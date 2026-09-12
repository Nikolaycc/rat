use crate::utils::ParseError;

pub enum Layer {
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
