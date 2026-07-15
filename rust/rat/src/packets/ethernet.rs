use zerocopy::{FromBytes, Immutable, KnownLayout, network_endian::U16};

use crate::utils::ParseError;

#[derive(FromBytes, Immutable, KnownLayout)]
#[repr(C, packed)]
pub struct EthernetFrame {
    pub dest_addr: [u8; 6],
    pub source_addr: [u8; 6],
    pub ty: U16,
}

impl EthernetFrame {
    pub fn parse(data: &[u8]) -> Result<&Self, ParseError> {
        EthernetFrame::ref_from_bytes(data).map_err(|_| ParseError::InvalidValue)
    }
}
