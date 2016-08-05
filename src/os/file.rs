use super::super::*;
use std::fs::File;
use super::{pread,pwrite};

impl ReadAt for File {
    #[inline]
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        pread(self, buf, offs)
    }
}

impl WriteAt for File {
    #[inline]
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        pwrite(self, buf, offs)
    }
}

/**
 * File can also be written via '&' (non-mutable reference) due to os intervention
 */
impl<'a> WriteAt for &'a File {
    #[inline]
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        pwrite(*self, buf, offs)
    }
}

#[cfg(test)]
mod test {
    use super::super::super::test_impl;
    #[test]
    fn base() {
        use tempfile;
        let f = tempfile::tempfile().unwrap();
        test_impl(f);
    }

    #[test]
    fn ref_() {
        use tempfile;
        let f = tempfile::tempfile().unwrap();
        test_impl(&f);
    }
}
