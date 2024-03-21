use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

use crate::player::position::Position;

use crate::Error;

pub mod position;

#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    id: Option<usize>,
    #[serde(skip)]
    socket: Option<TcpStream>,

    health: f64,
    position: Position,
}

impl Player {
    pub async fn new(id: usize, mut socket: TcpStream) -> Result<Self, Error> {
        let mut buffer = [0; 1024];
        socket.read(&mut buffer).await?;

        let size = buffer.iter().position(|&x| x == 0).unwrap_or(buffer.len());

        let player = serde_json::from_slice(&buffer[..size])?;

        Ok(Self {
            id: Some(id),
            socket: Some(socket),
            ..player
        })
    }

    pub fn id(&self) -> usize {
        self.id.unwrap()
    }

    pub fn socket(&self) -> &TcpStream {
        self.socket.as_ref().unwrap()
    }

    pub fn socket_mut(&mut self) -> &mut TcpStream {
        self.socket.as_mut().unwrap()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }

    pub fn state(&self) -> Result<String, Error> {
        serde_json::to_string(self).map_err(Into::into)
    }

    pub async fn request(&mut self) -> Result<(), Error> {
        // Ask the player for their new position.
        self.socket_mut().write_all(b"GetState").await?;

        let mut buffer = [0; 1024];
        let size = self.socket_mut().read(&mut buffer).await?;

        // Update the player state.
        let state: Self = serde_json::from_slice(&buffer[..size])?;
        *self = Self {
            id: self.id,
            socket: self.socket.take(),
            ..state
        };

        Ok(())
    }

    pub async fn inform(&mut self, data: &[u8]) -> Result<(), Error> {
        self.socket_mut().write_all(data).await?;

        Ok(())
    }
}
