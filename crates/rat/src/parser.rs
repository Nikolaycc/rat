use bytes::Bytes;
use std::any::TypeId;
use std::fmt;

use crate::packet::Packet;
use crate::registry::{FormatFn, ProtocolId, ProtocolRegistry};
use crate::utils::ParseError;

pub struct ParsedLayer {
    protocol: ProtocolId,
    type_id: TypeId,

    data: Bytes,
    header_len: usize,

    format: FormatFn,
}

pub struct ParseIter<'a> {
    registry: &'a ProtocolRegistry,
    current: Option<ProtocolId>,
    data: Option<Bytes>,
}

pub struct Parser {
    registry: ProtocolRegistry,
}

impl Parser {
    pub const fn new(registry: ProtocolRegistry) -> Self {
        Self { registry }
    }

    pub fn parse(&self, data: Bytes) -> ParseIter<'_> {
        ParseIter {
            registry: &self.registry,
            current: Some(self.registry.root()),
            data: Some(data),
        }
    }
}

impl ParsedLayer {
    #[inline]
    pub const fn protocol(&self) -> ProtocolId {
        self.protocol
    }

    #[inline]
    pub fn header(&self) -> &[u8] {
        &self.data[..self.header_len]
    }

    #[inline]
    pub fn payload(&self) -> &[u8] {
        &self.data[self.header_len..]
    }

    #[inline]
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    pub fn is<P: Packet>(&self) -> bool {
        self.type_id == TypeId::of::<P>()
    }

    #[inline]
    pub fn get<P: Packet>(&self) -> Option<&P> {
        if !self.is::<P>() {
            return None;
        }

        P::parse(self.header()).ok()
    }
}

impl fmt::Display for ParsedLayer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (self.format)(self.header(), f)
    }
}

impl Iterator for ParseIter<'_> {
    type Item = Result<ParsedLayer, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        let protocol = self.current?;

        let data = self.data.take()?;

        let descriptor = self
            .registry
            .protocol(protocol)
            .expect("invalid protocol id in registry");

        let parse = descriptor.parse;
        let format = descriptor.format;
        let type_id = descriptor.type_id;

        let meta = match parse(&data) {
            Ok(meta) => meta,
            Err(error) => {
                self.current = None;
                self.data = None;

                return Some(Err(error));
            }
        };

        if meta.header_len > meta.packet_len || meta.packet_len > data.len() {
            self.current = None;
            self.data = None;

            return Some(Err(ParseError::InvalidValue));
        }

        let next_protocol = meta
            .next
            .and_then(|selector| self.registry.resolve(protocol, selector));

        let layer_data = data.slice(..meta.packet_len);

        let next_data = next_protocol.map(|_| data.slice(meta.header_len..meta.packet_len));

        self.current = next_protocol;
        self.data = next_data;

        Some(Ok(ParsedLayer {
            protocol,
            type_id,
            data: layer_data,
            header_len: meta.header_len,
            format,
        }))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parser() {}
}
