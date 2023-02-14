use crate::error_handling::adsb_raw_error::ADSBRawError;
use deku::error::DekuError;
use hex::FromHexError;
use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum DeserializationError {
    SerdeError(serde_json::error::Error),
    DekuError(deku::error::DekuError),
    HexError(FromHexError),
    ADSBRawError(ADSBRawError),
}

impl std::fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DeserializationError::SerdeError(e) => write!(f, "Serde error: {}", e),
            DeserializationError::DekuError(e) => write!(f, "Deku error: {}", e),
            DeserializationError::HexError(e) => write!(f, "Hex error: {}", e),
            DeserializationError::ADSBRawError(e) => write!(f, "ADSB Raw error: {}", e),
        }
    }
}

impl From<FromHexError> for DeserializationError {
    fn from(value: FromHexError) -> Self {
        DeserializationError::HexError(value)
    }
}

impl From<SerdeError> for DeserializationError {
    fn from(value: SerdeError) -> Self {
        DeserializationError::SerdeError(value)
    }
}

impl From<DekuError> for DeserializationError {
    fn from(value: DekuError) -> Self {
        DeserializationError::DekuError(value)
    }
}

impl From<ADSBRawError> for DeserializationError {
    fn from(value: ADSBRawError) -> Self {
        DeserializationError::ADSBRawError(value)
    }
}
