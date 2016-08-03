extern crate libc;
extern crate posix_sys;
use super::super::*;
use std::io;
pub use std::os::unix::io::AsRawFd as AsRaw;

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

pub fn pread<F: AsRaw>(fd: &F, buf: &mut [u8], offs: u64) -> Result<usize>
{
    into_io_result(unsafe { posix_sys::pread(fd.as_raw_fd(), buf.as_mut_ptr() as *mut _, buf.len(), offs as libc::off_t) })
}

pub fn pwrite<F: AsRaw>(fd: &F, buf: &[u8], offs: u64) -> Result<usize>
{
    into_io_result(unsafe { posix_sys::pwrite(fd.as_raw_fd(), buf.as_ptr() as *const _, buf.len(), offs as libc::off_t) })
}
