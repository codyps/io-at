#[cfg(unix)]
pub mod unix;
#[cfg(windows)]
pub mod windows;

/* Expose the semi-compatible IoAtRaw types via the same name */
#[cfg(unix)]
pub use self::unix::{AsRaw,pread,pwrite};
#[cfg(windows)]
pub use self::windows::{AsRaw,pread,pwrite};

#[cfg(any(unix,windows))]
pub mod raw;
#[cfg(any(unix,windows))]
pub use self::raw::IoAtRaw;

#[cfg(any(unix,windows))]
pub mod file;
