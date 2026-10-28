use bytes::{Bytes, BytesMut};
use libc::{BIOCGBLEN, BIOCIMMEDIATE, BIOCSETIF};
use object_pool::Pool;
use std::io;
use std::os::fd::AsRawFd;
use std::sync::Arc;

use crate::capture::tokio::AsyncCapture;
use crate::interface::IFace;
use crate::sys::{
    self,
    bpf::{Bpf, bpf_wordalign},
};
use crate::utils::syscall;

#[cfg(feature = "experimental")]
pub mod tokio;

pub mod parallel;

pub type BufferPool = Arc<Pool<Box<[u8]>>>;

#[derive(Debug)]
pub struct RawPacket(Bytes);

impl RawPacket {
    #[must_use]
    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.0
    }

    #[must_use]
    #[inline]
    pub fn bytes(&self) -> Bytes {
        Bytes::clone(&self.0)
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

        if remaining < size_of::<Bpf>() {
            return None;
        }

        let (hdr, _) = Bpf::parse(&self.data[self.offset..]).ok()?;
        let header_len = hdr.bh_hdrlen.get() as usize;
        let captured_len = hdr.bh_caplen.get() as usize;

        if header_len < size_of::<Bpf>() {
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
    pub(crate) device: sys::Device,
    buf: BytesMut,
    pub(crate) buf_len: usize,
    interface: IFace,
}

impl Capture {
    /// Opens a BPF device and start capturing packets on `ifname`
    ///
    /// Opens `/dev/bpf0`, binds it to the interface, and enables immediate
    /// mode so packets delivered as soon as they arrive instead of
    /// waiting for the kernel buffer to fill. the returned capture is
    /// already [`Active`].
    ///
    /// This is only available on platform with BPF devices (macOS and the BSDs).
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - `/dev/bpf0` cannot be opened. Common causes are missing permissions
    ///   [`std::io::ErrorKind::PermissionDenied`] or the device already being
    ///   in use (`EBUSY`)
    /// - No interface named `ifname`
    ///   found [`std::io::ErrorKind::NotFound`].
    /// - Any of the `ioctl` calls that bind the device to the interface, read
    ///   the buffer size, or enable immediate mode fails.
    pub fn open(ifname: &str) -> io::Result<Self> {
        // let fd = open_bpf_device()?;
        // let fd: OwnedFd = fd.into();
        let device = sys::Device::open()?;

        let raw_fd = device.as_raw_fd();

        let interface = IFace::lookup(ifname)?;

        let ifreq = interface.to_interface_req()?;
        syscall!(ioctl(raw_fd, BIOCSETIF, &ifreq.0))?;

        let mut buf_len: u32 = 0;
        syscall!(ioctl(raw_fd, BIOCGBLEN, &mut buf_len))?;

        let enable: u32 = 1;
        syscall!(ioctl(raw_fd, BIOCIMMEDIATE, &enable))?;

        Ok(Capture {
            device,
            buf: BytesMut::zeroed(buf_len as usize),
            buf_len: buf_len as usize,
            interface,
        })
    }

    #[inline]
    pub fn set_nonblocking(&self, nonblocking: bool) -> io::Result<()> {
        self.device.set_nonblocking(nonblocking)
    }

    pub fn set_interface(&mut self, interface: IFace) -> io::Result<()> {
        let mut ifreq = interface.to_interface_req()?;
        syscall!(ioctl(self.device.as_raw_fd(), BIOCSETIF, &mut ifreq.0)).map(|_| {
            self.interface = interface;
        })
    }

    #[inline]
    pub fn set_interface_by_name(&mut self, name: &str) -> io::Result<()> {
        let interface = IFace::lookup(name)?;

        self.set_interface(interface)
    }

    #[inline]
    pub fn as_async(self) -> io::Result<AsyncCapture> {
        AsyncCapture::from(self)
    }
}

impl Iterator for Capture {
    type Item = CaptureBatch;

    fn next(&mut self) -> Option<Self::Item> {
        self.buf.fill(0);

        let _size = match self.device.read(&mut self.buf) {
            Err(why) => panic!("couldn't read {why}"),
            Ok(size) => size,
        };

        let buf = self.buf.clone().freeze();

        Some(CaptureBatch(buf))
    }
}
