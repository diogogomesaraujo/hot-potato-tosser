use crate::*;
use futures::{SinkExt, StreamExt};
use std::{error::Error, net::SocketAddr, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::Mutex,
};
use tokio_util::codec::{Framed, LinesCodec};

#[derive(Clone)]
pub struct Peer {
    pub address: String,
    pub server_address: String,
    pub next_peer_address: String,
    pub hot_potato_state: HotPotatoState,
}

impl Peer {
    pub fn new(address: String, server_address: String, next_peer_address: String) -> Self {
        Self {
            address,
            server_address,
            next_peer_address,
            hot_potato_state: HotPotatoState::NotHolding,
        }
    }

    pub async fn handle_previous_peer(
        previous_peer_stream: TcpStream,
        previous_peer_address: SocketAddr,
        next_peer_stream: TcpStream,
        current_peer_server: Arc<Mutex<Self>>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut previous_peer_lines = Framed::new(previous_peer_stream, LinesCodec::new());
        let mut next_peer_lines = Framed::new(next_peer_stream, LinesCodec::new());

        loop {
            tokio::select! {
                Some(Ok(hot_potato_string)) = previous_peer_lines.next() => {
                    match HotPotato::from_json_string(&hot_potato_string) {
                        Ok(hot_potato) => {
                            println!("Holding Hot Potato thrown by {}", previous_peer_address);
                            // TODO check if there is things to compute before throwing the potato

                            // get hold of hot potato
                            {
                                let mut current_peer_server = current_peer_server.lock().await;
                                current_peer_server.hot_potato_state = HotPotatoState::Holding(hot_potato);
                            }

                            // throw hot potato
                            {
                                next_peer_lines.send(hot_potato_string).await?;
                            }

                            {
                                let mut current_peer_server = current_peer_server.lock().await;
                                current_peer_server.hot_potato_state = HotPotatoState::NotHolding;
                            }
                        },
                        Err(_) => {
                            // Do something if the hot potato is compromised
                        }
                    }
                }
            }
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // client connections
        let server_stream = TcpStream::connect(&self.server_address).await?;
        let next_peer_stream = TcpStream::connect(&self.next_peer_address).await?;

        // let mut server_lines = Framed::new(server_stream, LinesCodec::new());

        // server connection
        let previous_peer_listener = TcpListener::bind(&self.address).await?;

        // create a thread-safe hot potato
        let current_peer = Arc::new(Mutex::new(self.clone()));

        let operation_server_thread = {
            let current_peer = Arc::clone(&current_peer);

            tokio::spawn(async move {
                let mut server_lines = Framed::new(server_stream, LinesCodec::new());

                tokio::select! {
                    Some(Ok(msg)) = server_lines.next() => {
                        if let Ok(hot_potato) = HotPotato::from_json_string(&msg) {
                            let mut current_peer = current_peer.lock().await;
                            current_peer.hot_potato_state = HotPotatoState::Holding(hot_potato);
                        }

                        if let Ok(operation_response) = Response::from_json_string(&msg) {
                            operation_response.print();
                        }
                    }
                }
            })
        };

        // open server connection for previous peer to join
        let hot_potato_thread = {
            let current_peer = Arc::clone(&current_peer);

            tokio::spawn(async move {
                let (previous_peer_stream, previous_peer_address) = previous_peer_listener
                    .accept()
                    .await
                    .expect("Failed to accept the previous peer's connection.");

                if let Err(e) = Self::handle_previous_peer(
                    previous_peer_stream,
                    previous_peer_address,
                    next_peer_stream,
                    current_peer,
                )
                .await
                {
                    eprintln!("{e}");
                };
            })
        };

        operation_server_thread.await?;
        hot_potato_thread.await?;

        Ok(())
    }
}
