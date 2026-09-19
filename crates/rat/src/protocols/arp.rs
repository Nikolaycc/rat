use crate::addrs::{IPv4Addr, MacAddr};
use crate::protocols::ethernet::EthernetFrame;
use rat_derive::packet;

#[packet(
    layer = DataLink,
    parent = EthernetFrame,
    selector = 0x0806
)]
pub struct ARPFrame {
    pub hardware_type: u16,
    pub protocol_type: u16,
    pub hardware_address_length: u8,
    pub protocol_address_length: u8,
    pub operation: u16,
    pub sender_hardware_address: MacAddr,
    pub sender_ip_address: IPv4Addr,
    pub target_hardware_address: MacAddr,
    pub target_ip_address: IPv4Addr,
}
