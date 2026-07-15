use std::ffi::CString;
use std::os::fd::{AsFd, AsRawFd, OwnedFd};
use std::path::Path;

use crate::utils::syscall;

pub(crate) fn open<S: AsRef<Path>>(path: &S, oflag: libc::c_int) -> std::io::Result<OwnedFd> {
    use std::os::fd::FromRawFd;

    let path = CString::new(path.as_ref().to_str().unwrap())?;

    syscall!(open(path.as_ptr(), oflag)).map(|fd| unsafe { OwnedFd::from_raw_fd(fd) })
}

pub(crate) fn read<F: AsRawFd + AsFd>(fd: &F, buf: &mut [u8]) -> std::io::Result<usize> {
    let raw_fd = fd.as_fd().as_raw_fd();

    syscall!(read(
        raw_fd,
        buf.as_mut_ptr().cast(),
        buf.len() as libc::size_t
    ))
    .map(|n| n as usize)
}
