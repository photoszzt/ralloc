use std::net::TcpListener;
use std::path::Path;
use std::path::PathBuf;
use std::process;
use std::sync::Arc;

use anyhow::anyhow;
use anyhow::Context as _;

use crate::rpc;

pub struct Worker {
    id: u8,
    count: u8,
    path: PathBuf,
    listener: Arc<TcpListener>,
    handle: process::Child,
    connection: rpc::Connection,
}

impl Worker {
    pub fn local(
        id: u8,
        count: u8,
        path: PathBuf,
        listener: Arc<TcpListener>,
    ) -> anyhow::Result<Self> {
        let (handle, connection) = spawn(id, count, &path, &listener)?;

        Ok(Worker {
            id,
            count,
            path,
            handle,
            listener,
            connection,
        })
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn send(&mut self, command: &[rpc::Command]) -> anyhow::Result<()> {
        self.connection.send(command)
    }

    pub fn wait(&mut self) -> anyhow::Result<()> {
        match self
            .handle
            .wait()
            .with_context(|| anyhow!("Failed to wait on worker {}", self.id))?
        {
            status if status.success() => Ok(()),
            status => Err(anyhow!("Worker {} failed with status: {}", self.id, status)),
        }
    }

    pub fn restart(&mut self) -> anyhow::Result<()> {
        self.wait()?;

        let (handle, connection) = spawn(self.id, self.count, &self.path, &self.listener)?;
        self.handle = handle;
        self.connection = connection;

        Ok(())
    }
}

fn spawn(
    id: u8,
    count: u8,
    path: &Path,
    listener: &TcpListener,
) -> anyhow::Result<(process::Child, rpc::Connection)> {
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

    Ok((handle, rpc::Connection::new(stream)))
}
