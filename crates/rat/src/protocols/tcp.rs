use crate::protocols::ipv4::IPv4Frame;
use rat_derive::packet;

#[packet(
    layer = Network,
    parent = IPv4Frame,
    selector = 6
)]
pub struct TCPFrame {
    #[packet(label = "Source Port")]
    pub source_port: u16,

    #[packet(label = "Destination Port")]
    pub destination_port: u16,

    #[packet(label = "Sequence Number")]
    pub sequence_number: u32,

    #[packet(label = "Acknowledgment Number")]
    pub acknowledgment_number: u32,

    #[packet(label = "Reserved")]
    pub reserved: u8,

    #[packet(label = "Flags")]
    pub flags: u8,

    #[packet(label = "Window Size")]
    pub window_size: u16,

    #[packet(label = "Checksim")]
    pub checksum: u16,

    #[packet(label = "Urgent Pointer")]
    pub urgent_pointer: u16,
}
