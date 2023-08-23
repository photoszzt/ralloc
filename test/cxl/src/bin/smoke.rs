use std::fs::File;
use std::mem;
use std::ptr;
use std::sync::atomic::Ordering;

use anyhow::Context;
use common_rs::AtomicOffsetPointer;
use cxl::sys;

fn main() -> anyhow::Result<()> {
    let cxl = File::options()
        .read(true)
        .write(true)
        .open("/dev/cxl_ivpci0")
        .context("Failed to open /dev/cxl_ivpci0: is the cxl_ivpci.ko module inserted?")?;
    eprintln!("Opened CXL device");

    sys::cxl_recover_meta(&cxl).context("Failed to call cxl_recover_meta ioctl")?;
    eprintln!("Recovered CXL metadata");

    let id = std::ffi::CString::new("sm").unwrap();
    unsafe {
        // ralloc requires slightly more memory (64KiB) for its own metadata?
        sys::RP_init(id.as_ptr(), 2u64.pow(30) + 64 * 2u64.pow(10), 0, 1);
        eprintln!("Initialized ralloc");

        const SIZE: usize = 100;

        let head = sys::RP_malloc(mem::size_of::<Node>());
        sys::RP_set_root(head, 0);

        let mut prev = head.cast::<Node>();
        ptr::addr_of_mut!((*prev).next).write(AtomicOffsetPointer::null());
        ptr::addr_of_mut!((*prev).data).write(0);

        for index in 1..SIZE {
            let next = sys::RP_malloc(mem::size_of::<Node>()).cast::<Node>();
            ptr::addr_of_mut!((*next).next).write(AtomicOffsetPointer::null());
            ptr::addr_of_mut!((*next).data).write(index as u64);

            (*prev).next.store(next, Ordering::Release);
            prev = next;
        }

        sys::RP_recover();

        let mut prev = head.cast_const().cast::<Node>();
        for index in 0..SIZE {
            assert_eq!((*prev).data, index as u64);
            let next = (*prev).next.load(Ordering::Acquire);
            sys::RP_free(prev as _);
            prev = next;
        }

        sys::RP_close();
    }

    Ok(())
}

struct Node {
    next: AtomicOffsetPointer<Node>,
    data: u64,
}
