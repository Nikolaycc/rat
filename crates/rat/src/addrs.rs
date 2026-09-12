use std::collections::HashMap;
use std::ffi;
use std::ffi::CString;
use std::fmt;
use std::io;
use std::io::ErrorKind;
use std::mem;
use std::net;
use std::net::Ipv6Addr;
use std::ptr;
use std::sync::Arc;
use zerocopy::{Immutable, KnownLayout, TryFromBytes, Unaligned};

use crate::utils::{syscall, syscallu};

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
        Ipv6Addr::from(self.octets).fmt(f)
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
    pub fn from_libc_sockaddr(sa: &libc::sockaddr) -> io::Result<Self> {
        let family = unsafe { ptr::read(sa) };

        match libc::c_int::from(family.sa_family) {
            libc::AF_INET => {
                let sin = unsafe { *(ptr::from_ref(sa).cast::<libc::sockaddr_in>()) };
                let bits = u32::from_be(sin.sin_addr.s_addr);
                Ok(SockAddr::IpV4(net::Ipv4Addr::from(bits)))
            }
            libc::AF_INET6 => {
                let sin6 = unsafe { *(ptr::from_ref(sa).cast::<libc::sockaddr_in6>()) };
                Ok(SockAddr::IpV6(net::Ipv6Addr::from(sin6.sin6_addr.s6_addr)))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Unsupported Protocol",
            )),
        }
    }
}

#[derive(Debug)]
pub struct NetworkInterface {
    pub name: Arc<str>,
    pub index: u32,
}

impl NetworkInterface {
    pub fn from_name(ifname: &str) -> io::Result<Self> {
        let name =
            CString::new(ifname).map_err(|error| io::Error::new(ErrorKind::InvalidInput, error))?;

        let index = syscallu!(if_nametoindex(name.as_ptr()))?;

        Ok(Self {
            name: Arc::from(ifname),
            index,
        })
    }

    #[must_use]
    pub fn to_interface_req(&self) -> io::Result<InterfaceReq> {
        let mut ifreq: libc::ifreq = unsafe { mem::zeroed() };
        ifreq.ifr_name = str_to_ifname(self.name.as_ref())?;

        Ok(InterfaceReq(ifreq))
    }
}

#[derive(Debug)]
pub struct NetworkInterfaceComp {
    /// Network address of this interface.
    pub address: Option<SockAddr>,
    /// Netmask of this interface.
    pub netmask: Option<SockAddr>,
    /// Broadcast address of this interface, if applicable.
    pub destination: Option<SockAddr>,
}

impl NetworkInterfaceComp {
    pub(crate) fn from_ifaddrs(ifa: &libc::ifaddrs) -> (String, Self) {
        let ifa_name = unsafe { ffi::CStr::from_ptr(ifa.ifa_name) };
        let ifa_addr = unsafe {
            match ifa.ifa_addr.as_ref() {
                Some(addr) => SockAddr::from_libc_sockaddr(addr),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid data ifa_addr maybe not exist",
                )),
            }
        };
        let ifa_netmask = unsafe {
            match ifa.ifa_netmask.as_ref() {
                Some(addr) => SockAddr::from_libc_sockaddr(addr),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid data ifa_netmask maybe not exist",
                )),
            }
        };
        let ifa_destination = unsafe {
            match ifa.ifa_dstaddr.as_ref() {
                Some(addr) => SockAddr::from_libc_sockaddr(addr),
                None => Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Invalid data ifa_dstaddr maybe not exist",
                )),
            }
        };

        (
            ifa_name.to_string_lossy().into_owned(),
            Self {
                address: ifa_addr.ok(),
                netmask: ifa_netmask.ok(),
                destination: ifa_destination.ok(),
            },
        )
    }
}

pub struct NetworkInterfaceIterator {
    base: *mut libc::ifaddrs,
    next: *mut libc::ifaddrs,
}

impl Drop for NetworkInterfaceIterator {
    fn drop(&mut self) {
        unsafe {
            libc::freeifaddrs(self.base);
        }
    }
}

impl Iterator for NetworkInterfaceIterator {
    type Item = (String, NetworkInterfaceComp);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match unsafe { self.next.as_ref() } {
            Some(ifaddr) => {
                self.next = ifaddr.ifa_next;
                Some(NetworkInterfaceComp::from_ifaddrs(ifaddr))
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct NetworkInterfaceMap {
    map: HashMap<String, Vec<NetworkInterfaceComp>>,
}

impl fmt::Display for NetworkInterfaceMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (name, addrs) in &self.map {
            writeln!(f, "{name}")?;
            for addr in addrs {
                let label = match addr.address {
                    Some(SockAddr::IpV6(_)) => "inet6",
                    _ => "inet ",
                };
                match &addr.address {
                    Some(ip) => write!(f, "    {label} {ip}")?,
                    None => write!(f, "    {label} <none>")?,
                }
                if let Some(mask) = &addr.netmask {
                    write!(f, "  netmask {mask}")?;
                }
                if let Some(dest) = &addr.destination {
                    write!(f, "  broadcast {dest}")?;
                }
                writeln!(f)?;
            }
        }
        Ok(())
    }
}

impl NetworkInterfaceMap {
    pub fn new() -> io::Result<Self> {
        let ifs = getifaddrs()?;

        Ok(NetworkInterfaceMap::from_iterator(ifs))
    }

    #[must_use]
    pub fn from_iterator(ifs: NetworkInterfaceIterator) -> Self {
        let mut map = HashMap::<String, Vec<NetworkInterfaceComp>>::new();

        for interface in ifs {
            map.entry(interface.0.clone())
                .or_default()
                .push(interface.1);
        }

        Self { map }
    }

    #[inline]
    pub fn get_components<N>(&self, name: &N) -> Option<&Vec<NetworkInterfaceComp>>
    where
        N: AsRef<str> + ?Sized,
    {
        self.map.get(name.as_ref())
    }

    #[inline]
    pub fn get<N>(&self, name: &N) -> Option<(NetworkInterface, &Vec<NetworkInterfaceComp>)>
    where
        N: AsRef<str> + ?Sized,
    {
        let comps = self.map.get(name.as_ref())?;
        let Ok(interface) = NetworkInterface::from_name(name.as_ref()) else {
            return None;
        };

        Some((interface, comps))
    }

    #[inline]
    pub fn contains_interface<N>(&self, name: &N) -> bool
    where
        N: AsRef<str> + ?Sized,
    {
        self.map.contains_key(name.as_ref())
    }
}

pub struct InterfaceReq(pub libc::ifreq);

#[inline]
pub(in crate::addrs) fn getifaddrs() -> io::Result<NetworkInterfaceIterator> {
    let mut addrs = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();

    unsafe {
        syscall!(getifaddrs(addrs.as_mut_ptr())).map(|_| NetworkInterfaceIterator {
            base: addrs.assume_init(),
            next: addrs.assume_init(),
        })
    }
}

// convert in bytes array first next we checking array size if is over 16 we need to return error
// next creating buf and fill with zeros and we iterate with zip bcs zips giving tuple but same value and same address and we can cast into i8
// and return buf
pub(in crate::addrs) fn str_to_ifname(name: &str) -> Result<[i8; 16], io::Error> {
    let bytes = name.as_bytes();

    // must fit with room for the trailing '\0' → max 15 chars
    if bytes.len() >= 16 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "interface name too long",
        ));
    }

    let mut buf = [0i8; 16]; // zero-filled → null terminator is automatic
    for (dst, &src) in buf.iter_mut().zip(bytes) {
        *dst = src.cast_signed(); // u8 → i8, same bits
    }
    Ok(buf)
}
