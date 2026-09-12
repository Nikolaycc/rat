use crate::addrs::{IPv4Addr, MacAddr};
use crate::protocols::ethernet::EthernetFrame;
use rat_derive::packet;

#[packet(
    layer = DataLink,
    parent = EthernetFrame,
    selector = 0x0806
)]
pub struct ARPFrame {
    #[packet(label = "Hardware Type")]
    pub hardware_type: u16,

    #[packet(label = "Protocol Type")]
    pub protocol_type: u16,

    #[packet(label = "Hardware Address Length")]
    pub hardware_address_length: u8,

    #[packet(label = "Protocol Address Length")]
    pub protocol_address_length: u8,

    #[packet(label = "Operation")]
    pub operation: u16,

    #[packet(label = "Sender Hardware Address")]
    pub sender_hardware_address: MacAddr,

    #[packet(label = "Sender IP Address")]
    pub sender_ip_address: IPv4Addr,

    #[packet(label = "Target Hardware Address")]
    pub target_hardware_address: MacAddr,

    #[packet(label = "Target IP Address")]
    pub target_ip_address: IPv4Addr,
}
