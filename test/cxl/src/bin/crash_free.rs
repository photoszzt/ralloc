use std::path::PathBuf;

use clap::Parser;
use cxl::rpc;
use cxl::Coordinator;
use rand::distributions::Uniform;
use rand::prelude::Distribution as _;
use rand::seq::SliceRandom as _;
use rand::SeedableRng as _;
use rand_xoshiro::Xoshiro256StarStar;
use rayon::prelude::IndexedParallelIterator;
use rayon::prelude::IntoParallelIterator;
use rayon::prelude::ParallelIterator;

#[derive(Parser)]
struct Command {
    /// Path to worker binary
    #[arg(short, long, default_value = "target/release/worker")]
    path: PathBuf,

    /// Port range to use for local workers
    #[arg(long, default_value = "10100")]
    port: u16,

    /// Number of worker processes
    #[arg(short, long)]
    workers: usize,

    #[arg(long, default_value = "35092")]
    seed: u64,

    /// Heap size (defaults to 7GiB + 64KiB)
    #[arg(short, long, default_value = "7516258304")]
    size: u64,

    /// Number of malloc/free operations in a batch
    #[arg(short, long, default_value = "100")]
    batch: usize,

    /// Number of batches to issue
    #[arg(short, long, default_value = "100")]
    rounds: usize,
}

fn main() -> anyhow::Result<()> {
    let command = Command::parse();

    let mut coordinator = Coordinator::new(&command.path, command.workers as u8)?;

    let uniform = Uniform::new_inclusive(1, 8193);
    let initialize = [rpc::Command::Init {
        id: String::from("cf"),
        size: command.size,
    }];

    for worker in coordinator.workers() {
        worker.send(&initialize)?;
    }

    coordinator
        .workers()
        .into_par_iter()
        .enumerate()
        .try_for_each(|(id, worker)| -> anyhow::Result<()> {
            let mut workload = vec![rpc::Command::Malloc { size: 0 }; command.batch * 2];
            let mut rng = Xoshiro256StarStar::seed_from_u64(command.seed + id as u64);
            let mut shuffle = (0..command.batch).collect::<Vec<_>>();

            for round in 0..command.rounds {
                for malloc in &mut workload[..command.batch] {
                    *malloc = rpc::Command::Malloc {
                        size: uniform.sample(&mut rng),
                    };
                }

                shuffle.shuffle(&mut rng);

                for (free, index) in workload[command.batch..].iter_mut().zip(&shuffle) {
                    *free = rpc::Command::Free {
                        index: index + (round * command.batch),
                    };
                }

                worker.send(&workload)?;
            }

            Ok(())
        })?;

    Ok(())
}
