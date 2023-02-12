extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate log;

use deku::prelude::*;
use error_handling::deserialization_error::DeserializationError;
use message_types::acars::AcarsMessage;
use message_types::adsb_json::AdsbJsonMessage;
use message_types::adsb_raw::AdsbRawMessage;
use message_types::vdlm2::Vdlm2Message;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod error_handling;
pub mod message_types;

/// Common return type for all serialisation/deserialisation functions.
///
/// This serves as a wrapper for `serde_json::Error` as the Error type.
pub type MessageResult<T> = Result<T, DeserializationError>;

/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
pub trait DecodeMessage {
    fn decode_message(&self) -> MessageResult<DecodedMessage>;
}

/// Provides functionality for decoding a `String` to `DecodedMessage`.
///
/// This does not consume the `String`.
impl DecodeMessage for String {
    fn decode_message(&self) -> MessageResult<DecodedMessage> {
        let serde = serde_json::from_str(self);

        if !serde.is_err() {
            return serde.map_err(|e| DeserializationError::SerdeError(e));
        }

        //let bincode = bincode::deserialize::<DecodedMessage>(self.as_bytes());
        let deku_decoded = AdsbRawMessage::from_bytes((self.as_bytes(), 0));

        if !deku_decoded.is_err() {
            // FIXME: This is hardly idiomatic Rust, but I just want it to work
            let (_, deku_decoded) = deku_decoded.unwrap();
            return MessageResult::Ok(DecodedMessage::AdsbRawMessage(deku_decoded));
        }

        Err(DeserializationError::All(
            serde.unwrap_err(),
            deku_decoded.unwrap_err(),
        ))
    }
}

/// Provides functionality for decoding a `str` to `DecodedMessage`.
///
/// This does not consume the `str`.
impl DecodeMessage for str {
    fn decode_message(&self) -> MessageResult<DecodedMessage> {
        let serde = serde_json::from_str(self);

        if !serde.is_err() {
            return serde.map_err(|e| DeserializationError::SerdeError(e));
        }

        //let bincode = bincode::deserialize::<DecodedMessage>(self.as_bytes());

        let deku_decoded = AdsbRawMessage::from_bytes((self.as_bytes(), 0));

        if !deku_decoded.is_err() {
            // FIXME: This is hardly idiomatic Rust, but I just want it to work
            let (_, deku_decoded) = deku_decoded.unwrap();
            return MessageResult::Ok(DecodedMessage::AdsbRawMessage(deku_decoded));
        }

        Err(DeserializationError::All(
            serde.unwrap_err(),
            deku_decoded.unwrap_err(),
        ))
    }
}

impl DecodeMessage for Vec<u8> {
    fn decode_message(&self) -> MessageResult<DecodedMessage> {
        let serde = serde_json::from_slice(self);

        if !serde.is_err() {
            return serde.map_err(|e| DeserializationError::SerdeError(e));
        }

        //let bincode = bincode::deserialize::<DecodedMessage>(self);

        let deku_decoded = AdsbRawMessage::from_bytes((self, 0));

        if !deku_decoded.is_err() {
            // FIXME: This is hardly idiomatic Rust, but I just want it to work
            let (_, deku_decoded) = deku_decoded.unwrap();
            return MessageResult::Ok(DecodedMessage::AdsbRawMessage(deku_decoded));
        }

        Err(DeserializationError::All(
            serde.unwrap_err(),
            deku_decoded.unwrap_err(),
        ))
    }
}

/// Implementation of `DecodedMessage`.
impl DecodedMessage {
    /// Converts `DecodedMessage` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        trace!("Converting {:?} to a string", &self);
        match serde_json::to_string(self) {
            Err(to_string_error) => Err(DeserializationError::SerdeError(to_string_error)),
            Ok(string) => Ok(string),
        }
    }

    /// Converts `DecodedMessage` to `String` and appends a `\n` to the end.
    pub fn to_string_newline(&self) -> MessageResult<String> {
        trace!("Converting {:?} to a string and appending a newline", &self);
        match serde_json::to_string(self) {
            Err(to_string_error) => Err(DeserializationError::SerdeError(to_string_error)),
            Ok(string) => Ok(format!("{}\n", string)),
        }
    }

    /// Converts `DecodedMessage` to a `String` encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        trace!("Converting {:?} into a string and encoding as bytes", &self);
        match self.to_string() {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes()),
        }
    }

    /// Converts `DecodedMessage` to a `String` terminated with a `\n` and encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes_newline(&self) -> MessageResult<Vec<u8>> {
        trace!(
            "Converting {:?} into a string, appending a newline and encoding as bytes",
            &self
        );
        match self.to_string_newline() {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes()),
        }
    }

    /// Clears a station name that may be set for either `Vdlm2Message` or `AcarsMessage`.
    pub fn clear_station_name(&mut self) {
        trace!("Clearing the station name for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.clear_station_name(),
            DecodedMessage::AcarsMessage(acars) => acars.clear_station_name(),
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Sets a station name to the provided value for either `Vdlm2Message` or `AcarsMessage`.
    pub fn set_station_name(&mut self, station_name: &str) {
        trace!(
            "Setting the station name to {} for {:?}",
            station_name,
            &self
        );
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.set_station_name(station_name),
            DecodedMessage::AcarsMessage(acars) => acars.set_station_name(station_name),
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Clears any proxy details that may be set for either `Vdlm2Message` or `AcarsMessage`.
    pub fn clear_proxy_details(&mut self) {
        trace!("Clearing the proxy details for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.clear_proxy_details(),
            DecodedMessage::AcarsMessage(acars) => acars.clear_proxy_details(),
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Sets proxy details to the provided details and sets `proxied` to true.
    ///
    /// This invokes `AppDetails::new()` for either `Vdlm2Message` or `AcarsMessage` and updates the record.
    pub fn set_proxy_details(&mut self, proxied_by: &str, acars_router_version: &str) {
        trace!(
            "Setting the proxy details for {:?} to include proxy {} and router version {}",
            &self,
            proxied_by,
            acars_router_version
        );
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => {
                vdlm2.set_proxy_details(proxied_by, acars_router_version)
            }
            DecodedMessage::AcarsMessage(acars) => {
                acars.set_proxy_details(proxied_by, acars_router_version)
            }
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Clears the time details from the message.
    pub fn clear_time(&mut self) {
        trace!("Clearing the time for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.clear_time(),
            DecodedMessage::AcarsMessage(acars) => acars.clear_time(),
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Retrieves the time information from the message.
    pub fn get_time(&self) -> Option<f64> {
        trace!("Getting the time from {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.get_time(),
            DecodedMessage::AcarsMessage(acars) => acars.get_time(),
            DecodedMessage::AdsbJsonMessage(adsb_json) => adsb_json.get_time(),
            DecodedMessage::AdsbRawMessage(adsb_raw) => adsb_raw.get_time(),
        }
    }

    /// Clears the `freq_skew` field from a `Vdlm2Message`.
    pub fn clear_freq_skew(&mut self) {
        trace!("Clearing the frequency skew for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.clear_freq_skew(),
            DecodedMessage::AcarsMessage(_) => {}
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Clears the `hdr_bits_fixed` field from a `Vdlm2Message`.
    pub fn clear_hdr_bits_fixed(&mut self) {
        trace!("Clearing the hdr bits fixed for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.clear_hdr_bits_fixed(),
            DecodedMessage::AcarsMessage(_) => {}
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Clears the `noise_level` field from a `Vdlm2Message`.
    pub fn clear_noise_level(&mut self) {
        trace!("Clearing the noise level for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.clear_noise_level(),
            DecodedMessage::AcarsMessage(_) => {}
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Clears the `octets_corrected_by_fec` field from a `Vdlm2Message`.
    pub fn clear_octets_corrected_by_fec(&mut self) {
        trace!("Clearing the octets corrected by fec for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.clear_octets_corrected_by_fec(),
            DecodedMessage::AcarsMessage(_) => {}
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Clears the `sig_level` field from a `Vdlm2Message`.
    pub fn clear_sig_level(&mut self) {
        trace!("Clearing the signal level for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(vdlm2) => vdlm2.clear_sig_level(),
            DecodedMessage::AcarsMessage(_) => {}
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Clears the `channel` field from a `AcarsMessage`.
    pub fn clear_channel(&mut self) {
        trace!("Clearing the channel for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(_) => {}
            DecodedMessage::AcarsMessage(acars) => acars.clear_channel(),
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Clears the `error` field from a `AcarsMessage`.
    pub fn clear_error(&mut self) {
        trace!("Clearing the error field for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(_) => {}
            DecodedMessage::AcarsMessage(acars) => acars.clear_error(),
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }

    /// Clears the `level` field from a `AcarsMessage`.
    pub fn clear_level(&mut self) {
        trace!("Clearing the level field for {:?}", &self);
        match self {
            DecodedMessage::Vdlm2Message(_) => {}
            DecodedMessage::AcarsMessage(acars) => acars.clear_level(),
            DecodedMessage::AdsbJsonMessage(_) => {}
            DecodedMessage::AdsbRawMessage(_) => {}
        }
    }
}

/// This will automagically serialise to either a `Vdlm2Message` or `AcarsMessage`.
///
/// This simplifies the handling of messaging by not needing to identify it first.
/// It handles identification by looking at the provided data and seeing which format matches it best.
#[allow(clippy::large_enum_variant)]
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum DecodedMessage {
    Vdlm2Message(Vdlm2Message),
    AcarsMessage(AcarsMessage),
    AdsbJsonMessage(AdsbJsonMessage),
    AdsbRawMessage(AdsbRawMessage),
}

impl Default for DecodedMessage {
    fn default() -> Self {
        Self::Vdlm2Message(Default::default())
    }
}

/// This struct lives here because it is used by both `Vdlm2Message` and `AcarsMessage`.
///
/// This does not normally exist on `AcarsMessage` and has been added as part of the implementation for the acars_router project.
/// ```
/// use acars_vdlm2_parser::AppDetails;
/// let app_details: AppDetails = AppDetails { name: "test_name".to_string(), ver: "test_ver".to_string(), proxied: None, proxied_by: None, acars_router_version: None, acars_router_uuid: None };
/// let app_details_string: Result<String, serde_json::Error> = serde_json::to_string(&app_details);
/// let expected_result = r#"{"name":"test_name","ver":"test_ver"}"#;
/// assert!(app_details_string.as_ref().is_ok());
/// assert_eq!(app_details_string.as_ref().unwrap(), expected_result, "Was expecting {} but received {}", expected_result, app_details_string.as_ref().unwrap());
/// ```
#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct AppDetails {
    pub name: String,
    pub ver: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proxied_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acars_router_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acars_router_uuid: Option<String>,
}

impl AppDetails {
    /// Creates a new instance of `AppDetails` with the provided details.
    /// ```
    /// use acars_vdlm2_parser::AppDetails;
    /// let manual: AppDetails = AppDetails { name: "".to_string(), ver: "".to_string(), proxied: Some(true), proxied_by: Some("test".to_string()), acars_router_version: Some("1.0.4".to_string()), acars_router_uuid: Some("00000000-0000-0000-0000-000000000000".to_string()) };
    /// let mut generated: AppDetails = AppDetails::new("test", "1.0.4");
    /// generated.acars_router_uuid = Some("00000000-0000-0000-0000-000000000000".to_string());
    /// assert_eq!(manual, generated);
    /// ```
    pub fn new(proxied_by: &str, acars_router_version: &str) -> Self {
        Self {
            name: "".to_string(),
            ver: "".to_string(),
            proxied: Some(true),
            proxied_by: Some(proxied_by.to_string()),
            acars_router_version: Some(acars_router_version.to_string()),
            acars_router_uuid: Some(Uuid::new_v4().to_string()),
        }
    }
    /// Updates an existing entry of `AppDetails` with the provided details.
    /// ```
    /// use acars_vdlm2_parser::AppDetails;
    /// let manual_vdlm2: AppDetails = AppDetails { name: "dumpvdl2".to_string(), ver: "2.2.0".to_string(), proxied: Some(true), proxied_by: Some("acars_router".to_string()), acars_router_version: Some("1.0.12".to_string()), acars_router_uuid: Some("00000000-0000-0000-0000-000000000000".to_string()) };
    /// let mut vdlm2: AppDetails = AppDetails { name: "dumpvdl2".to_string(), ver: "2.2.0".to_string(), proxied: None, proxied_by: None, acars_router_version: None, acars_router_uuid: Some("00000000-0000-0000-0000-000000000000".to_string())  };
    /// let manual_acars: AppDetails = AppDetails { name: "acarsdec". to_string(), ver: "3.7".to_string(), proxied: Some(true), proxied_by: Some("acars_router".to_string()), acars_router_version: Some("1.0.12".to_string()), acars_router_uuid: Some("00000000-0000-0000-0000-000000000000".to_string()) };
    /// let mut acars: AppDetails = AppDetails { name: "acarsdec". to_string(), ver: "3.7".to_string(), proxied: None, proxied_by: None, acars_router_version: None, acars_router_uuid: Some("00000000-0000-0000-0000-000000000000".to_string())  };
    /// vdlm2.proxy("acars_router", "1.0.12");
    /// acars.proxy("acars_router", "1.0.12");
    /// assert_eq!(vdlm2, manual_vdlm2);
    /// assert_eq!(acars, manual_acars);
    /// ```
    pub fn proxy(&mut self, proxied_by: &str, acars_router_version: &str) {
        self.proxied = Some(true);
        self.proxied_by = Some(proxied_by.to_string());
        self.acars_router_version = Some(acars_router_version.to_string());
        if self.acars_router_uuid.is_none() {
            self.acars_router_uuid = Some(Uuid::new_v4().to_string());
        }
    }
    /// Removes the proxy information from an existing `AppDetails`.
    /// ```
    /// use acars_vdlm2_parser::AppDetails;
    /// let mut vdlm2: AppDetails = AppDetails { name: "dumpvdl2".to_string(), ver: "2.2.0".to_string(), proxied: Some(true), proxied_by: Some("acars_router".to_string()), acars_router_version: Some("1.0.12".to_string()), acars_router_uuid: Some("00000000-0000-0000-0000-000000000000".to_string()) };
    /// let manual_vdlm2: AppDetails = AppDetails { name: "dumpvdl2".to_string(), ver: "2.2.0".to_string(), proxied: None, proxied_by: None, acars_router_version: None, acars_router_uuid: None };
    /// let mut acars: AppDetails = AppDetails { name: "acarsdec". to_string(), ver: "3.7".to_string(), proxied: Some(true), proxied_by: Some("acars_router".to_string()), acars_router_version: Some("1.0.12".to_string()), acars_router_uuid: Some("00000000-0000-0000-0000-000000000000".to_string()) };
    /// let manual_acars: AppDetails = AppDetails { name: "acarsdec". to_string(), ver: "3.7".to_string(), proxied: None, proxied_by: None, acars_router_version: None, acars_router_uuid: None };
    /// vdlm2.remove_proxy();
    /// acars.remove_proxy();
    /// assert_eq!(vdlm2, manual_vdlm2);
    /// assert_eq!(acars, manual_acars);
    /// ```
    pub fn remove_proxy(&mut self) {
        self.proxied = None;
        self.proxied_by = None;
        self.acars_router_version = None;
        self.acars_router_uuid = None;
    }
}
