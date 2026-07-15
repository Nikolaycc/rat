use crate::addrs::MacAddr;
use crate::utils::ParseError;

// we need here zero copy!!
#[derive(Debug)]
pub struct EthernetFrame {
    pub dest_addr: MacAddr,
    pub source_addr: MacAddr,
    pub ty: u16,
}

impl<'a> TryFrom<&'a [u8]> for EthernetFrame {
    type Error = ParseError;

    fn try_from(data: &'a [u8]) -> Result<Self, Self::Error> {
        const HEADER_SIZE: usize = 14;

        if data.len() < HEADER_SIZE {
            return Err(ParseError::PacketTooShort {
                expected: HEADER_SIZE,
                actual: data.len(),
            });
        }

        let dest_bytes: [u8; 6] = data[0..6]
            .try_into()
            .map_err(|_| ParseError::InvalidValue)?;

        let dest_addr = MacAddr::new(dest_bytes);

        let source_bytes: [u8; 6] = data[6..12]
            .try_into()
            .map_err(|_| ParseError::InvalidValue)?;

        let source_addr = MacAddr::new(source_bytes);

        let ty = u16::from_be_bytes(
            data[12..14]
                .try_into()
                .map_err(|_| ParseError::InvalidValue)?,
        );

        Ok(Self {
            dest_addr,
            source_addr,
            ty,
        })
    }
}
