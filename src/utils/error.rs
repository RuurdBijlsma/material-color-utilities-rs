use thiserror::Error;

#[derive(Debug, Error)]
pub enum ColorParseError {
    #[error("Hex string must be 6 or 8 characters (plus optional #)")]
    InvalidLength,

    #[error("Invalid hex characters: {0}")]
    InvalidHex(#[from] std::num::ParseIntError),
}