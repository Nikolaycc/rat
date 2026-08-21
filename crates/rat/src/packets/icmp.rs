use crate::packets::ip::IPFrame;
use rat_derive::packet;

#[packet(
    layer = Network,
    parent = IPFrame,
    selector = 1
)]
pub struct ICMPFrame {
    pub typ: u8,
    pub code: u8,
    pub checksum: u16,
    pub extended_header: u32,
}
