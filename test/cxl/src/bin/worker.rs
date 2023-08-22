use std::collections::HashMap;
use std::ffi;
use std::net::SocketAddr;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

use anyhow::anyhow;
use anyhow::Context as _;
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
    id: usize,

    #[arg(short, long)]
    address: SocketAddr,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let options = Options::parse();
    let id = options.id;

    let mut rng = Xoshiro256StarStar::seed_from_u64(options.id as u64);
    let mut addresses = HashMap::new();
    let mut allocations = 0;
    let mut connection = TcpStream::connect(options.address)
        .map(rpc::Connection::new)
        .with_context(|| anyhow!("[{}]: failed to connect to {}", id, options.address))?;

    log::info!("[{}]: connected to {}", id, options.address);

    loop {
        for command in connection.receive()? {
            log::trace!("[{}]: received {:?}", id, command);

            match command {
                rpc::Command::Crash { delay, random } => {
                    if delay == 0 {
                        log::info!("[{}]: crashing now!", id);
                        std::process::abort()
                    }

                    let duration = match random {
                        false => delay,
                        true => Uniform::from(0..=delay).sample(&mut rng),
                    };

                    log::info!("[{}]: crashing in {:.3}s...", id, duration as f64 / 1e9);

                    std::thread::spawn(move || {
                        thread::sleep(Duration::from_nanos(duration));
                        std::process::abort()
                    });
                }
                rpc::Command::Init { id: heap_id, size } => {
                    let heap_id_ = ffi::CString::new(heap_id.clone())
                        .expect("Coordinator sent null byte in path");

                    let restart = match unsafe { sys::RP_init(heap_id_.as_ptr(), size) } {
                        0 => false,
                        1 => true,
                        _ => unreachable!(),
                    };

                    log::info!("[{}]: initializing {}, restart: {}", id, heap_id, restart);
                }
                rpc::Command::Malloc { size } => {
                    let address = unsafe {
                        let address = sys::RP_malloc(size);
                        let fill = size as u8;
                        libc::memset(address, fill as i32, size);
                        for offset in 0..size {
                            assert_eq!(address.cast::<u8>().add(offset).read(), fill);
                        }
                        address
                    };

                    let index = allocations;
                    addresses.insert(index, address);
                    allocations += 1;
                }
                rpc::Command::Free { index } => {
                    unsafe { sys::RP_free(addresses[&index]) }
                    addresses.remove(&index);
                }
                rpc::Command::Exit => {
                    log::info!("[{}]: exiting cleanly...", id);
                    unsafe {
                        sys::RP_close();
                    }
                    std::process::exit(0);
                }
            }
        }
    }
}
