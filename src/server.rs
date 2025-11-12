use crate::*;
use color_print::cformat;
use futures::{SinkExt, StreamExt};
use std::{error::Error, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Barrier, Mutex},
};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Clone)]
pub struct Server {
    pub own_address: String,
    pub number_of_peers: usize,
}

impl Server {
    pub fn new(own_address: String, number_of_peers: usize) -> Self {
        Self {
            own_address,
            number_of_peers,
        }
    }

    async fn handle(
        stream: TcpStream,
        _server: Arc<Mutex<Self>>,
        barrier: Arc<Barrier>,
        starts_with_hot_potato: Arc<Mutex<bool>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let lines = Framed::new(stream, LinesCodec::new());
        let (mut writer, mut reader) = lines.split::<String>();

        // wait for all participants to join
        barrier.wait().await;

        log::info(&cformat!("Send <bold>starting flag</bold> to peer."));
        writer.send(StartFlag(true).to_json_string()?).await?;
        writer.flush().await?;

        {
            let mut starts_with_hot_potato = starts_with_hot_potato.lock().await;

            if *starts_with_hot_potato {
                log::info(&cformat!(
                    "Sending <yellow, bold>hot potato</yellow, bold> to a peer."
                ));
                *starts_with_hot_potato = false;
                writer.send(StartFlag(true).to_json_string()?).await?;
                writer.flush().await?;
            }
        }

        loop {
            tokio::select! {
                Some(Ok(line)) = reader.next() => {
                    match serde_json::from_str::<Request>(&line) {
                        Ok(request) => {
                            let response = request.to_response();

                            request.print();
                            response.print();

                            writer.send(response.to_json_string()?).await?;
                        }
                        Err(_) => {
                            writer.send(Response::Err(0, 0, cformat!("The request had <bold>incorrect formatting</bold>.")).to_json_string()?).await?;
                        }
                    }
                }
            }
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(&self.own_address).await?;
        let server = Arc::new(Mutex::new(self.clone()));

        let barrier = Arc::new(Barrier::new(self.number_of_peers));
        let starts_with_hot_potato = Arc::new(Mutex::new(true));

        loop {
            let (peer_stream, _peer_address) = listener.accept().await?;

            log::info(&cformat!("Accepted a <bold>connection</bold>."));

            let server = server.clone();
            let barrier = barrier.clone();
            let starts_with_hot_potato = starts_with_hot_potato.clone();

            let _handle = tokio::spawn(async move {
                if let Err(e) =
                    Self::handle(peer_stream, server, barrier, starts_with_hot_potato).await
                {
                    log::error(&format!("{e}"));
                };
            });
        }
    }
}
