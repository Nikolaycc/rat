use crate::{protocols::ProtocolKey, utils::ParseError};

pub(crate) struct DecodeResult {
    pub h_len: usize,
    pub p_len: usize,
    pub next: Option<ProtocolKey>,
}

pub(crate) type DecodeFn = fn(&[u8]) -> Result<DecodeResult, ParseError>;

pub(crate) struct ProtocolDescriptor {}

pub(crate) struct Route {}

pub struct ProtocolRegistry {}
