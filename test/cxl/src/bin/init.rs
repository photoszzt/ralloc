use std::ffi::CString;
use std::fs::File;
use std::os::fd::AsRawFd as _;
use std::path::PathBuf;
use std::ptr;

use anyhow::anyhow;
use anyhow::Context;
use clap::Parser;
use cxl::sys;

#[derive(Parser)]
struct Command {
    #[arg(short, long, default_value = "/dev/cxl_ivpci0")]
    path: PathBuf,

    #[arg(long, default_value = "7516258304")]
    heap_size: u64,

    #[arg(long, default_value = "17179869184")]
    zero_size: usize,

    #[arg(short, long)]
    zero: bool,
}

fn main() -> anyhow::Result<()> {
    let command = Command::parse();
    let cxl = File::options()
        .read(true)
        .write(true)
        .open(&command.path)
        .with_context(|| {
            anyhow!(
                "Failed to open {}: is the cxl_ivpci.ko module inserted?",
                command.path.display()
            )
        })?;
    eprintln!("Opened CXL device");

    if command.zero {
        unsafe {
            let cxl = libc::mmap64(
                ptr::null_mut(),
                command.zero_size,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED,
                cxl.as_raw_fd(),
                0,
            );
            eprintln!("Mapped CXL device at {:x?}", cxl);

            if cxl == libc::MAP_FAILED {
                panic!("Failed to mmap CXL device");
            }

            libc::memset(cxl, 0, command.zero_size);
            eprintln!("Wrote {} bytes to CXL", command.zero_size);
            libc::munmap(cxl, command.zero_size);
        }
    }

    sys::cxl_init_meta(&cxl).context("Failed to call cxl_init_meta ioctl")?;
    eprintln!("Initialized CXL metadata");

    let id = CString::new("cf").unwrap();
    unsafe {
        sys::RP_init(id.as_ptr(), command.heap_size, 0, 0);
    }
    eprintln!("Initialized ralloc metadata");

    Ok(())
}
