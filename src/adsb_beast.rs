use crate::{DeserializatonError, MessageResult};
use bincode;
use serde::{Deserialize, Serialize};

// TODO: Verify an adsb beast packet is 120 bits long
pub const ADSB_BEAST_PACKET_SIZE: usize = 120;

/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
///
/// This is intended for specifically decoding to `ADSBMessage`.
pub trait NewAdsbBeastMessage {
    fn to_adsb_beast(&self) -> MessageResult<AdsbBeastMessage>;
}

/// Implementing `.to_adsb_beast()` for the type `String`.
///
/// This does not consume the `String`.
impl NewAdsbBeastMessage for String {
    fn to_adsb_beast(&self) -> MessageResult<AdsbBeastMessage> {
        match bincode::deserialize(self.as_bytes()) {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializatonError::BoxError(e)),
        }
    }
}

/// Supporting `.to_adsb_beast()` for the type `str`.
///
/// This does not consume the `str`.
impl NewAdsbBeastMessage for str {
    fn to_adsb_beast(&self) -> MessageResult<AdsbBeastMessage> {
        match bincode::deserialize(self.as_bytes()) {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializatonError::BoxError(e)),
        }
    }
}

impl NewAdsbBeastMessage for Vec<u8> {
    fn to_adsb_beast(&self) -> MessageResult<AdsbBeastMessage> {
        println!("whoa");
        match bincode::deserialize(self) {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializatonError::BoxError(e)),
        }
    }
}

impl NewAdsbBeastMessage for [u8] {
    fn to_adsb_beast(&self) -> MessageResult<AdsbBeastMessage> {
        println!("made it here");
        match bincode::deserialize(self) {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializatonError::BoxError(e)),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct AdsbBeastMessage {
    /// Transponder Capability
    pub capability: Capability,
    // ICAO aircraft address
    // pub icao: ICAO,
    // /// Message, extended Squitter
    // pub me: ME,
    // /// Parity/Interrogator ID
    // pub pi: ICAO,
}

/// Transponder level and additional information (3.1.2.5.2.2.1)
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Capability {
    /// Level 1 transponder (surveillance only), and either airborne or on the ground
    AG_UNCERTAIN = 0x00,
    Reserved,
    /// Level 2 or above transponder, on ground
    AG_GROUND = 0x04,
    /// Level 2 or above transponder, airborne
    AG_AIRBORNE = 0x05,
    /// Level 2 or above transponder, either airborne or on ground
    AG_UNCERTAIN2 = 0x06,
    /// DR field is not equal to 0, or fs field equal 2, 3, 4, or 5, and either airborne or on
    /// ground
    AG_UNCERTAIN3 = 0x07,
}

impl Default for Capability {
    fn default() -> Self {
        Capability::AG_UNCERTAIN
    }
}

impl AdsbBeastMessage {
    /// Converts `ADSBsMessage` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        match serde_json::to_string(self) {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializatonError::SerdeError(e)),
        }
    }

    /// Converts `ADSBJsonMessage` to `String` and appends a `\n` to the end.
    pub fn to_string_newline(&self) -> MessageResult<String> {
        match serde_json::to_string(self) {
            Err(to_string_error) => Err(DeserializatonError::SerdeError(to_string_error)),
            Ok(string) => Ok(format!("{}\n", string)),
        }
    }

    /// Converts `ADSBJsonMessage` to a `String` encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        match self.to_string() {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes()),
        }
    }

    /// Converts `ADSBJsonMessage` to a `String` terminated with a `\n` and encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes_newline(&self) -> MessageResult<Vec<u8>> {
        match self.to_string_newline() {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes()),
        }
    }

    pub fn get_time(&self) -> Option<f64> {
        // self.now.as_ref().copied()
        Some(0.0)
    }
}
