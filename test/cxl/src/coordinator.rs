use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::net::TcpListener;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context as _;

use crate::rpc;
use crate::Worker;

pub struct Coordinator {
    workers: Vec<Worker>,
}

impl Coordinator {
    pub fn new(worker: &Path, workers: u8) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
            .map(Arc::new)
            .context("[C]: failed to bind to localhost:0")?;

        let workers = (0..workers)
            .map(|id| Worker::local(id, workers, worker.to_path_buf(), Arc::clone(&listener)))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { workers })
    }

    pub fn workers(&mut self) -> &mut [Worker] {
        &mut self.workers
    }
}

impl Drop for Coordinator {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            if let Err(error) = worker.send(&[rpc::Command::Exit]) {
                log::warn!(
                    "[C]: failed to send exit to worker {}: {:?}",
                    worker.id(),
                    error,
                );
            }
        }

        for worker in &mut self.workers {
            if let Err(error) = worker.wait() {
                log::warn!("[C]: failed to wait on worker {}: {:?}", worker.id(), error);
            }
        }
    }
}
