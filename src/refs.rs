use super::*;

/**
 * Given a type T that impliments WriteAt, impliment WriteAt for &mut T
 */
impl<'a, T: WriteAt> WriteAt for &'a mut T {
    #[inline]
    fn write_at(&mut self, buf: &[u8], offs: u64) -> Result<usize> {
        (*self).write_at(buf, offs)
    }
}

/**
 * Given that type &T impliments ReadAt, impliment ReadAt for &T
 */
impl<'a, T: ReadAt> ReadAt for &'a mut T {
    fn read_at(&self, buf: &mut [u8], offs: u64) -> Result<usize> {
        let src : &T = self;
        src.read_at(buf, offs)
    }
}

/**
 * Given a type T that impliments ReadAt, impliment ReadAt for &T and &mut T
 */
impl<'a, T: ReadAt> ReadAt for &'a T {
    #[inline]
    fn read_at(&self, buf: &mut [u8], offs: u64) -> Result<usize> {
        (*self).read_at(buf, offs)
    }
}
