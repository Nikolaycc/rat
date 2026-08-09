use std::fmt;
use zerocopy::{Immutable, KnownLayout, TryFromBytes, network_endian::U16};

use crate::addrs::MacAddr;
use crate::packets::{Layer, OSILayer, Packet};
use crate::utils::ParseError;

/*
 * The number of bytes in an ethernet (MAC) address.
 */
pub const ETHER_ADDR_LEN: usize = 6;

/*
 * The number of bytes in the type field.
 */
pub const ETHER_TYPE_LEN: usize = 2;

/*
 * The number of bytes in the trailing CRC field.
 */
pub const ETHER_CRC_LEN: usize = 4;

/*
 * The length of the combined header.
 */
pub const ETHER_HDR_LEN: usize = ETHER_ADDR_LEN * 2 + ETHER_TYPE_LEN;

/*
 * The minimum packet length.
 */
pub const ETHER_MIN_LEN: usize = 64;

/*
 * The maximum packet length.
 */
pub const ETHER_MAX_LEN: usize = 1518;

#[derive(Debug)]
#[repr(u16)]
pub enum EtherType {
    Pup,        /* PUP protocol */
    IP,         /* IP protocol */
    ARP,        /* Addr. resolution protocol */
    Revarp,     /* reverse Addr. resolution protocol */
    Vlan,       /* IEEE 802.1Q VLAN tagging */
    IPv6,       /* IPv6 */
    Pae,        /* EAPOL PAE/802.1x */
    Wai,        /* WAI Authentication Protocol */
    RsnPreauth, /* 802.11i / RSN Pre-Authentication */
    Ptp,        /* IEEE 1588 Precision Time Protocol */
    Loopback,   /* used to test interfaces */
    Unknown(u16),
}

impl From<u16> for EtherType {
    fn from(typ: u16) -> Self {
        match typ {
            0x0200 => Self::Pup,
            0x0800 => Self::IP,
            0x0806 => Self::ARP,
            0x8035 => Self::Revarp,
            0x8100 => Self::Vlan,
            0x86dd => Self::IPv6,
            0x888e => Self::Pae,
            0x88b4 => Self::Wai,
            0x88c7 => Self::RsnPreauth,
            0x88f7 => Self::Ptp,
            0x9000 => Self::Loopback,
            other => Self::Unknown(other),
        }
    }
}

impl fmt::Display for EtherType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Pup => write!(f, "PUP"),
            Self::IP => write!(f, "IPv4"),
            Self::ARP => write!(f, "ARP"),
            Self::Revarp => write!(f, "RARP"),
            Self::Vlan => write!(f, "802.1Q VLAN"),
            Self::IPv6 => write!(f, "IPv6"),
            Self::Pae => write!(f, "EAPOL"),
            Self::Wai => write!(f, "WAI"),
            Self::RsnPreauth => write!(f, "RSN Pre-Authentication"),
            Self::Ptp => write!(f, "PTP"),
            Self::Loopback => write!(f, "Loopback"),
            Self::Unknown(c) => write!(f, "Unknown({c})"),
        }
    }
}

#[derive(TryFromBytes, Immutable, KnownLayout)]
#[repr(C, packed)]
pub struct EthernetFrame {
    pub dst: MacAddr,
    pub src: MacAddr,
    pub ty: U16,
}

impl Packet for EthernetFrame {
    const LAYER: Layer = Layer::OSI(OSILayer::Physical);

    fn parse(data: &[u8]) -> Result<&Self, ParseError> {
        Self::try_ref_from_bytes(&data[..ETHER_HDR_LEN]).map_err(|_| ParseError::InvalidValue)
    }
}
