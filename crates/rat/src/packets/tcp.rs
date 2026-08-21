use crate::packets::ip::IPFrame;
use rat_derive::packet;

#[packet(
    layer = Network,
    parent = IPFrame,
    selector = 6
)]
pub struct TCPFrame {
    pub source_port: u16,
    pub destination_port: u16,
    pub sequence_number: u32,
    pub acknowledgment_number: u32,
    pub reserved: u8,
    pub flags: u8,
    pub window_size: u16,
    pub checksum: u16,
    pub urgent_pointer: u16,
}
