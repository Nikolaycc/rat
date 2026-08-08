use crate::utils::ParseError;

pub mod bpf;
pub mod ethernet;

pub trait Packet {
    fn parse(frame: &[u8]) -> Result<&Self, ParseError>;
}
