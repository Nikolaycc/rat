use zerocopy::{
    Immutable, KnownLayout, TryFromBytes,
    network_endian::{U16, U32},
};

use crate::utils::ParseError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct TimeVal32 {
    pub tv_sec: i32,
    pub tv_usec: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, TryFromBytes, Immutable, KnownLayout)]
#[repr(C)]
pub struct BPFFrame {
    pub bh_tstamp: TimeVal32,
    pub bh_caplen: U32,
    pub bh_datalen: U32,
    pub bh_hdrlen: U16,
}

impl BPFFrame {
    pub fn parse(frame: &[u8]) -> Result<&Self, ParseError> {
        BPFFrame::try_ref_from_bytes(&frame[..size_of::<BPFFrame>()])
            .map_err(|_| ParseError::InvalidValue)
    }
}
