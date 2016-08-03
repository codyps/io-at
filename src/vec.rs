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

/* TODO: eliminate duplication here */
impl<'a> ReadAt for Vec<u8> {
    #[inline]
    fn read_at(&self, buf: &mut[u8], offs: u64) -> Result<usize> {
        /* use &[u8] impl */
        (&self[..]).read_at(buf, offs)
    }
}

impl WriteAt for Vec<u8> {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        let offs = as_usize(offs);
        if buf.len() == 0 {
            return Ok(0);
        }
        if offs > self.len() {
            /* fill extra space with zeros */
            self.resize(offs, 0);
            self.extend_from_slice(buf);
        } else {
            /* 2 pieces:
             *  - copy_from_slice() what fits
             *  - extend_from_slice() what doesn't
             */
            let l = {
                let r = &mut self[offs..];
                let l = cmp::min(buf.len(), r.len());
                let r = &mut r[..l];
                let buf = &buf[..l];
                r.copy_from_slice(buf);
                l
            };

            if l < buf.len() {
                self.extend_from_slice(&buf[l..]);
            }
        }
        Ok(buf.len())
    }
}

#[test]
fn do_t() {
    let x = vec![];
    super::test_impl(x);
}
