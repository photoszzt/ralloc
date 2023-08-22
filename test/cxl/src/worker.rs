use std::net::TcpListener;
use std::path::Path;
use std::process;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::rpc;

pub struct Worker {
    id: u8,
    handle: process::Child,
    connection: rpc::Connection,
}

impl Worker {
    pub fn local(
        id: u8,
        count: u8,
        path: &Path,
        listener: &mut TcpListener,
    ) -> anyhow::Result<Self> {
        let address = listener
            .local_addr()
            .context("[C]: failed to get local address")?;

        let handle = process::Command::new(path)
            .args([
                "--process-id",
                &id.to_string(),
                "--process-count",
                &count.to_string(),
                "--address",
                &address.to_string(),
            ])
            .spawn()?;

        let (stream, address) = listener
            .accept()
            .context("[C]: failed to accept connection")?;

        log::info!("[C]: connected to {}", address);

        Ok(Worker {
            id,
            handle,
            connection: rpc::Connection::new(stream),
        })
    }

    pub fn send(&mut self, command: &[rpc::Command]) -> anyhow::Result<()> {
        self.connection.send(command)
    }

    pub fn wait(mut self) -> anyhow::Result<()> {
        match self
            .handle
            .wait()
            .with_context(|| anyhow!("Failed to wait on worker {}", self.id))?
        {
            status if status.success() => Ok(()),
            status => Err(anyhow!("Worker {} failed with status: {}", self.id, status)),
        }
    }
}
