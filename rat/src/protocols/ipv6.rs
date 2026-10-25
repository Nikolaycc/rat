use crate::addrs::IPv6Addr;
use crate::protocols::ethernet::EthernetFrame;
use rat_derive::packet;

#[packet(
    layer = DataLink,
    parent = EthernetFrame,
    selector = 0x86dd
)]
pub struct IPv6Frame {
    version_tc_flow: u32,

    #[packet(label = "Payload Length")]
    payload_length: u16,

    #[packet(label = "Next Header", next)]
    next_header: u8,

    #[packet(label = "Hop Limit")]
    hop_limit: u8,

    #[packet(label = "Source Address")]
    src_addr: IPv6Addr,

    #[packet(label = "Destination Address")]
    dst_addr: IPv6Addr,
}
