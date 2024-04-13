use serde::{Serialize, Deserialize};
use crate::{AppDetails, MessageResult};


/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
///
/// This is intended for specifically decoding to `ImslMessage`.
pub trait NewIrdmMessage {
    fn to_irdm(&self) -> MessageResult<IrdmMessage>;
}

/// Implementing `.to_irdm()` for the type `String`.
///
/// This does not consume the `String`.
impl NewIrdmMessage for String {
    fn to_irdm(&self) -> MessageResult<IrdmMessage> {
        serde_json::from_str(self)
    }
}

/// Supporting `.to_irdm()` for the type `str`.
///
/// This does not consume the `str`.
impl NewIrdmMessage for str {
    fn to_irdm(&self) -> MessageResult<IrdmMessage> {
        serde_json::from_str(self)
    }
}

impl IrdmMessage {

    /// Converts `IrdmMessage` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        serde_json::to_string(self)
    }
    
    /// Converts `IrdmMessage` to `String` and appends a `\n` to the end.
    pub fn to_string_newline(&self) -> MessageResult<String> {
        match serde_json::to_string(self) {
            Err(to_string_error) => Err(to_string_error),
            Ok(string) => Ok(format!("{}\n", string))
        }
    }

    /// Converts `IrdmMessage` to a `String` encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        match self.to_string() {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
        }
    }
    
    /// Converts `IrdmMessage` to a `String` terminated with a `\n` and encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes_newline(&self) -> MessageResult<Vec<u8>> {
        match self.to_string_newline() {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
        }
    }

    pub fn get_time(&self) -> Option<f64> {
        self.timestamp.as_ref().copied()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Default)]
pub struct IrdmMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
}
