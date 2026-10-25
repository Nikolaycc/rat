use crate::protocols::ipv4::IPv4Frame;
use rat_derive::packet;

#[packet(
    layer = Network,
    parent = IPv4Frame,
    selector = 1
)]
pub struct ICMPFrame {
    pub typ: u8,
    pub code: u8,
    pub checksum: u16,
    pub extended_header: u32,
}
