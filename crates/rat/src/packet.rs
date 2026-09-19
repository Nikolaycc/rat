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

pub trait Packet: Sized + 'static {
    const LAYER: Layer;
    const SELECTOR: u32;

    fn parse(frame: &[u8]) -> Result<&Self, ParseError>;

    fn parent_type_id() -> Option<std::any::TypeId>;

    fn next_selector(&self) -> Option<u32>;

    #[inline]
    fn header_len(&self) -> usize {
        std::mem::size_of::<Self>()
    }

    #[inline]
    fn packet_len(&self) -> Option<usize> {
        None
    }
}
