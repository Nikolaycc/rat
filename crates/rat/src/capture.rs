use crate::addrs::NetworkInterface;
use crate::packets::Packet;
use crate::packets::bpf::BPFFrame;
use bytes::{Bytes, BytesMut};
use libc::{BIOCGBLEN, BIOCIMMEDIATE, BIOCSETIF};
use std::io::ErrorKind;
use std::os::fd::{AsRawFd, OwnedFd};
use std::path::Path;

use crate::addrs::NetworkInterfaceMap;
use crate::io::{open, read};
use crate::utils::syscall;

pub struct RawPacket(pub Bytes);

impl RawPacket {
    #[must_use]
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.0
    }
}

pub struct CaptureBatch(Bytes);

impl IntoIterator for CaptureBatch {
    type Item = RawPacket;
    type IntoIter = CaptureIter;

    fn into_iter(self) -> Self::IntoIter {
        CaptureIter {
            data: self.0.clone(),
            offset: 0usize,
        }
    }
}

pub struct CaptureIter {
    data: Bytes,
    offset: usize,
}

impl Iterator for CaptureIter {
    type Item = RawPacket;

    fn next(&mut self) -> Option<Self::Item> {
        let remaining = self.data.len().checked_sub(self.offset)?;

        if remaining < size_of::<BPFFrame>() {
            return None;
        }

        let hdr = BPFFrame::parse(&self.data[self.offset..]).ok()?;

        let header_len = hdr.bh_hdrlen.get() as usize;
        let captured_len = hdr.bh_caplen.get() as usize;

        if header_len < size_of::<BPFFrame>() {
            return None;
        }

        let packet_start = self.offset.checked_add(header_len)?;

        let packet_end = packet_start.checked_add(captured_len)?;

        if packet_end > self.data.len() {
            return None;
        }

        let packet = self.data.slice(packet_start..packet_end);

        let unaligned_len = header_len.checked_add(captured_len)?;

        let record_len = bpf_wordalign(unaligned_len);

        self.offset = self
            .offset
            .checked_add(record_len)
            .unwrap_or(self.data.len());

        Some(RawPacket(packet))
    }
}

pub struct Capture {
    fd: OwnedFd,
    buf: BytesMut,
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
            buf: BytesMut::zeroed(buflen as usize),
            interface,
        })
    }
}

impl Iterator for Capture {
    type Item = CaptureBatch;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.fill(0);

        let _size = match read(&self.fd, &mut self.buf) {
            Err(why) => panic!("couldn't read {why}"),
            Ok(size) => size,
        };

        let buf = self.buf.clone().freeze();

        Some(CaptureBatch(buf))
    }
}

#[inline]
pub(in crate::capture) const fn bpf_wordalign(length: usize) -> usize {
    let alignment = libc::BPF_ALIGNMENT as usize;
    (length + alignment - 1) & !(alignment - 1)
}
