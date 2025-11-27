use crate::*;
use color_print::cformat;
use futures::{SinkExt, StreamExt};
use std::{error::Error, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Barrier, Notify, RwLock},
    time::timeout,
};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Clone)]
pub struct Server {
    pub own_address: String,
    pub number_of_peers: usize,
    pub timeout_flag: Flag,
}

impl Server {
    pub fn new(own_address: String, number_of_peers: usize) -> Self {
        Self {
            own_address,
            number_of_peers,
            timeout_flag: Flag::new(false),
        }
    }

    async fn handle(
        stream: TcpStream,
        barrier: Arc<Barrier>,
        starts_with_hot_potato: Flag,
        notify_timeout: Arc<Notify>,
        server: Arc<RwLock<Server>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let lines = Framed::new(stream, LinesCodec::new());
        let (mut writer, mut reader) = lines.split::<String>();

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
            match timeout(TIMEOUT_DURATION, reader.next()).await {
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

                    log::debug("Here should be the logic to send the potato!");

                    server.write().await.timeout_flag.write(false).await;
                }
            })
        };

        loop {
            let (peer_stream, _peer_address) = listener.accept().await?;

            log::info(&cformat!("Accepted a <bold>connection</bold>."));

            let server = server.clone();
            let barrier = barrier.clone();
            let starts_with_hot_potato = starts_with_hot_potato.clone();
            let notify_timeout = notify_timeout.clone();

            let _handle = tokio::spawn(async move {
                if let Err(e) = Self::handle(
                    peer_stream,
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
