pub mod rpc;
pub mod sys;

use std::io;
use std::net::TcpStream;
use std::path::Path;
use std::process;
use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Context as _;

pub struct Worker {
    handle: process::Child,
    connection: rpc::Connection,
}

impl Worker {
    pub fn local(path: &Path, port: u16) -> anyhow::Result<Self> {
        let address = format!("localhost:{port}");
        let handle = process::Command::new(path)
            .arg("--address")
            .arg(&address)
            .spawn()?;

        // TODO: use more robust mechanism?
        thread::sleep(Duration::from_secs(1));

        let connection = TcpStream::connect(&address)
            .with_context(|| anyhow!("Failed to connect to {}", address))
            .map(rpc::Connection::new)?;

        Ok(Worker { handle, connection })
    }

    pub fn send(&mut self, command: &[rpc::Command]) -> anyhow::Result<Vec<rpc::Response>> {
        self.connection.send(command)
    }
}
