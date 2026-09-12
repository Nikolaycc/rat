pub mod arp;
pub mod ethernet;
pub mod icmp;
pub mod ipv4;
pub mod ipv6;
pub mod ospf;
pub mod tcp;
pub mod udp;

#[repr(transparent)]
pub struct ProtocolId(u16);

#[repr(transparent)]
pub struct ProtocolKey(u32);
