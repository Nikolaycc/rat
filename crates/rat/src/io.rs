use std::ffi::CString;
use std::io::ErrorKind;
use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};
use std::path::Path;

use crate::utils::syscall;

pub fn set_nonblocking<F>(fd: &F) -> std::io::Result<()>
where
    F: AsRawFd,
{
    let raw = fd.as_raw_fd();
    let flags = syscall!(fcntl(raw, libc::F_GETFL)).expect("here in flags");
    let _res =
        syscall!(fcntl(raw, libc::F_SETFL, flags | libc::O_NONBLOCK,)).expect("here is F_SETFL");
    Ok(())
}

#[allow(unused)]
pub fn open<S>(path: &S, oflag: i32) -> std::io::Result<OwnedFd>
where
    S: AsRef<Path>,
{
    let path_str = path
        .as_ref()
        .to_str()
        .ok_or(std::io::Error::from(ErrorKind::InvalidInput))?;
    let cpath = CString::new(path_str)?;

    syscall!(open(cpath.as_ptr(), oflag)).map(|fd| unsafe { OwnedFd::from_raw_fd(fd) })
}

#[allow(unused)]
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
    .map(isize::cast_unsigned)
}
