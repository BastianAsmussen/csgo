use clap::Parser;

use error::Error;

use crate::player::Player;
use crate::server::Server;

use tokio::net::TcpListener;

mod error;
mod player;
mod server;

/// A simple game server.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// The number of players to wait for.
    #[arg(short, long, default_value = "1")]
    count: usize,

    /// The port to listen on.
    #[arg(short, long, default_value = "7512")]
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let args = Args::parse();

    let mut players = Vec::with_capacity(args.count);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;
    while players.len() < args.count {
        let (socket, _) = listener.accept().await?;

        let player = Player::new(players.len(), socket).await?;
        players.push(player);
    }

    let mut server = Server::new(players);
    server.run().await?;

    Ok(())
}
