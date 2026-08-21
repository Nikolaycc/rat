use crate::packets::ip::IPFrame;
use rat_derive::packet;

#[packet(
    layer = Network,
    parent = IPFrame,
    selector = 89
)]
pub struct OSPFFrame {
    pub version: u8,
    pub typ: u8,
    pub packet_length: u16,
    pub router_id: u32,
    pub area_id: u32,
    pub checksum: u16,
    pub autype: u16,
    pub authentication: u64,
}
