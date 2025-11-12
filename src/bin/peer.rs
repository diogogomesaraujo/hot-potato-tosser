use clap::Parser;
use std::{error::Error, time::Duration};
use token_ring::{log, peer};
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(index = 1)]
    self_address: String,

    #[arg(index = 2)]
    server_address: String,

    #[arg(index = 3)]
    next_peer_address: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Args::parse();
    log::clear();

    let mut peer = peer::Peer::new(
        args.self_address,
        args.server_address,
        args.next_peer_address,
    );

    loop {
        peer.run().await;
        sleep(Duration::from_secs(5)).await;
    }
}
