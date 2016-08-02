extern crate winapi;
extern crate kernel32;
use super::super::*;
use self::winapi::{OVERLAPPED,DWORD};
use std::{mem,ptr,io};
pub use std::os::windows::io::AsRawHandle as AsRaw;

fn ov(offs: u64) -> OVERLAPPED
{
    OVERLAPPED {
        Internal: 0,
        InternalHigh: 0,
        Offset: offs as DWORD,
        OffsetHigh: (offs >> (mem::size_of::<DWORD>() * 8)) as DWORD,
        hEvent: ptr::null_mut(),
    }
}

pub fn pread<F: AsRaw>(fd: &F, buf: &mut [u8], offs: u64) -> Result<usize>
{
    let mut lr = 0 as DWORD;
    let mut ovl = ov(offs);
    let r = unsafe {kernel32::ReadFile(fd.as_raw_handle(), buf.as_mut_ptr() as *mut _, buf.len() as DWORD, &mut lr, &mut ovl) };
    if r != 0 {
        Ok(lr as usize)
    } else {
        Err(io::Error::last_os_error())
    }
}

pub fn pwrite<F: AsRaw>(fd: &F, buf: &[u8], offs: u64) -> Result<usize>
{
    let mut lr = 0 as DWORD;
    let mut ovl = ov(offs);
    let r = unsafe {kernel32::WriteFile(fd.as_raw_handle(), buf.as_ptr() as *const _, buf.len() as DWORD, &mut lr, &mut ovl) };
    if r != 0 {
        Ok(lr as usize)
    } else {
        Err(io::Error::last_os_error())
    }
}
