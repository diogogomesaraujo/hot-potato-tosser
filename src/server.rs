use crate::{sync::Flag, *};
use color_print::cformat;
use futures::{SinkExt, StreamExt};
use std::{collections::HashMap, error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{self, Sender},
        Barrier, RwLock,
    },
};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Clone)]
pub struct Server {
    pub own_address: String,
    pub number_of_peers: usize,
    pub timeout_flag: Flag,
    pub connections: HashMap<SocketAddr, Sender<HotPotato>>,
    pub who_noticed_failure_address: Option<SocketAddr>,
}

impl Server {
    pub fn new(own_address: String, number_of_peers: usize) -> Self {
        Self {
            own_address,
            number_of_peers,
            timeout_flag: Flag::new(false),
            connections: HashMap::new(),
            who_noticed_failure_address: None,
        }
    }

    async fn handle(
        stream: TcpStream,
        address: SocketAddr,
        barrier: Arc<Barrier>,
        starts_with_hot_potato: Flag,
        server: Arc<RwLock<Server>>,
        blocking_flag: bool,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let lines = Framed::new(stream, LinesCodec::new());
        let (writer, mut reader) = lines.split::<String>();

        let writer = Arc::new(RwLock::new(writer));

        let (tx, mut rx) = mpsc::channel::<HotPotato>(1);

        {
            server.write().await.connections.insert(address, tx);
        }

        // wait for all participants to join
        if blocking_flag {
            barrier.wait().await;
        }

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
                        // log::info(&cformat!("Sent a new potato to <bold>{}</bold>.", address));
                    }
                }
            });
        }

        loop {
            match reader.next().await {
                Some(Ok(line)) => match serde_json::from_str::<ServerRequest>(&line) {
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
                    Err(_) => match RedistributeHotPotato::from_json_string(&line) {
                        Ok(_) => {
                            let noticed_address = server.read().await.who_noticed_failure_address;
                            match noticed_address {
                                None => {
                                    server.write().await.who_noticed_failure_address =
                                        Some(address);
                                    writer
                                        .write()
                                        .await
                                        .send(HotPotato::new().to_json_string()?)
                                        .await?;
                                }
                                Some(noticed_address) => {
                                    server.read().await.connections[&noticed_address]
                                        .send(HotPotato::new())
                                        .await?;
                                }
                            }
                        }
                        Err(_) => {
                            writer
                                .write()
                                .await
                                .send(
                                    ServerResponse::Err(
                                        0,
                                        0,
                                        cformat!(
                                            "The request had <bold>incorrect formatting</bold>."
                                        ),
                                    )
                                    .to_json_string()?,
                                )
                                .await?;
                        }
                    },
                },
                _ => {}
            }
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(&self.own_address).await?;
        let server = Arc::new(RwLock::new(self.clone()));

        let barrier = Arc::new(Barrier::new(self.number_of_peers));
        let starts_with_hot_potato = Flag::new(true);

        for _ in 0..self.number_of_peers {
            let (peer_stream, peer_address) = listener.accept().await?;

            log::info(&cformat!("Accepted a <bold>connection</bold>."));

            let server = server.clone();
            let barrier = barrier.clone();
            let starts_with_hot_potato = starts_with_hot_potato.clone();

            let _handle = tokio::spawn(async move {
                if let Err(e) = Self::handle(
                    peer_stream,
                    peer_address,
                    barrier,
                    starts_with_hot_potato,
                    server,
                    true,
                )
                .await
                {
                    log::error(&format!("{e}"));
                };
            });
        }

        loop {
            let (peer_stream, peer_address) = listener.accept().await?;

            log::info(&cformat!("Accepted a <bold>connection</bold>."));

            let server = server.clone();
            let barrier = barrier.clone();
            let starts_with_hot_potato = starts_with_hot_potato.clone();

            let _handle = tokio::spawn(async move {
                if let Err(e) = Self::handle(
                    peer_stream,
                    peer_address,
                    barrier,
                    starts_with_hot_potato,
                    server,
                    false,
                )
                .await
                {
                    log::error(&format!("{e}"));
                };
            });
        }
    }
}
