use serde::{Serialize, Deserialize};
use crate::{AppDetails, MessageResult};

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

    /// Converts `Vdlm2Message` to a `String` encoded as bytes.
    ///
    /// The output is stored returned as a `Vec<u8>`.
    pub fn to_bytes(&self) -> MessageResult<Vec<u8>> {
        let string_conversion: MessageResult<String> = self.to_string();
        match string_conversion {
            Err(conversion_failed) => Err(conversion_failed),
            Ok(string) => Ok(string.into_bytes())
        }
    }

    /// Clears a station name that may be set for `Vdlm2Message`.
    pub fn clear_station_name(&mut self) {
        self.vdl2.station = None;
    }

    /// Sets a station name to the provided value for `Vdlm2Message`.
    pub fn set_station_name(&mut self, station_name: &str) {
        self.vdl2.station = Some(station_name.to_string());
    }

    /// Clears any proxy details that may be set for `Vdlm2Message`.
    pub fn clear_proxy_details(&mut self) {
        self.vdl2.app = None;
    }

    /// Sets proxy details to the provided details and sets `proxied` to true.
    ///
    /// This invokes `AppDetails::new()` for `Vdlm2Message` and updates the record.
    pub fn set_proxy_details(&mut self, proxied_by: &str, acars_router_version: &str) {
        self.vdl2.app = Some(AppDetails::new(proxied_by, acars_router_version));
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Vdlm2Message {
    pub vdl2: Vdlm2Body
}

// The following items have been removed from the below structs as they are currently being explicitly ignored.
//
// Vdlm2Body
// pub freq_skew: f64,
// pub hdr_bits_fixed: u16,
// pub noise_level: f64,
// pub octets_corrected_by_fec: u16,
// pub sig_level: f64,
// pub t: TBlock

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct TBlock {
//     pub sec: u64,
//     pub usec: u64
// }


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Vdlm2Body {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub app: Option<AppDetails>,
    pub avlc: AvlcData,
    pub burst_len_octets: u16,
    pub freq: u64,
    pub idx: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub station: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
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
    pub acars: Option<AvlcAcars>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct DstBlock {
    pub addr: String,
    #[serde(rename = "type")]
    pub vehicle_type: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct SrcBlock {
    pub addr: String,
    pub status: String,
    #[serde(rename = "type")]
    pub source_type: String
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct XidBlock {
    pub err: bool,
    pub pub_params: Vec<XidParam>,
    #[serde(rename = "type")]
    pub xid_type: String,
    #[serde(rename = "type_descr")]
    pub xid_type_descr: String,
    pub vdl_params: Vec<XidParam>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct XidParam {
    pub name: String,
    pub value: ParamValueType
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(untagged)]
pub enum ParamValueType {
    String(String),
    VecInteger(Vec<u64>),
    VecString(Vec<String>),
    CoOrdinates(CoOrdinates),
    I32(i32),
    RetrySequence {
        retry: i32,
        seq: i32
    },
    AltLoc {
        alt: i32,
        loc: CoOrdinates
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct CoOrdinates {
    lat: f64,
    lon: f64
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct AvlcAcars {
    pub err: bool,
    pub crc_ok: bool,
    pub more: bool,
    pub reg: String,
    pub mode: String,
    pub label: String,
    pub blk_id: String,
    pub ack: String,
    pub flight: String,
    pub msg_num: String,
    pub msg_num_seq: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sublabel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mfi: Option<String>,
    pub msg_text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arinc622: Option<Arinc622>
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Arinc622 {
    pub msg_type: String,
    pub crc_ok: bool,
    pub gs_addr: String,
    pub air_addr: String,
    pub adsc: AdscEntry
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct AdscEntry {
    pub tags: Vec<AdscTags>,
    pub err: bool
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "snake_case")]
pub enum AdscTags {
    Ack {
        contract_num: u16
    },
    BasicReport {
        lat: f64,
        lon: f64,
        alt: i64,
        ts_sec: f64,
        pos_accuracy_nm: f64,
        nav_redundancy: bool,
        tcas_avail: bool
    }
}