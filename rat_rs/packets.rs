use std::slice;
use super::binding::{packets as raw, RatPacketT, TimeVal};

#[derive(Debug, Clone)]
pub enum PacketHeader {
    Ethernet(EthernetHeader),
    Arp(ArpHeader),
    Ip(IpHeader),
    Transport(TransportHeader),
}

pub trait Packet {
    fn timestamp(&self) -> TimeVal;
    fn raw_data(&self) -> &[u8];
    fn payload(&self) -> &[u8];
    fn headers(&self) -> &[PacketHeader];
}

#[derive(Debug, Clone)]
pub struct ParsedPacket {
    timestamp: TimeVal,
    raw_data: Vec<u8>,
    headers: Vec<PacketHeader>,
    payload: Vec<u8>,
}

impl ParsedPacket {
    pub unsafe fn from_raw(raw_pkt: &RatPacketT) -> Self {
        let data = slice::from_raw_parts(raw_pkt.raw_data, raw_pkt.length).to_vec();

        let mut headers = Vec::new();
        if let Some(eth) = raw_pkt.eth.as_ref() {
            headers.push(PacketHeader::Ethernet(EthernetHeader::from(*eth)));
        }
        if let Some(arp) = raw_pkt.arp.as_ref() {
            headers.push(PacketHeader::Arp(ArpHeader::from(*arp)));
        }
        if let Some(ip) = raw_pkt.ip.as_ref() {
            headers.push(PacketHeader::Ip(IpHeader::from(*ip)));
        }
        if let Some(tcp) = raw_pkt.tcp.as_ref() {
            headers.push(PacketHeader::Transport(TransportHeader::Tcp(TcpHeader::from(*tcp))));
        } else if let Some(udp) = raw_pkt.udp.as_ref() {
            headers.push(PacketHeader::Transport(TransportHeader::Udp(UdpHeader::from(*udp))));
        }

        let payload = slice::from_raw_parts(raw_pkt.payload, raw_pkt.payload_length).to_vec();

        ParsedPacket { timestamp: raw_pkt.timestamp, raw_data: data, headers, payload }
    }
}

impl Packet for ParsedPacket {
    fn timestamp(&self) -> TimeVal { self.timestamp }
    fn raw_data(&self) -> &[u8] { &self.raw_data }
    fn payload(&self) -> &[u8] { &self.payload }
    fn headers(&self) -> &[PacketHeader] { &self.headers }
}

impl IntoIterator for ParsedPacket {
    type Item = PacketHeader;
    type IntoIter = std::vec::IntoIter<PacketHeader>;
    fn into_iter(self) -> Self::IntoIter {
        self.headers.into_iter()
    }
}

impl<'a> IntoIterator for &'a ParsedPacket {
    type Item = &'a PacketHeader;
    type IntoIter = std::slice::Iter<'a, PacketHeader>;
    fn into_iter(self) -> Self::IntoIter {
        self.headers.iter()
    }
}

#[derive(Debug, Clone)]
pub struct EthernetHeader {
    pub source: [u8; raw::RAT_HA_ADDR_LEN],
    pub destination: [u8; raw::RAT_HA_ADDR_LEN],
    pub ethertype: u16,
}

impl From<raw::RatEthernetHeader> for EthernetHeader {
    fn from(h: raw::RatEthernetHeader) -> Self {
        EthernetHeader { source: h.shost, destination: h.dhost, ethertype: u16::from_be(h.ether_type) }
    }
}

#[derive(Debug, Clone)]
pub struct ArpHeader {
    pub sender_hw: [u8; raw::RAT_HA_ADDR_LEN],
    pub sender_ip: [u8; 4],
    pub target_hw: [u8; raw::RAT_HA_ADDR_LEN],
    pub target_ip: [u8; 4],
    pub operation: u16,
}

impl From<raw::RatArpHeader> for ArpHeader {
    fn from(h: raw::RatArpHeader) -> Self {
        ArpHeader {
            sender_hw: h.sender_hardware_addr,
            sender_ip: h.sender_ip_addr,
            target_hw: h.target_hardware_addr,
            target_ip: h.target_ip_addr,
            operation: u16::from_be(h.operation),
        }
    }
}

#[derive(Debug, Clone)]
pub struct IpHeader {
    pub src: u32,
    pub dst: u32,
    pub protocol: u8,
    pub ttl: u8,
}

impl From<raw::RatIpHeader> for IpHeader {
    fn from(h: raw::RatIpHeader) -> Self {
        IpHeader { src: u32::from_be(h.src_addr), dst: u32::from_be(h.dst_addr), protocol: h.protocol, ttl: h.ttl }
    }
}

#[derive(Debug, Clone)]
pub enum TransportHeader {
    Tcp(TcpHeader),
    Udp(UdpHeader),
}

#[derive(Debug, Clone)]
pub struct TcpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub seq: u32,
    pub ack: u32,
    pub flags: u8,
}

impl From<raw::RatTcpHeader> for TcpHeader {
    fn from(h: raw::RatTcpHeader) -> Self {
        TcpHeader { src_port: u16::from_be(h.src_port), dst_port: u16::from_be(h.dst_port), seq: u32::from_be(h.seq_num), ack: u32::from_be(h.ack_num), flags: h.flags }
    }
}

#[derive(Debug, Clone)]
pub struct UdpHeader {
    pub src_port: u16,
    pub dst_port: u16,
    pub length: u16,
}

impl From<raw::RatUdpHeader> for UdpHeader {
    fn from(h: raw::RatUdpHeader) -> Self {
        UdpHeader { src_port: u16::from_be(h.src_port), dst_port: u16::from_be(h.dst_port), length: u16::from_be(h.length) }
    }
}
