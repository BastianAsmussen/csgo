use error::Error;

use crate::player::Player;
use crate::server::Server;

use tokio::net::TcpListener;

mod error;
mod player;
mod server;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let mut players = Vec::new();

    let listener = TcpListener::bind("0.0.0.0:7512").await?;
    while players.len() < 1 {
        let (socket, _) = listener.accept().await?;

        let player = Player::new(players.len(), socket).await?;
        players.push(player);
    }

    let mut server = Server::new(players);
    server.run().await?;

    Ok(())
}
