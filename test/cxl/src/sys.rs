#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::fs::File;
use std::io;
use std::os::fd::AsRawFd as _;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

const IOCTL_MAGIC: u32 = 0xf;

ioctl_sys::ioctl!(try none _cxl_init_meta with IOCTL_MAGIC, 9);

pub fn cxl_init_meta(cxl: &File) -> io::Result<()> {
    unsafe { _cxl_init_meta(cxl.as_raw_fd()) }
}

ioctl_sys::ioctl!(try none _cxl_recover_meta with IOCTL_MAGIC, 10);

pub fn cxl_recover_meta(cxl: &File) -> io::Result<()> {
    unsafe { _cxl_recover_meta(cxl.as_raw_fd()) }
}
