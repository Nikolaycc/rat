use std::collections::HashMap;
use std::{
    any::{TypeId, type_name},
    fmt,
};
use thiserror::Error;

use crate::packet::Packet;
use crate::protocols::{
    arp::ARPFrame, ethernet::EthernetFrame, icmp::ICMPFrame, ipv4::IPv4Frame, ipv6::IPv6Frame,
    ospf::OSPFFrame, tcp::TCPFrame, udp::UDPFrame,
};
use crate::utils::ParseError;

#[derive(Error, Debug)]
pub enum RegistryError {
    #[error("protocol registry supports at most {max}")]
    TooManyProtocols { max: usize },

    #[error("protocol {0} is registred more than once")]
    DuplicateProtocol(String),

    #[error("parent protocol required by '{0}' is not registred")]
    MissingParent(String),

    #[error("protocol registry has no root protocol")]
    MissingRoot,

    #[error("root protocol is registred more than once")]
    MultipleRoots,

    #[error("duplicate route for protocol '{parent}' with selector {selector}")]
    DuplicateRoute {
        parent: ProtocolId,
        selector: ProtocolKey,
    },
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
pub struct ProtocolId(u16);

impl fmt::Display for ProtocolId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:X}", self.0)
    }
}

impl ProtocolId {
    #[must_use]
    #[inline]
    pub const fn index(&self) -> usize {
        self.0 as usize
    }
}

#[derive(Debug, Hash, Eq, PartialEq, PartialOrd, Ord, Clone, Copy)]
#[repr(transparent)]
pub struct ProtocolKey(u32);

impl ProtocolKey {
    #[must_use]
    #[inline]
    pub const fn new(key: u32) -> Self {
        Self(key)
    }

    #[must_use]
    #[inline]
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl fmt::Display for ProtocolKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

pub(crate) type ParseFn = fn(&[u8]) -> Result<ParseMeta, ParseError>;
pub(crate) type FormatFn = fn(&[u8], &mut fmt::Formatter<'_>) -> fmt::Result;

fn parse_protocol<P>(data: &[u8]) -> Result<ParseMeta, ParseError>
where
    P: Packet,
{
    let packet = P::parse(data)?;

    let header_len = packet.header_len();

    let packet_len = packet.packet_len().unwrap_or(data.len());

    if header_len > packet_len || packet_len < data.len() {
        return Err(ParseError::InvalidValue);
    }

    Ok(ParseMeta {
        header_len,
        packet_len,
        next: packet.next_selector().map(ProtocolKey::new),
    })
}

fn format_protocol<P>(data: &[u8], f: &mut fmt::Formatter<'_>) -> fmt::Result
where
    P: Packet + fmt::Display,
{
    let packet = P::parse(data).map_err(|_| fmt::Error)?;

    fmt::Display::fmt(packet, f)
}

pub struct ParseMeta {
    pub header_len: usize,
    pub packet_len: usize,
    pub next: Option<ProtocolKey>,
}

#[derive(Debug, Clone)]
pub(crate) struct PendingProtocol {
    pub type_id: TypeId,
    pub name: String,
    pub parent: Option<TypeId>,
    pub selector: ProtocolKey,
    pub parse_fn: ParseFn,
    pub format_fn: FormatFn,
}

pub struct ProtocolRegistryBuilder {
    root: Option<PendingProtocol>,
    protocols: Vec<PendingProtocol>,
}

impl ProtocolRegistryBuilder {
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            root: None,
            protocols: Vec::new(),
        }
    }

    pub fn new_with_root<P>() -> Self
    where
        P: Packet + fmt::Display,
    {
        if P::parent_type_id().is_some() {
            panic!("Root protocol cannot have a parent protocol")
        }

        Self {
            root: Some(PendingProtocol {
                type_id: TypeId::of::<P>(),
                name: type_name::<P>().to_owned(),
                parent: P::parent_type_id(),
                selector: ProtocolKey(P::SELECTOR),
                parse_fn: parse_protocol::<P>,
                format_fn: format_protocol::<P>,
            }),
            protocols: Vec::new(),
        }
    }

    pub fn root<P>(&mut self) -> &mut Self
    where
        P: Packet + fmt::Display,
    {
        if P::parent_type_id().is_some() {
            panic!("Root protocol cannot have a parent protocol")
        }

        self.protocols.insert(
            0,
            PendingProtocol {
                type_id: TypeId::of::<P>(),
                name: type_name::<P>().to_owned(),
                parent: P::parent_type_id(),
                selector: ProtocolKey(P::SELECTOR),
                parse_fn: parse_protocol::<P>,
                format_fn: format_protocol::<P>,
            },
        );

        self
    }

    pub fn register<P>(&mut self) -> &mut Self
    where
        P: Packet + fmt::Display,
    {
        if P::parent_type_id().is_none() {
            panic!("Protocol must have a parent protocol.")
        }

        self.protocols.push(PendingProtocol {
            type_id: TypeId::of::<P>(),
            name: type_name::<P>().to_owned(),
            parent: P::parent_type_id(),
            selector: ProtocolKey(P::SELECTOR),
            parse_fn: parse_protocol::<P>,
            format_fn: format_protocol::<P>,
        });

        self
    }

    #[inline]
    pub fn defaults(&mut self) -> &mut Self {
        self.root::<EthernetFrame>()
            .register::<IPv4Frame>()
            .register::<IPv6Frame>()
            .register::<ARPFrame>()
            .register::<ICMPFrame>()
            .register::<OSPFFrame>()
            .register::<TCPFrame>()
            .register::<UDPFrame>()
    }

    pub fn build(&mut self) -> Result<ProtocolRegistry, RegistryError> {
        if self.protocols.len() > u16::MAX as usize {
            return Err(RegistryError::TooManyProtocols {
                max: u16::MAX as usize,
            });
        }

        let mut ids = HashMap::with_capacity(self.protocols.len());

        if let Some(ref root) = self.root {
            self.protocols.insert(0, root.to_owned());
        }

        for (index, protocol) in self.protocols.iter().enumerate() {
            let id = ProtocolId(index as u16);

            if ids.insert(protocol.type_id, id).is_some() {
                return Err(RegistryError::DuplicateProtocol(protocol.name.clone()));
            }
        }

        let mut routes_by_parent: Vec<Vec<Route>> =
            (0..self.protocols.len()).map(|_| Vec::new()).collect();

        let mut root = None;

        for (index, protocol) in self.protocols.iter().enumerate() {
            let child_id = ProtocolId(index as u16);

            match protocol.parent {
                None => {
                    if root.is_some() {
                        return Err(RegistryError::MultipleRoots);
                    }

                    root = Some(child_id);
                }
                Some(parent_type_id) => {
                    let parent_id = *ids
                        .get(&parent_type_id)
                        .ok_or(RegistryError::MissingParent(protocol.name.to_owned()))?;

                    let routes = &mut routes_by_parent[parent_id.index()];

                    if routes
                        .iter()
                        .any(|route| route.selector == protocol.selector)
                    {
                        return Err(RegistryError::DuplicateRoute {
                            parent: parent_id,

                            selector: protocol.selector,
                        });
                    }

                    routes.push(Route {
                        selector: protocol.selector,

                        child: child_id,
                    });
                }
            }
        }

        let root = root.ok_or(RegistryError::MissingRoot)?;

        for routes in &mut routes_by_parent {
            routes.sort_unstable_by_key(|route| route.selector);
        }

        let route_count = routes_by_parent.iter().map(Vec::len).sum();

        let mut routes = Vec::with_capacity(route_count);

        let mut descriptors = Vec::with_capacity(self.protocols.len());

        for (index, pending) in self.protocols.iter().enumerate() {
            let protocol_routes = &routes_by_parent[index];

            let route_start = routes.len();

            routes.extend_from_slice(protocol_routes);

            descriptors.push(ProtocolDescriptor {
                type_id: pending.type_id,
                parse: pending.parse_fn,
                format: pending.format_fn,
                route_start,
                route_len: protocol_routes.len(),
            });
        }

        Ok(ProtocolRegistry {
            root,
            protocols: descriptors.into_boxed_slice(),
            routes: routes.into_boxed_slice(),
        })
    }
}

impl Default for ProtocolRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct Route {
    pub selector: ProtocolKey,
    pub child: ProtocolId,
}

// Protocol Descriptor
#[derive(Debug, Clone, Copy)]
pub(crate) struct ProtocolDescriptor {
    pub type_id: TypeId,
    pub parse: ParseFn,
    pub format: FormatFn,
    pub route_start: usize,
    pub route_len: usize,
}

#[derive(Debug)]
pub struct ProtocolRegistry {
    root: ProtocolId,

    protocols: Box<[ProtocolDescriptor]>,
    routes: Box<[Route]>,
}

impl ProtocolRegistry {
    #[must_use]
    pub fn builder() -> ProtocolRegistryBuilder {
        ProtocolRegistryBuilder::new()
    }

    #[inline]
    pub fn root(&self) -> ProtocolId {
        self.root
    }

    #[inline]
    pub(crate) fn protocol(&self, id: ProtocolId) -> Option<&ProtocolDescriptor> {
        self.protocols.get(id.index())
    }

    #[inline]
    pub fn resolve(&self, parent: ProtocolId, selector: ProtocolKey) -> Option<ProtocolId> {
        let descriptor = self.protocol(parent)?;

        let start = descriptor.route_start;
        let end = start + descriptor.route_len;

        self.routes[start..end]
            .iter()
            .find(|route| route.selector == selector)
            .map(|route| route.child)
    }
}

#[cfg(test)]
mod tests {
    use crate::protocols::{
        arp::ARPFrame, ethernet::EthernetFrame, ipv4::IPv4Frame, ipv6::IPv6Frame, tcp::TCPFrame,
    };

    use super::*;

    #[test]
    fn registry_builder_root() {
        let registry = ProtocolRegistryBuilder::new_with_root::<EthernetFrame>()
            .register::<IPv4Frame>()
            .register::<IPv6Frame>()
            .register::<ARPFrame>()
            .register::<TCPFrame>()
            .build()
            .unwrap();

        assert_eq!(registry.root, ProtocolId(0));
        assert_eq!(
            registry.resolve(registry.root(), ProtocolKey::new(0x0800)),
            Some(ProtocolId(1))
        );
    }
}
