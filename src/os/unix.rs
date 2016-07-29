#![cfg(unix)]
extern crate libc;
use super::super::*;
use std::ops::{Deref, DerefMut};
use std::io;
use std::os::unix::io::AsRawFd;

mod ffi {
    use super::libc;
    extern "C" {
        pub fn pread(fd: libc::c_int, buf: *mut libc::c_void, len: libc::size_t, offs: libc::off_t) -> libc::ssize_t;
        pub fn pwrite(fd: libc::c_int, buf: *const libc::c_void, len: libc::size_t, offs: libc::off_t) -> libc::ssize_t;
    }
}

/* ideally this would generalize over any return value we can ask "is non-negative", but
 * there isn't a built in trait for that and defining it ourselves would be work
 */
fn into_io_result(r: libc::ssize_t) -> Result<usize>
{
    if r >= 0 {
        Ok(r as usize)
    } else {
        Err(io::Error::last_os_error())
    }
}

fn pread<F: AsRawFd>(fd: &F, buf: &mut [u8], offs: libc::off_t) -> Result<usize>
{
    into_io_result(unsafe { ffi::pread(fd.as_raw_fd(), buf.as_mut_ptr() as *mut _, buf.len(), offs) })
}

fn pwrite<F: AsRawFd>(fd: &F, buf: &[u8], offs: libc::off_t) -> Result<usize>
{
    into_io_result(unsafe { ffi::pwrite(fd.as_raw_fd(), buf.as_ptr() as *const _, buf.len(), offs) })
}


#[derive(Debug, Eq, PartialEq)]
pub struct IoAtRaw<S: AsRawFd>(S);
impl<S: AsRawFd> From<S> for IoAtRaw<S> {
    fn from(v: S) -> Self {
        IoAtRaw(v)
    }
}

impl<S: AsRawFd> ReadAt for IoAtRaw<S> {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        pread(&self.0, buf, offs as libc::off_t)
    }
}

impl<S: AsRawFd> WriteAt for IoAtRaw<S> {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        pwrite(&self.0, buf, offs as libc::off_t)
    }
}

impl<T: AsRawFd> Deref for IoAtRaw<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        &self.0
    }
}

impl<T: AsRawFd> DerefMut for IoAtRaw<T> {
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
