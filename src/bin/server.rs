use clap::Parser;
use hot_potato_tosser::{log, server};
use std::{error::Error, time::Duration};
use tokio::time::sleep;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(index = 1)]
    self_address: String,

    #[arg(index = 2)]
    number_of_peers: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let args = Args::parse();
    log::clear();

    let server = server::Server::new(args.self_address, args.number_of_peers);

    loop {
        sleep(Duration::from_secs(server.number_of_peers as u64)).await;
        if let Err(e) = server.run().await {
            eprintln!("{e}");
        }
    }
}
