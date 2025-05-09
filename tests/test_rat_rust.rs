use std::ptr;
use std::mem::MaybeUninit;
use std::ffi::c_void;

unsafe extern "C" fn packet_handler(packet: *mut Rat::binding::RatPacketT, _data: *mut c_void) {
    if !packet.is_null() {
        let packet_ref = &*packet;
        println!("Received packet of length: {}", packet_ref.length);
    }
}

fn main() -> Result<(), String> {
    unsafe {
        let mut devices: [MaybeUninit<Rat::binding::RatDevice>; Rat::binding::MAX_INTERFACES] = 
            MaybeUninit::uninit().assume_init();
        
        let devices_ptr = devices.as_mut_ptr() as *mut Rat::binding::RatDevice;
        
	let devices_size = Rat::binding::device_lookup(devices_ptr);
	if devices_size == 0 {
            println!("No network devices found");
            return Err("No network devices found".to_string());
        }
        
        let device_idx = Rat::binding::device_pick(devices_ptr, devices_size as usize);
        if device_idx < 0 {
            println!("No network devices index found");
            return Err("No network device index found".to_string());
        }
        
        let mut cap = MaybeUninit::<Rat::binding::RatCap>::zeroed().assume_init();
        let device_ptr = devices_ptr.add(device_idx as usize);
        
        Rat::binding::cap_create(&mut cap, device_ptr, ptr::null_mut(), 0);
        
        Rat::binding::cap_loop(&mut cap, packet_handler, 2);
        
        Rat::binding::cap_destroy(&mut cap);
    }
    
    Ok(())
}
