use std::{
    fs::{File, OpenOptions},
    io::{self, ErrorKind},
};

use zerocopy::{
    Immutable, KnownLayout, TryFromBytes,
    native_endian::{I32, U16, U32},
};

use crate::error::ParseError;

#[derive(Debug, PartialEq, Eq, TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct TimeVal32 {
    pub tv_sec: I32,
    pub tv_usec: I32,
}

#[derive(Debug, PartialEq, Eq, TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct BPFFrame {
    pub bh_tstamp: TimeVal32,
    pub bh_caplen: U32,
    pub bh_datalen: U32,
    pub bh_hdrlen: U16,
    // __padding: [u8; 2],
}

impl BPFFrame {
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
        Self::try_ref_from_prefix(frame).map_err(|_| ParseError::InvalidValue)
    }
}

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
pub(crate) fn open_bpf_device() -> io::Result<File> {
    const MAX_DEVICES: u32 = 256;

    for n in 0..MAX_DEVICES {
        let path = format!("/dev/bpf{n}");

        match OpenOptions::new().read(true).open(&path) {
            Ok(file) => return Ok(file),
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
