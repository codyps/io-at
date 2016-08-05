/* Ensure that other crates can impl ReadAt & WriteAt for their refs */
extern crate io_at;

struct Foo(Vec<u8>);
impl<'a> io_at::ReadAt for &'a Foo {
    fn read_at(&self, buf: &mut [u8], offs: u64) -> io_at::Result<usize> {
        self.0.read_at(buf, offs)
    }
}

impl<'a> io_at::WriteAt for &'a mut Foo {
    fn write_at(&mut self, buf: &[u8], offs: u64) -> io_at::Result<usize> {
        self.0.write_at(buf, offs)
    }
}

fn impls_read_at<T: io_at::ReadAt>(_: T) {}
fn impls_write_at<T: io_at::WriteAt>(_: T) {}

#[test]
fn test() {
    let mut f = Foo(vec![0;2]);

    impls_read_at(&f);
    impls_write_at(&mut f);
}
