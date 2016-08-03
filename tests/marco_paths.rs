#[macro_use]
extern crate io_at;

struct Foo(Vec<u8>);
impl io_at::ReadAt for Foo {
    fn read_at(&self, buf: &mut [u8], offs: u64) -> io_at::Result<usize> {
        self.0.read_at(buf, offs)
    }
}

impl io_at::WriteAt for Foo {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> io_at::Result<usize> {
        self.0.write_at(buf, offs)
    }
}

fn impls_read_at<T: io_at::ReadAt>(_: T) {}
fn impls_write_at<T: io_at::WriteAt>(_: T) {}

#[test]
fn all() {
    impl_ref_io_at!{Foo}
    let mut f = Foo(vec![0u8;10]);
    impls_read_at(&f);
    impls_read_at(&mut f);
    impls_write_at(&mut f);
}
