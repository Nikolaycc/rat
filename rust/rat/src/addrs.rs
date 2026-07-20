use std::collections::HashMap;

use crate::utils::{str_to_ifname, syscall};

#[derive(Debug)]
pub struct MacAddr {
    bytes: [u8; 6],
}

impl MacAddr {
    /// Creates a new `MacAddr` struct from the given bytes.
    pub fn new(bytes: [u8; 6]) -> MacAddr {
        MacAddr { bytes }
    }

    pub fn bytes(self) -> [u8; 6] {
        self.bytes
    }
}

impl From<[u8; 6]> for MacAddr {
    fn from(v: [u8; 6]) -> Self {
        MacAddr::new(v)
    }
}

impl std::fmt::Display for MacAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let _ = write!(
            f,
            "{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}:{:<02X}",
            self.bytes[0],
            self.bytes[1],
            self.bytes[2],
            self.bytes[3],
            self.bytes[4],
            self.bytes[5]
        );

        Ok(())
    }
}

#[derive(Debug)]
pub enum SockAddr {
    IpV4(std::net::Ipv4Addr),
    IpV6(std::net::Ipv6Addr),
}

impl std::fmt::Display for SockAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SockAddr::IpV4(addr) => write!(f, "{addr}"),
            SockAddr::IpV6(addr) => write!(f, "{addr}"),
        }
    }
}

impl SockAddr {
    pub fn from_libc_sockaddr(sa: &libc::sockaddr) -> std::io::Result<Self> {
        let family = unsafe { std::ptr::read(sa) };

        match family.sa_family as libc::c_int {
            libc::AF_INET => {
                let sin = unsafe { *(std::ptr::from_ref(sa) as *const libc::sockaddr_in) };
                let bits = u32::from_be(sin.sin_addr.s_addr);
                Ok(SockAddr::IpV4(std::net::Ipv4Addr::from(bits)))
            }
            libc::AF_INET6 => {
                let sin6 = unsafe { *(std::ptr::from_ref(sa) as *const libc::sockaddr_in6) };
                Ok(SockAddr::IpV6(std::net::Ipv6Addr::from(
                    sin6.sin6_addr.s6_addr,
                )))
            }
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::Unsupported,
                "Unsupported Protocol",
            )),
        }
    }
}

#[derive(Debug)]
pub struct InterfaceAddress {
    /// Name of the network interface.
    pub name: String,
    /// Network address of this interface.
    pub address: Option<SockAddr>,
    /// Netmask of this interface.
    pub netmask: Option<SockAddr>,
    /// Broadcast address of this interface, if applicable.
    pub destination: Option<SockAddr>,
}

impl InterfaceAddress {
    pub(crate) fn from_ifaddrs(ifa: &libc::ifaddrs) -> Self {
        let ifa_name = unsafe { std::ffi::CStr::from_ptr(ifa.ifa_name) };
        let ifa_addr = unsafe {
            match ifa.ifa_addr.as_ref() {
                Some(addr) => SockAddr::from_libc_sockaddr(addr),
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid data ifa_addr maybe not exist",
                )),
            }
        };
        let ifa_netmask = unsafe {
            match ifa.ifa_netmask.as_ref() {
                Some(addr) => SockAddr::from_libc_sockaddr(addr),
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid data ifa_netmask maybe not exist",
                )),
            }
        };
        let ifa_destination = unsafe {
            match ifa.ifa_dstaddr.as_ref() {
                Some(addr) => SockAddr::from_libc_sockaddr(addr),
                None => Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Invalid data ifa_dstaddr maybe not exist",
                )),
            }
        };

        return Self {
            name: ifa_name.to_string_lossy().into_owned(),
            address: ifa_addr.ok(),
            netmask: ifa_netmask.ok(),
            destination: ifa_destination.ok(),
        };
    }
}

pub struct InterfaceAddressIterator {
    base: *mut libc::ifaddrs,
    next: *mut libc::ifaddrs,
}

impl Drop for InterfaceAddressIterator {
    fn drop(&mut self) {
        unsafe {
            libc::freeifaddrs(self.base);
        }
    }
}

impl Iterator for InterfaceAddressIterator {
    type Item = InterfaceAddress;
    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match unsafe { self.next.as_ref() } {
            Some(ifaddr) => {
                self.next = ifaddr.ifa_next;
                Some(InterfaceAddress::from_ifaddrs(ifaddr))
            }
            None => None,
        }
    }
}

pub(crate) fn getifaddrs() -> std::io::Result<InterfaceAddressIterator> {
    let mut addrs = std::mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();

    unsafe {
        syscall!(getifaddrs(addrs.as_mut_ptr())).map(|_| InterfaceAddressIterator {
            base: addrs.assume_init(),
            next: addrs.assume_init(),
        })
    }
}

#[derive(Debug)]
pub struct InterfaceMap {
    map: HashMap<String, Vec<InterfaceAddress>>,
}

impl std::fmt::Display for InterfaceMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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

impl InterfaceMap {
    pub fn new() -> std::io::Result<Self> {
        let ifs = getifaddrs()?;

        Ok(InterfaceMap::from_iterator(ifs))
    }

    pub fn from_iterator(ifs: InterfaceAddressIterator) -> Self {
        let mut map = HashMap::<String, Vec<InterfaceAddress>>::new();

        for interface in ifs.into_iter() {
            map.entry(interface.name.clone())
                .or_default()
                .push(interface);
        }

        Self { map }
    }

    pub fn get<N: AsRef<str> + ?Sized>(&self, name: &N) -> Option<&Vec<InterfaceAddress>> {
        self.map.get(name.as_ref())
    }

    pub fn contains_interface<N: AsRef<str> + ?Sized>(&self, name: &N) -> bool {
        self.map.contains_key(name.as_ref())
    }

    pub fn to_interface_req<N: AsRef<str> + ?Sized>(
        &self,
        name: &N,
    ) -> std::io::Result<InterfaceReq> {
        if self.contains_interface(name) {
            let mut ifreq: libc::ifreq = unsafe { std::mem::zeroed() };
            ifreq.ifr_name = str_to_ifname(name.as_ref()).unwrap();

            Ok(InterfaceReq(ifreq))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Interface {} not found", name.as_ref()),
            ))
        }
    }
}

pub struct InterfaceReq(pub libc::ifreq);
