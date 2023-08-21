use std::fs::File;

use anyhow::Context;
use cxl::sys;

fn main() -> anyhow::Result<()> {
    let cxl = File::options()
        .read(true)
        .write(true)
        .open("/dev/cxl_ivpci0")
        .context("Failed to open /dev/cxl_ivpci0: is the cxl_ivpci.ko module inserted?")?;
    eprintln!("Opened CXL device");

    sys::cxl_init_meta(&cxl).context("Failed to call cxl_init_meta ioctl")?;
    eprintln!("Initialized CXL metadata");

    Ok(())
}
