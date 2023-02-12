use deku::error::DekuError;
use hex::FromHexError;
use serde_json::Error as SerdeError;

#[derive(Debug)]
pub enum DeserializationError {
    SerdeError(serde_json::error::Error),
    // TODO: rename this to something more appropriate
    DekuError(deku::error::DekuError),
    HexError(FromHexError),
    All(serde_json::error::Error, deku::error::DekuError),
}

impl std::fmt::Display for DeserializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            DeserializationError::SerdeError(e) => write!(f, "Serde error: {}", e),
            DeserializationError::DekuError(e) => write!(f, "Deku error: {}", e),
            DeserializationError::HexError(e) => write!(f, "Hex error: {}", e),
            DeserializationError::All(e, e2) => write!(f, "Serde error: {}, Box error: {}", e, e2),
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
