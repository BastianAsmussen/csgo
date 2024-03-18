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

    pub async fn inform(&mut self, others: &[Self]) -> Result<(), Error> {
        let json = serde_json::to_string(&self)?;
        for other in others {
            if other.id() == self.id() {
                continue;
            }

            // other.socket().write_all(json.as_bytes()).await?;
        }

        Ok(())
    }
}
