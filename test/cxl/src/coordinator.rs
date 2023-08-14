use std::path::Path;
use std::thread;

use crate::rpc;
use crate::Worker;

pub struct Coordinator {
    workers: Vec<Worker>,
}

impl Coordinator {
    pub fn new(seed: u64, path: &Path, workers: usize) -> anyhow::Result<Self> {
        let workers = (0..workers)
            .map(|id| Worker::local(path, 10100 + id as u16, seed + id as u64 + 1))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { workers })
    }

    pub fn send<II, I>(&mut self, workloads: II)
    where
        II: IntoIterator<Item = I>,
        I: AsRef<[rpc::Command]> + Send,
    {
        thread::scope(|scope| {
            let handles = self
                .workers
                .iter_mut()
                .zip(workloads)
                .map(|(worker, workload)| scope.spawn(move || worker.send(workload.as_ref())))
                .collect::<Vec<_>>();

            handles
                .into_iter()
                .map(|handle| handle.join())
                .collect::<Result<Vec<_>, _>>()
        })
        .expect("Failed to collect worker responses");
    }
}
