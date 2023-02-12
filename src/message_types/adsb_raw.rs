// With MASSIVE thanks to https://github.com/rsadsb/adsb_deku

use crate::{DeserializationError, MessageResult};

use core::{
    clone::Clone,
    cmp::PartialEq,
    convert::From,
    f64, fmt,
    fmt::Write,
    fmt::{Debug, Error},
    marker::Copy,
    option::{Option::None, Option::Some},
    prelude::rust_2021::derive,
    result,
    result::Result::Ok,
    stringify, write, writeln,
};
use deku::bitvec::{BitSlice, Msb0};
use deku::prelude::*;
use serde::{Deserialize, Serialize};

/// Trait for performing a decode if you wish to apply it to types other than the defaults done in this library.
///
/// The originating data must be in JSON format and have support for providing a `str`, and will not consume the source.
///
/// This is intended for specifically decoding to `ADSBMessage`.
pub trait NewAdsbRawMessage {
    fn to_adsb_raw(&self) -> MessageResult<AdsbRawMessage>;
}

/// Implementing `.to_adsb_raw()` for the type `String`.
///
/// This does not consume the `String`.
impl NewAdsbRawMessage for String {
    fn to_adsb_raw(&self) -> MessageResult<AdsbRawMessage> {
        match AdsbRawMessage::from_bytes((self.as_bytes().as_ref(), 0)) {
            Ok((_, v)) => Ok(v),
            Err(e) => Err(DeserializationError::DekuError(e)),
        }
    }
}

/// Supporting `.to_adsb_raw()` for the type `str`.
///
/// This does not consume the `str`.
impl NewAdsbRawMessage for str {
    fn to_adsb_raw(&self) -> MessageResult<AdsbRawMessage> {
        match AdsbRawMessage::from_bytes((self.as_bytes().as_ref(), 0)) {
            Ok((_, v)) => Ok(v),
            Err(e) => Err(DeserializationError::DekuError(e)),
        }
    }
}

impl NewAdsbRawMessage for Vec<u8> {
    fn to_adsb_raw(&self) -> MessageResult<AdsbRawMessage> {
        match AdsbRawMessage::from_bytes((self.as_ref(), 0)) {
            Ok((_, v)) => Ok(v),
            Err(e) => Err(DeserializationError::DekuError(e)),
        }
    }
}

impl NewAdsbRawMessage for [u8] {
    fn to_adsb_raw(&self) -> MessageResult<AdsbRawMessage> {
        match AdsbRawMessage::from_bytes((self.as_ref(), 0)) {
            Ok((_, v)) => Ok(v),
            Err(e) => Err(DeserializationError::DekuError(e)),
        }
    }
}

/// Downlink ADS-B Packet
#[derive(Debug, PartialEq, DekuRead, Clone, Serialize, Deserialize)]

pub struct AdsbRawMessage {
    /// Starting with 5 bit identifier, decode packet
    pub df: DF,
    /// Calculated from all bits, used as ICAO for Response packets
    #[deku(reader = "Self::read_crc(df, deku::input_bits)")]
    pub crc: u32,
}

impl AdsbRawMessage {
    /// Read rest as CRC bits
    fn read_crc<'b>(
        df: &DF,
        rest: &'b BitSlice<u8, Msb0>,
    ) -> result::Result<(&'b BitSlice<u8, Msb0>, u32), DekuError> {
        const MODES_LONG_MSG_BYTES: usize = 14;
        const MODES_SHORT_MSG_BYTES: usize = 7;

        let bit_len = if let Ok(id) = df.deku_id() {
            if id & 0x10 != 0 {
                MODES_LONG_MSG_BYTES * 8
            } else {
                MODES_SHORT_MSG_BYTES * 8
            }
        } else {
            // In this case, it's the DF::CommD, which has multiple ids
            MODES_LONG_MSG_BYTES * 8
        };

        let (_, remaining_bytes, _) = rest.domain().region().unwrap();
        let crc = modes_checksum(remaining_bytes, bit_len)?;
        Ok((rest, crc))
    }
}

/// Downlink Format (3.1.2.3.2.1.2)
///
/// Starting with 5 bits, decode the rest of the message as the correct data packets
#[derive(Debug, PartialEq, DekuRead, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "5")]
pub enum DF {
    /// 17: Extended Squitter, Downlink Format 17 (3.1.2.8.6)
    ///
    /// Civil aircraft ADS-B message
    #[deku(id = "17")]
    ADSB(Adsb),

    /// 11: (Mode S) All-call reply, Downlink format 11 (2.1.2.5.2.2)
    #[deku(id = "11")]
    AllCallReply {
        /// CA: Capability
        capability: Capability,
        /// AA: Address Announced
        icao: ICAO,
        /// PI: Parity/Interrogator identifier
        p_icao: ICAO,
    },

    /// 0: (Mode S) Short Air-Air Surveillance, Downlink Format 0 (3.1.2.8.2)
    #[deku(id = "0")]
    ShortAirAirSurveillance {
        /// VS: Vertical Status
        #[deku(bits = "1")]
        vs: u8,
        /// CC:
        #[deku(bits = "1")]
        cc: u8,
        /// Spare
        #[deku(bits = "1")]
        unused: u8,
        /// SL: Sensitivity level, ACAS
        #[deku(bits = "3")]
        sl: u8,
        /// Spare
        #[deku(bits = "2")]
        unused1: u8,
        /// RI: Reply Information
        #[deku(bits = "4")]
        ri: u8,
        /// Spare
        #[deku(bits = "2")]
        unused2: u8,
        /// AC: altitude code
        altitude: AC13Field,
        /// AP: address, parity
        parity: ICAO,
    },

    /// 4: (Mode S) Surveillance Altitude Reply, Downlink Format 4 (3.1.2.6.5)
    #[deku(id = "4")]
    SurveillanceAltitudeReply {
        /// FS: Flight Status
        fs: FlightStatus,
        /// DR: DownlinkRequest
        dr: DownlinkRequest,
        /// UM: Utility Message
        um: UtilityMessage,
        /// AC: AltitudeCode
        ac: AC13Field,
        /// AP: Address/Parity
        ap: ICAO,
    },

    /// 5: (Mode S) Surveillance Identity Reply (3.1.2.6.7)
    #[deku(id = "5")]
    SurveillanceIdentityReply {
        /// FS: Flight Status
        fs: FlightStatus,
        /// DR: Downlink Request
        dr: DownlinkRequest,
        /// UM: UtilityMessage
        um: UtilityMessage,
        /// ID: Identity
        id: IdentityCode,
        /// AP: Address/Parity
        ap: ICAO,
    },

    /// 16: (Mode S) Long Air-Air Surveillance Downlink Format 16 (3.1.2.8.3)
    #[deku(id = "16")]
    LongAirAir {
        #[deku(bits = "1")]
        vs: u8,
        #[deku(bits = "2")]
        spare1: u8,
        #[deku(bits = "3")]
        sl: u8,
        #[deku(bits = "2")]
        spare2: u8,
        #[deku(bits = "4")]
        ri: u8,
        #[deku(bits = "2")]
        spare3: u8,
        /// AC: altitude code
        altitude: AC13Field,
        /// MV: message, acas
        #[deku(count = "7")]
        mv: Vec<u8>,
        /// AP: address, parity
        parity: ICAO,
    },

    /// 18: Extended Squitter/Supplementary, Downlink Format 18 (3.1.2.8.7)
    ///
    /// Non-Transponder-based ADS-B Transmitting Subsystems and TIS-B Transmitting equipment.
    /// Equipment that cannot be interrogated.
    #[deku(id = "18")]
    TisB {
        /// Enum containing message
        cf: ControlField,
        /// PI: parity/interrogator identifier
        pi: ICAO,
    },

    /// 19: Extended Squitter Military Application, Downlink Format 19 (3.1.2.8.8)
    #[deku(id = "19")]
    ExtendedQuitterMilitaryApplication {
        /// Reserved
        #[deku(bits = "3")]
        af: u8,
    },

    /// 20: COMM-B Altitude Reply (3.1.2.6.6)
    #[deku(id = "20")]
    CommBAltitudeReply {
        /// FS: Flight Status
        flight_status: FlightStatus,
        /// DR: Downlink Request
        dr: DownlinkRequest,
        /// UM: Utility Message
        um: UtilityMessage,
        /// AC: Altitude Code
        alt: AC13Field,
        /// MB Message, Comm-B
        bds: BDS,
        /// AP: address/parity
        parity: ICAO,
    },

    /// 21: COMM-B Reply, Downlink Format 21 (3.1.2.6.8)
    #[deku(id = "21")]
    CommBIdentityReply {
        /// FS: Flight Status
        fs: FlightStatus,
        /// DR: Downlink Request
        dr: DownlinkRequest,
        /// UM: Utility Message
        um: UtilityMessage,
        /// ID: Identity
        #[deku(
            bits = "13",
            endian = "big",
            map = "|squawk: u32| -> Result<_, DekuError> {Ok(decode_id13_field(squawk))}"
        )]
        id: u32,
        /// MB Message, Comm-B
        bds: BDS,
        /// AP address/parity
        parity: ICAO,
    },

    /// 24..=31: Comm-D(ELM), Downlink Format 24 (3.1.2.7.3)
    #[deku(id_pat = "24..=31")]
    CommDExtendedLengthMessage {
        /// Spare - 1 bit
        #[deku(bits = "1")]
        spare: u8,
        /// KE: control, ELM
        ke: KE,
        /// ND: number of D-segment
        #[deku(bits = "4")]
        nd: u8,
        /// MD: message, Comm-D, 80 bits
        #[deku(count = "10")]
        md: Vec<u8>,
        /// AP: address/parity
        parity: ICAO,
    },
}

#[derive(Serialize, Deserialize, Debug, Clone, DekuRead, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Adsb {
    // Transponder Capability
    pub capability: Capability,
    // ICAO aircraft address
    pub icao: ICAO,
    // // Message, extended Squitter
    pub me: ME,
    // // Parity/Interrogator ID
    pub pi: ICAO,
}

/// ICAO Address; Mode S transponder code
#[derive(
    Debug,
    PartialEq,
    Eq,
    PartialOrd,
    DekuRead,
    DekuWrite,
    Hash,
    Copy,
    Clone,
    Ord,
    Serialize,
    Deserialize,
)]
pub struct ICAO(pub [u8; 3]);

impl core::fmt::Display for ICAO {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:02x}", self.0[0])?;
        write!(f, "{:02x}", self.0[1])?;
        write!(f, "{:02x}", self.0[2])?;
        Ok(())
    }
}

impl core::str::FromStr for ICAO {
    type Err = core::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let num = u32::from_str_radix(s, 16)?;
        let bytes = num.to_be_bytes();
        let num = [bytes[1], bytes[2], bytes[3]];
        Ok(Self(num))
    }
}

/// ADS-B Message, 5 first bits are known as Type Code (TC)
///
/// reference: ICAO 9871 (A.2.3.1)
#[derive(Debug, PartialEq, DekuRead, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "5")]
pub enum ME {
    #[deku(id_pat = "9..=18")]
    AirbornePositionBaroAltitude(Altitude),

    #[deku(id = "19")]
    AirborneVelocity(AirborneVelocity),

    #[deku(id = "0")]
    NoPosition([u8; 6]),

    #[deku(id_pat = "1..=4")]
    AircraftIdentification(Identification),

    #[deku(id_pat = "5..=8")]
    SurfacePosition(SurfacePosition),

    #[deku(id_pat = "20..=22")]
    AirbornePositionGNSSAltitude(Altitude),

    #[deku(id = "23")]
    Reserved0([u8; 6]),

    #[deku(id_pat = "24")]
    SurfaceSystemStatus([u8; 6]),

    #[deku(id_pat = "25..=27")]
    Reserved1([u8; 6]),

    #[deku(id = "28")]
    AircraftStatus(AircraftStatus),

    #[deku(id = "29")]
    TargetStateAndStatusInformation(TargetStateAndStatusInformation),

    #[deku(id = "30")]
    AircraftOperationalCoordination([u8; 6]),

    #[deku(id = "31")]
    AircraftOperationStatus(OperationStatus),
}

impl ME {
    /// `to_string` with DF.id() input
    pub(crate) fn to_string(
        &self,
        icao: ICAO,
        address_type: &str,
        capability: Capability,
        is_transponder: bool,
    ) -> result::Result<String, Error> {
        let transponder = match is_transponder {
            true => " ",
            false => " (Non-Transponder) ",
        };
        let mut f = String::new();
        match self {
            ME::NoPosition(_) => {
                writeln!(f, " Extended Squitter{transponder}No position information")?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
            }
            ME::AircraftIdentification(Identification { tc, ca, cn }) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Aircraft identification and category"
                )?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
                writeln!(f, "  Ident:         {cn}")?;
                writeln!(f, "  Category:      {tc}{ca}")?;
            }
            ME::SurfacePosition(..) => {
                writeln!(f, " Extended Squitter{transponder}Surface position")?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
            }
            ME::AirbornePositionBaroAltitude(altitude) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Airborne position (barometric altitude)"
                )?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
                write!(f, "{altitude}")?;
            }
            ME::AirborneVelocity(airborne_velocity) => match &airborne_velocity.sub_type {
                AirborneVelocitySubType::GroundSpeedDecoding(_) => {
                    writeln!(
                        f,
                        " Extended Squitter{transponder}Airborne velocity over ground, subsonic"
                    )?;
                    writeln!(f, "  Address:       {icao} {address_type}")?;
                    writeln!(f, "  Air/Ground:    {capability}")?;
                    writeln!(
                        f,
                        "  GNSS delta:    {}{} ft",
                        airborne_velocity.gnss_sign, airborne_velocity.gnss_baro_diff
                    )?;
                    if let Some((heading, ground_speed, vertical_rate)) =
                        airborne_velocity.calculate()
                    {
                        writeln!(f, "  Heading:       {}", libm::ceil(heading as f64))?;
                        writeln!(
                            f,
                            "  Speed:         {} kt groundspeed",
                            libm::floor(ground_speed)
                        )?;
                        writeln!(
                            f,
                            "  Vertical rate: {} ft/min {}",
                            vertical_rate, airborne_velocity.vrate_src
                        )?;
                    } else {
                        writeln!(f, "  Invalid packet")?;
                    }
                }
                AirborneVelocitySubType::AirspeedDecoding(airspeed_decoding) => {
                    writeln!(
                        f,
                        " Extended Squitter{transponder}Airspeed and heading, subsonic",
                    )?;
                    writeln!(f, "  Address:       {icao} {address_type}")?;
                    writeln!(f, "  Air/Ground:    {capability}")?;
                    writeln!(f, "  IAS:           {} kt", airspeed_decoding.airspeed)?;
                    if airborne_velocity.vrate_value > 0 {
                        writeln!(
                            f,
                            "  Baro rate:     {}{} ft/min",
                            airborne_velocity.vrate_sign,
                            (airborne_velocity.vrate_value - 1) * 64
                        )?;
                    }
                    writeln!(f, "  NACv:          {}", airborne_velocity.nac_v)?;
                }
                AirborneVelocitySubType::Reserved0(_) | AirborneVelocitySubType::Reserved1(_) => {
                    writeln!(
                        f,
                        " Extended Squitter{transponder}Airborne Velocity status (reserved)",
                    )?;
                    writeln!(f, "  Address:       {icao} {address_type}")?;
                }
            },
            ME::AirbornePositionGNSSAltitude(altitude) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Airborne position (GNSS altitude)",
                )?;
                writeln!(f, "  Address:      {icao} {address_type}")?;
                write!(f, "{altitude}")?;
            }
            ME::Reserved0(_) | ME::Reserved1(_) => {
                writeln!(f, " Extended Squitter{transponder}Unknown")?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
            }
            ME::SurfaceSystemStatus(_) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Reserved for surface system status",
                )?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
            }
            ME::AircraftStatus(AircraftStatus {
                emergency_state,
                squawk,
                ..
            }) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Emergency/priority status",
                )?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
                writeln!(f, "  Squawk:        {squawk:x?}")?;
                writeln!(f, "  Emergency/priority:    {emergency_state}")?;
            }
            ME::TargetStateAndStatusInformation(target_info) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Target state and status (V2)",
                )?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
                writeln!(f, "  Target State and Status:")?;
                writeln!(f, "    Target altitude:   MCP, {} ft", target_info.altitude)?;
                writeln!(f, "    Altimeter setting: {} millibars", target_info.qnh)?;
                if target_info.is_heading {
                    writeln!(f, "    Target heading:    {}", target_info.heading)?;
                }
                if target_info.tcas {
                    write!(f, "    ACAS:              operational ")?;
                    if target_info.autopilot {
                        write!(f, "autopilot ")?;
                    }
                    if target_info.vnac {
                        write!(f, "vnav ")?;
                    }
                    if target_info.alt_hold {
                        write!(f, "altitude-hold ")?;
                    }
                    if target_info.approach {
                        write!(f, " approach")?;
                    }
                    writeln!(f)?;
                } else {
                    writeln!(f, "    ACAS:              NOT operational")?;
                }
                writeln!(f, "    NACp:              {}", target_info.nacp)?;
                writeln!(f, "    NICbaro:           {}", target_info.nicbaro)?;
                writeln!(f, "    SIL:               {} (per sample)", target_info.sil)?;
                writeln!(f, "    QNH:               {} millibars", target_info.qnh)?;
            }
            ME::AircraftOperationalCoordination(_) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Aircraft Operational Coordination",
                )?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
            }
            ME::AircraftOperationStatus(OperationStatus::Airborne(opstatus_airborne)) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Aircraft operational status (airborne)",
                )?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
                write!(f, "  Aircraft Operational Status:\n{opstatus_airborne}")?;
            }
            ME::AircraftOperationStatus(OperationStatus::Surface(opstatus_surface)) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Aircraft operational status (surface)",
                )?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
                writeln!(f, "  Air/Ground:    {capability}")?;
                write!(f, "  Aircraft Operational Status:\n {opstatus_surface}")?;
            }
            ME::AircraftOperationStatus(OperationStatus::Reserved(..)) => {
                writeln!(
                    f,
                    " Extended Squitter{transponder}Aircraft operational status (reserved)",
                )?;
                writeln!(f, "  Address:       {icao} {address_type}")?;
            }
        }
        Ok(f)
    }
}

/// [`ME::AirborneVelocity`] && [`AirborneVelocitySubType::GroundSpeedDecoding`]
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct GroundSpeedDecoding {
    pub ew_sign: Sign,
    #[deku(endian = "big", bits = "10")]
    pub ew_vel: u16,
    pub ns_sign: Sign,
    #[deku(endian = "big", bits = "10")]
    pub ns_vel: u16,
}

/// [`ME::AirborneVelocity`] && [`AirborneVelocitySubType::AirspeedDecoding`]
#[derive(Debug, PartialEq, Eq, DekuRead, Clone, Serialize, Deserialize)]
pub struct AirspeedDecoding {
    #[deku(bits = "1")]
    pub status_heading: u8,
    #[deku(endian = "big", bits = "10")]
    pub mag_heading: u16,
    #[deku(bits = "1")]
    pub airspeed_type: u8,
    #[deku(
        endian = "big",
        bits = "10",
        map = "|airspeed: u16| -> result::Result<_, DekuError> {Ok(if airspeed > 0 { airspeed - 1 } else { 0 })}"
    )]
    pub airspeed: u16,
}

/// Aircraft Operational Status Subtype
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "3")]
pub enum OperationStatus {
    #[deku(id = "0")]
    Airborne(OperationStatusAirborne),

    #[deku(id = "1")]
    Surface(OperationStatusSurface),

    #[deku(id_pat = "2..=7")]
    Reserved(#[deku(bits = "5")] u8, [u8; 5]),
}

/// [`ME::AircraftOperationStatus`] && [`OperationStatus`] == 0
///
/// Version 2 support only
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct OperationStatusAirborne {
    /// CC (16 bits)
    pub capability_class: CapabilityClassAirborne,

    /// OM
    pub operational_mode: OperationalMode,

    #[deku(pad_bytes_before = "1")] // reserved: OM last 8 bits (diff for airborne/surface)
    pub version_number: ADSBVersion,

    #[deku(bits = "1")]
    pub nic_supplement_a: u8,

    #[deku(bits = "4")]
    pub navigational_accuracy_category: u8,

    #[deku(bits = "2")]
    pub geometric_vertical_accuracy: u8,

    #[deku(bits = "2")]
    pub source_integrity_level: u8,

    #[deku(bits = "1")]
    pub barometric_altitude_integrity: u8,

    #[deku(bits = "1")]
    pub horizontal_reference_direction: u8,

    #[deku(bits = "1")]
    #[deku(pad_bits_after = "1")] // reserved
    pub sil_supplement: u8,
}

impl fmt::Display for OperationStatusAirborne {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "   Version:            {}", self.version_number)?;
        writeln!(f, "   Capability classes:{}", self.capability_class)?;
        writeln!(f, "   Operational modes: {}", self.operational_mode)?;
        writeln!(f, "   NIC-A:              {}", self.nic_supplement_a)?;
        writeln!(
            f,
            "   NACp:               {}",
            self.navigational_accuracy_category
        )?;
        writeln!(
            f,
            "   GVA:                {}",
            self.geometric_vertical_accuracy
        )?;
        writeln!(
            f,
            "   SIL:                {} (per hour)",
            self.source_integrity_level
        )?;
        writeln!(
            f,
            "   NICbaro:            {}",
            self.barometric_altitude_integrity
        )?;
        if self.horizontal_reference_direction == 1 {
            writeln!(f, "   Heading reference:  magnetic north")?;
        } else {
            writeln!(f, "   Heading reference:  true north")?;
        }
        Ok(())
    }
}

/// [`ME::AircraftOperationStatus`]
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct CapabilityClassAirborne {
    #[deku(bits = "2", assert_eq = "0")]
    pub reserved0: u8,

    /// TCAS Operational
    #[deku(bits = "1")]
    pub acas: u8,

    /// 1090ES IN
    ///
    /// bit 12
    #[deku(bits = "1")]
    pub cdti: u8,

    #[deku(bits = "2", assert_eq = "0")]
    pub reserved1: u8,

    #[deku(bits = "1")]
    pub arv: u8,
    #[deku(bits = "1")]
    pub ts: u8,
    #[deku(bits = "2")]
    #[deku(pad_bits_after = "6")] //reserved
    pub tc: u8,
}

impl fmt::Display for CapabilityClassAirborne {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.acas == 1 {
            write!(f, " ACAS")?;
        }
        if self.cdti == 1 {
            write!(f, " CDTI")?;
        }
        if self.arv == 1 {
            write!(f, " ARV")?;
        }
        if self.ts == 1 {
            write!(f, " TS")?;
        }
        if self.tc == 1 {
            write!(f, " TC")?;
        }
        Ok(())
    }
}

/// [`ME::AircraftOperationStatus`] && [`OperationStatus`] == 1
///
/// Version 2 support only
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct OperationStatusSurface {
    /// CC (14 bits)
    pub capability_class: CapabilityClassSurface,

    /// CC L/W codes
    #[deku(bits = "4")]
    pub lw_codes: u8,

    /// OM
    pub operational_mode: OperationalMode,

    /// OM last 8 bits (diff for airborne/surface)
    // TODO: parse:
    // http://www.anteni.net/adsb/Doc/1090-WP30-18-DRAFT_DO-260B-V42.pdf
    // 2.2.3.2.7.2.4.7 “GPS Antenna Offset” OM Code Subfield in Aircraft Operational Status Messages
    pub gps_antenna_offset: u8,

    pub version_number: ADSBVersion,

    #[deku(bits = "1")]
    pub nic_supplement_a: u8,

    #[deku(bits = "4")]
    #[deku(pad_bits_after = "2")] // reserved
    pub navigational_accuracy_category: u8,

    #[deku(bits = "2")]
    pub source_integrity_level: u8,

    #[deku(bits = "1")]
    pub barometric_altitude_integrity: u8,

    #[deku(bits = "1")]
    pub horizontal_reference_direction: u8,

    #[deku(bits = "1")]
    #[deku(pad_bits_after = "1")] // reserved
    pub sil_supplement: u8,
}

impl fmt::Display for OperationStatusSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "  Version:            {}", self.version_number)?;
        writeln!(f, "   NIC-A:              {}", self.nic_supplement_a)?;
        write!(f, "{}", self.capability_class)?;
        write!(f, "   Capability classes:")?;
        if self.lw_codes != 0 {
            writeln!(f, " L/W={}", self.lw_codes)?;
        } else {
            writeln!(f)?;
        }
        write!(f, "   Operational modes: {}", self.operational_mode)?;
        writeln!(f)?;
        writeln!(
            f,
            "   NACp:               {}",
            self.navigational_accuracy_category
        )?;
        writeln!(
            f,
            "   SIL:                {} (per hour)",
            self.source_integrity_level
        )?;
        writeln!(
            f,
            "   NICbaro:            {}",
            self.barometric_altitude_integrity
        )?;
        if self.horizontal_reference_direction == 1 {
            writeln!(f, "   Heading reference:  magnetic north")?;
        } else {
            writeln!(f, "   Heading reference:  true north")?;
        }
        Ok(())
    }
}

/// [`ME::AircraftOperationStatus`]
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct CapabilityClassSurface {
    /// 0, 0 in current version, reserved as id for later versions
    #[deku(bits = "2", assert_eq = "0")]
    pub reserved0: u8,

    /// Position Offset Applied
    #[deku(bits = "1")]
    pub poe: u8,

    /// Aircraft has ADS-B 1090ES Receive Capability
    #[deku(bits = "1")]
    #[deku(pad_bits_after = "2")] // reserved
    pub es1090: u8,

    /// Class B2 Ground Vehicle transmitting with less than 70 watts
    #[deku(bits = "1")]
    pub b2_low: u8,

    /// Aircraft has ADS-B UAT Receive Capability
    #[deku(bits = "1")]
    pub uat_in: u8,

    /// Nagivation Accuracy Category for Velocity
    #[deku(bits = "3")]
    pub nac_v: u8,

    /// NIC Supplement used on the Surface
    #[deku(bits = "1")]
    pub nic_supplement_c: u8,
}

impl fmt::Display for CapabilityClassSurface {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "   NIC-C:              {}", self.nic_supplement_c)?;
        writeln!(f, "   NACv:               {}", self.nac_v)?;
        Ok(())
    }
}

/// `OperationMode` field not including the last 8 bits that are different for Surface/Airborne
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct OperationalMode {
    /// (0, 0) in Version 2, reserved for other values
    #[deku(bits = "2", assert_eq = "0")]
    reserved: u8,

    #[deku(bits = "1")]
    tcas_ra_active: bool,

    #[deku(bits = "1")]
    ident_switch_active: bool,

    #[deku(bits = "1")]
    reserved_recv_atc_service: bool,

    #[deku(bits = "1")]
    single_antenna_flag: bool,

    #[deku(bits = "2")]
    system_design_assurance: u8,
}

impl fmt::Display for OperationalMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.tcas_ra_active {
            write!(f, " TCAS")?;
        }
        if self.ident_switch_active {
            write!(f, " IDENT_SWITCH_ACTIVE")?;
        }
        if self.reserved_recv_atc_service {
            write!(f, " ATC")?;
        }
        if self.single_antenna_flag {
            write!(f, " SAF")?;
        }
        if self.system_design_assurance != 0 {
            write!(f, " SDA={}", self.system_design_assurance)?;
        }
        Ok(())
    }
}

/// ADS-B Defined from different ICAO documents
///
/// reference: ICAO 9871 (5.3.2.3)
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "3")]
pub enum ADSBVersion {
    #[deku(id = "0")]
    DOC9871AppendixA,
    #[deku(id = "1")]
    DOC9871AppendixB,
    #[deku(id = "2")]
    DOC9871AppendixC,
}

impl fmt::Display for ADSBVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.deku_id().unwrap())
    }
}

/// Control Field (B.3) for [`crate::DF::TisB`]
///
/// reference: ICAO 9871
#[derive(Debug, PartialEq, DekuRead, Clone, Serialize, Deserialize)]
pub struct ControlField {
    t: ControlFieldType,
    /// AA: Address, Announced
    pub aa: ICAO,
    /// ME: message, extended quitter
    pub me: ME,
}

impl fmt::Display for ControlField {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            self.me.to_string(
                self.aa,
                &format!("{}", self.t),
                Capability::AG_UNCERTAIN3,
                false,
            )?
        )
    }
}

#[derive(Debug, PartialEq, Eq, DekuRead, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "3")]
#[allow(non_camel_case_types)]
pub enum ControlFieldType {
    /// ADS-B Message from a non-transponder device
    #[deku(id = "0")]
    ADSB_ES_NT,

    /// Reserved for ADS-B for ES/NT devices for alternate address space
    #[deku(id = "1")]
    ADSB_ES_NT_ALT,

    /// Code 2, Fine Format TIS-B Message
    #[deku(id = "2")]
    TISB_FINE,

    /// Code 3, Coarse Format TIS-B Message
    #[deku(id = "3")]
    TISB_COARSE,

    /// Code 4, Coarse Format TIS-B Message
    #[deku(id = "4")]
    TISB_MANAGE,

    /// Code 5, TIS-B Message for replay ADS-B Message
    ///
    /// Anonymous 24-bit addresses
    #[deku(id = "5")]
    TISB_ADSB_RELAY,

    /// Code 6, TIS-B Message, Same as DF=17
    #[deku(id = "6")]
    TISB_ADSB,

    /// Code 7, Reserved
    #[deku(id = "7")]
    Reserved,
}

impl fmt::Display for ControlFieldType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s_type = match self {
            Self::ADSB_ES_NT | Self::ADSB_ES_NT_ALT => "(ADS-B)",
            Self::TISB_COARSE | Self::TISB_ADSB_RELAY | Self::TISB_FINE => "(TIS-B)",
            Self::TISB_MANAGE | Self::TISB_ADSB => "(ADS-R)",
            Self::Reserved => "(unknown addressing scheme)",
        };
        write!(f, "{s_type}")
    }
}

/// Table: A-2-97
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct AircraftStatus {
    pub sub_type: AircraftStatusType,
    pub emergency_state: EmergencyState,
    #[deku(
        bits = "13",
        endian = "big",
        map = "|squawk: u32| -> Result<_, DekuError> {Ok(decode_id13_field(squawk))}"
    )]
    pub squawk: u32,
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "3")]
pub enum AircraftStatusType {
    #[deku(id = "0")]
    NoInformation,
    #[deku(id = "1")]
    EmergencyPriorityStatus,
    #[deku(id = "2")]
    ACASRaBroadcast,
    #[deku(id_pat = "_")]
    Reserved,
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "3")]
pub enum EmergencyState {
    None = 0,
    General = 1,
    Lifeguard = 2,
    MinimumFuel = 3,
    NoCommunication = 4,
    UnlawfulInterference = 5,
    DownedAircraft = 6,
    Reserved2 = 7,
}

impl fmt::Display for EmergencyState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::None => "no emergency",
            Self::General => "general",
            Self::Lifeguard => "lifeguard",
            Self::MinimumFuel => "minimum fuel",
            Self::NoCommunication => "no communication",
            Self::UnlawfulInterference => "unflawful interference",
            Self::DownedAircraft => "downed aircraft",
            Self::Reserved2 => "reserved2",
        };
        write!(f, "{s}")?;
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct OperationCodeSurface {
    #[deku(bits = "1")]
    pub poe: u8,
    #[deku(bits = "1")]
    pub cdti: u8,
    #[deku(bits = "1")]
    pub b2_low: u8,
    #[deku(bits = "3")]
    #[deku(pad_bits_before = "6")]
    pub lw: u8,
}

#[derive(Debug, PartialEq, Eq, DekuRead, Clone, Serialize, Deserialize)]
pub struct Identification {
    pub tc: TypeCoding,

    #[deku(bits = "3")]
    pub ca: u8,

    /// N-Number / Tail Number
    #[deku(reader = "aircraft_identification_read(deku::rest)")]
    pub cn: String,
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "5")]
pub enum TypeCoding {
    D = 1,
    C = 2,
    B = 3,
    A = 4,
}

impl fmt::Display for TypeCoding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::D => "D",
                Self::C => "C",
                Self::B => "B",
                Self::A => "A",
            }
        )
    }
}

/// Target State and Status (§2.2.3.2.7.1)
#[derive(Copy, Clone, Debug, PartialEq, DekuRead, Serialize, Deserialize)]
pub struct TargetStateAndStatusInformation {
    // TODO Support Target State and Status defined in DO-260A, ADS-B Version=1
    // TODO Support reserved 2..=3
    #[deku(bits = "2")]
    pub subtype: u8,
    #[deku(bits = "1")]
    pub is_fms: bool,
    #[deku(
        bits = "12",
        endian = "big",
        map = "|altitude: u32| -> Result<_, DekuError> {Ok(if altitude > 1 {(altitude - 1) * 32} else {0} )}"
    )]
    pub altitude: u32,
    #[deku(
        bits = "9",
        endian = "big",
        map = "|qnh: u32| -> Result<_, DekuError> {if qnh == 0 { Ok(0.0) } else { Ok(800.0 + ((qnh - 1) as f32) * 0.8)}}"
    )]
    pub qnh: f32,
    #[deku(bits = "1")]
    pub is_heading: bool,
    #[deku(
        bits = "9",
        endian = "big",
        map = "|heading: u16| -> Result<_, DekuError> {Ok(heading as f32 * 180.0 / 256.0)}"
    )]
    pub heading: f32,
    #[deku(bits = "4")]
    pub nacp: u8,
    #[deku(bits = "1")]
    pub nicbaro: u8,
    #[deku(bits = "2")]
    pub sil: u8,
    #[deku(bits = "1")]
    pub mode_validity: bool,
    #[deku(bits = "1")]
    pub autopilot: bool,
    #[deku(bits = "1")]
    pub vnac: bool,
    #[deku(bits = "1")]
    pub alt_hold: bool,
    #[deku(bits = "1")]
    pub imf: bool,
    #[deku(bits = "1")]
    pub approach: bool,
    #[deku(bits = "1")]
    pub tcas: bool,
    #[deku(bits = "1")]
    #[deku(pad_bits_after = "2")] // reserved
    pub lnav: bool,
}

/// [`ME::AirborneVelocity`]
#[derive(Debug, PartialEq, Eq, DekuRead, Clone, Serialize, Deserialize)]
pub struct AirborneVelocity {
    #[deku(bits = "3")]
    pub st: u8,
    #[deku(bits = "5")]
    pub nac_v: u8,
    #[deku(ctx = "*st")]
    pub sub_type: AirborneVelocitySubType,
    pub vrate_src: VerticalRateSource,
    pub vrate_sign: Sign,
    #[deku(endian = "big", bits = "9")]
    pub vrate_value: u16,
    #[deku(bits = "2")]
    pub reverved: u8,
    pub gnss_sign: Sign,
    #[deku(
        bits = "7",
        map = "|gnss_baro_diff: u16| -> Result<_, DekuError> {Ok(if gnss_baro_diff > 1 {(gnss_baro_diff - 1)* 25} else { 0 })}"
    )]
    pub gnss_baro_diff: u16,
}

impl AirborneVelocity {
    /// Return effective (`heading`, `ground_speed`, `vertical_rate`) for groundspeed
    #[must_use]
    pub fn calculate(&self) -> Option<(f32, f64, i16)> {
        if let AirborneVelocitySubType::GroundSpeedDecoding(ground_speed) = &self.sub_type {
            let v_ew = f64::from((ground_speed.ew_vel as i16 - 1) * ground_speed.ew_sign.value());
            let v_ns = f64::from((ground_speed.ns_vel as i16 - 1) * ground_speed.ns_sign.value());
            let h = libm::atan2(v_ew, v_ns) * (360.0 / (2.0 * f64::consts::PI));
            let heading = if h < 0.0 { h + 360.0 } else { h };

            let vrate = self
                .vrate_value
                .checked_sub(1)
                .and_then(|v| v.checked_mul(64))
                .map(|v| (v as i16) * self.vrate_sign.value());

            if let Some(vrate) = vrate {
                return Some((heading as f32, libm::hypot(v_ew, v_ns), vrate));
            }
        }
        None
    }
}

/// Airborne Velocity Message “Subtype” Code Field Encoding
#[derive(Debug, PartialEq, Eq, DekuRead, Clone, Serialize, Deserialize)]
#[deku(ctx = "st: u8", id = "st")]
pub enum AirborneVelocitySubType {
    #[deku(id = "0")]
    Reserved0(#[deku(bits = "22")] u32),

    #[deku(id_pat = "1..=2")]
    GroundSpeedDecoding(GroundSpeedDecoding),

    #[deku(id_pat = "3..=4")]
    AirspeedDecoding(AirspeedDecoding),

    #[deku(id_pat = "5..=7")]
    Reserved1(#[deku(bits = "22")] u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, DekuRead, Serialize, Deserialize)]
#[deku(type = "u8", bits = "3")]
pub enum AirborneVelocityType {
    Subsonic = 1,
    Supersonic = 3,
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(ctx = "t: AirborneVelocityType")]
pub struct AirborneVelocitySubFields {
    pub dew: DirectionEW,
    #[deku(reader = "Self::read_v(deku::rest, t)")]
    pub vew: u16,
    pub dns: DirectionNS,
    #[deku(reader = "Self::read_v(deku::rest, t)")]
    pub vns: u16,
}

impl AirborneVelocitySubFields {
    fn read_v(
        rest: &BitSlice<u8, Msb0>,
        t: AirborneVelocityType,
    ) -> result::Result<(&BitSlice<u8, Msb0>, u16), DekuError> {
        match t {
            AirborneVelocityType::Subsonic => {
                u16::read(rest, (deku::ctx::Endian::Big, deku::ctx::BitSize(10)))
                    .map(|(rest, value)| (rest, value - 1))
            }
            AirborneVelocityType::Supersonic => {
                u16::read(rest, (deku::ctx::Endian::Big, deku::ctx::BitSize(10)))
                    .map(|(rest, value)| (rest, 4 * (value - 1)))
            }
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, DekuRead, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum DirectionEW {
    WestToEast = 0,
    EastToWest = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, DekuRead, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum DirectionNS {
    SouthToNorth = 0,
    NorthToSouth = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, DekuRead, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum SourceBitVerticalRate {
    GNSS = 0,
    Barometer = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, DekuRead, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum SignBitVerticalRate {
    Up = 0,
    Down = 1,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, DekuRead, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum SignBitGNSSBaroAltitudesDiff {
    Above = 0,
    Below = 1,
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum VerticalRateSource {
    BarometricPressureAltitude = 0,
    GeometricAltitude = 1,
}

impl fmt::Display for VerticalRateSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::BarometricPressureAltitude => "barometric",
                Self::GeometricAltitude => "GNSS",
            }
        )
    }
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct SurfacePosition {
    #[deku(bits = "7")]
    pub mov: u8,
    pub s: StatusForGroundTrack,
    #[deku(bits = "7")]
    pub trk: u8,
    #[deku(bits = "1")]
    pub t: bool,
    pub f: CPRFormat,
    #[deku(bits = "17", endian = "big")]
    pub lat_cpr: u32,
    #[deku(bits = "17", endian = "big")]
    pub lon_cpr: u32,
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum StatusForGroundTrack {
    Invalid = 0,
    Valid = 1,
}

/// Transponder level and additional information (3.1.2.5.2.2.1)
#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize, DekuRead, DekuWrite)]
#[allow(non_camel_case_types)]
#[deku(type = "u8", bits = "3")]
pub enum Capability {
    /// Level 1 transponder (surveillance only), and either airborne or on the ground
    AG_UNCERTAIN = 0x00,
    #[deku(id_pat = "0x01..=0x03")]
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

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::AG_UNCERTAIN => "uncertain1",
                Self::Reserved => "reserved",
                Self::AG_GROUND => "ground",
                Self::AG_AIRBORNE => "airborne",
                Self::AG_UNCERTAIN2 => "uncertain2",
                Self::AG_UNCERTAIN3 => "airborne?",
            }
        )
    }
}

impl Default for Capability {
    fn default() -> Self {
        Capability::AG_UNCERTAIN
    }
}

impl AdsbRawMessage {
    /// Converts `ADSBsMessage` to `String`.
    pub fn to_string(&self) -> MessageResult<String> {
        match serde_json::to_string(self) {
            Ok(v) => Ok(v),
            Err(e) => Err(DeserializationError::SerdeError(e)),
        }
    }

    /// Converts `ADSBJsonMessage` to `String` and appends a `\n` to the end.
    pub fn to_string_newline(&self) -> MessageResult<String> {
        match serde_json::to_string(self) {
            Err(to_string_error) => Err(DeserializationError::SerdeError(to_string_error)),
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

/// Latitude, Longitude and Altitude information
#[derive(Debug, PartialEq, Eq, DekuRead, Default, Copy, Clone, Serialize, Deserialize)]
pub struct Altitude {
    #[deku(bits = "5")]
    pub tc: u8,
    pub ss: SurveillanceStatus,
    #[deku(bits = "1")]
    pub saf_or_imf: u8,
    #[deku(reader = "Self::read(deku::rest)")]
    pub alt: Option<u16>,
    /// UTC sync or not
    #[deku(bits = "1")]
    pub t: bool,
    /// Odd or even
    pub odd_flag: CPRFormat,
    #[deku(bits = "17", endian = "big")]
    pub lat_cpr: u32,
    #[deku(bits = "17", endian = "big")]
    pub lon_cpr: u32,
}

impl fmt::Display for Altitude {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let altitude = self.alt.map_or_else(
            || "None".to_string(),
            |altitude| format!("{altitude} ft barometric"),
        );
        writeln!(f, "  Altitude:      {altitude}")?;
        writeln!(f, "  CPR type:      Airborne")?;
        writeln!(f, "  CPR odd flag:  {}", self.odd_flag)?;
        writeln!(f, "  CPR latitude:  ({})", self.lat_cpr)?;
        writeln!(f, "  CPR longitude: ({})", self.lon_cpr)?;
        Ok(())
    }
}

impl Altitude {
    /// `decodeAC12Field`
    fn read(rest: &BitSlice<u8, Msb0>) -> Result<(&BitSlice<u8, Msb0>, Option<u16>), DekuError> {
        let (rest, num) = u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::BitSize(12)))?;

        let q = num & 0x10;

        if q > 0 {
            let n = ((num & 0x0fe0) >> 1) | (num & 0x000f);
            let n = n * 25;
            if n > 1000 {
                // TODO: maybe replace with Result->Option
                Ok((rest, u16::try_from(n - 1000).ok()))
            } else {
                Ok((rest, None))
            }
        } else {
            let mut n = ((num & 0x0fc0) << 1) | (num & 0x003f);
            n = decode_id13_field(n);
            if let Ok(n) = mode_a_to_mode_c(n) {
                Ok((rest, u16::try_from(n * 100).ok()))
            } else {
                Ok((rest, None))
            }
        }
    }
}

/// SPI Condition
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "2")]
pub enum SurveillanceStatus {
    NoCondition = 0,
    PermanentAlert = 1,
    TemporaryAlert = 2,
    SPICondition = 3,
}

impl Default for SurveillanceStatus {
    fn default() -> Self {
        Self::NoCondition
    }
}

pub(crate) fn decode_id13_field(id13_field: u32) -> u32 {
    let mut hex_gillham: u32 = 0;

    if id13_field & 0x1000 != 0 {
        hex_gillham |= 0x0010;
    } // Bit 12 = C1
    if id13_field & 0x0800 != 0 {
        hex_gillham |= 0x1000;
    } // Bit 11 = A1
    if id13_field & 0x0400 != 0 {
        hex_gillham |= 0x0020;
    } // Bit 10 = C2
    if id13_field & 0x0200 != 0 {
        hex_gillham |= 0x2000;
    } // Bit  9 = A2
    if id13_field & 0x0100 != 0 {
        hex_gillham |= 0x0040;
    } // Bit  8 = C4
    if id13_field & 0x0080 != 0 {
        hex_gillham |= 0x4000;
    } // Bit  7 = A4
      //if id13_field & 0x0040 != 0 {hex_gillham |= 0x0800;} // Bit  6 = X  or M
    if id13_field & 0x0020 != 0 {
        hex_gillham |= 0x0100;
    } // Bit  5 = B1
    if id13_field & 0x0010 != 0 {
        hex_gillham |= 0x0001;
    } // Bit  4 = D1 or Q
    if id13_field & 0x0008 != 0 {
        hex_gillham |= 0x0200;
    } // Bit  3 = B2
    if id13_field & 0x0004 != 0 {
        hex_gillham |= 0x0002;
    } // Bit  2 = D2
    if id13_field & 0x0002 != 0 {
        hex_gillham |= 0x0400;
    } // Bit  1 = B4
    if id13_field & 0x0001 != 0 {
        hex_gillham |= 0x0004;
    } // Bit  0 = D4

    hex_gillham
}

pub(crate) fn mode_a_to_mode_c(mode_a: u32) -> result::Result<u32, &'static str> {
    let mut five_hundreds: u32 = 0;
    let mut one_hundreds: u32 = 0;

    // check zero bits are zero, D1 set is illegal; C1,,C4 cannot be Zero
    if (mode_a & 0xffff_8889) != 0 || (mode_a & 0x0000_00f0) == 0 {
        return Err("Invalid altitude");
    }

    if mode_a & 0x0010 != 0 {
        one_hundreds ^= 0x007;
    } // C1
    if mode_a & 0x0020 != 0 {
        one_hundreds ^= 0x003;
    } // C2
    if mode_a & 0x0040 != 0 {
        one_hundreds ^= 0x001;
    } // C4

    // Remove 7s from OneHundreds (Make 7->5, snd 5->7).
    if (one_hundreds & 5) == 5 {
        one_hundreds ^= 2;
    }

    // Check for invalid codes, only 1 to 5 are valid
    if one_hundreds > 5 {
        return Err("Invalid altitude");
    }

    // if mode_a & 0x0001 {five_hundreds ^= 0x1FF;} // D1 never used for altitude
    if mode_a & 0x0002 != 0 {
        five_hundreds ^= 0x0ff;
    } // D2
    if mode_a & 0x0004 != 0 {
        five_hundreds ^= 0x07f;
    } // D4

    if mode_a & 0x1000 != 0 {
        five_hundreds ^= 0x03f;
    } // A1
    if mode_a & 0x2000 != 0 {
        five_hundreds ^= 0x01f;
    } // A2
    if mode_a & 0x4000 != 0 {
        five_hundreds ^= 0x00f;
    } // A4

    if mode_a & 0x0100 != 0 {
        five_hundreds ^= 0x007;
    } // B1
    if mode_a & 0x0200 != 0 {
        five_hundreds ^= 0x003;
    } // B2
    if mode_a & 0x0400 != 0 {
        five_hundreds ^= 0x001;
    } // B4

    // Correct order of one_hundreds.
    if five_hundreds & 1 != 0 && one_hundreds <= 6 {
        one_hundreds = 6 - one_hundreds;
    }

    let n = (five_hundreds * 5) + one_hundreds;
    if n >= 13 {
        Ok(n - 13)
    } else {
        Err("Invalid altitude")
    }
}

/// Even / Odd
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum CPRFormat {
    Even = 0,
    Odd = 1,
}

impl Default for CPRFormat {
    fn default() -> Self {
        Self::Even
    }
}

impl fmt::Display for CPRFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Even => "even",
                Self::Odd => "odd",
            }
        )
    }
}

/// Positive / Negative
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum Sign {
    Positive = 0,
    Negative = 1,
}

impl Sign {
    #[must_use]
    pub fn value(&self) -> i16 {
        match self {
            Self::Positive => 1,
            Self::Negative => -1,
        }
    }
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Positive => "",
                Self::Negative => "-",
            }
        )
    }
}

const CHAR_LOOKUP: &[u8; 64] = b"#ABCDEFGHIJKLMNOPQRSTUVWXYZ##### ###############0123456789######";

pub(crate) fn aircraft_identification_read(
    rest: &BitSlice<u8, Msb0>,
) -> Result<(&BitSlice<u8, Msb0>, String), DekuError> {
    let mut inside_rest = rest;

    let mut chars = vec![];
    for _ in 0..=6 {
        let (for_rest, c) = <u8>::read(inside_rest, deku::ctx::BitSize(6))?;
        if c != 32 {
            chars.push(c);
        }
        inside_rest = for_rest;
    }
    let encoded = chars
        .into_iter()
        .map(|b| CHAR_LOOKUP[b as usize] as char)
        .collect::<String>();

    Ok((inside_rest, encoded))
}

/// Airborne / Ground and SPI
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "3")]
pub enum FlightStatus {
    NoAlertNoSPIAirborne = 0b000,
    NoAlertNoSPIOnGround = 0b001,
    AlertNoSPIAirborne = 0b010,
    AlertNoSPIOnGround = 0b011,
    AlertSPIAirborneGround = 0b100,
    NoAlertSPIAirborneGround = 0b101,
    Reserved = 0b110,
    NotAssigned = 0b111,
}

impl fmt::Display for FlightStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::NoAlertNoSPIAirborne
                | Self::AlertSPIAirborneGround
                | Self::NoAlertSPIAirborneGround => "airborne?",
                Self::NoAlertNoSPIOnGround => "ground?",
                Self::AlertNoSPIAirborne => "airborne",
                Self::AlertNoSPIOnGround => "ground",
                _ => "reserved",
            }
        )
    }
}

/// Type of `DownlinkRequest`
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "5")]
pub enum DownlinkRequest {
    None = 0b00000,
    RequestSendCommB = 0b00001,
    CommBBroadcastMsg1 = 0b00100,
    CommBBroadcastMsg2 = 0b00101,
    #[deku(id_pat = "_")]
    Unknown,
}

/// 13 bit encoded altitude
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct AC13Field(#[deku(reader = "Self::read(deku::rest)")] pub u16);

impl AC13Field {
    // TODO Add unit
    fn read(rest: &BitSlice<u8, Msb0>) -> result::Result<(&BitSlice<u8, Msb0>, u16), DekuError> {
        let (rest, num) = u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::BitSize(13)))?;

        let m_bit = num & 0x0040;
        let q_bit = num & 0x0010;

        if m_bit != 0 {
            // TODO: this might be wrong?
            Ok((rest, 0))
        } else if q_bit != 0 {
            let n = ((num & 0x1f80) >> 2) | ((num & 0x0020) >> 1) | (num & 0x000f);
            let n = n * 25;
            if n > 1000 {
                Ok((rest, (n - 1000) as u16))
            } else {
                // TODO: add error
                Ok((rest, 0))
            }
        } else {
            // TODO 11 bit gillham coded altitude
            if let Ok(n) = mode_a_to_mode_c(decode_id13_field(num)) {
                Ok((rest, (100 * n) as u16))
            } else {
                Ok((rest, 0))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct UtilityMessage {
    #[deku(bits = "4")]
    pub iis: u8,
    pub ids: UtilityMessageType,
}

/// Message Type
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "2")]
pub enum UtilityMessageType {
    NoInformation = 0b00,
    CommB = 0b01,
    CommC = 0b10,
    CommD = 0b11,
}

/// Uplink / Downlink
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "1")]
pub enum KE {
    DownlinkELMTx = 0,
    UplinkELMAck = 1,
}

#[derive(Debug, PartialEq, Eq, DekuRead, Clone, Serialize, Deserialize)]
#[deku(type = "u8", bits = "8")]
pub enum BDS {
    /// (1, 0) Table A-2-16
    #[deku(id = "0x00")]
    Empty([u8; 6]),

    /// (1, 0) Table A-2-16
    #[deku(id = "0x10")]
    DataLinkCapability(DataLinkCapability),

    /// (2, 0) Table A-2-32
    #[deku(id = "0x20")]
    AircraftIdentification(#[deku(reader = "aircraft_identification_read(deku::rest)")] String),

    #[deku(id_pat = "_")]
    Unknown([u8; 6]),
}

impl fmt::Display for BDS {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty(_) => {
                writeln!(f, "Comm-B format: empty response")?;
            }
            Self::AircraftIdentification(s) => {
                writeln!(f, "Comm-B format: BDS2,0 Aircraft identification")?;
                writeln!(f, "  Ident:         {s}")?;
            }
            Self::DataLinkCapability(_) => {
                writeln!(f, "Comm-B format: BDS1,0 Datalink capabilities")?;
            }
            Self::Unknown(_) => {
                writeln!(f, "Comm-B format: unknown format")?;
            }
        }
        Ok(())
    }
}

/// To report the data link capability of the Mode S transponder/data link installation
#[derive(Debug, PartialEq, Eq, DekuRead, Clone, Serialize, Deserialize)]
pub struct DataLinkCapability {
    #[deku(bits = "1")]
    #[deku(pad_bits_after = "5")] // reserved
    pub continuation_flag: bool,
    #[deku(bits = "1")]
    pub overlay_command_capability: bool,
    #[deku(bits = "1")]
    pub acas: bool,
    #[deku(bits = "7")]
    pub mode_s_subnetwork_version_number: u8,
    #[deku(bits = "1")]
    pub transponder_enhanced_protocol_indicator: bool,
    #[deku(bits = "1")]
    pub mode_s_specific_services_capability: bool,
    #[deku(bits = "3")]
    pub uplink_elm_average_throughput_capability: u8,
    #[deku(bits = "4")]
    pub downlink_elm: u8,
    #[deku(bits = "1")]
    pub aircraft_identification_capability: bool,
    #[deku(bits = "1")]
    pub squitter_capability_subfield: bool,
    #[deku(bits = "1")]
    pub surveillance_identifier_code: bool,
    #[deku(bits = "1")]
    pub common_usage_gicb_capability_report: bool,
    #[deku(bits = "4")]
    pub reserved_acas: u8,
    pub bit_array: u16,
}

/// 13 bit identity code
#[derive(Debug, PartialEq, Eq, DekuRead, Copy, Clone, Serialize, Deserialize)]
pub struct IdentityCode(#[deku(reader = "Self::read(deku::rest)")] pub u16);

impl IdentityCode {
    fn read(rest: &BitSlice<u8, Msb0>) -> result::Result<(&BitSlice<u8, Msb0>, u16), DekuError> {
        let (rest, num) = u32::read(rest, (deku::ctx::Endian::Big, deku::ctx::BitSize(13)))?;

        let c1 = (num & 0b1_0000_0000_0000) >> 12;
        let a1 = (num & 0b0_1000_0000_0000) >> 11;
        let c2 = (num & 0b0_0100_0000_0000) >> 10;
        let a2 = (num & 0b0_0010_0000_0000) >> 9;
        let c4 = (num & 0b0_0001_0000_0000) >> 8;
        let a4 = (num & 0b0_0000_1000_0000) >> 7;
        let b1 = (num & 0b0_0000_0010_0000) >> 5;
        let d1 = (num & 0b0_0000_0001_0000) >> 4;
        let b2 = (num & 0b0_0000_0000_1000) >> 3;
        let d2 = (num & 0b0_0000_0000_0100) >> 2;
        let b4 = (num & 0b0_0000_0000_0010) >> 1;
        let d4 = num & 0b0_0000_0000_0001;

        let a = a4 << 2 | a2 << 1 | a1;
        let b = b4 << 2 | b2 << 1 | b1;
        let c = c4 << 2 | c2 << 1 | c1;
        let d = d4 << 2 | d2 << 1 | d1;

        let num: u16 = (a << 12 | b << 8 | c << 4 | d) as u16;
        Ok((rest, num))
    }
}

pub const CRC_TABLE: [u32; 256] = [
    0x0000_0000,
    0x00ff_f409,
    0x0000_1c1b,
    0x00ff_e812,
    0x0000_3836,
    0x00ff_cc3f,
    0x0000_242d,
    0x00ff_d024,
    0x0000_706c,
    0x00ff_8465,
    0x0000_6c77,
    0x00ff_987e,
    0x0000_485a,
    0x00ff_bc53,
    0x0000_5441,
    0x00ff_a048,
    0x0000_e0d8,
    0x00ff_14d1,
    0x0000_fcc3,
    0x00ff_08ca,
    0x0000_d8ee,
    0x00ff_2ce7,
    0x0000_c4f5,
    0x00ff_30fc,
    0x0000_90b4,
    0x00ff_64bd,
    0x0000_8caf,
    0x00ff_78a6,
    0x0000_a882,
    0x00ff_5c8b,
    0x0000_b499,
    0x00ff_4090,
    0x0001_c1b0,
    0x00fe_35b9,
    0x0001_ddab,
    0x00fe_29a2,
    0x0001_f986,
    0x00fe_0d8f,
    0x0001_e59d,
    0x00fe_1194,
    0x0001_b1dc,
    0x00fe_45d5,
    0x0001_adc7,
    0x00fe_59ce,
    0x0001_89ea,
    0x00fe_7de3,
    0x0001_95f1,
    0x00fe_61f8,
    0x0001_2168,
    0x00fe_d561,
    0x0001_3d73,
    0x00fe_c97a,
    0x0001_195e,
    0x00fe_ed57,
    0x0001_0545,
    0x00fe_f14c,
    0x0001_5104,
    0x00fe_a50d,
    0x0001_4d1f,
    0x00fe_b916,
    0x0001_6932,
    0x00fe_9d3b,
    0x0001_7529,
    0x00fe_8120,
    0x0003_8360,
    0x00fc_7769,
    0x0003_9f7b,
    0x00fc_6b72,
    0x0003_bb56,
    0x00fc_4f5f,
    0x0003_a74d,
    0x00fc_5344,
    0x0003_f30c,
    0x00fc_0705,
    0x0003_ef17,
    0x00fc_1b1e,
    0x0003_cb3a,
    0x00fc_3f33,
    0x0003_d721,
    0x00fc_2328,
    0x0003_63b8,
    0x00fc_97b1,
    0x0003_7fa3,
    0x00fc_8baa,
    0x0003_5b8e,
    0x00fc_af87,
    0x0003_4795,
    0x00fc_b39c,
    0x0003_13d4,
    0x00fc_e7dd,
    0x0003_0fcf,
    0x00fc_fbc6,
    0x0003_2be2,
    0x00fc_dfeb,
    0x0003_37f9,
    0x00fc_c3f0,
    0x0002_42d0,
    0x00fd_b6d9,
    0x0002_5ecb,
    0x00fd_aac2,
    0x0002_7ae6,
    0x00fd_8eef,
    0x0002_66fd,
    0x00fd_92f4,
    0x0002_32bc,
    0x00fd_c6b5,
    0x0002_2ea7,
    0x00fd_daae,
    0x0002_0a8a,
    0x00fd_fe83,
    0x0002_1691,
    0x00fd_e298,
    0x0002_a208,
    0x00fd_5601,
    0x0002_be13,
    0x00fd_4a1a,
    0x0002_9a3e,
    0x00fd_6e37,
    0x0002_8625,
    0x00fd_722c,
    0x0002_d264,
    0x00fd_266d,
    0x0002_ce7f,
    0x00fd_3a76,
    0x0002_ea52,
    0x00fd_1e5b,
    0x0002_f649,
    0x00fd_0240,
    0x0007_06c0,
    0x00f8_f2c9,
    0x0007_1adb,
    0x00f8_eed2,
    0x0007_3ef6,
    0x00f8_caff,
    0x0007_22ed,
    0x00f8_d6e4,
    0x0007_76ac,
    0x00f8_82a5,
    0x0007_6ab7,
    0x00f8_9ebe,
    0x0007_4e9a,
    0x00f8_ba93,
    0x0007_5281,
    0x00f8_a688,
    0x0007_e618,
    0x00f8_1211,
    0x0007_fa03,
    0x00f8_0e0a,
    0x0007_de2e,
    0x00f8_2a27,
    0x0007_c235,
    0x00f8_363c,
    0x0007_9674,
    0x00f8_627d,
    0x0007_8a6f,
    0x00f8_7e66,
    0x0007_ae42,
    0x00f8_5a4b,
    0x0007_b259,
    0x00f8_4650,
    0x0006_c770,
    0x00f9_3379,
    0x0006_db6b,
    0x00f9_2f62,
    0x0006_ff46,
    0x00f9_0b4f,
    0x0006_e35d,
    0x00f9_1754,
    0x0006_b71c,
    0x00f9_4315,
    0x0006_ab07,
    0x00f9_5f0e,
    0x0006_8f2a,
    0x00f9_7b23,
    0x0006_9331,
    0x00f9_6738,
    0x0006_27a8,
    0x00f9_d3a1,
    0x0006_3bb3,
    0x00f9_cfba,
    0x0006_1f9e,
    0x00f9_eb97,
    0x0006_0385,
    0x00f9_f78c,
    0x0006_57c4,
    0x00f9_a3cd,
    0x0006_4bdf,
    0x00f9_bfd6,
    0x0006_6ff2,
    0x00f9_9bfb,
    0x0006_73e9,
    0x00f9_87e0,
    0x0004_85a0,
    0x00fb_71a9,
    0x0004_99bb,
    0x00fb_6db2,
    0x0004_bd96,
    0x00fb_499f,
    0x0004_a18d,
    0x00fb_5584,
    0x0004_f5cc,
    0x00fb_01c5,
    0x0004_e9d7,
    0x00fb_1dde,
    0x0004_cdfa,
    0x00fb_39f3,
    0x0004_d1e1,
    0x00fb_25e8,
    0x0004_6578,
    0x00fb_9171,
    0x0004_7963,
    0x00fb_8d6a,
    0x0004_5d4e,
    0x00fb_a947,
    0x0004_4155,
    0x00fb_b55c,
    0x0004_1514,
    0x00fb_e11d,
    0x0004_090f,
    0x00fb_fd06,
    0x0004_2d22,
    0x00fb_d92b,
    0x0004_3139,
    0x00fb_c530,
    0x0005_4410,
    0x00fa_b019,
    0x0005_580b,
    0x00fa_ac02,
    0x0005_7c26,
    0x00fa_882f,
    0x0005_603d,
    0x00fa_9434,
    0x0005_347c,
    0x00fa_c075,
    0x0005_2867,
    0x00fa_dc6e,
    0x0005_0c4a,
    0x00fa_f843,
    0x0005_1051,
    0x00fa_e458,
    0x0005_a4c8,
    0x00fa_50c1,
    0x0005_b8d3,
    0x00fa_4cda,
    0x0005_9cfe,
    0x00fa_68f7,
    0x0005_80e5,
    0x00fa_74ec,
    0x0005_d4a4,
    0x00fa_20ad,
    0x0005_c8bf,
    0x00fa_3cb6,
    0x0005_ec92,
    0x00fa_189b,
    0x0005_f089,
    0x00fa_0480,
];

pub fn modes_checksum(message: &[u8], bits: usize) -> result::Result<u32, DekuError> {
    let mut rem: u32 = 0;
    let n = bits / 8;

    if (n < 3) || (message.len() < n) {
        return Err(DekuError::Incomplete(NeedSize::new(4)));
    }

    for i in 0..(n - 3) {
        rem =
            (rem << 8) ^ CRC_TABLE[(u32::from(message[i]) ^ ((rem & 0x00ff_0000) >> 16)) as usize];
        rem &= 0x00ff_ffff;
    }

    let msg_1 = u32::from(message[n - 3]) << 16;
    let msg_2 = u32::from(message[n - 2]) << 8;
    let msg_3 = u32::from(message[n - 1]);
    let xor_term: u32 = msg_1 ^ msg_2 ^ msg_3;

    rem ^= xor_term;

    Ok(rem)
}
