use std::ffi;
use std::net::SocketAddr;
use std::net::TcpListener;

use anyhow::Context as _;
use clap::Parser;
use cxl::rpc;
use cxl::sys;

#[derive(Parser)]
struct Options {
    #[arg(short, long)]
    address: SocketAddr,
}

fn main() -> anyhow::Result<()> {
    let options = Options::parse();

    let listener = TcpListener::bind(options.address)?;

    loop {
        let (stream, address) = listener.accept()?;
        log::info!("Accepted connection from {}", address);

        let mut connection = rpc::Connection::new(stream);
        let mut buffer = Vec::new();

        loop {
            let responses = connection.receive()?.into_iter().map(|command| {
                log::info!("{:?}", command);

                match command {
                    rpc::Command::Crash => panic!(),
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
                        let address = unsafe { sys::RP_malloc(size) } as usize;
                        rpc::Response::Malloc { address }
                    }
                    rpc::Command::Free { address } => {
                        unsafe { sys::RP_free(address as *mut ffi::c_void) }
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
