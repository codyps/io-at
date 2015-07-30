// TODO: we don't need most of nix, pull it out and kill a dep
extern crate nix;
extern crate libc; // just for offs_t

#[cfg(test)]
extern crate tempfile;

pub use std::io::Result;

use std::ops::{Deref, DerefMut};
use std::convert::From;
use std::sync::Mutex;
use std::io::{SeekFrom, Seek, Read, Write};

pub trait ReadAt {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize>;
}

pub trait WriteAt {
    fn write_at(&self, buf: &[u8], offs: u64) -> Result<usize>;

    /* XXX: this impl is very similar to Write::write_all, is there some way to generalize? */
    fn write_all_at(&self, mut buf: &[u8], mut offs: u64) -> Result<()> {
        use std::io::{Error, ErrorKind};
        while !buf.is_empty() {
            match self.write_at(buf, offs) {
                Ok(0) => return Err(Error::new(ErrorKind::WriteZero,
                                               "failed to write whole buffer")),
                Ok(n) => {
                    buf = &buf[n..];
                    offs += n as u64;
                },
                Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }
}


/*
 * Adaptors:
 */
/*
pub struct<T: WriteAt> RateLimitWrite<T> {
    ts : u64;
    bytes_per_sec : u64;
    inner: T;
}

pub struct<T> Take<T> {
    max_offs: u64;
    inner: T;
}

*/

pub struct BlockLimitWrite<T: WriteAt> {
    max_per_block: usize,
    inner: T,
}

impl<T: WriteAt> BlockLimitWrite<T> {
    pub fn new(v: T, max_per_block: usize) -> Self {
        BlockLimitWrite { max_per_block: max_per_block, inner: v }
    }
}

impl<T: WriteAt + ReadAt> ReadAt for BlockLimitWrite<T> {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        self.inner.read_at(buf, offs)
    }
}

impl<T: WriteAt> WriteAt for BlockLimitWrite<T> {
    fn write_at(&self, buf: &[u8], offs: u64) -> Result<usize> {
        self.inner.write_at(&buf[..std::cmp::min(buf.len(), self.max_per_block)], offs)
    }
}

impl<T: WriteAt> Deref for BlockLimitWrite<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        &self.inner
    }
}

impl<T: WriteAt> DerefMut for BlockLimitWrite<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.inner
    }
}


pub struct LockedSeek<T: Seek> {
    inner: Mutex<T>,
}

/* FIXME: only allow for if T is also Read || Write */
impl<T: Seek> From<T> for LockedSeek<T> {
    fn from(v: T) -> Self {
        LockedSeek { inner: Mutex::new(v) }
    }
}

impl<T: Seek + Read> ReadAt for LockedSeek<T> {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        let mut f = self.inner.lock().unwrap();
        try!(f.seek(SeekFrom::Start(offs)));
        f.read(buf)
    }
}

impl<T: Seek + Write> WriteAt for LockedSeek<T> {
    fn write_at(&self, buf: &[u8], offs: u64) -> Result<usize> {
        let mut f = self.inner.lock().unwrap();
        try!(f.seek(SeekFrom::Start(offs)));
        f.write(buf)
    }
}

#[test]
fn do_t_locked_seek() {
    use tempfile;
    let f = tempfile::TempFile::new().unwrap();
    let at = LockedSeek::from(f);
    test_impl(at);
}

#[test]
fn do_t_block_limit() {
    use tempfile;
    let f = tempfile::TempFile::new().unwrap();
    let at = BlockLimitWrite::new(LockedSeek::from(f), 2);
    test_impl(at);

    let f = tempfile::TempFile::new().unwrap();
    let at = BlockLimitWrite::new(LockedSeek::from(f), 2);
    assert_eq!(at.write_at(&[1u8, 2, 3], 0).unwrap(), 2);
}

#[cfg(test)]
fn test_impl<T: ReadAt + WriteAt>(at: T) {
    let x = [1u8, 4, 9, 5];

    /* write at start */
    at.write_all_at(&x, 0).unwrap();
    let mut res = [0u8; 4];

    /* read at start */
    assert_eq!(at.read_at(&mut res, 0).unwrap(), 4);
    assert_eq!(&res, &x);

    /* read at middle */
    assert_eq!(at.read_at(&mut res, 1).unwrap(), 3);
    assert_eq!(&res[..3], &x[1..]);

    /* write at middle */
    at.write_all_at(&x, 1).unwrap();

    assert_eq!(at.read_at(&mut res, 0).unwrap(), 4);
    assert_eq!(&res, &[1u8, 1, 4, 9]);

    assert_eq!(at.read_at(&mut res, 4).unwrap(), 1);
    assert_eq!(&res[..1], &[5u8]);
}

pub mod os {
    pub mod unix {
        use nix;
        use libc;
        use super::super::*;
        use std::ops::{Deref, DerefMut};
        use std::io;
        use std::os::unix::io::AsRawFd;

        pub struct IoAtRaw<S: AsRawFd>(pub S);

        fn nix_to_io<T>(x: nix::Result<T>) -> io::Result<T> {
            x.map_err(|v| match v {
                nix::Error::Sys(errno) => io::Error::from_raw_os_error(errno as i32),
                nix::Error::InvalidPath => io::Error::new(io::ErrorKind::InvalidInput, "InvalidPath"),
            })
        }

        impl<S: AsRawFd> ReadAt for IoAtRaw<S> {
            fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
                nix_to_io(nix::sys::uio::pread(self.0.as_raw_fd(), buf, offs as libc::off_t))
            }
        }

        impl<S: AsRawFd> WriteAt for IoAtRaw<S> {
            fn write_at(&self, buf: &[u8], offs: u64) -> Result<usize> {
                nix_to_io(nix::sys::uio::pwrite(self.0.as_raw_fd(), buf, offs as libc::off_t))
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
            let f = tempfile::TempFile::new().unwrap();
            let at = IoAtRaw(f);
            super::super::test_impl(at);
        }
    }
}
