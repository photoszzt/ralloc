use std::path::PathBuf;

use clap::Parser;
use cxl::rpc;
use cxl::Coordinator;
use rand::distributions::Uniform;
use rand::prelude::Distribution as _;
use rand::seq::SliceRandom as _;
use rand::SeedableRng as _;
use rand_xoshiro::Xoshiro256StarStar;

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

    #[arg(short, long, default_value = "35092")]
    seed: u64,
}

fn main() -> anyhow::Result<()> {
    let command = Command::parse();
    let mut rng = Xoshiro256StarStar::seed_from_u64(command.seed);

    let mut coordinator = Coordinator::new(command.seed, &command.path, 2)?;

    let sizes = Uniform::new_inclusive(1, 8192);

    let mut workload = vec![rpc::Command::Init {
        id: String::from("cf"),
        size: 2u64.pow(30),
    }];

    workload.extend((0..100).map(|_| rpc::Command::Malloc {
        size: sizes.sample(&mut rng),
    }));

    let mut frees = (0..100).collect::<Vec<_>>();
    frees.shuffle(&mut rng);

    workload.extend(frees.into_iter().map(|index| rpc::Command::Free { index }));

    coordinator.send([&workload, &workload]);
    coordinator.wait()?;
    Ok(())
}
