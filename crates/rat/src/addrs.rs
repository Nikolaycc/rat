use std::fmt;
use std::io;
use std::net;
use std::ptr;
use zerocopy::{Immutable, KnownLayout, TryFromBytes, Unaligned};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromBytes, Immutable, KnownLayout, Unaligned,
)]
#[repr(transparent)]
pub struct IPv6Addr {
    octets: [u8; 16],
}

impl IPv6Addr {
    #[must_use]
    #[inline]
    pub const fn octets(&self) -> &[u8; 16] {
        &self.octets
    }
}

impl fmt::Display for IPv6Addr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        net::Ipv6Addr::from(self.octets).fmt(f)
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromBytes, Immutable, KnownLayout, Unaligned,
)]
#[repr(transparent)]
pub struct IPv4Addr {
    octets: [u8; 4],
}

impl IPv4Addr {
    #[must_use]
    #[inline]
    pub const fn new(a: u8, b: u8, c: u8, d: u8) -> IPv4Addr {
        IPv4Addr {
            octets: [a, b, c, d],
        }
    }

    #[must_use]
    #[inline]
    pub const fn octets(&self) -> [u8; 4] {
        self.octets
    }
}

impl fmt::Display for IPv4Addr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = write!(
            f,
            "{}.{}.{}.{}",
            self.octets[0], self.octets[1], self.octets[2], self.octets[3]
        );

        Ok(())
    }
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Hash, TryFromBytes, Immutable, KnownLayout, Unaligned,
)]
#[repr(transparent)]
pub struct MacAddr {
    octets: [u8; 6],
}

impl MacAddr {
    /// Creates a new `MacAddr` struct from the given bytes.
    #[must_use]
    #[inline]
    pub const fn octets(&self) -> &[u8; 6] {
        &self.octets
    }
}

impl fmt::Display for MacAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let _ = write!(
            f,
            "{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}",
            self.octets[0],
            self.octets[1],
            self.octets[2],
            self.octets[3],
            self.octets[4],
            self.octets[5]
        );

        Ok(())
    }
}

#[derive(Debug)]
pub enum SockAddr {
    IpV4(net::Ipv4Addr),
    IpV6(net::Ipv6Addr),
}

impl fmt::Display for SockAddr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SockAddr::IpV4(addr) => write!(f, "{addr}"),
            SockAddr::IpV6(addr) => write!(f, "{addr}"),
        }
    }
}

impl SockAddr {
    /// Converts a C `sockaddr` into a [`SockAddr`].
    ///
    /// Supports `AF_INET` (IPv4) and `AF_INET6` (IPv6). The port, flow info,
    /// and scope ID are ignored; only the IP address is extracted.
    ///
    /// # Errors
    ///
    /// Returns an [`io::ErrorKind::Unsupported`] error if `sa.sa_family` is
    /// neither `AF_INET` nor `AF_INET6`.
    ///
    /// # Safety
    ///
    /// `sa` must point to a fully initialized socket address structure that
    /// is at least as large as the type its `sa_family` field indicates:
    /// `sockaddr_in` for `AF_INET` and `sockaddr_in6` for `AF_INET6`.
    ///
    /// Because `&libc::sockaddr` only guarantees 16 bytes, the caller
    /// typically obtains `sa` by casting from a `sockaddr_in6`,
    /// `sockadd_storage`, or a buffer filled by the OS (for example by
    /// `getsockname` or `recvfrom`).
    pub unsafe fn from_libc_sockaddr(sa: &libc::sockaddr) -> io::Result<Self> {
        match libc::c_int::from(sa.sa_family) {
            libc::AF_INET => {
                let sin_ptr = ptr::from_ref(sa).cast::<libc::sockaddr_in>();

                // SAFETY: `sa_family == AF_INET`, so caller guarantees
                // (see `# Safety`) that `sa` points to a full `sockaddr_in`.
                // `read_unaligned` is used because `sockaddr` has a weaker
                // alignment than `sockaddr_in`.
                let sin = unsafe { ptr::read_unaligned(sin_ptr) };

                let bits = u32::from_be(sin.sin_addr.s_addr);
                Ok(SockAddr::IpV4(net::Ipv4Addr::from(bits)))
            }
            libc::AF_INET6 => {
                let sin6_ptr = ptr::from_ref(sa).cast::<libc::sockaddr_in6>();

                // SAFETY: `sa_family == AF_INET6`, so caller guarantees
                // (see `# Safety`) that `sa` points to a full `sockaddr_in6`,
                // which is larger than `sockaddr`. `read_unaligned` is used
                // for the same alignment reason as above.
                let sin6 = unsafe { ptr::read_unaligned(sin6_ptr) };

                Ok(SockAddr::IpV6(net::Ipv6Addr::from(sin6.sin6_addr.s6_addr)))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Unsupported Protocol",
            )),
        }
    }
}
