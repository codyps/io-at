/*
 * Behavior of cursor in files differs between windows & unix. Windows docs appear to imply unix
 * behavior (ie: writing to an offset not updating the internal cursor), but testing shows
 * otherwise.
 *
 * We can't fix this without using some not-so-great locking around write_at/read_at on windows.
 */
extern crate io_at;
extern crate tempfile;

use io_at::WriteAt;
use std::io::Seek;

#[test]
fn test() {
    let x = [56u8, 1, 66];
    let mut f = tempfile::tempfile().unwrap();

    f.write_all_at(&x, 0).unwrap();

    #[cfg(unix)]
    assert_eq!(f.seek(std::io::SeekFrom::Current(0)).unwrap(), 0);
    #[cfg(windows)]
    assert_eq!(f.seek(std::io::SeekFrom::Current(0)).unwrap(), 3);
}
