pub mod addrs;
pub mod capture;
mod io;
pub mod packet;
pub mod parser;
pub mod protocols;
pub mod registry;
pub mod utils;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_of_bpf_frame() {
        // assert_eq!(size_of::<bpf::BPFFrame>(), size_of::<libc::bpf_hdr>());
        // assert_eq!(align_of::<bpf::BPFFrame>(), align_of::<libc::bpf_hdr>());
    }

    #[test]
    fn header_len_defined_with_macro() {
        use bytes::Bytes;
        use packet::Packet;
        use parser::Parser;
        use protocols::{ethernet::EthernetFrame, ipv4::IPv4Frame};
        use registry::ProtocolRegistry;

        let registry = ProtocolRegistry::builder()
            .root::<EthernetFrame>()
            .register::<IPv4Frame>()
            .build()
            .unwrap();

        let parser = Parser::new(registry);

        let packet = Bytes::from_owner(b"&\xf1\x16\xd3A\xf2([\x0c8i\x15\x08\0E\0\0\xae\0\0@\07\x11\xb5\xef\xa2\x9f\xc6\x02\xc0\xa8d\x05\x01\xbb\xc8\xcc\0\x9ai\x81[ \xa6\x0euVjF\x99\x8bK+\xd6\x0feV\xd8\x9fR\xc0i\x8f\xadI|W\xe2\xf0Y\xf7\xe5\x0c\x13\x0c\xb3TM5\x03\x98SCN\xafk\xeaP\xbf\xdc\xf8\"\xd2\xb9%D\x1dR\x06~\xa2h^\xeel\xd2\xd7\x19\xfe\xf6\xed\x14!\x84\x03\x01\x0c\x14\x8c\xc7w,\xd8\xf3\xf4\x839\x8em~\x1c\x8fq*\xc4]JZ\xe8c\x95\xaf\xb37\xf1sOPb\xa9\xc3\xc0\x18L\xba\xaa\xeb5\x05\x8d\xaf\xcb\xa6\x8e\xc1o\x8bE\xae\"\xa7\x89X\xd6\x01\xb6\"$\x9b\xd3\xe5\x08\xf3\xd1B\xf8\x13");

        for layer in parser.parse(packet) {
            let layer = layer.unwrap();

            if let Some(ipv4) = layer.get::<IPv4Frame>() {
                assert_eq!(ipv4.header_len(), ((ipv4.version_ihl & 0x0f) * 4) as usize);
                break;
            }
        }
    }
}
