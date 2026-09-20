use bytes::{Bytes, BytesMut};
use libc::{BIOCGBLEN, BIOCIMMEDIATE, BIOCSETIF};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{ErrorKind, Read};
use std::marker::PhantomData;
use std::os::fd::AsRawFd;

use crate::addrs::NetworkInterface;
use crate::addrs::NetworkInterfaceMap;
use crate::capture::bpf::BPFFrame;
use crate::capture::tokio::AsyncCapture;
use crate::utils::syscall;

#[derive(Debug)]
pub struct RawPacket(pub Bytes);

impl RawPacket {
    #[must_use]
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    #[inline]
    pub fn bytes(&self) -> Bytes {
        self.0.clone()
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
    pub(crate) data: Bytes,
    pub(crate) offset: usize,
}

impl Iterator for CaptureIter {
    type Item = RawPacket;

    fn next(&mut self) -> Option<Self::Item> {
        let remaining = self.data.len().checked_sub(self.offset)?;

        if remaining < size_of::<BPFFrame>() {
            return None;
        }

        let (hdr, _) = BPFFrame::parse(&self.data[self.offset..]).ok()?;

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

pub trait State {}

pub struct Active;
pub struct Inactive;

impl State for Active {}
impl State for Inactive {}

pub struct Capture<S: State> {
    pub(crate) fd: File,
    buf: BytesMut,
    pub(crate) buf_len: usize,
    interface: NetworkInterface,
    _p: PhantomData<S>,
}

impl Capture<Inactive> {
    #[must_use]
    pub fn new(ifname: &str) -> std::io::Result<Capture<Active>> {
        let fd = OpenOptions::new().read(true).open("/dev/bpf0")?;
        // let fd: OwnedFd = fd.into();

        let raw_fd = fd.as_raw_fd();

        let interfaces = NetworkInterfaceMap::new()?;
        let (interface, _) = interfaces
            .get(ifname)
            .ok_or(std::io::Error::from(ErrorKind::NotFound))?;

        let ifreq = interface.to_interface_req()?;
        syscall!(ioctl(raw_fd, BIOCSETIF, &ifreq.0))?;

        let mut buf_len: u32 = 0;
        syscall!(ioctl(raw_fd, BIOCGBLEN, &mut buf_len))?;

        let enable: u32 = 1;
        syscall!(ioctl(raw_fd, BIOCIMMEDIATE, &enable))?;

        Ok(Capture {
            fd,
            buf: BytesMut::zeroed(buf_len as usize),
            buf_len: buf_len as usize,
            interface,
            _p: PhantomData,
        })
    }
}

impl Capture<Active> {
    #[inline]
    pub fn set_interface(&mut self, interface: NetworkInterface) -> std::io::Result<()> {
        let mut ifreq = interface.to_interface_req()?;
        syscall!(ioctl(self.fd.as_raw_fd(), BIOCSETIF, &mut ifreq.0)).map(|_| {
            self.interface = interface;
        })
    }

    #[inline]
    pub fn set_interface_with_name(&mut self, name: &str) -> std::io::Result<()> {
        let interface = NetworkInterface::from_name(name)?;

        self.set_interface(interface)
    }

    #[inline]
    pub fn as_async(self) -> std::io::Result<AsyncCapture> {
        AsyncCapture::from(self)
    }
}

impl Iterator for Capture<Active> {
    type Item = CaptureBatch;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.fill(0);

        let _size = match self.fd.read(&mut self.buf) {
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
