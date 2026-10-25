use crate::protocols::ipv4::IPv4Frame;
use rat_derive::packet;

#[packet(
    layer = Transport,
    parent = IPv4Frame,
    selector = 17
)]
pub struct UDPFrame {
    #[packet(label = "Source Port")]
    pub src: u16,

    #[packet(label = "Destination Port")]
    pub dst: u16,

    #[packet(label = "Length")]
    pub lenght: u16,

    #[packet(label = "Checksum")]
    pub checksum: u16,
}
