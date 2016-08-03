/**
 * Given a type T that impliments WriteAt, impliment WriteAt for &mut T
 */
#[macro_export]
macro_rules! impl_ref_write_at {
    ($t:ty) => {
        impl<'a> $crate::WriteAt for &'a mut $t {
            #[inline]
            fn write_at(&mut self, buf: &[u8], offs: u64) -> $crate::Result<usize> {
                (*self).write_at(buf, offs)
            }
        }
    }
}

/**
 * Given that type &T impliments ReadAt, impliment ReadAt for &T
 */
#[macro_export]
macro_rules! impl_ref_read_at_mut {
    ($t:ty) => {
        impl<'a> $crate::ReadAt for &'a mut $t {
            fn read_at(&self, buf: &mut [u8], offs: u64) -> $crate::Result<usize> {
                let src : & $t = self;
                src.read_at(buf, offs)
            }
        }
    }
}

/**
 * Given a type T that impliments ReadAt, impliment ReadAt for &T and &mut T
 */
#[macro_export]
macro_rules! impl_ref_read_at {
    ($t:ty) => {
        impl<'a> $crate::ReadAt for &'a $t {
            #[inline]
            fn read_at(&self, buf: &mut [u8], offs: u64) -> $crate::Result<usize> {
                (*self).read_at(buf, offs)
            }
        }

        impl_ref_read_at_mut!{$t}
    }
}

/**
 * Given a type T that impliments ReadAt and WriteAt, impliment ReadAt for &T and &mut T, and
 * WriteAt for &mut T
 */
#[macro_export]
macro_rules! impl_ref_io_at {
    ($t:ty) => {
        impl_ref_write_at!{$t}
        impl_ref_read_at!{$t}
    }
}
