use std::path::Path;

use crate::rpc;
use crate::Worker;

pub struct Coordinator {
    workers: Vec<Worker>,
}

impl Coordinator {
    pub fn new(seed: u64, path: &Path, workers: u8) -> anyhow::Result<Self> {
        let workers = (0..workers)
            .map(|id| Worker::local(id, path, 10100 + id as u16, seed + id as u64 + 1))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { workers })
    }

    pub fn workers(&mut self) -> &mut [Worker] {
        &mut self.workers
    }

    pub fn wait(mut self) -> anyhow::Result<()> {
        for worker in &mut self.workers {
            worker.send(&[rpc::Command::Exit])?;
        }

        self.workers.into_iter().try_for_each(Worker::wait)
    }
}
