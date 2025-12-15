#![allow(non_snake_case)]

pub mod binding;
pub mod packets;

use std::mem::zeroed;
use std::ptr;
use binding as ratc;

#[derive(Debug)]
pub struct RatCap {
    pub device: ratc::RatDevice,
    cap: ratc::RatCap,
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
        
        let device = &devices[picked as usize];

        let res = unsafe {
            ratc::cap_create(
                &mut cap as *mut _,
                device as *const _,
				ptr::null_mut(),
				0,
            )
        };
		if res < 0 {
			return Err("Failed to create capture.".into());
		}
		
        Ok(Self { device: *device, cap })
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
        let device = &devices[target];

		let res = unsafe {
            ratc::cap_create(
                &mut cap as *mut _,
                device as *const _,
				ptr::null_mut(),
				0,
            )
        };
		if res < 0 {
			return Err("Failed to create capture.".into());
		}
		
        Ok(Self { device: *device, cap })
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
