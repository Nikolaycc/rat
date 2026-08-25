use zerocopy::{
    Immutable, KnownLayout, TryFromBytes, Unaligned,
    native_endian::{I32, U16, U32},
};

use crate::utils::ParseError;

#[derive(Debug, PartialEq, Eq, TryFromBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C)]
pub struct TimeVal32 {
    pub tv_sec: I32,
    pub tv_usec: I32,
}

#[derive(Debug, PartialEq, Eq, TryFromBytes, Immutable, KnownLayout, Unaligned)]
#[repr(C)]
pub struct BPFFrame {
    pub bh_tstamp: TimeVal32,
    pub bh_caplen: U32,
    pub bh_datalen: U32,
    pub bh_hdrlen: U16,
}

impl BPFFrame {
    #[must_use]
    pub fn parse(frame: &[u8]) -> Result<&Self, ParseError> {
        BPFFrame::try_ref_from_bytes(&frame[..size_of::<BPFFrame>()])
            .map_err(|_| ParseError::InvalidValue)
    }
}
