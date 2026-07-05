use libc::{BIOCGBLEN, BIOCIMMEDIATE, BIOCSETIF, ifreq, ioctl};
use std::mem;
use std::os::fd::AsRawFd;
use std::path::Path;

mod addrs;
mod io;
mod utils;

use crate::addrs::{InterfaceMap, MacAddr, getifaddrs};
use crate::io::{open, read};
use crate::utils::{ParseError, inspect_bpf_buffer, str_to_ifname};

#[derive(Debug)]
struct EthernetFrame<'a> {
    dest_addr: MacAddr,
    source_addr: MacAddr,
    ty: u16,
    payload: &'a [u8],
}

impl<'a> TryFrom<&'a [u8]> for EthernetFrame<'a> {
    type Error = ParseError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        const HEADER_SIZE: usize = 14;

        if data.len() < HEADER_SIZE {
            return Err(ParseError::PacketTooShort {
                expected: HEADER_SIZE,
                actual: data.len(),
            });
        }

        let dest_bytes: [u8; 6] = data[0..6]
            .try_into()
            .map_err(|_| ParseError::InvalidValue)?;

        let dest_addr = MacAddr::new(dest_bytes);

        let source_bytes: [u8; 6] = data[6..12]
            .try_into()
            .map_err(|_| ParseError::InvalidValue)?;

        let source_addr = MacAddr::new(source_bytes);

        let ty = u16::from_be_bytes(
            data[12..14]
                .try_into()
                .map_err(|_| ParseError::InvalidValue)?,
        );

        Ok(Self {
            dest_addr,
            source_addr,
            ty,
            payload: &data[HEADER_SIZE..],
        })
    }
}

fn main() {
    let interfaces = InterfaceMap::new().unwrap();
    println!("{interfaces}");

    let bpf_path = Path::new("/dev/bpf0");

    let fd = match open(&bpf_path, libc::O_RDWR) {
        Err(why) => panic!("couldn't open {}: {}", bpf_path.display(), why),
        Ok(file) => file,
    };

    let raw_fd = fd.as_raw_fd();
    let enable = 1;
    let ret = unsafe { ioctl(raw_fd, BIOCIMMEDIATE, &enable) };
    if ret < 0 {
        panic!(
            "failed to ioctl BIOCIMMEDIATE {}",
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

    let mut ifreq = interfaces.to_interface_req("en1").unwrap();

    let ret = unsafe { ioctl(raw_fd, BIOCSETIF, &mut ifreq.0) };
    if ret < 0 {
        panic!(
            "failed to ioctl BIOCSETIF: {}",
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
            match EthernetFrame::try_from(packet) {
                Ok(ethernet) => {
                    println!(
                        "EthernetFrame: dest_addr: {}, source_addr: {}, ty: 0x{:04X}, payload size: {}",
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
