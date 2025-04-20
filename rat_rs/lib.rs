#![allow(non_snake_case)]

pub mod binding;

use std::ffi::c_void;
use std::mem::zeroed;
use binding as ratc;

pub struct RatCap {
    pub device: ratc::RatDevice,
    cap: ratc::RatCap,
    current_packet: Option<ratc::RatPacketT>,
}

impl RatCap {
    pub fn new() -> Result<Self, String> {
        let mut devices = [ratc::RatDevice {
            name: [0; 16],
            mtu: 0,
        }; ratc::MAX_INTERFACES];

        let found = unsafe { ratc::device_lookup(devices.as_mut_ptr()) };
        if found <= 0 {
            return Err("No interfaces found.".into());
        }

        let picked = unsafe { ratc::device_pick(devices.as_mut_ptr(), found as usize) };
        if picked < 0 {
            return Err("Failed to pick interface.".into());
        }

        let mut cap: ratc::RatCap = unsafe { zeroed() };
        let mut current_packet: Option<ratc::RatPacketT> = None;

        let device = &devices[picked as usize];

        unsafe {
            ratc::cap_create(
                &mut cap as *mut _,
                device as *const _,
                &mut current_packet as *mut _ as *mut c_void,
                0,
            );
        }

        Ok(Self { device: *device, cap, current_packet })
    }

    pub fn from(device_name: &str) -> Result<Self, String> {
        let mut devices = [ratc::RatDevice {
            name: [0; 16],
            mtu: 0,
        }; ratc::MAX_INTERFACES];

        let found = unsafe { ratc::device_lookup(devices.as_mut_ptr()) };
        if found <= 0 {
            return Err("No interfaces found.".into());
        }

        let target = devices
            .iter()
            .take(found as usize)
            .position(|dev| {
                let cstr = unsafe {
                    std::ffi::CStr::from_ptr(dev.name.as_ptr() as *const i8)
                };
                cstr.to_str().map(|s| s == device_name).unwrap_or(false)
            })
            .ok_or("Device not found by name")?;

        let mut cap: ratc::RatCap = unsafe { zeroed() };
        let mut current_packet: Option<ratc::RatPacketT> = None;

        let device = &devices[target];

        unsafe {
            ratc::cap_create(
                &mut cap as *mut _,
                device as *const _,
                &mut current_packet as *mut _ as *mut c_void,
                0,
            );
        }

        Ok(Self { device: *device, cap, current_packet })
    }

    pub fn capture_one(&mut self) -> Option<ratc::RatPacketT> {
	let mut packet: ratc::RatPacketT = unsafe { zeroed() };
	
        unsafe {
            ratc::cap_loop_w(&mut self.cap as *mut _, &mut packet as *mut ratc::RatPacketT, 1);
        }

	Some(packet)
    }
}

impl Iterator for RatCap {
    type Item = ratc::RatPacketT;

    fn next(&mut self) -> Option<Self::Item> {
        self.capture_one()
    }
}

impl Drop for RatCap {
    fn drop(&mut self) {
        unsafe {
            ratc::cap_destroy(&mut self.cap as *mut _);
        }
    }
}
