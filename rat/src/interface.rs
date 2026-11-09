use std::collections::HashMap;
use std::ffi::{self, CString};
use std::fmt;
use std::io::{self, ErrorKind};
use std::mem;

use crate::addrs::SockAddr;
use crate::utils::{syscall, syscallu};

#[derive(Debug)]
pub struct Interface {
    pub name: String,
    pub index: u32,
}

impl Interface {
    /// Look up a network interface by name.
    ///
    /// Resolves `ifname` (for example `"eth0"` or `"lo"`) to its kernel
    /// interface index. The index is read once, so the returned [`Interface`]
    /// can become stale if the interface is removed or recreated later.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    ///
    /// - `ifname` contains an interior NUL Byte
    ///   ([`io::ErrorKind::InvaildInput`]).
    /// - No interface with that name exists, or the lookup fails for another
    ///   reason. The error is the OS error reported by `if_nametoindex`
    ///   (usually `ENODEV`).
    pub fn lookup(ifname: &str) -> io::Result<Self> {
        let name =
            CString::new(ifname).map_err(|error| io::Error::new(ErrorKind::InvalidInput, error))?;

        let index = syscallu!(if_nametoindex(name.as_ptr()))?;

        Ok(Self {
            name: ifname.to_owned(),
            index,
        })
    }

    /// Creates an [`InterfaceReq`] for use with interface `ioctl` calls.
    ///
    /// The returned request has only the interface name filled in and every
    /// other field zeroed, so the caller (or the `ioctl`) sets the rest.
    ///
    /// # Errors
    ///
    /// Returns an error if the interface name cannot be converted to a
    /// C interface name, for example because it is too long to fit in
    /// `ifr_name` (`IFNAMSIZ`) or contains a NUL byte.
    pub fn to_interface_req(&self) -> io::Result<InterfaceReq> {
        // SAFETY: [`libc::ifreq`] is a plain C struct made of integers, byte
        // arrays, and unions of those. The all-zero bit pattern is a valid
        // value for every field, and it is the conventional initial state
        // for an `ifreq`
        let mut ifreq: libc::ifreq = unsafe { mem::zeroed() };
        ifreq.ifr_name = str_to_ifname(self.name.as_ref())?;

        Ok(InterfaceReq(ifreq))
    }
}

#[derive(Debug)]
pub struct InterfaceComp {
    /// Network address of this interface.
    pub address: Option<SockAddr>,
    /// Netmask of this interface.
    pub netmask: Option<SockAddr>,
    /// Broadcast address of this interface, if applicable.
    pub destination: Option<SockAddr>,
}

impl InterfaceComp {
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

pub struct InterfaceIterator {
    base: *mut libc::ifaddrs,
    next: *mut libc::ifaddrs,
}

impl Drop for InterfaceIterator {
    fn drop(&mut self) {
        unsafe {
            libc::freeifaddrs(self.base);
        }
    }
}

impl Iterator for InterfaceIterator {
    type Item = (String, InterfaceComp);

    fn next(&mut self) -> Option<<Self as Iterator>::Item> {
        match unsafe { self.next.as_ref() } {
            Some(ifaddr) => {
                self.next = ifaddr.ifa_next;
                Some(InterfaceComp::from_ifaddrs(ifaddr))
            }
            None => None,
        }
    }
}

#[derive(Debug)]
pub struct InterfaceMap {
    map: HashMap<String, Vec<InterfaceComp>>,
}

impl fmt::Display for InterfaceMap {
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

impl InterfaceMap {
    /// Creates an [`InterfaceMap`] for display network interface list.
    ///
    /// Caller gives a list of network interface with components.
    ///
    /// # Errors
    ///
    /// Returns an error if the lookup fails for another reason.
    /// The error is the OS error reported by `getifaaddrs`.
    pub fn new() -> io::Result<Self> {
        let ifs = getifaddrs()?;

        Ok(InterfaceMap::from_iterator(ifs))
    }

    #[must_use]
    pub fn from_iterator(ifs: InterfaceIterator) -> Self {
        let mut map = HashMap::<String, Vec<InterfaceComp>>::new();

        for interface in ifs {
            map.entry(interface.0.clone())
                .or_default()
                .push(interface.1);
        }

        Self { map }
    }

    #[inline]
    #[must_use]
    pub fn get_components(&self, name: &str) -> Option<&Vec<InterfaceComp>> {
        self.map.get(name)
    }

    #[inline]
    #[must_use]
    pub fn get(&self, name: &str) -> Option<(Interface, &Vec<InterfaceComp>)> {
        let comps = self.map.get(name)?;
        let Ok(interface) = Interface::lookup(name) else {
            return None;
        };

        Some((interface, comps))
    }

    #[inline]
    #[must_use]
    pub fn contains_interface(&self, name: &str) -> bool {
        self.map.contains_key(name)
    }
}

pub struct InterfaceReq(pub libc::ifreq);

#[inline]
pub(crate) fn getifaddrs() -> io::Result<InterfaceIterator> {
    let mut addrs = mem::MaybeUninit::<*mut libc::ifaddrs>::uninit();

    unsafe {
        syscall!(getifaddrs(addrs.as_mut_ptr())).map(|_| InterfaceIterator {
            base: addrs.assume_init(),
            next: addrs.assume_init(),
        })
    }
}

// convert in bytes array first next we checking array size if is over 16 we need to return error
// next creating buf and fill with zeros and we iterate with zip bcs zips giving tuple but same value and same address and we can cast into i8
// and return buf
pub(crate) fn str_to_ifname(name: &str) -> Result<[i8; 16], io::Error> {
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
