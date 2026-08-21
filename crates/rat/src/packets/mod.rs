use crate::utils::ParseError;

pub mod arp;
pub mod bpf;
pub mod ethernet;
pub mod icmp;
pub mod ip;
pub mod ospf;
pub mod tcp;
pub mod udp;

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
    const SELECTOR: u32;

    fn parse(frame: &[u8]) -> Result<&Self, ParseError>;

    fn parent_type_id() -> Option<std::any::TypeId>;

    fn next_selector(&self) -> Option<u32>;
}
