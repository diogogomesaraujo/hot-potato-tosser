use crate::*;
use color_print::cformat;
use futures::{SinkExt, StreamExt};
use std::{collections::HashMap, error::Error, net::SocketAddr, str::FromStr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Sender},
        Barrier, Notify, RwLock,
    },
    time::timeout,
};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Clone)]
pub struct AddressMap(pub HashMap<SocketAddr, SocketAddr>);

impl AddressMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn push_addresses(
        &mut self,
        addresses: &Addresses,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.0.insert(
            SocketAddr::from_str(&addresses.own_address)?,
            SocketAddr::from_str(&addresses.peer_address)?,
        );
        Ok(())
    }

    pub fn push_socket_addresses(
        &mut self,
        addresses: (SocketAddr, SocketAddr),
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.0.insert(addresses.0, addresses.1);
        Ok(())
    }

    pub fn get(&self, address: SocketAddr) -> Option<SocketAddr> {
        match self.0.get(&address) {
            Some(addr) => Some(addr.clone()),
            _ => None,
        }
    }

    pub fn get_value(&self, address: SocketAddr) -> Option<SocketAddr> {
        for (k, v) in &self.0 {
            if v == &address {
                return Some(*k);
            }
        }
        None
    }
}

#[derive(Clone)]
pub struct Server {
    pub own_address: String,
    pub number_of_peers: usize,
    pub timeout_flag: Flag,
    pub connections: HashMap<SocketAddr, Sender<HotPotato>>,
    pub peer_order: AddressMap,
    pub own_address_mapping: AddressMap,
    pub failing_peer_address: Option<SocketAddr>,
}

impl Server {
    pub fn new(own_address: String, number_of_peers: usize) -> Self {
        Self {
            own_address,
            number_of_peers,
            timeout_flag: Flag::new(false),
            connections: HashMap::new(),
            peer_order: AddressMap::new(),
            own_address_mapping: AddressMap::new(),
            failing_peer_address: None,
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
        let (writer, mut reader) = lines.split::<String>();

        let writer = Arc::new(RwLock::new(writer));

        let (tx, mut rx) = mpsc::channel::<HotPotato>(1);

        {
            server.write().await.connections.insert(address, tx);
        }

        if let Some(Ok(msg)) = reader.next().await {
            if let Ok(addresses) = serde_json::from_str::<Addresses>(&msg) {
                server.write().await.peer_order.push_addresses(&addresses)?;

                server
                    .write()
                    .await
                    .own_address_mapping
                    .push_socket_addresses((
                        address,
                        SocketAddr::from_str(&addresses.own_address)?,
                    ))?;
            }
        }

        // wait for all participants to join
        barrier.wait().await;

        log::info(&cformat!("Send <bold>starting flag</bold> to peer."));
        writer
            .write()
            .await
            .send(StartFlag(true).to_json_string()?)
            .await?;
        writer.write().await.flush().await?;

        {
            if starts_with_hot_potato.read().await {
                log::info(&cformat!(
                    "Sending <yellow, bold>hot potato</yellow, bold> to a peer."
                ));
                starts_with_hot_potato.write(false).await;
                writer
                    .write()
                    .await
                    .send(StartFlag(true).to_json_string()?)
                    .await?;
                writer.write().await.flush().await?;
            }
        }

        {
            let writer = writer.clone();
            tokio::spawn(async move {
                loop {
                    if let Some(hot_potato) = rx.recv().await {
                        if let Err(_) = writer
                            .write()
                            .await
                            .send(hot_potato.to_json_string().unwrap())
                            .await
                        {
                            log::error("Couldn't send hot potato to next peer.");
                            break;
                        }
                        log::info(&cformat!("Sent a new potato to <bold>{}</bold>.", address));
                    }
                }
            });
        }

        loop {
            match timeout(TIMEOUT_DURATION, reader.next()).await {
                Ok(Some(Ok(line))) => match serde_json::from_str::<ServerRequest>(&line) {
                    Ok(request) => {
                        let response = request.to_response();

                        request.print();
                        response.print();

                        writer
                            .write()
                            .await
                            .send(response.to_json_string()?)
                            .await?;
                    }
                    Err(_) => {
                        writer
                            .write()
                            .await
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
                    server.write().await.failing_peer_address = Some(address);
                    notify_timeout.notify_one();
                    return Ok(());
                }
                _ => {}
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

                    {
                        let failing_peer = match server.read().await.failing_peer_address {
                            Some(fp) => fp,
                            _ => {
                                log::error("Couldn't get the failing peer address.");
                                return;
                            }
                        };

                        let address = match server
                            .read()
                            .await
                            .own_address_mapping
                            .get(failing_peer)
                        {
                            Some(a) => a,
                            _ => {
                                log::error("Couldn't get the faling peer address from own_address_mapping.");
                                return;
                            }
                        };

                        let next_peer_address = match server.read().await.peer_order.get(address) {
                            Some(npa) => npa,
                            _ => {
                                log::error(
                                    "Couldn't get the next peer address from failing peer address.",
                                );
                                return;
                            }
                        };

                        let next_peer_address = match server
                            .read()
                            .await
                            .own_address_mapping
                            .get_value(next_peer_address)
                        {
                            Some(npa) => npa,
                            _ => {
                                log::error(
                                    "Couldn't get the next peer address from own_address_mapping.",
                                );
                                return;
                            }
                        };

                        match server.read().await.connections.get(&next_peer_address) {
                            Some(tx) => {
                                if let Err(_) = tx.send(HotPotato::new()).await {
                                    log::error("Couldn't send the new hot potato to next peer.");
                                    return;
                                }
                            }
                            _ => {
                                log::error("Couldn't get the next peer address sender.");
                                return;
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
