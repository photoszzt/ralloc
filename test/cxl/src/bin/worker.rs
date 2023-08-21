use std::collections::HashMap;
use std::ffi;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use clap::Parser;
use cxl::rpc;
use cxl::sys;
use rand::distributions::Distribution as _;
use rand::distributions::Uniform;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;

#[derive(Parser)]
struct Options {
    #[arg(short, long)]
    address: SocketAddr,

    #[arg(short, long)]
    seed: u64,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let options = Options::parse();
    let listener = TcpListener::bind(options.address)?;
    let mut rng = Xoshiro256StarStar::seed_from_u64(options.seed);

    let mut addresses = HashMap::new();
    let mut allocations = 0;

    loop {
        let (stream, address) = listener.accept()?;
        log::info!("Accepted connection from {}", address);

        let mut connection = rpc::Connection::new(stream);

        loop {
            for command in connection.receive()? {
                log::info!("{:?}", command);

                match command {
                    rpc::Command::Crash { delay, random } => {
                        if delay == 0 {
                            std::process::abort()
                        }

                        let duration = match random {
                            false => delay,
                            true => Uniform::from(0..=delay).sample(&mut rng),
                        };

                        log::info!("Crashing in {:.3}s...", duration as f64 / 1e9);

                        std::thread::spawn(move || {
                            thread::sleep(Duration::from_nanos(duration));
                            std::process::abort()
                        });
                    }
                    rpc::Command::Init { id, size } => {
                        let id_ = ffi::CString::new(id.clone())
                            .expect("Coordinator sent null byte in path");
                        match unsafe { sys::RP_init(id_.as_ptr(), size) } {
                            0 => log::info!("Initializing {}: no restart", id),
                            1 => log::info!("Initializing {}: restarted!", id),
                            _ => unreachable!(),
                        };
                    }
                    rpc::Command::Malloc { size } => {
                        let address = unsafe { sys::RP_malloc(size) };
                        let index = allocations;

                        addresses.insert(index, address);
                        allocations += 1;
                    }
                    rpc::Command::Free { index } => {
                        unsafe { sys::RP_free(addresses[&index]) }
                        addresses.remove(&index);
                    }
                    rpc::Command::Exit => {
                        log::info!("Exiting cleanly...");
                        unsafe {
                            sys::RP_close();
                        }
                        std::process::exit(0);
                    }
                }
            }
        }
    }
}
