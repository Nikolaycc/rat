use libc::{BIOCGBLEN, BIOCIMMEDIATE, BIOCSETIF, ioctl};
use std::os::fd::AsRawFd;
use std::path::Path;

use rat::addrs::InterfaceMap;
use rat::io::{open, read};
use rat::packets::ethernet::EthernetFrame;
use rat::utils::inspect_bpf_buffer;

fn main() {
    let interfaces = InterfaceMap::new().unwrap();
    println!("{interfaces}");

    let bpf_path = Path::new("/dev/bpf0");

    let fd = match open(&bpf_path, libc::O_RDWR) {
        Err(why) => panic!("couldn't open {}: {}", bpf_path.display(), why),
        Ok(file) => file,
    };

    let raw_fd = fd.as_raw_fd();

    let mut ifreq = interfaces.to_interface_req("en1").unwrap();

    let ret = unsafe { ioctl(raw_fd, BIOCSETIF, &mut ifreq.0) };
    if ret < 0 {
        panic!(
            "failed to ioctl BIOCSETIF: {}",
            std::io::Error::last_os_error()
        );
    }

    let mut buflen: u32 = 0;
    let ret = unsafe { ioctl(raw_fd, BIOCGBLEN, &mut buflen) };
    if ret < 0 {
        panic!(
            "failed to ioctl BIOCGBLEN {}",
            std::io::Error::last_os_error()
        );
    }

    let enable: u32 = 1;
    let ret = unsafe { ioctl(raw_fd, BIOCIMMEDIATE, &enable) };
    if ret < 0 {
        panic!(
            "failed to ioctl BIOCIMMEDIATE {}",
            std::io::Error::last_os_error()
        );
    }

    let mut buf: Vec<u8> = vec![0u8; buflen as usize];

    loop {
        buf.fill(0);
        let size = match read(&fd, &mut buf) {
            Err(why) => panic!("couldn't read {why}"),
            Ok(size) => size,
        };

        inspect_bpf_buffer(&buf[..size], |packet| {
            match EthernetFrame::parse(&packet[..14]) {
                Ok(ethernet) => {
                    println!(
                        "EthernetFrame: dest_addr: {:?}, source_addr: {:?}, ty: 0x{:04X}, payload size: {}",
                        ethernet.dest_addr, ethernet.source_addr, ethernet.ty, size,
                    );
                }
                Err(error) => {
                    eprintln!("Ethernet parse error: {error:?}");
                }
            }
        });
    }
}
