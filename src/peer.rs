//! Module that implements all the building blocks used to create the peer.

use crate::*;
use futures::{SinkExt, StreamExt};
use rand::{rng, RngCore};
use std::{collections::VecDeque, error::Error, sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, Notify},
    time::sleep,
};
use tokio_util::codec::{Framed, LinesCodec};

pub type RequestQueue = VecDeque<ServerRequest>;

#[derive(Clone)]
pub struct Peer {
    pub address: String,
    pub server_address: String,
    pub next_peer_address: String,
    pub hot_potato_state: HotPotatoState,
    pub request_queue: RequestQueue,
}

impl Peer {
    pub fn new(address: String, server_address: String, next_peer_address: String) -> Self {
        let mut rng = rand::rng();

        Self {
            address,
            server_address,
            next_peer_address,
            hot_potato_state: HotPotatoState::NotHolding,
            request_queue: RequestQueue::from([ServerRequest::generate(&mut rng)]),
        }
    }

    pub async fn handle_previous_peer(
        previous_peer_stream: TcpStream,
        current_peer_server: Arc<Mutex<Self>>,
        holding_hot_potato_notify: Arc<Notify>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut previous_peer_lines = Framed::new(previous_peer_stream, LinesCodec::new());

        loop {
            if let Some(Ok(hot_potato_string)) = previous_peer_lines.next().await {
                if let Ok(hot_potato) = HotPotato::from_json_string(&hot_potato_string) {
                    // get hold of hot potato
                    {
                        let mut current_peer_server = current_peer_server.lock().await;
                        current_peer_server.hot_potato_state = HotPotatoState::Holding(hot_potato);
                        holding_hot_potato_notify.notify_one();
                    }

                    /*log::debug(&cformat!(
                        "Currently holding <yellow, bold>hot potato</yellow, bold>"
                    )); */
                }
            } else {
                return Err("Couldn't receive hot potato from previous peer.".into());
            }
        }
    }

    pub async fn run(&mut self) {
        let mut rng = rng();

        // client connections
        let server_stream = match TcpStream::connect(&self.server_address).await {
            Ok(stream) => stream,
            Err(_) => {
                log::error("Failed to connect to the server.");
                return;
            }
        };
        let mut server_lines = Framed::new(server_stream, LinesCodec::new());

        // open a server for previous Peer to connect
        let previous_peer_listener = match TcpListener::bind(&self.address).await {
            Ok(listener) => listener,
            Err(_) => {
                log::error("Couldn't open connection for the previous peer.");
                return;
            }
        };

        //send addresses
        if let Err(_) = server_lines
            .send(
                match Addresses::new(self.address.clone(), self.next_peer_address.clone())
                    .to_json_string()
                {
                    Ok(msg) => msg,
                    Err(_) => {
                        log::error("Couldn't parse the addresses to send to the server.");
                        return;
                    }
                },
            )
            .await
        {
            log::error("Couldn't send addresses to the server.");
            return;
        }

        // receive starting flag
        let _ = match server_lines.next().await {
            Some(Ok(line))
                if matches!(
                    StartFlag::from_json_string(&line).expect("(StartFlag) Shouldn't fail."),
                    StartFlag(_)
                ) => {}
            _ => {
                log::error("Couldn't receive the starting flag from the server.");
            }
        };

        // create a thread-safe state instance
        let current_peer = Arc::new(Mutex::new(self.clone()));

        // connect to the next Peer's server
        let next_peer_stream = match TcpStream::connect(&self.next_peer_address).await {
            Ok(stream) => stream,
            Err(_) => {
                log::error("Couldn't connect to the next peer.");
                return;
            }
        };

        let holding_hot_potato_notify = Arc::new(Notify::new());
        let (mut server_writer, mut server_reader) = server_lines.split::<String>();

        // thread that handles the server connection
        let operation_server_thread = {
            let current_peer = Arc::clone(&current_peer);
            let holding_hot_potato_notify = holding_hot_potato_notify.clone();

            tokio::spawn(async move {
                loop {
                    if let Some(Ok(msg)) = server_reader.next().await {
                        if let Ok(hot_potato) = HotPotato::from_json_string(&msg) {
                            let mut current_peer = current_peer.lock().await;

                            current_peer.hot_potato_state = HotPotatoState::Holding(hot_potato);
                            holding_hot_potato_notify.notify_one();

                            /*log::debug(&cformat!(
                                "Currently holding <yellow, bold>hot potato</yellow, bold>"
                            ));*/
                        }

                        if let Ok(operation_response) = ServerResponse::from_json_string(&msg) {
                            operation_response.print();
                        }
                    }
                }
            })
        };

        // open server connection for previous peer to join
        let previous_peer_thread = {
            let current_peer = Arc::clone(&current_peer);
            let holding_hot_potato_notify = holding_hot_potato_notify.clone();

            tokio::spawn(async move {
                loop {
                    let (previous_peer_stream, _previous_peer_address) =
                        match previous_peer_listener.accept().await {
                            Ok(connection) => connection,
                            Err(_) => continue,
                        };

                    if let Err(e) = Self::handle_previous_peer(
                        previous_peer_stream,
                        current_peer.clone(),
                        holding_hot_potato_notify.clone(),
                    )
                    .await
                    {
                        log::error(&format!("{e}"));
                    };
                }
            })
        };

        let calc_then_throw_hot_potato_thread = {
            let current_peer = Arc::clone(&current_peer);
            let holding_hot_potato_notify = holding_hot_potato_notify.clone();
            let mut next_peer_lines = Framed::new(next_peer_stream, LinesCodec::new());

            tokio::spawn(async move {
                loop {
                    holding_hot_potato_notify.notified().await;

                    let (hot_potato_string, has_requests) = {
                        let mut current_peer = current_peer.lock().await;

                        let hot_potato_string = match &current_peer.hot_potato_state {
                            HotPotatoState::Holding(hot_potato) => hot_potato.to_json_string().ok(),
                            _ => None,
                        };

                        let has_requests = !current_peer.request_queue.is_empty();

                        // Send all operations request to server
                        while let Some(operation_request) = current_peer.request_queue.pop_front() {
                            operation_request.print();
                            if let Err(_) = server_writer
                                .send(operation_request.to_json_string().unwrap_or("".to_string()))
                                .await
                            {
                                log::error("Couldn't send operation request to server.");
                            }
                        }

                        current_peer.hot_potato_state = HotPotatoState::NotHolding;

                        (hot_potato_string, has_requests)
                    }; // Lock released here

                    // Send hot potato after releasing lock
                    if let Some(potato_str) = hot_potato_string {
                        if let Err(_) = next_peer_lines.send(potato_str).await {
                            log::error("Couldn't send hot potato to next peer.");
                        }
                    }
                }
            })
        };

        let generate_potato_work_thread = {
            let current_peer = current_peer.clone();

            let mut seed: [u8; 32] = [0u8; 32];
            rng.fill_bytes(&mut seed);

            let mut poisson_process = Poisson::new(RATE, &mut seed);

            tokio::spawn(async move {
                loop {
                    sleep(Duration::from_secs_f64(
                        poisson_process.time_for_next_event(),
                    ))
                    .await;
                    current_peer
                        .lock()
                        .await
                        .request_queue
                        .push_back(ServerRequest::generate(&mut poisson_process.rng));
                }
            })
        };

        if let Err(_) = tokio::try_join!(
            operation_server_thread,
            previous_peer_thread,
            calc_then_throw_hot_potato_thread,
            generate_potato_work_thread
        ) {
            log::error("Couldn't close all threads");
        }
    }
}
