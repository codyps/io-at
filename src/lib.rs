#[cfg(test)]
extern crate tempfile;

pub use std::io::Result;

use std::ops::{Deref, DerefMut};
use std::convert::From;
use std::sync::Mutex;
use std::io::{SeekFrom, Seek, Read, Write};

/**
 * Read data at an offset
 *
 * Note that compared to Read::read, ReadAt::read_at does not borrow T mutably as it does not need
 * to modify an internal cursor.
 */
pub trait ReadAt {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize>;
}

/**
 * Write data at an offset
 */
pub trait WriteAt {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize>;

    /* XXX: this impl is very similar to Write::write_all, is there some way to generalize? */
    fn write_all_at(&mut self, mut buf: &[u8], mut offs: u64) -> Result<()> {
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
*/

/**
 * Adapt a WriteAt and/or ReadAt to a Write and/or Read by tracking a single internal offset and
 * modifying it.
 */
#[derive(Debug, Eq, PartialEq)]
pub struct Cursor<T> {
    offs: u64,
    inner: T,
}

impl<T> Cursor<T> {
    pub fn new(v: T, first_offs: u64) -> Self {
        Cursor { inner: v, offs: first_offs }
    }
}

impl<T: WriteAt> Write for Cursor<T> {
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let r = self.inner.write_at(buf, self.offs);
        if let Ok(v) = r {
            self.offs += v as u64;
        }
        r
    }

    fn flush(&mut self) -> Result<()> {
        Ok(())
    }
}

impl<T: ReadAt> Read for Cursor<T> {
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let r = self.inner.read_at(buf, self.offs);
        if let Ok(v) = r {
            self.offs += v as u64;
        }
        r
    }
}

#[test]
fn do_t_cursor() {
    use tempfile;
    let f = tempfile::tempfile().unwrap();
    let _at = Cursor::new(LockedSeek::from(f), 5);

    /* TODO: test it */
}

/**
 * Limit the maximum offset of a WriteAt and/or ReadAt.
 *
 * This can be useful when trying to simulate a fixed size item (like a block device) with a normal
 * file.
 */
#[derive(Debug, Eq, PartialEq)]
pub struct Take<T> {
    max_offs: u64,
    inner: T,
}

impl<T> Take<T> {
    pub fn new(v: T, max_offs: u64) -> Self {
        Take { max_offs : max_offs, inner: v }
    }

    pub fn len(&self) -> u64 {
        self.max_offs
    }
}

impl<T: ReadAt> ReadAt for Take<T> {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        let last = std::cmp::min(buf.len() as u64 + offs, self.max_offs);
        if offs > last {
            Ok(0)
        } else {
            self.inner.read_at(&mut buf[..(last - offs) as usize], offs)
        }
    }
}

impl<T: WriteAt> WriteAt for Take<T> {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        let last = std::cmp::min(buf.len() as u64 + offs, self.max_offs);
        if offs > last {
            Ok(0)
        } else {
            self.inner.write_at(&buf[..(last - offs) as usize], offs)
        }
    }
}

impl<T> Deref for Take<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        &self.inner
    }
}

impl<T> DerefMut for Take<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.inner
    }
}

#[test]
fn do_t_take() {
    use tempfile;
    let f = tempfile::tempfile().unwrap();
    let at = Take::new(LockedSeek::from(f), 5);
    test_impl(at);

    /* Partial write */
    let f = tempfile::tempfile().unwrap();
    let mut at = Take::new(LockedSeek::from(f), 5);
    assert_eq!(at.write_at(&[11u8, 2, 3, 4], 4).unwrap(), 1);

    /* Partial read */
    let mut res = [0u8; 4];
    assert_eq!(at.read_at(&mut res, 4).unwrap(), 1);
    assert_eq!(&res[..1], &[11]);

    /* At the end none write */
    assert_eq!(at.write_at(&[12u8, 13], 5).unwrap(), 0);

    /* At the end none read */
    assert_eq!(at.read_at(&mut res, 5).unwrap(), 0);

    /* Past the end none write */
    assert_eq!(at.write_at(&[12u8, 13], 6).unwrap(), 0);

    /* Past the end none read */
    assert_eq!(at.read_at(&mut res, 6).unwrap(), 0);
}

/**
 * Limit the amount of data accepted in a single call to WriteAt (ReadAt is unaffected)
 *
 * This is primarily useful for testing that handling of incomplete writes work properly.
 */
#[derive(Debug, Eq, PartialEq)]
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
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
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

#[test]
fn do_t_block_limit() {
    use tempfile;
    let f = tempfile::tempfile().unwrap();
    let at = BlockLimitWrite::new(LockedSeek::from(f), 2);
    test_impl(at);

    let f = tempfile::tempfile().unwrap();
    let mut at = BlockLimitWrite::new(LockedSeek::from(f), 2);
    assert_eq!(at.write_at(&[1u8, 2, 3], 0).unwrap(), 2);
}

/**
 * Allow a Seek + (Read and/or Write) to impliment WriteAt and/or ReadAt
 *
 * While in most cases more specific adaptations will be more efficient, this allows any Seek to
 * impliment WriteAt or ReadAt with not additional restrictions.
 */
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
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        let mut f = self.inner.lock().unwrap();
        try!(f.seek(SeekFrom::Start(offs)));
        f.write(buf)
    }
}

#[test]
fn do_t_locked_seek() {
    use tempfile;
    let f = tempfile::tempfile().unwrap();
    let at = LockedSeek::from(f);
    test_impl(at);
}

#[cfg(test)]
fn test_impl<T: ReadAt + WriteAt>(mut at: T) {
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

/**
 * OS specific implimentations of WriteAt and/or ReadAt
 */
pub mod os {
    #[cfg(unix)]
    pub mod unix;
    #[cfg(windows)]
    pub mod windows;
}
