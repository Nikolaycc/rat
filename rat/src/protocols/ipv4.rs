use crate::addrs::IPv4Addr;
use crate::protocols::ethernet::EthernetFrame;
use rat_derive::packet;

#[packet(
    layer = DataLink,
    parent = EthernetFrame,
    selector = 0x0800,
    header_len = ((self.version_ihl & 0x0f) * 4) as usize,
    packet_len = self.total_length.get() as usize
)]
pub struct IPv4Frame {
    #[packet(label = "Version IHL")]
    pub version_ihl: u8,
    #[packet(label = "TOS")]
    pub tos: u8,
    pub total_length: u16,
    #[packet(label = "ID")]
    pub id: u16,
    pub flags_fragment: u16,
    #[packet(label = "Time To Live")]
    pub ttl: u8,
    #[packet(next)]
    pub protocol: u8,
    pub checksum: u16,
    pub source_address: IPv4Addr,
    pub destination_address: IPv4Addr,
}
