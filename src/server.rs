use crate::*;
use color_print::cformat;
use futures::{SinkExt, StreamExt};
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, UnboundedSender},
        Barrier, Notify, RwLock,
    },
    time::timeout,
};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Clone)]
pub struct PeerMapping {
    pub address_to_sender: HashMap<SocketAddr, UnboundedSender<HotPotato>>,
    pub own_to_own_next_peer: HashMap<String, (String, SocketAddr)>,
}

#[derive(Clone)]
pub struct Server {
    pub own_address: String,
    pub number_of_peers: usize,
    pub timeout_flag: Flag,
    pub peer_mapping: PeerMapping,
    pub peer_to_send_potato_to: Option<String>,
}

impl PeerMapping {
    pub fn new() -> Self {
        Self {
            address_to_sender: HashMap::new(),
            own_to_own_next_peer: HashMap::new(),
        }
    }
}

impl Server {
    pub fn new(own_address: String, number_of_peers: usize) -> Self {
        Self {
            own_address,
            number_of_peers,
            timeout_flag: Flag::new(false),
            peer_mapping: PeerMapping::new(),
            peer_to_send_potato_to: None,
        }
    }

    async fn handle(
        stream: TcpStream,
        address: SocketAddr,
        barrier: Arc<Barrier>,
        starts_with_hot_potato: Flag,
        notify_timeout: Arc<Notify>,
        server: Arc<RwLock<Server>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let lines = Framed::new(stream, LinesCodec::new());
        let (mut writer, mut reader) = lines.split::<String>();

        let (tx, mut rx) = mpsc::unbounded_channel::<HotPotato>();

        // receive addresses
        let address_string = {
            let mut address_string = String::new();
            if let Some(Ok(line)) = reader.next().await {
                match Addresses::from_json_string(&line) {
                    Ok(addresses) => {
                        address_string = addresses.own.clone();
                        server
                            .write()
                            .await
                            .peer_mapping
                            .address_to_sender
                            .insert(address.clone(), tx);
                        server
                            .write()
                            .await
                            .peer_mapping
                            .own_to_own_next_peer
                            .insert(addresses.own, (addresses.next_peer, address));
                    }
                    Err(_) => {
                        log::error("Couldn't parse the addresses received.");
                    }
                }
            }
            address_string
        };

        // wait for all participants to join
        barrier.wait().await;

        log::info(&cformat!("Send <bold>starting flag</bold> to peer."));
        writer.send(StartFlag(true).to_json_string()?).await?;
        writer.flush().await?;

        {
            if starts_with_hot_potato.read().await {
                log::info(&cformat!(
                    "Sending <yellow, bold>hot potato</yellow, bold> to a peer."
                ));
                starts_with_hot_potato.write(false).await;
                writer.send(StartFlag(true).to_json_string()?).await?;
                writer.flush().await?;
            }
        }

        loop {
            tokio::select! {
                Some(hot_potato) = rx.recv() => {
                    writer.send(hot_potato.to_json_string().unwrap_or("".to_string())).await?;
                }
                timeout = timeout(TIMEOUT_DURATION, reader.next()) => {
                    match timeout {
                        Ok(Some(Ok(line))) => match serde_json::from_str::<ServerRequest>(&line) {
                            Ok(request) => {
                                let response = request.to_response();

                                request.print();
                                response.print();

                                writer.send(response.to_json_string()?).await?;
                            }
                            Err(_) => {
                                writer
                                    .send(
                                        ServerResponse::Err(
                                            0,
                                            0,
                                            cformat!("The request had <bold>incorrect formatting</bold>."),
                                        )
                                        .to_json_string()?,
                                    )
                                    .await?;
                            }
                        },
                        Ok(None) if !server.read().await.timeout_flag.read().await => {
                            server.write().await.timeout_flag.write(true).await;
                            let next_peer_address = match &server.read().await.peer_mapping.own_to_own_next_peer.get(&address_string) {
                                Some(addresses) => {
                                    addresses.0.clone()
                                },
                                None => {
                                    log::error("Failed to get the addresses for the string address given.");
                                    return Err("Failed to get the addresses for the string address given.".into());
                                }
                            };
                            server.write().await.peer_to_send_potato_to = Some(next_peer_address);
                            notify_timeout.notify_one();
                            return Ok(());
                        }
                        _ if !server.read().await.timeout_flag.read().await => {
                            server.write().await.timeout_flag.write(true).await;
                            let next_peer_address = match &server.read().await.peer_mapping.own_to_own_next_peer.get(&address_string) {
                                Some(addresses) => {
                                    addresses.0.clone()
                                },
                                None => {
                                    log::error("Failed to get the addresses for the string address given.");
                                    return Err("Failed to get the addresses for the string address given.".into());
                                }
                            };
                            server.write().await.peer_to_send_potato_to = Some(next_peer_address);
                            notify_timeout.notify_one();
                            return Ok(());
                        }
                        _ => {
                            return Ok(());
                        }
                    }
                }
            }
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(&self.own_address).await?;
        let server = Arc::new(RwLock::new(self.clone()));

        let barrier = Arc::new(Barrier::new(self.number_of_peers));
        let starts_with_hot_potato = Flag::new(true);
        let notify_timeout = Arc::new(Notify::new());

        let _throw_potato_on_timeout_thread = {
            let notify_timeout = notify_timeout.clone();
            let server = server.clone();

            tokio::spawn(async move {
                loop {
                    notify_timeout.notified().await;

                    if let Some(peer_to_send_potato_to) =
                        &server.read().await.peer_to_send_potato_to
                    {
                        if let Some((_, address)) = &server
                            .read()
                            .await
                            .peer_mapping
                            .own_to_own_next_peer
                            .get(peer_to_send_potato_to)
                        {
                            if let Some(tx) = &server
                                .read()
                                .await
                                .peer_mapping
                                .address_to_sender
                                .get(address)
                            {
                                log::info(&cformat!(
                                    "Sending a new <bold>hot potato</bold> to <bold>{}</bold>.",
                                    peer_to_send_potato_to
                                ));

                                if let Err(_) = tx.send(HotPotato::new()) {
                                    log::error("Couldn't redistribute the hot potato.");
                                    continue;
                                }
                            }
                        }
                    }

                    server.write().await.timeout_flag.write(false).await;
                }
            })
        };

        loop {
            let (peer_stream, peer_address) = listener.accept().await?;

            log::info(&cformat!("Accepted a <bold>connection</bold>."));

            let server = server.clone();
            let barrier = barrier.clone();
            let starts_with_hot_potato = starts_with_hot_potato.clone();
            let notify_timeout = notify_timeout.clone();

            let _handle = tokio::spawn(async move {
                if let Err(e) = Self::handle(
                    peer_stream,
                    peer_address,
                    barrier,
                    starts_with_hot_potato,
                    notify_timeout,
                    server,
                )
                .await
                {
                    log::error(&format!("{e}"));
                };
            });
        }
    }
}
