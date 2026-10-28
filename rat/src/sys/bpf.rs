use std::{
    fs::OpenOptions,
    io::{self, ErrorKind},
    os::fd::{AsFd, AsRawFd, BorrowedFd, OwnedFd, RawFd},
};

use zerocopy::{
    FromBytes, Immutable, KnownLayout,
    native_endian::{U16, U32},
};

use crate::{error::ParseError, io::read, utils::syscall};

const BPF_MAX_DEVICES: u32 = 256;

#[cfg(any(target_os = "macos", target_os = "ios"))]
type Time = zerocopy::native_endian::I32;

#[cfg(target_os = "openbsd")]
type Time = zerocopy::native_endian::U32;

#[cfg(all(
    any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"),
    target_pointer_width = "64"
))]
type Time = zerocopy::native_endian::I64;

#[cfg(all(
    any(target_os = "freebsd", target_os = "netbsd", target_os = "dragonfly"),
    target_pointer_width = "32"
))]
type Time = zerocopy::native_endian::I32;

// FIXME: potential bug
#[derive(Debug, PartialEq, Eq, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct TimeVal {
    pub tv_sec: Time,
    pub tv_usec: Time,
}

#[derive(Debug, PartialEq, Eq, FromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct Bpf {
    // FIXME: potential bug
    pub bh_tstamp: TimeVal,

    pub bh_caplen: U32,
    pub bh_datalen: U32,
    pub bh_hdrlen: U16,
    // __padding: [u8; 2],
}

impl Bpf {
    /// Parses a [`BPFFrame`] from the start of `frame`.
    ///
    /// Returns a reference of the frame and the bytes that follow it. The data is
    /// not copied; the reference points into 'frame'.
    ///
    /// # Errors
    ///
    /// Returns an [`ParseError::InvalidValue`] if `frame` is too short to hold a
    /// [`BPFFrame`], or if its bytes are not a valid [`BPFFrame`].
    pub fn parse(frame: &[u8]) -> Result<(&Self, &[u8]), ParseError> {
        Self::ref_from_prefix(frame).map_err(|_| ParseError::InvalidValue)
    }
}

pub struct BpfDevice(OwnedFd);

impl BpfDevice {
    /// Opens the first available BPF device.
    ///
    /// BPF devices are exclusive, so each open capture needs its own
    /// `/dev/bpfN`. This tries `/dev/bpf0`, `/dev/bpf1`, and so on, and skips
    /// devices that are busy.
    ///
    /// # Errors
    ///
    /// Returns the OS error if opening fails for any reason other than the
    /// device being busy (for example [`ErrorKind::PermissionDenied`]).
    /// Returns [`ErrorKind::NotFound`] if every device is busy or non exist.
    #[must_use]
    pub fn open() -> io::Result<Self> {
        for n in 0..BPF_MAX_DEVICES {
            let path = format!("/dev/bpf{n}");

            match OpenOptions::new().read(true).open(&path) {
                Ok(file) => return Ok(Self(OwnedFd::from(file))),
                Err(e) if e.raw_os_error() == Some(libc::EBUSY) => continue,
                Err(e) if e.kind() == ErrorKind::NotFound => break,
                Err(e) => return Err(e),
            }
        }

        Err(io::Error::new(
            ErrorKind::NotFound,
            "no available /dev/bpf device",
        ))
    }

    pub fn read(&self, buf: &mut [u8]) -> io::Result<usize> {
        read(&self.0, buf)
    }

    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        let raw = self.0.as_raw_fd();

        let mut opt: libc::c_int = if nonblocking { 1 } else { 0 };

        let _ = syscall!(ioctl(raw, libc::FIONBIO, &mut opt))?;

        Ok(())
    }
}

impl AsFd for BpfDevice {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.0.as_fd()
    }
}

impl AsRawFd for BpfDevice {
    fn as_raw_fd(&self) -> RawFd {
        self.0.as_raw_fd()
    }
}

#[inline]
pub(crate) const fn bpf_wordalign(length: usize) -> usize {
    let alignment = libc::BPF_ALIGNMENT as usize;
    (length + alignment - 1) & !(alignment - 1)
}
