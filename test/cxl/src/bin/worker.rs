use std::collections::HashMap;
use std::ffi;
use std::net::SocketAddr;
use std::net::TcpListener;
use std::thread;
use std::time::Duration;

use anyhow::Context as _;
use clap::Parser;
use cxl::rpc;
use cxl::sys;
use rand::distributions::Distribution as _;
use rand::distributions::Uniform;

#[derive(Parser)]
struct Options {
    #[arg(short, long)]
    address: SocketAddr,
}

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let options = Options::parse();
    let listener = TcpListener::bind(options.address)?;

    let mut buffer = Vec::new();
    let mut addresses = HashMap::new();
    let mut allocations = 0;

    loop {
        let (stream, address) = listener.accept()?;
        log::info!("Accepted connection from {}", address);

        let mut connection = rpc::Connection::new(stream);

        loop {
            let responses = connection.receive()?.into_iter().map(|command| {
                log::info!("{:?}", command);

                match command {
                    rpc::Command::Crash { delay, random } => {
                        if delay == 0 {
                            std::process::abort()
                        }

                        let duration = match random {
                            false => delay,
                            true => Uniform::from(0..=delay).sample(&mut rand::thread_rng()),
                        };

                        log::info!("Crashing in {:.3}s...", duration as f64 / 1e9);

                        std::thread::spawn(move || {
                            thread::sleep(Duration::from_nanos(duration));
                            std::process::abort()
                        });

                        rpc::Response::Crash
                    }
                    rpc::Command::Init { id, size } => {
                        let id = ffi::CString::new(id).expect("Coordinator sent null byte in path");
                        let restart = match unsafe { sys::RP_init(id.as_ptr(), size) } {
                            0 => false,
                            1 => true,
                            _ => unreachable!(),
                        };

                        rpc::Response::Init { restart }
                    }
                    rpc::Command::Malloc { size } => {
                        let address = unsafe { sys::RP_malloc(size) };
                        let index = allocations;

                        addresses.insert(index, address);
                        allocations += 1;

                        rpc::Response::Malloc { index: allocations }
                    }
                    rpc::Command::Free { index } => {
                        unsafe { sys::RP_free(addresses[&index]) }
                        addresses.remove(&index);
                        rpc::Response::Free
                    }
                }
            });

            buffer.clear();
            buffer.extend(responses);
            connection
                .respond(&buffer)
                .context("Failed to send response")?;
        }
    }
}
