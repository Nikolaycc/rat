use crate::utils::ParseError;

pub mod bpf;
pub mod ethernet;

pub enum Layer {
    PRH, // Packet Record Header
    OSI(OSILayer),
}

pub enum OSILayer {
    Physical,
    DataLink,
    Network,
    Transport,
    Session,
    Presentation,
    Application,
}

pub trait Packet {
    const LAYER: Layer;

    fn parse(frame: &[u8]) -> Result<&Self, ParseError>;
}
