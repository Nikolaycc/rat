use crate::packets::ip::IPFrame;
use rat_derive::packet;

#[packet(
    layer = Transport,
    parent = IPFrame,
    selector = 17
)]
pub struct UDPFrame {
    pub src: u16,
    pub dst: u16,
    pub lenght: u16,
    pub checksum: u16,
}
