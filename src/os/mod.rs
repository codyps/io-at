use super::*;
use std::ops::{Deref, DerefMut};

#[cfg(unix)]
pub mod unix;
#[cfg(windows)]
pub mod windows;

/* Expose the semi-compatible IoAtRaw types via the same name */
#[cfg(unix)]
use self::unix::{AsRaw,pread,pwrite};
#[cfg(windows)]
use self::windows::{AsRaw,pread,pwrite};

#[derive(Debug, Eq, PartialEq)]
pub struct IoAtRaw<S: AsRaw>(S);
impl<S: AsRaw> From<S> for IoAtRaw<S> {
    fn from(v: S) -> Self {
        IoAtRaw(v)
    }
}

impl<S: AsRaw> ReadAt for IoAtRaw<S> {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        pread(&self.0, buf, offs)
    }
}

impl<S: AsRaw> WriteAt for IoAtRaw<S> {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        pwrite(&self.0, buf, offs)
    }
}

impl<T: AsRaw> Deref for IoAtRaw<T> {
    type Target = T;
    fn deref<'a>(&'a self) -> &'a T {
        &self.0
    }
}

impl<T: AsRaw> DerefMut for IoAtRaw<T> {
    fn deref_mut<'a>(&'a mut self) -> &'a mut T {
        &mut self.0
    }
}

#[test]
fn do_t() {
    use tempfile;
    let f = tempfile::tempfile().unwrap();
    let at = IoAtRaw::from(f);
    super::test_impl(at);
}
