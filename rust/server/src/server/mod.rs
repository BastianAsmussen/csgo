use crate::{error::Error, Player};

#[derive(Debug)]
pub struct Server {
    players: Vec<Player>,
}

impl Server {
    pub fn new(players: Vec<Player>) -> Self {
        Self { players }
    }

    pub fn players(&self) -> &[Player] {
        &self.players
    }

    pub fn players_mut(&mut self) -> &mut [Player] {
        &mut self.players
    }

    pub async fn run(&mut self) -> Result<(), Error> {
        loop {
            // First, request all players states.
            for player in self.players_mut() {
                // TODO: player.request().await?;
            }

            // Then, inform all players about the others states.
            for player in self.players_mut() {
                // TODO: player.inform(&self.players).await?;
            }
        }
    }
}
