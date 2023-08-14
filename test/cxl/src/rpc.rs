use std::net::TcpStream;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Command {
    Crash { delay: u64, random: bool },
    Init { id: String, size: u64 },
    Malloc { size: usize },
    Free { index: usize },
}

pub struct Connection(TcpStream);

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self(stream)
    }

    pub fn send(&mut self, commands: &[Command]) -> anyhow::Result<()> {
        bincode::serialize_into(&mut self.0, commands)?;
        Ok(())
    }

    pub fn receive(&mut self) -> anyhow::Result<Vec<Command>> {
        bincode::deserialize_from(&mut self.0).map_err(anyhow::Error::from)
    }
}
