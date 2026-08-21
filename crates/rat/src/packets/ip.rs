use crate::addrs::IPAddr;
use crate::packets::ethernet::EthernetFrame;
use rat_derive::packet;

#[packet(
    layer = DataLink,
    parent = EthernetFrame,
    selector = 0x0800
)]
pub struct IPFrame {
    pub version_ihl: u8,
    pub tos: u8,
    pub total_length: u16,
    pub id: u16,
    pub flags_fragment: u16,
    pub ttl: u8,
    #[packet(next)]
    pub protocol: u8,
    pub checksum: u16,
    pub source_address: IPAddr,
    pub destination_address: IPAddr,
}
