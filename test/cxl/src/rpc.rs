use std::net::TcpStream;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Crash,
    Init { id: String, size: u64 },
    Malloc { size: usize },
    Free { address: usize },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Response {
    Init { restart: bool },
    Malloc { address: usize },
    Free,
}

pub struct Connection(TcpStream);

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self(stream)
    }

    pub fn send(&mut self, commands: &[Command]) -> anyhow::Result<Response> {
        bincode::serialize_into(&mut self.0, commands)?;
        bincode::deserialize_from(&mut self.0).map_err(anyhow::Error::from)
    }

    pub fn receive(&mut self) -> anyhow::Result<Vec<Command>> {
        bincode::deserialize_from(&mut self.0).map_err(anyhow::Error::from)
    }

    pub fn respond(&mut self, responses: &[Response]) -> anyhow::Result<()> {
        bincode::serialize_into(&mut self.0, responses).map_err(anyhow::Error::from)
    }
}
