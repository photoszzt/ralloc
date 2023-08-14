pub mod rpc;
pub mod sys;

use std::net::TcpStream;
use std::path::Path;
use std::process;
use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Context as _;

pub struct Worker {
    _handle: process::Child,
    connection: rpc::Connection,
}

impl Worker {
    pub fn local(path: &Path, port: u16, seed: u64) -> anyhow::Result<Self> {
        let address = format!("localhost:{port}");
        let handle = process::Command::new(path)
            .args(["--address", &address, "--seed", &seed.to_string()])
            .spawn()?;

        // TODO: use more robust mechanism?
        thread::sleep(Duration::from_secs(1));

        let connection = TcpStream::connect(&address)
            .with_context(|| anyhow!("Failed to connect to {}", address))
            .map(rpc::Connection::new)?;

        Ok(Worker {
            _handle: handle,
            connection,
        })
    }

    pub fn send(&mut self, command: &[rpc::Command]) -> anyhow::Result<()> {
        self.connection.send(command)
    }
}
