//! Module that implements all the building blocks used to create the peer.

use crate::*;
use futures::{SinkExt, StreamExt};
use rand::{rng, RngCore};
use std::{collections::VecDeque, error::Error, sync::Arc, time::Duration};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{Mutex, Notify, RwLock},
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
        current_peer_server: &Arc<Mutex<Self>>,
        holding_hot_potato_notify: &Arc<Notify>,
        previous_peer_notify: &Arc<Notify>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut previous_peer_lines = Framed::new(previous_peer_stream, LinesCodec::new());

        loop {
            match previous_peer_lines.next().await {
                Some(Ok(hot_potato_string)) => {
                    if let Ok(hot_potato) = HotPotato::from_json_string(&hot_potato_string) {
                        // get hold of hot potato
                        {
                            let mut current_peer_server = current_peer_server.lock().await;
                            current_peer_server.hot_potato_state =
                                HotPotatoState::Holding(hot_potato);
                            holding_hot_potato_notify.notify_one();
                        }

                        /*log::debug(&cformat!(
                            "Currently holding <yellow, bold>hot potato</yellow, bold>"
                        )); */
                    }
                }
                _ => {
                    log::error("Couldn't receive the hot potato from previous peer.");
                    previous_peer_notify.notify_waiters();
                    return Err("".into());
                }
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

        // receive starting flag
        let _ = match server_lines.next().await {
            Some(Ok(line))
                if matches!(
                    StartFlag::from_json_string(&line).expect("(StartFlag) Shouldn't fail."),
                    StartFlag(_)
                ) =>
            {
                log::info("Received authorization from the server to start the protocol.");
            }
            _ => {
                log::error("Couldn't receive the starting flag from the server.");
            }
        };

        // create a thread-safe state instance
        let current_peer = Arc::new(Mutex::new(self.clone()));

        let holding_hot_potato_notify = Arc::new(Notify::new());
        let previous_peer_lost_potato_notify = Arc::new(Notify::new());
        let (server_writer, mut server_reader) = server_lines.split::<String>();

        let server_writer = Arc::new(RwLock::new(server_writer));

        // thread that handles the server connection
        let operation_server_thread = {
            let current_peer = Arc::clone(&current_peer);
            let holding_hot_potato_notify = holding_hot_potato_notify.clone();
            let previous_peer_lost_potato_notify = previous_peer_lost_potato_notify.clone();
            let server_writer = server_writer.clone();

            tokio::spawn(async move {
                loop {
                    tokio::select! {
                        Some(Ok(msg)) = server_reader.next() => {
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
                        () = previous_peer_lost_potato_notify.notified() => {
                            if let Err(_) = server_writer.write().await.send(RedistributeHotPotato().to_json_string().unwrap()).await {
                                log::error("Couldn't send the request for new hot potato to the server.");
                            }
                        }
                    }
                }
            })
        };

        // open server connection for previous peer to join
        let previous_peer_thread = {
            let current_peer = Arc::clone(&current_peer);
            let holding_hot_potato_notify = holding_hot_potato_notify.clone();
            let previous_peer_lost_potato_notify = previous_peer_lost_potato_notify.clone();

            tokio::spawn(async move {
                loop {
                    let (previous_peer_stream, _previous_peer_address) = previous_peer_listener
                        .accept()
                        .await
                        .expect("Failed to accept the previous peer's connection.");

                    log::warning("Successfully connected to previous peer.");

                    if let Err(_) = Self::handle_previous_peer(
                        previous_peer_stream,
                        &current_peer,
                        &holding_hot_potato_notify,
                        &previous_peer_lost_potato_notify,
                    )
                    .await
                    {
                        continue;
                    };
                }
            })
        };

        let calc_then_throw_hot_potato_thread = {
            let current_peer = Arc::clone(&current_peer);
            let holding_hot_potato_notify = holding_hot_potato_notify.clone();
            let previous_peer_lost_potato_notify = previous_peer_lost_potato_notify.clone();

            let server_writer = server_writer.clone();

            tokio::spawn(async move {
                loop {
                    let next_peer_address = { current_peer.lock().await.next_peer_address.clone() };

                    log::debug("Trying to connect to next peer.");

                    // connect to the next Peer's server
                    let next_peer_stream = match TcpStream::connect(next_peer_address).await {
                        Ok(stream) => stream,
                        Err(_) => {
                            log::error("Couldn't connect to the next peer.");
                            continue;
                        }
                    };

                    let mut next_peer_lines = Framed::new(next_peer_stream, LinesCodec::new());

                    log::warning("Successfully connected to next peer.");

                    loop {
                        holding_hot_potato_notify.notified().await;
                        {
                            let mut current_peer = current_peer.lock().await;
                            if let HotPotatoState::Holding(hot_potato) =
                                &current_peer.hot_potato_state
                            {
                                if let Ok(hot_potato_string) = hot_potato.to_json_string() {
                                    // send all operations request to server
                                    while let Some(operation_request) =
                                        current_peer.request_queue.pop_front()
                                    {
                                        operation_request.print();
                                        server_writer
                                            .write()
                                            .await
                                            .send(
                                                operation_request
                                                    .to_json_string()
                                                    .expect("Couldn't parse operation request."),
                                            )
                                            .await
                                            .expect("Couldn't send operation request to server.");
                                    }

                                    // sleep(Duration::from_secs(2)).await;
                                    if let Err(_) = next_peer_lines.send(hot_potato_string).await {
                                        // log::error("Couldn't send potato to next peer.");
                                        previous_peer_lost_potato_notify.notify_one();
                                    }
                                }
                            }
                            current_peer.hot_potato_state = HotPotatoState::NotHolding;
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

        if let Err(_) = operation_server_thread.await {
            log::error("Operation Server Thread failded.");
        }
        if let Err(_) = previous_peer_thread.await {
            log::error("Previous Peer Thread failded.");
        }
        if let Err(_) = calc_then_throw_hot_potato_thread.await {
            log::error("Calculate then Throw Hot Potato Thread failded.");
        }
        if let Err(_) = generate_potato_work_thread.await {
            log::error("Generate Potato Work Thread failded.");
        }
    }
}
