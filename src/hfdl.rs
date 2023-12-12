use std::num::ParseFloatError;

use serde::{Serialize, Deserialize};
use serde_json::Value;
use crate::{AppDetails, MessageResult};


/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
///
/// This is intended for specifically decoding to `HfdlMessage`.
pub trait NewHfdlMessage {
    fn to_hfdl(&self) -> MessageResult<HfdlMessage>;
}

/// Implementing `.to_hfdl()` for the type `String`.
///
/// This does not consume the `String`.
impl NewHfdlMessage for String {
    fn to_hfdl(&self) -> MessageResult<HfdlMessage> {
        serde_json::from_str(self)
    }
}

/// Supporting `.to_hfdl()` for the type `str`.
///
/// This does not consume the `str`.
impl NewHfdlMessage for str {
    fn to_hfdl(&self) -> MessageResult<HfdlMessage> {
        serde_json::from_str(self)
    }
}

impl HfdlMessage {

    /// Converts `HfdlMessage` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        serde_json::to_string(self)
    }


    /// Converts `HfdlMessage` to `String` and appends a `\n` to the end.
    pub fn to_string_newline(&self) -> MessageResult<String> {
        let data = serde_json::to_string(self);
        match data {
            Err(to_string_error) => Err(to_string_error),
            Ok(string) => Ok(format!("{}\n", string))
        }
    }

    /// Converts `HfdlMessage` to a `String` encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
        }
    }


    /// Converts `HfdlMessage` to a `String` terminated with a `\n` and encoded as bytes.
    ///
    /// The output is returned as a `Vec<u8>`.
    pub fn to_bytes_newline(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string_newline();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
        }
    }

    /// Clears a station name that may be set for `HfdlMessage`.
    /// ```
    /// use acars_vdlm2_parser::hfdl::{HfdlBody, HfdlMessage};
    /// let mut new_hfdl_message: HfdlMessage = HfdlMessage { hfdl: HfdlBody { station: Some("test_station".to_string()), ..Default::default() } };
    /// assert!(&new_hfdl_message.hfdl.station.is_some());
    /// new_hfdl_message.clear_station_name();
    /// assert!(new_hfdl_message.hfdl.station.is_none());
    /// ```
    pub fn clear_station_name(&mut self) {
        self.hfdl.station = None;
    }

    /// Sets a station name to the provided value for `HfdlMessage`.
    pub fn set_station_name(&mut self, station_name: &str) {
        self.hfdl.station = Some(station_name.to_string());
    }

    /// Clears any proxy details that may be set for `HfdlMessage`.
    pub fn clear_proxy_details(&mut self) {
        if let Some(app_details) = self.hfdl.app.as_mut() {
            app_details.remove_proxy();
        }
    }

    /// Sets proxy details to the provided details and sets `proxied` to true.
    ///
    /// This invokes `AppDetails::new()` for `HfdlMessage` if there is no app block.
    /// This invokes `AppDetails::proxy()` for `HfdlMessage` if there is an app block to add proxy details.
    pub fn set_proxy_details(&mut self, proxied_by: &str, acars_router_version: &str) {
        match self.hfdl.app.as_mut() {
            None => self.hfdl.app = Some(AppDetails::new(proxied_by, acars_router_version)),
            Some(app_details) => app_details.proxy(proxied_by, acars_router_version)
        }
    }

    pub fn clear_time(&mut self) {
        self.hfdl.t = None;
    }

    pub fn get_time(&self) -> Option<f64> {
        match &self.hfdl.t {
            None => None,
            Some(time_block) => {
                // This will do until there's a more elegant solution found.
                let build_float_string: String = format!("{}.{}", time_block.sec, time_block.usec);
                let parse_f64: Result<f64, ParseFloatError> = build_float_string.parse::<f64>();
                match parse_f64 {
                    Err(_) => None,
                    Ok(value) => Some(value)
                }
            }
        }
    }

    pub fn clear_freq_skew(&mut self) {
        self.hfdl.freq_skew = None;
    }

    pub fn clear_noise_level(&mut self) {
        self.hfdl.noise_level = None;
    }

    pub fn clear_sig_level(&mut self) {
        self.hfdl.sig_level = None;
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct HfdlMessage {
    pub hfdl: HfdlBody,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct HfdlBody {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppDetails>,
    pub freq: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub noise_level: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sig_level: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<TBlock>,
    pub bit_rate: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub freq_skew: Option<f64>,
    pub slot: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lpdu: Option<LPDU>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spdu: Option<SPDU>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct SPDU {
    err: bool,
    src: SPDUorLPDUSource,
    spdu_version: u8,
    rls: bool,
    iso: bool,
    change_note: String,
    frame_index: u16,
    frame_offset: u8,
    min_priority: u8,
    systable_version: u8,
    gs_status: Vec<SPDUGroundStationStatus>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct SPDUGroundStationStatus {
    gs: SPDUorLPDUSource,
    utc_sync: bool,
    freqs: Vec<FreqId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct TBlock {
    pub sec: u64,
    pub usec: u64
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDU {
    err: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    dst: Option<SPDUorLPDUSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    src: Option<SPDUorLPDUSource>,
    #[serde(rename = "type")]
    lpdu_type: LPDUType,
    #[serde(skip_serializing_if = "Option::is_none")]
    ac_info: Option<LPDUAircraftInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hfnpdu: Option<LPDUHfnPdu>,
    #[serde(skip_serializing_if = "Option::is_none")]
    assigned_ac_id: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<LPDUReason>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUReason {
    code: u16,
    descr: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUAcars {
    err: bool,
    crc_ok: bool,
    more: bool,
    reg: String,
    mode: String,
    label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    sublabel: Option<String>,
    blk_id: String,
    ack: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    flight: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_num: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_num_seq: Option<String>,
    msg_text: String,
    #[serde(rename = "media-adv", skip_serializing_if = "Option::is_none")]
    media_advisory: Option<LPDUAcarsMediaAdvisory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mfi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    arinc622: Option<Arinc622>,
    #[serde(skip_serializing_if = "Option::is_none")]
    miam: Option<Miam>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Miam {
    pub single_transfer: MiamSingleTransfer,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct MiamSingleTransfer {
    pub miam_core: MiamCore,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct MiamCore {
    pub version: u8,
    pub pdu_type: u8,
    pub ack: MiamCoreAck
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct MiamCoreAck {
    pub pdu_len: u16,
    pub aircraft_id: String,
    pub msg_ack_num: u16,
    pub ack_xfer_result: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Arinc622 {
    pub msg_type: String,
    pub crc_ok: bool,
    pub gs_addr: String,
    pub air_addr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpdlc: Option<CPDLC>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adsc: Option<ADSC>,
}

// TODO: I think VDLM and HFDL share the same ADSC and CPDLC structures, so this should be moved to a common location.
// Also, I really think this should be enumerated out in to structs/enums instead of using serde_json::Value.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ADSC {
    pub tags: Vec<Value>,
    pub err: bool
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct CPDLC {
    pub err: bool,
    pub atc_uplink_msg: ATCUplinkMsg,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCUplinkMessageElementId {
    pub choice_label: String,
    pub choice: String,
    pub data: ATCData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCData {
    #[serde(skip_serializing_if = "Option::is_none")]
    free_text: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icao_facility_designation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    freq: Option<ATCFreq>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icao_unit_name_freq: Option<ATCIcaoUnitNameFreq>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alt: Option<ATCDataAlt>,
    #[serde(skip_serializing_if = "Option::is_none")]
    alt_alt: Option<Vec<ATCDataBlockAlt>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCDataBlockAlt {
    alt: ATCDataAlt
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCDataAlt {
    choice: String,
    data: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCIcaoUnitNameFreq {
    icao_unit_name: ATCICAOUnitName,
    freq: ATCFreq,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCICAOUnitName {
    #[serde(skip_serializing_if = "Option::is_none")]
    icao_facility_id: Option<ICAOFacilityId>,
    icao_facility_function: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ICAOFacilityId {
    choice: String,
    data: ICAOFacilityIdData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ICAOFacilityIdData {
    #[serde(skip_serializing_if = "Option::is_none")]
    icao_facility_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    icao_facility_designation: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCFreq {
    pub choice: String,
    pub data: ATCFreqData,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCFreqData {
    vhf: ATCFreqDataType,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCFreqDataType {
    val: f64,
    unit: FrequencyLabel
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(try_from = "String")]
pub enum FrequencyLabel {
    #[default]
    MHz,
}

impl TryFrom<String> for FrequencyLabel {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "MHz" => Ok(FrequencyLabel::MHz),
            _ => Err(format!("Unknown FrequencyLabel: {}", value))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCUplinkMsg {
    pub header: ATCUplinkHeader,
    pub atc_uplink_msg_element_id: ATCUplinkMessageElementId
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ATCUplinkHeader {
    msg_id: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    msg_ref: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp: Option<UTCTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUAcarsMediaAdvisory {
    err: bool,
    version: u8,
    current_link: LPDUAcarsMediaAdvisoryLink,
    links_avail: Vec<LPDUACARSMediaAdivsoryLinksAvailble>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUAcarsMediaAdvisoryLink {
    code: String,
    descr: String,
    established: bool,
    time: UTCTime
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUACARSMediaAdivsoryLinksAvailble {
    code: String,
    descr: String,
}

#[derive(Serialize, Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct SPDUorLPDUSource {
    #[serde(rename = "type")]
    source_type: LPDUSrcType,
    id: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    ac_info: Option<LPDUAircraftInfo>,
}

#[derive(Deserialize, Debug, Clone, Eq, PartialEq, Default)]
#[serde(try_from = "String", deny_unknown_fields)]
pub enum LPDUSrcType {
    #[default]
    Aircraft,
    GroundStation
}

// Helper to serialize the enum back to the original string.
impl Serialize for LPDUSrcType {
    fn serialize<S>(&self, serializer: S) -> Result<<S as serde::Serializer>::Ok, <S as serde::Serializer>::Error>
        where
            S: serde::Serializer {
        match self {
            LPDUSrcType::Aircraft => serializer.serialize_str("Aircraft"),
            LPDUSrcType::GroundStation => serializer.serialize_str("Ground station"),
        }
    }
}

impl TryFrom<String> for LPDUSrcType {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "Aircraft" => Ok(LPDUSrcType::Aircraft),
            "Ground station" => Ok(LPDUSrcType::GroundStation),
            _ => Err(format!("Unknown LPDUSrcType: {}", value))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUType {
    name: String,
    id: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUAircraftInfo {
    icao: String
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUHfnPdu {
    err: bool,
    #[serde(rename = "type")]
    lpdu_type: LPDUType,
    #[serde(skip_serializing_if = "Option::is_none")]
    flight_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pos: Option<Position>,
    #[serde(skip_serializing_if = "Option::is_none")]
    utc_time: Option<UTCTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    freq_data: Option<Vec<LPDUFreqData>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    version: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    time: Option<UTCTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    flight_leg_num: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    gs: Option<SPDUorLPDUSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    frequency: Option<FreqId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    freq_search_cnt: Option<LPDUHfnPduCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hfdl_disabled_duration: Option<LPDUHfnPduDisabledCount>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pdu_stats: Option<PDUStats>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_freq_change_cause: Option<LastFreqChangeCause>,
    #[serde(skip_serializing_if = "Option::is_none")]
    acars: Option<LPDUAcars>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LastFreqChangeCause {
    code: u8,
    descr: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PDUStats {
    mpdus_rx_ok_cnt: PDUStatCounts,
    mpdus_rx_err_cnt: PDUStatCounts,
    mpdus_tx_cnt: PDUStatCounts,
    mpdus_delivered_cnt: PDUStatCounts,
    spdus_rx_ok_cnt: u16,
    spdus_missed_cnt: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct PDUStatCounts {
    #[serde(rename = "300bps")]
    three_hundred_bps: u8,
    #[serde(rename = "600bps")]
    six_hundred_bps: u8,
    #[serde(rename = "1200bps")]
    twelve_hundred_bps: u8,
    #[serde(rename = "1800bps")]
    eighteen_hundred_bps: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUHfnPduDisabledCount {
    this_leg: u16,
    prev_leg: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUHfnPduCount {
    cur_leg: u16,
    prev_leg: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct Position {
    lat: f64,
    lon: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct UTCTime {
    hour: u8,
    min: u8,
    sec: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LPDUFreqData {
    gs: SPDUorLPDUSource,
    listening_on_freqs: Vec<FreqId>,
    heard_on_freqs: Vec<FreqId>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct FreqId {
    id: u16,
}