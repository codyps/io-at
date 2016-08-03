use super::super::*;
use std::fs::File;
use super::{pread,pwrite};

impl ReadAt for File {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        pread(self, buf, offs)
    }
}

impl WriteAt for File {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        pwrite(self, buf, offs)
    }
}

#[test]
fn do_t() {
    use tempfile;
    let f = tempfile::tempfile().unwrap();
    super::super::test_impl(f);
}

impl_ref_io_at!{File}
