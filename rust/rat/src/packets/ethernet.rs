use std::fmt;
use zerocopy::{Immutable, KnownLayout, TryFromBytes, network_endian::U16};

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
    Pup = 0x0200,        /* PUP protocol */
    IP = 0x0800,         /* IP protocol */
    ARP = 0x0806,        /* Addr. resolution protocol */
    Revarp = 0x8035,     /* reverse Addr. resolution protocol */
    Vlan = 0x8100,       /* IEEE 802.1Q VLAN tagging */
    IPv6 = 0x86dd,       /* IPv6 */
    Pae = 0x888e,        /* EAPOL PAE/802.1x */
    Wai = 0x88b4,        /* WAI Authentication Protocol */
    RsnPreauth = 0x88c7, /* 802.11i / RSN Pre-Authentication */
    Ptp = 0x88f7,        /* IEEE 1588 Precision Time Protocol */
    Loopback = 0x9000,   /* used to test interfaces */
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
        }
    }
}

#[derive(TryFromBytes, Immutable, KnownLayout)]
#[repr(C, packed)]
pub struct EthernetFrame {
    pub dest_addr: [u8; 6],
    pub source_addr: [u8; 6],
    pub ty: U16,
}

impl EthernetFrame {
    #[must_use]
    pub fn parse(data: &[u8]) -> Result<&Self, ParseError> {
        Self::try_ref_from_bytes(&data[..ETHER_HDR_LEN]).map_err(|_| ParseError::InvalidValue)
    }
}
