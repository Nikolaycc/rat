use crate::addrs::NetworkInterface;
use crate::packets::Packet;
use crate::packets::bpf::BPFFrame;
use libc::{BIOCGBLEN, BIOCIMMEDIATE, BIOCSETIF};
use std::io::ErrorKind;
use std::os::fd::{AsRawFd, OwnedFd};
use std::path::Path;

use crate::addrs::NetworkInterfaceMap;
use crate::io::{open, read};
use crate::utils::syscall;

pub struct Capture {
    fd: OwnedFd,
    buf: Vec<u8>,
    interface: NetworkInterface,
}

impl Capture {
    #[cfg(any(
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "macos"
    ))]
    #[inline]
    pub fn set_interface(&mut self, interface: NetworkInterface) -> std::io::Result<()> {
        let mut ifreq = interface.to_interface_req()?;
        syscall!(ioctl(self.fd.as_raw_fd(), BIOCSETIF, &mut ifreq.0)).map(|_| {
            self.interface = interface;
        })
    }

    #[cfg(any(
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "macos"
    ))]
    #[inline]
    pub fn set_interface_with_name(&mut self, name: &str) -> std::io::Result<()> {
        let interface = NetworkInterface::from_name(name)?;

        self.set_interface(interface)
    }

    #[must_use]
    #[cfg(any(
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "freebsd",
        target_os = "macos"
    ))]
    pub fn new(ifname: &str) -> std::io::Result<Self> {
        let bpf_path = Path::new("/dev/bpf0");
        let fd = open(&bpf_path, libc::O_RDWR)?;

        let raw_fd = fd.as_raw_fd();

        let interfaces = NetworkInterfaceMap::new()?;
        let (interface, _) = interfaces
            .get(ifname)
            .ok_or(std::io::Error::from(ErrorKind::NotFound))?;

        let ifreq = interface.to_interface_req()?;
        syscall!(ioctl(raw_fd, BIOCSETIF, &ifreq.0))?;

        let mut buflen: u32 = 0;
        syscall!(ioctl(raw_fd, BIOCGBLEN, &mut buflen))?;

        let enable: u32 = 1;
        syscall!(ioctl(raw_fd, BIOCIMMEDIATE, &enable))?;

        Ok(Self {
            fd,
            buf: vec![0u8; buflen as usize],
            interface,
        })
    }

    pub fn run_loop<F>(&mut self, cb: F)
    where
        F: Fn(&[u8]),
    {
        loop {
            self.buf.fill(0);
            let _size = match read(&self.fd, &mut self.buf) {
                Err(why) => panic!("couldn't read {why}"),
                Ok(size) => size,
            };

            let mut offset = 0usize;

            while offset + size_of::<libc::bpf_hdr>() <= self.buf.len() {
                dbg!(offset);

                let buffer = &self.buf;
                let hdr = BPFFrame::parse(buffer.split_at(offset).1).unwrap();

                dbg!(hdr);

                let header_len = hdr.bh_hdrlen.get() as usize;
                let captured_len = hdr.bh_caplen.get() as usize;

                if header_len == 0 || captured_len == 0 {
                    break;
                }

                let Some(packet_start) = offset.checked_add(header_len) else {
                    break;
                };

                let Some(packet_end) = packet_start.checked_add(captured_len) else {
                    break;
                };

                if packet_end > buffer.len() {
                    eprintln!(
                        "invalid BPF record: packet_end={packet_end}, buffer_len={}",
                        buffer.len()
                    );
                    break;
                }

                cb(&buffer[packet_start..packet_end]);

                let record_len = bpf_wordalign(header_len + captured_len);

                if record_len == 0 {
                    break;
                }

                let Some(next_offset) = offset.checked_add(record_len) else {
                    break;
                };

                if next_offset > buffer.len() {
                    break;
                }

                offset = next_offset;
            }
        }
    }
}

#[inline]
pub(in crate::capture) const fn bpf_wordalign(length: usize) -> usize {
    let alignment = libc::BPF_ALIGNMENT as usize;
    (length + alignment - 1) & !(alignment - 1)
}
