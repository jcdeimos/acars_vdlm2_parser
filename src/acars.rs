use serde::{Serialize, Deserialize};
use crate::{AppDetails, MessageResult};


/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
///
/// This is intended for specifically decoding to `AcarsMessage`.
pub trait NewAcarsMessage {
    fn to_acars(&self) -> MessageResult<AcarsMessage>;
}

/// Implementing `.to_acars()` for the type `String`.
///
/// This does not consume the `String`.
impl NewAcarsMessage for String {
    fn to_acars(&self) -> MessageResult<AcarsMessage> {
        serde_json::from_str(self)
    }
}

/// Supporting `.to_acars()` for the type `str`.
///
/// This does not consume the `str`.
impl NewAcarsMessage for str {
    fn to_acars(&self) -> MessageResult<AcarsMessage> {
        serde_json::from_str(self)
    }
}

impl AcarsMessage {

    /// Converts `AcarsMessage` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        serde_json::to_string(self)
    }

    /// Converts `AcarsMessage` to a `String` encoded as bytes.
    ///
    /// The output is stored returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
        }
    }

    /// Clears a station name that may be set for `AcarsMessage`.
    pub fn clear_station_name(&mut self) {
        self.station_id = None;
    }

    /// Sets a station name to the provided value for `AcarsMessage`.
    pub fn set_station_name(&mut self, station_name: &str) {
        self.station_id = Some(station_name.to_string());
    }

    /// Clears any proxy details that may be set for `AcarsMessage`.
    pub fn clear_proxy_details(&mut self) {
        self.app = None;
    }

    /// Sets proxy details to the provided details and sets `proxied` to true.
    ///
    /// This invokes `AppDetails::new()` for `AcarsMessage` and updates the record.
    pub fn set_proxy_details(&mut self, proxied_by: &str, acars_router_version: &str) {
        self.app = Some(AppDetails::new(proxied_by, acars_router_version));
    }

    pub fn clear_time(&mut self) {
        self.timestamp = None;
    }

    pub fn get_time(&self) -> Option<f64> {
        match &self.timestamp {
            None => None,
            Some(timestamp) => Some(*timestamp)
        }
    }
}

// The following entries have been removed from the below structs as they are explicitly not wanted.
// AcarsMessage:
// pub channel: u16,
// #[serde(skip_serializing_if = "Option::is_none")]
// pub error: Option<u8>,
// #[serde(skip_serializing_if = "Option::is_none")]
// pub level: Option<LevelType>,

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct AcarsMessage {
    pub freq: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppDetails>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assstat: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icao: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toaddr: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_response: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_onground: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ack: Option<AckType>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msgno: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flight: Option<String>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum LevelType {
    I32(i32),
    Float64(f64)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum AckType {
    String(String),
    Bool(bool)
}