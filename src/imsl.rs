use serde::{Serialize, Deserialize};
use crate::MessageResult;


/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
///
/// This is intended for specifically decoding to `ImslMessage`.
pub trait NewImslMessage {
    fn to_imsl(&self) -> MessageResult<ImslMessage>;
}

/// Implementing `.to_imsl()` for the type `String`.
///
/// This does not consume the `String`.
impl NewImslMessage for String {
    fn to_imsl(&self) -> MessageResult<ImslMessage> {
        serde_json::from_str(self)
    }
}

/// Supporting `.to_imsl()` for the type `str`.
///
/// This does not consume the `str`.
impl NewImslMessage for str {
    fn to_imsl(&self) -> MessageResult<ImslMessage> {
        serde_json::from_str(self)
    }
}

impl ImslMessage {

    /// Converts `ImslMessage` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        serde_json::to_string(self)
    }
    
    /// Converts `ImslMessage` to `String` and appends a `\n` to the end.
    pub fn to_string_newline(&self) -> MessageResult<String> {
        match serde_json::to_string(self) {
            Err(to_string_error) => Err(to_string_error),
            Ok(string) => Ok(format!("{}\n", string))
        }
    }

    /// Converts `ImslMessage` to a `String` encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        match self.to_string() {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
        }
    }
    
    /// Converts `ImslMessage` to a `String` terminated with a `\n` and encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes_newline(&self) -> MessageResult<Vec<u8>> {
        match self.to_string_newline() {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
        }
    }

    pub fn get_time(&self) -> Option<f64> {
        Some(self.timestamp.as_ref().copied().unwrap() as f64)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Default)]
pub struct ImslMessage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bi: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub has_text: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub more_to_come: Option<bool>,
    pub msg_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plane_reg: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal_unit: Option<SignalUnitBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceBody>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tak: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freq: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct SignalUnitBody {
    pub aes_id: i64,
    pub ges_id: i64,
    pub message_type: i64,
    pub no_of_bytes_in_last_su: i64,
    pub q_no: i64,
    pub ref_no: i64,
    pub seq_no: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct SourceBody {
    pub app: AppBody,
    pub station_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct AppBody {
    pub name: String,
    pub version: String,
}
