use crate::addrs::IPv4Addr;
use crate::protocols::ethernet::EthernetFrame;
use rat_derive::packet;

#[packet(
    layer = DataLink,
    parent = EthernetFrame,
    selector = 0x0800
)]
pub struct IPv4Frame {
    #[packet(label = "Version IHL")]
    pub version_ihl: u8,

    pub tos: u8,

    #[packet(label = "Total length")]
    pub total_length: u16,

    #[packet(label = "ID")]
    pub id: u16,

    #[packet(label = "Flags Fragment")]
    pub flags_fragment: u16,

    #[packet(label = "Time To Live")]
    pub ttl: u8,

    #[packet(label = "Protocol", next)]
    pub protocol: u8,

    #[packet(label = "Checksum")]
    pub checksum: u16,

    #[packet(label = "Source Address")]
    pub source_address: IPv4Addr,

    #[packet(label = "Destination address")]
    pub destination_address: IPv4Addr,
}
