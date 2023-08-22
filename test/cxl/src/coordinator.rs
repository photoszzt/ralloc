use std::net::Ipv4Addr;
use std::net::SocketAddrV4;
use std::net::TcpListener;
use std::path::Path;

use anyhow::Context as _;

use crate::rpc;
use crate::Worker;

pub struct Coordinator {
    _listener: TcpListener,
    workers: Vec<Worker>,
}

impl Coordinator {
    pub fn new(worker: &Path, workers: u8) -> anyhow::Result<Self> {
        let mut listener = TcpListener::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
            .context("[C]: failed to bind to localhost:0")?;

        let workers = (0..workers)
            .map(|id| Worker::local(id, workers, worker, &mut listener))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            _listener: listener,
            workers,
        })
    }

    pub fn workers(&mut self) -> &mut [Worker] {
        &mut self.workers
    }
}

impl Drop for Coordinator {
    fn drop(&mut self) {
        for worker in &mut self.workers {
            worker
                .send(&[rpc::Command::Exit])
                .expect("[C]: failed to send exit command");
        }

        self.workers
            .drain(..)
            .try_for_each(Worker::wait)
            .expect("[C]: failed to wait on workers");
    }
}
