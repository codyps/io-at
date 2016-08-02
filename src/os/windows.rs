extern crate winapi;
extern crate kernel32;
use super::super::*;
use std::ops::{Deref,DerefMut};
use std::os::windows::io::AsRawHandle;
use self::winapi::{OVERLAPPED,DWORD};
use std::{mem,ptr,io};

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

fn pread<F: AsRawHandle>(fd: &F, buf: &mut [u8], offs: u64) -> Result<usize>
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

fn pwrite<F: AsRawHandle>(fd: &F, buf: &[u8], offs: u64) -> Result<usize>
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

#[derive(Debug, Eq, PartialEq)]
pub struct IoAtRaw<S: AsRawHandle>(S);
impl<S: AsRawHandle> From<S> for IoAtRaw<S> {
    fn from(v: S) -> Self {
        IoAtRaw(v)
    }
}

impl<S: AsRawHandle> ReadAt for IoAtRaw<S> {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        pread(&self.0, buf, offs)
    }
}

impl<S: AsRawHandle> WriteAt for IoAtRaw<S> {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        pwrite(&self.0, buf, offs)
    }
}

impl<T: AsRawHandle> Deref for IoAtRaw<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        &self.0
    }
}

impl<T: AsRawHandle> DerefMut for IoAtRaw<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.0
    }
}

#[test]
fn do_t() {
    use tempfile;
    let f = tempfile::tempfile().unwrap();
    let at = IoAtRaw::from(f);
    super::super::test_impl(at);
}
