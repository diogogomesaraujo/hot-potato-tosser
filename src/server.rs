use futures::{SinkExt, StreamExt};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, Mutex},
};
use tokio_util::codec::{Framed, LinesCodec};

use crate::*;
use std::{error::Error, net::SocketAddr, sync::Arc};

#[derive(Clone)]
pub struct Server {
    pub own_address: String,
    // pub peer_addresses: HashMap<SocketAddr, Tx>,
}

pub struct PeerConnecton {
    // pub receiver: Rx,
    // pub sender: Tx,
    pub address: SocketAddr,
}

impl Server {
    pub fn new(own_address: String) -> Self {
        Self {
            own_address,
            // peer_addresses: HashMap::new(),
        }
    }

    async fn handle(
        stream: TcpStream,
        address: SocketAddr,
        server: Arc<Mutex<Self>>,
        hot_potato: Option<HotPotato>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let lines = Framed::new(stream, LinesCodec::new());
        let (mut writer, mut reader) = lines.split::<String>();
        let (tx, rx) = mpsc::unbounded_channel::<String>();

        let mut participant = PeerConnecton { address };

        // send the hot potato to a peer in the beginning of the program
        if let Some(hot_potato) = hot_potato {
            writer.send(hot_potato.to_json_string()?).await?;
        }

        loop {
            tokio::select! {
                Some(Ok(_operation_request)) = reader.next() => {
                    // do operation and send response
                }
            }
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        let listener = TcpListener::bind(&self.own_address).await?;
        let server = Arc::new(Mutex::new(self.clone()));

        let send_hot_potato = Arc::new(Mutex::new(true));

        loop {
            let (peer_stream, peer_address) = listener.accept().await?;

            let server = server.clone();
            let send_hot_potato = send_hot_potato.clone();

            let _handle = tokio::spawn(async move {
                let mut send_hot_potato = send_hot_potato.lock().await;
                if let Err(e) = Self::handle(
                    peer_stream,
                    peer_address,
                    server,
                    match &*send_hot_potato {
                        true => {
                            *send_hot_potato = false;
                            Some(HotPotato::new())
                        }
                        _ => None,
                    },
                )
                .await
                {
                    eprintln!("{e}");
                };
            });
        }
    }
}
