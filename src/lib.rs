extern crate serde;
extern crate serde_json;

use crate::acars::AcarsMessage;
use crate::vdlm2::Vdlm2Message;
use serde::{Deserialize, Serialize};

pub mod acars;
pub mod vdlm2;

/// Common return type for all serialisation/deserialisation functions.
///
/// This serves as a wrapper for `serde_json::Error` as the Error type.
pub type MessageResult<T> = Result<T, serde_json::Error>;

/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
pub trait DecodeMessage {
    fn decode_message(&self) -> MessageResult<AcarsVdlm2Message>;
}

/// Provides functionality for decoding a `String` to `AcarsVdlm2Message`.
///
/// This does not consume the `String`.
impl DecodeMessage for String {
    fn decode_message(&self) -> MessageResult<AcarsVdlm2Message> {
        serde_json::from_str(self)
    }
}

/// Provides functionality for decoding a `str` to `AcarsVdlm2Message`.
///
/// This does not consume the `str`.
impl DecodeMessage for str {
    fn decode_message(&self) -> MessageResult<AcarsVdlm2Message> {
        serde_json::from_str(self)
    }
}

/// Implementation of `AcarsVdlm2Message`.
impl AcarsVdlm2Message {
    /// Converts `AcarsVdlm2Message` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        serde_json::to_string(self)
    }
    
    /// Converts `AcarsVdlm2Message` to `String` and appends a `\n` to the end.
    pub fn to_string_newline(&self) -> MessageResult<String> {
        let data = serde_json::to_string(self);
        match data {
            Err(to_string_error) => Err(to_string_error),
            Ok(string) => Ok(format!("{}\n", string))
        }
    }

    /// Converts `AcarsVdlm2Message` to a `String` encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes()),
        }
    }
    
    /// Converts `AcarsVdlm2Message` to a `String` terminated with a `\n` and encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes_newline(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string_newline();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
        }
    }

    /// Clears a station name that may be set for either `Vdlm2Message` or `AcarsMessage`.
    pub fn clear_station_name(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.clear_station_name(),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.clear_station_name(),
        }
    }

    /// Sets a station name to the provided value for either `Vdlm2Message` or `AcarsMessage`.
    pub fn set_station_name(&mut self, station_name: &str) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.set_station_name(station_name),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.set_station_name(station_name),
        }
    }

    /// Clears any proxy details that may be set for either `Vdlm2Message` or `AcarsMessage`.
    pub fn clear_proxy_details(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.clear_proxy_details(),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.clear_proxy_details(),
        }
    }

    /// Sets proxy details to the provided details and sets `proxied` to true.
    ///
    /// This invokes `AppDetails::new()` for either `Vdlm2Message` or `AcarsMessage` and updates the record.
    pub fn set_proxy_details(
        &mut self,
        proxied_by: &str,
        acars_router_version: &str,
    ) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => {
                vdlm2.set_proxy_details(proxied_by, acars_router_version)
            }
            AcarsVdlm2Message::AcarsMessage(acars) => {
                acars.set_proxy_details(proxied_by, acars_router_version)
            }
        }
    }

    /// Clears the time details from the message.
    pub fn clear_time(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.clear_time(),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.clear_time(),
        }
    }

    /// Retrieves the time information from the message.
    pub fn get_time(&self) -> Option<f64> {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) =>
                vdlm2.get_time(),
            AcarsVdlm2Message::AcarsMessage(acars) =>
                acars.get_time(),
        }
    }
    
    /// Clears the `freq_skew` field from a `Vdlm2Message`.
    pub fn clear_freq_skew(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_freq_skew(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `hdr_bits_fixed` field from a `Vdlm2Message`.
    pub fn clear_hdr_bits_fixed(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_hdr_bits_fixed(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `noise_level` field from a `Vdlm2Message`.
    pub fn clear_noise_level(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_noise_level(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `octets_corrected_by_fec` field from a `Vdlm2Message`.
    pub fn clear_octets_corrected_by_fec(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_octets_corrected_by_fec(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `sig_level` field from a `Vdlm2Message`.
    pub fn clear_sig_level(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(vdlm2) => vdlm2.clear_sig_level(),
            AcarsVdlm2Message::AcarsMessage(_) => {}
        }
    }
    
    /// Clears the `channel` field from a `AcarsMessage`.
    pub fn clear_channel(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(_) => {}
            AcarsVdlm2Message::AcarsMessage(acars) => acars.clear_channel()
        }
    }
    
    /// Clears the `error` field from a `AcarsMessage`.
    pub fn clear_error(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(_) => {}
            AcarsVdlm2Message::AcarsMessage(acars) => acars.clear_error()
        }
    }
    
    /// Clears the `level` field from a `AcarsMessage`.
    pub fn clear_level(&mut self) {
        match self {
            AcarsVdlm2Message::Vdlm2Message(_) => {}
            AcarsVdlm2Message::AcarsMessage(acars) => acars.clear_level()
        }
    }
}

/// This will automagically serialise to either a `Vdlm2Message` or `AcarsMessage`.
///
/// This simplifies the handling of messaging by not needing to identify it first.
/// It handles identification by looking at the provided data and seeing which format matches it best.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum AcarsVdlm2Message {
    Vdlm2Message(Vdlm2Message),
    AcarsMessage(AcarsMessage),
}

/// This struct lives here because it is used by both `Vdlm2Message` and `AcarsMessage`.
///
/// This does not normally exist on `AcarsMessage` and has been added as part of the implementation for the acars_router project.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct AppDetails {
    pub name: String,
    pub ver: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acars_router_version: Option<String>,
}

impl AppDetails {
    /// Creates a new instance of `AppDetails` with the provided details.
    /// ```
    /// use acars_vdlm2_parser::AppDetails;
    /// let manual: AppDetails = AppDetails { name: "".to_string(), ver: "".to_string(),proxied: Some(true), proxied_by: Some("test".to_string()), acars_router_version: Some("1.0.4".to_string()) };
    /// let generated: AppDetails = AppDetails::new("test", "1.0.4");
    /// assert_eq!(manual, generated);
    /// ```
    pub fn new(proxied_by: &str, acars_router_version: &str) -> Self {
        Self {
            name: "".to_string(),
            ver: "".to_string(),
            proxied: Some(true),
            proxied_by: Some(proxied_by.to_string()),
            acars_router_version: Some(acars_router_version.to_string()),
        }
    }
}
