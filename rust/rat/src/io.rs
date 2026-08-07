use std::ffi::CString;
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::path::Path;

use crate::utils::syscall;

pub fn open<S>(path: &S, oflag: libc::c_int) -> std::io::Result<OwnedFd>
where
    S: AsRef<Path>,
{
    use std::os::fd::FromRawFd;

    let path = CString::new(path.as_ref().to_str().unwrap())?;

    syscall!(open(path.as_ptr(), oflag)).map(|fd| unsafe { OwnedFd::from_raw_fd(fd) })
}

pub fn read<F>(fd: &F, buf: &mut [u8]) -> std::io::Result<usize>
where
    F: AsRawFd + AsFd,
{
    let raw_fd = fd.as_fd().as_raw_fd();

    syscall!(read(
        raw_fd,
        buf.as_mut_ptr().cast(),
        buf.len() as libc::size_t
    ))
    .map(|n| isize::cast_unsigned(n))
}
