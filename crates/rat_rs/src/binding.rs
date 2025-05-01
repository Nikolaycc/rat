#![allow(non_snake_case)]

use std::ffi;
use std::fmt;

pub const MAX_INTERFACES: usize = 23;

type TimeT = i64;
type SusecondsT = i64;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct TimeVal {
    tv_sec: TimeT,
    tv_usec: SusecondsT,
}

impl fmt::Display for TimeVal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const SECONDS_PER_MINUTE: u64 = 60;
        const SECONDS_PER_HOUR: u64 = 3600;
        const SECONDS_PER_DAY: u64 = 86400;

        let total_secs = self.tv_sec as u64;
        let micros = self.tv_usec as u64;

        let seconds_today = total_secs % SECONDS_PER_DAY;

        let hours = seconds_today / SECONDS_PER_HOUR;
        let minutes = (seconds_today % SECONDS_PER_HOUR) / SECONDS_PER_MINUTE;
        let seconds = seconds_today % SECONDS_PER_MINUTE;

        write!(
            f,
            "{:02}:{:02}:{:02}.{:06}",
            hours, minutes, seconds, micros
        )
    }
}

pub mod packets {
    pub const RAT_HA_ADDR_LEN: usize = 6;

    #[repr(C, packed)]
    #[derive(Debug, Copy, Clone)]
    pub struct RatHardwareAddr {
        pub octet: [u8; RAT_HA_ADDR_LEN],
    }

    #[repr(C, packed)]
    #[derive(Debug, Copy, Clone)]
    pub struct RatEthernetHeader {
        pub dhost: [u8; RAT_HA_ADDR_LEN],
        pub shost: [u8; RAT_HA_ADDR_LEN],
        pub ether_type: u16,
    }

    #[repr(C, packed)]
    #[derive(Debug, Copy, Clone)]
    pub struct RatArpHeader {
        pub hardware_type: u16,
        pub protocol_type: u16,
        pub hardware_addr_len: u8,
        pub protocol_addr_len: u8,
        pub operation: u16,
        pub sender_hardware_addr: [u8; RAT_HA_ADDR_LEN],
        pub sender_ip_addr: [u8; 4],
        pub target_hardware_addr: [u8; RAT_HA_ADDR_LEN],
        pub target_ip_addr: [u8; 4],
    }

    #[repr(C, packed)]
    #[derive(Debug, Copy, Clone)]
    pub struct RatIpHeader {
        pub version_ihl: u8,
        pub tos: u8,
        pub total_length: u16,
        pub id: u16,
        pub flags_fragment: u16,
        pub ttl: u8,
        pub protocol: u8,
        pub checksum: u16,
        pub src_addr: u32,
        pub dst_addr: u32,
    }

    #[repr(C, packed)]
    #[derive(Debug, Copy, Clone)]
    pub struct RatTcpHeader {
        pub src_port: u16,
        pub dst_port: u16,
        pub seq_num: u32,
        pub ack_num: u32,
        pub data_offset_reserved: u8,
        pub flags: u8,
        pub window_size: u16,
        pub checksum: u16,
        pub urgent_ptr: u16,
    }

    #[repr(C, packed)]
    #[derive(Debug, Copy, Clone)]
    pub struct RatUdpHeader {
        pub src_port: u16,
        pub dst_port: u16,
        pub length: u16,
        pub checksum: u16,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct RatPacket {
        pub raw_data: *mut u8,
        pub length: usize,
        pub timestamp: super::TimeVal,
        
        pub eth: *mut RatEthernetHeader,
        pub arp: *mut RatArpHeader,
        pub ip: *mut RatIpHeader,
        pub tcp: *mut RatTcpHeader,
        pub udp: *mut RatUdpHeader,
        
        pub payload: *mut u8,
        pub payload_length: usize,
    }

    extern "C" {
        #[link_name="__rat_packet_parse"]
        pub fn packet_parse(packet: *mut RatPacket, header_type: u8, remaining: usize);
    }
}

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct RatDevice {
    pub name: [u8; 16],
    pub mtu: u32,
}

pub type RatDeviceT = RatDevice;
pub type RatPacketT = packets::RatPacket;

#[repr(C)]
#[derive(Debug)]
pub struct RatCap {
    fd: i32,
    device: RatDeviceT,
    timeout: u32,
    buffer_size: usize,
    user_data: *mut ffi::c_void,
}

pub type RatCapCb = unsafe extern "C" fn(packet: *mut RatPacketT, data: *mut ffi::c_void);

extern "C" {
    #[link_name="rat_device_lookup"]
    pub fn device_lookup(devices: *mut RatDevice)-> i32;
    
    #[link_name="rat_device_pick"]
    pub fn device_pick(devices: *mut RatDevice, devices_len: usize) -> i32;
    
    #[link_name="rat_cap_create"]
    pub fn cap_create(cap: *mut RatCap, device: *const RatDevice, user_data: *mut ffi::c_void, timeout: u32) -> i32;
    
    #[link_name="rat_cap_loop"]
    pub fn cap_loop(cap: *mut RatCap, cb: RatCapCb, packet_count: u32) -> i32;

    #[link_name="rat_cap_loop_w"]
    pub fn cap_loop_w(cap: *mut RatCap, pk: *mut RatPacketT, packet_count: u32) -> i32;
    
    #[link_name="rat_cap_destroy"]
    pub fn cap_destroy(cap: *mut RatCap);
}
