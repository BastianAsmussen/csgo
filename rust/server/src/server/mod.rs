use crate::{error::Error, Player};

use tokio::io::AsyncWriteExt;

#[derive(Debug)]
pub struct Server {
    players: Vec<Player>,
}

impl Server {
    pub fn new(players: Vec<Player>) -> Self {
        Self { players }
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            // First, request all players states.
            for player in self.players.iter_mut() {
                player.request().await?;
            }

            // Then, inform all players about the others states.
            let states: Vec<_> = self
                .players
                .iter()
                .map(|p| p.state())
                .collect::<Result<_, _>>()?;

            for (player, state) in self.players.iter_mut().zip(states) {
                player.inform(state.as_bytes()).await?;
            }
        }
    }
}
