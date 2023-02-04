use crate::{AppDetails, MessageResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::num::ParseFloatError;

/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
///
/// This is intended for specifically decoding to `Vdlm2Message`.
pub trait NewVdlm2Message {
    fn to_vdlm2(&self) -> MessageResult<Vdlm2Message>;
}

/// Implementing `.to_vdlm2()` for the type `String`.
///
/// This does not consume the `String`.
impl NewVdlm2Message for String {
    fn to_vdlm2(&self) -> MessageResult<Vdlm2Message> {
        serde_json::from_str(self)
    }
}

/// Supporting `.to_vdlm2()` for the type `str`.
///
/// This does not consume the `str`.
impl NewVdlm2Message for str {
    fn to_vdlm2(&self) -> MessageResult<Vdlm2Message> {
        serde_json::from_str(self)
    }
}

/// Implementation of `Vdlm2Message`.
impl Vdlm2Message {
    /// Converts `Vdlm2Message` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        serde_json::to_string(self)
    }

    /// Converts `Vdlm2Message` to `String` and appends a `\n` to the end.
    pub fn to_string_newline(&self) -> MessageResult<String> {
        let data = serde_json::to_string(self);
        match data {
            Err(to_string_error) => Err(to_string_error),
            Ok(string) => Ok(format!("{}\n", string)),
        }
    }

    /// Converts `Vdlm2Message` to a `String` encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes()),
        }
    }

    /// Converts `Vdlm2Message` to a `String` terminated with a `\n` and encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes_newline(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string_newline();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes()),
        }
    }

    /// Clears a station name that may be set for `Vdlm2Message`.
    /// ```
    /// use acars_vdlm2_parser::vdlm2::{Vdlm2Body, Vdlm2Message};
    /// let mut new_vdlm2_message: Vdlm2Message = Vdlm2Message { vdl2: Vdlm2Body { station: Some("test_station".to_string()), ..Default::default() } };
    /// assert!(&new_vdlm2_message.vdl2.station.is_some());
    /// new_vdlm2_message.clear_station_name();
    /// assert!(new_vdlm2_message.vdl2.station.is_none());
    /// ```
    pub fn clear_station_name(&mut self) {
        self.vdl2.station = None;
    }

    /// Sets a station name to the provided value for `Vdlm2Message`.
    pub fn set_station_name(&mut self, station_name: &str) {
        self.vdl2.station = Some(station_name.to_string());
    }

    /// Clears any proxy details that may be set for `Vdlm2Message`.
    pub fn clear_proxy_details(&mut self) {
        if let Some(app_details) = self.vdl2.app.as_mut() {
            app_details.remove_proxy();
        }
    }

    /// Sets proxy details to the provided details and sets `proxied` to true.
    ///
    /// This invokes `AppDetails::new()` for `Vdlm2Message` if there is no app block.
    /// This invokes `AppDetails::proxy()` for `Vdlm2Message` if there is an app block to add proxy details.
    pub fn set_proxy_details(&mut self, proxied_by: &str, acars_router_version: &str) {
        match self.vdl2.app.as_mut() {
            None => self.vdl2.app = Some(AppDetails::new(proxied_by, acars_router_version)),
            Some(app_details) => app_details.proxy(proxied_by, acars_router_version),
        }
    }

    pub fn clear_time(&mut self) {
        self.vdl2.t = None;
    }

    pub fn get_time(&self) -> Option<f64> {
        match &self.vdl2.t {
            None => None,
            Some(time_block) => {
                // This will do until there's a more elegant solution found.
                let build_float_string: String = format!("{}.{}", time_block.sec, time_block.usec);
                let parse_f64: Result<f64, ParseFloatError> = build_float_string.parse::<f64>();
                match parse_f64 {
                    Err(_) => None,
                    Ok(value) => Some(value),
                }
            }
        }
    }

    pub fn clear_freq_skew(&mut self) {
        self.vdl2.freq_skew = None;
    }

    pub fn clear_hdr_bits_fixed(&mut self) {
        self.vdl2.hdr_bits_fixed = None;
    }

    pub fn clear_noise_level(&mut self) {
        self.vdl2.noise_level = None;
    }

    pub fn clear_octets_corrected_by_fec(&mut self) {
        self.vdl2.octets_corrected_by_fec = None;
    }

    pub fn clear_sig_level(&mut self) {
        self.vdl2.sig_level = None;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Vdlm2Message {
    pub vdl2: Vdlm2Body,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Vdlm2Body {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppDetails>,
    pub avlc: AvlcData,
    pub burst_len_octets: u16,
    pub freq: u64,
    pub idx: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freq_skew: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hdr_bits_fixed: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_level: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub octets_corrected_by_fec: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig_level: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<TBlock>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct TBlock {
    pub sec: u64,
    pub usec: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AvlcData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cmd: Option<String>,
    pub cr: String,
    pub dst: DstBlock,
    pub frame_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pf: Option<bool>,
    pub src: SrcBlock,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xid: Option<XidBlock>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rseq: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sseq: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub poll: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acars: Option<AvlcAcars>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct DstBlock {
    pub addr: String,
    #[serde(rename = "type")]
    pub vehicle_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct SrcBlock {
    pub addr: String,
    pub status: String,
    #[serde(rename = "type")]
    pub source_type: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct XidBlock {
    pub err: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pub_params: Option<Vec<XidParam>>,
    #[serde(rename = "type")]
    pub xid_type: String,
    #[serde(rename = "type_descr")]
    pub xid_type_descr: String,
    pub vdl_params: Vec<XidParam>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct XidParam {
    pub name: String,
    pub value: ParamValueType,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum ParamValueType {
    String(String),
    VecInteger(Vec<u64>),
    VecString(Vec<String>),
    CoOrdinates(CoOrdinates),
    I32(i32),
    RetrySequence {
        retry: i32,
        seq: i32,
    },
    AltLoc {
        alt: i32,
        loc: CoOrdinates,
    },
    ProtocolViolation {
        cause_code: u16,
        cause_descr: String,
        delay: u16,
        additional_data: Vec<u16>,
    },
    LCRCause {
        cause_code: u16,
        cause_descr: String,
        delay: u16,
    },
    AutoTune {
        freq_mhz: f64,
        modulation_support: Vec<String>,
    },
}

impl Default for ParamValueType {
    fn default() -> Self {
        Self::String("".to_string())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct CoOrdinates {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AvlcAcars {
    pub err: bool,
    pub crc_ok: bool,
    pub more: bool,
    pub reg: String,
    pub mode: String,
    pub label: String,
    pub blk_id: String,
    pub ack: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_num: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub msg_num_seq: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfi: Option<String>,
    pub msg_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arinc622: Option<Arinc622>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Arinc622 {
    pub msg_type: String,
    pub crc_ok: bool,
    pub gs_addr: String,
    pub air_addr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adsc: Option<AdscEntry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpdlc: Option<CPDLC>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AdscEntry {
    pub tags: Vec<Value>,
    pub err: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum AdscTagGroups {
    ReportInterval { interval_secs: u16 },
}

impl Default for AdscTagGroups {
    fn default() -> Self {
        Self::ReportInterval { interval_secs: 0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, Default)]
pub struct AdscWaypoint {
    pub lat: f64,
    pub lon: f64,
    pub alt: i32,
    pub eta_sec: Option<i16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct NonCompMessageGroup {
    pub noncomp_tag: i64,
    pub noncomp_cause: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct AdscEventData {
    pub alt: i64,
    pub lat: f64,
    pub lon: f64,
    pub ts_sec: f64,
    pub tcas_avail: bool,
    pub nav_redundancy: bool,
    pub pos_accuracy_nm: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct CPDLC {
    pub err: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atc_downlink_msg: Option<ATCDownlinkMsg>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct ATCDownlinkMsg {
    pub header: ATCDownlinkMsgHeader,
    pub atc_downlink_msg_element_id: ATCDownlinkMsgElementID,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct ATCDownlinkTimestamp {
    pub hour: u16,
    pub min: u16,
    pub sec: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct ATCDownlinkData {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ver_num: Option<u16>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct ATCDownlinkMsgHeader {
    pub msg_id: u16,
    pub msg_ref: Option<u16>,
    pub timestamp: ATCDownlinkTimestamp,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Default)]
pub struct ATCDownlinkMsgElementID {
    pub choice_label: String,
    pub choice: String,
    pub data: ATCDownlinkData,
}
