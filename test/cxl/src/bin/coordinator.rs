use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Context as _;
use clap::Parser;
use cxl::rpc;
use cxl::Coordinator;
use rand::distributions::Uniform;
use rand::prelude::Distribution as _;
use rand::seq::SliceRandom as _;
use rand::SeedableRng as _;
use rand_xoshiro::Xoshiro256StarStar;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::ParallelIterator;

#[derive(Parser)]
struct Command {
    /// Path to worker binary
    #[arg(short, long, default_value = "target/release/worker")]
    path: PathBuf,

    /// Number of worker processes
    #[arg(short, long)]
    workers: usize,

    #[arg(long, default_value = "35092")]
    seed: u64,

    /// Heap id
    #[arg(short, long, default_value = "rs")]
    heap_id: String,

    /// Heap size (defaults to 7GiB + 64KiB)
    #[arg(short, long, default_value = "7516258304")]
    heap_size: u64,

    /// Number of malloc/free operations in a batch
    #[arg(short, long, default_value = "100")]
    batch: usize,

    /// Number of batches to issue
    #[arg(short, long, default_value = "100")]
    rounds: usize,

    #[command(subcommand)]
    crash: Crash,
}

#[derive(Parser)]
enum Crash {
    Zero,
    One {
        #[arg(short, long, default_value = "0")]
        id: u8,

        #[arg(short, long)]
        restart: bool,
    },
}

fn main() -> anyhow::Result<()> {
    let command = Command::parse();

    let mut coordinator = Coordinator::new(&command.path, command.workers as u8)?;

    let initialize = [rpc::Command::Init {
        id: command.heap_id.clone(),
        size: command.heap_size,
    }];

    for worker in coordinator.workers() {
        worker.send(&initialize)?;
    }

    match &command.crash {
        Crash::Zero => coordinator
            .workers()
            .into_par_iter()
            .try_for_each(|worker| command.crash_free(worker))?,
        Crash::One { id, restart } => {
            coordinator
                .workers()
                .into_par_iter()
                .try_for_each(|worker| {
                    if worker.id() == *id {
                        command.crash(worker, *restart)
                    } else {
                        command.crash_free(worker)
                    }
                })?
        }
    }

    Ok(())
}

impl Command {
    fn crash_free(&self, worker: &mut cxl::Worker) -> anyhow::Result<()> {
        let mut workload = Workload::new(self.batch, self.seed);

        for _ in 0..self.rounds {
            worker
                .send(workload.next())
                .with_context(|| anyhow!("Failed to send batch to worker {}", worker.id()))?;
        }

        Ok(())
    }

    fn crash(&self, worker: &mut cxl::Worker, restart: bool) -> anyhow::Result<()> {
        let mut workload = Workload::new(self.batch, self.seed);
        let crash = Uniform::new(0, self.rounds).sample(&mut workload.rng);

        for _ in 0..crash {
            worker
                .send(workload.next())
                .with_context(|| anyhow!("Failed to send batch to worker {}", worker.id()))?;
        }

        worker.send(&[rpc::Command::Crash {
            delay: 0,
            random: false,
        }])?;

        if !restart {
            return Ok(());
        }

        worker.restart()?;

        worker.send(&[
            rpc::Command::Init {
                id: self.heap_id.clone(),
                size: self.heap_size,
            },
            rpc::Command::Recover,
        ])?;

        for _ in crash..self.rounds {
            worker
                .send(workload.next())
                .with_context(|| anyhow!("Failed to send batch to worker {}", worker.id()))?;
        }

        Ok(())
    }
}

struct Workload {
    buffer: Vec<rpc::Command>,
    rng: Xoshiro256StarStar,
    round: usize,
    shuffle: Vec<usize>,
    sizes: Uniform<usize>,
}

impl Workload {
    fn new(batch: usize, seed: u64) -> Self {
        let rng = Xoshiro256StarStar::seed_from_u64(seed);
        let shuffle = (0..batch).collect::<Vec<_>>();
        let sizes = Uniform::new_inclusive(1, 8193);
        Self {
            buffer: vec![rpc::Command::Exit; batch * 2],
            rng,
            round: 0,
            shuffle,
            sizes,
        }
    }

    fn next(&mut self) -> &[rpc::Command] {
        let batch = self.buffer.len() / 2;

        for malloc in &mut self.buffer[..batch] {
            *malloc = rpc::Command::Malloc {
                size: self.sizes.sample(&mut self.rng),
            };
        }

        self.shuffle.shuffle(&mut self.rng);

        for (free, index) in self.buffer[batch..].iter_mut().zip(&self.shuffle) {
            *free = rpc::Command::Free {
                index: index + (self.round * batch),
            };
        }

        self.round += 1;
        &self.buffer
    }
}
