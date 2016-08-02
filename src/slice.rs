/* TODO: provide a wrapper type impl for Deref<[u8]> and DerefMut<[u8]> */
use super::*;
use std::{usize,cmp};

/* TODO: see if there is a cannonical way of handling this for any 2 numerics (without an extra
 * crate) */
fn as_usize(v: u64) -> usize {
    /* clamping cast */
    if v > (usize::MAX as u64) {
        usize::MAX
    } else {
        v as usize
    }
}

fn ra(src: &[u8], buf: &mut[u8], offs: u64) -> Result<usize> {
    let offs = as_usize(offs);
    if offs > src.len() {
        return Ok(0);
    }

    let r = &src[offs..];
    let l = cmp::min(buf.len(), r.len());
    let r = &r[..l];
    let buf = &mut buf[..l];

    buf.copy_from_slice(r);
    Ok(l)
}

/* TODO: eliminate duplication here */
impl<'a> ReadAt for &'a mut [u8] {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        ra(self, buf, offs)
    }
}

impl<'a> ReadAt for &'a [u8] {
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        ra(self, buf, offs)
    }
}

impl<'a> WriteAt for &'a mut [u8] {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        let offs = as_usize(offs);
        if offs > self.len() {
            return Ok(0);
        }

        let r = &mut self[offs..];
        let l = cmp::min(buf.len(), r.len());
        let r = &mut r[..l];
        let buf = &buf[..l];

        r.copy_from_slice(buf);
        Ok(l)
    }
}

#[test]
fn do_t() {
    let mut x = [0u8;128];
    super::test_impl(&mut x[..]);
}

