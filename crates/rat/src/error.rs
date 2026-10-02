use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("packet is too short: expected at least {expected} bytes, got {actual}")]
    Truncated { expected: usize, actual: usize },

    #[error("invalid header length")]
    InvalidHeaderLength,

    #[error("invalid packet length")]
    InvalidPacketLength,

    #[error("invalid field value")]
    InvalidValue,

    #[error("packet data is not correctly aligned")]
    InvalidAlignment,
}
