use std::fs::File;
use std::ptr;

use anyhow::Context;
use cxl::sys;

fn main() -> anyhow::Result<()> {
    let cxl = File::options()
        .read(true)
        .write(true)
        .open("/dev/cxl_ivpci0")
        .context("Failed to open /dev/cxl_ivpci0: is the cxl_ivpci.ko module inserted?")?;
    eprintln!("Opened CXL device");

    sys::cxl_recover_meta(&cxl).context("Failed to call cxl_init_meta ioctl")?;
    eprintln!("Recovered CXL metadata");

    let id = std::ffi::CString::new("sm").unwrap();
    unsafe {
        // ralloc requires slightly more memory (64KiB) for its own metadata?
        sys::RP_init(id.as_ptr(), 2u64.pow(30) + 64 * 2u64.pow(10));
        eprintln!("Initialized ralloc");

        const SIZE: usize = 8000;

        let pointer = sys::RP_malloc(SIZE).cast::<u8>();
        assert_ne!(pointer, ptr::null_mut());
        eprintln!("Allocated {} bytes at {:x?}", SIZE, pointer);

        for offset in 0..SIZE {
            *pointer.add(offset) = offset as u8;
        }
        for offset in 0..SIZE {
            assert_eq!(*pointer.add(offset), offset as u8);
        }
        eprintln!("Verified {} bytes", SIZE);

        sys::RP_free(pointer.cast());
        eprintln!("Freed {:x?}", pointer);
    }

    Ok(())
}
