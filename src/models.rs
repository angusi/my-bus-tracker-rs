//! Models representing data types returned by the My Bus Tracker API
#![allow(similar_names)]

use std::fmt::{self, Display, Formatter};
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer};
use chrono::prelude::*;
use std::ops::Deref;

#[derive(Clone, Debug)]
pub struct Timetable {
    pub stop_id: String,
    pub service_reference: String,
    pub destination_reference: String,
    pub operator_id: Operator,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusTimes {
    pub bus_times: Vec<BusTime>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusTime {
    pub operator_id: Operator,
    pub stop_id: String,
    pub stop_name: String,
    #[serde(rename = "refService")]
    pub service_reference: String,
    #[serde(rename = "mnemoService")]
    pub service_mnemonic: String,
    #[serde(rename = "nameService")]
    pub service_name: String,
    #[serde(rename = "refDest")]
    pub destination_reference: Option<String>,
    #[serde(rename = "nameDest")]
    pub destination_name: Option<String>,
    #[serde(rename = "timeDatas")]
    pub times: Vec<TimeData>,
    pub global_disruption: bool,
    pub service_disruption: bool,
    pub bus_stop_disruption: bool,
    pub service_diversion: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeData {
    pub day: u8,
    pub time: String,
    pub minutes: u8,
    pub reliability: Reliability,
    #[serde(rename = "type")]
    pub stop_type: StopType,
    pub terminus: String,
    pub journey_id: String,
    pub bus_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Reliability {
    #[serde(rename = "B")]
    Delayed,
    #[serde(rename = "D")]
    Delocated,
    #[serde(rename = "F")]
    RealTimeNotLowFloorEquipped,
    #[serde(rename = "H")]
    RealTimeLowFloorEquipped,
    #[serde(rename = "I")]
    Immobilized,
    #[serde(rename = "N")]
    Neutralized,
    #[serde(rename = "R")]
    RadioFault,
    #[serde(rename = "T")]
    Estimated,
    #[serde(rename = "V")]
    Diverted,
}
#[derive(Clone, Debug, Deserialize)]
pub enum StopType {
    #[serde(rename = "D")]
    Terminus,
    #[serde(rename = "N")]
    Normal,
    #[serde(rename = "P")]
    PartRoute,
    #[serde(rename = "R")]
    Reference,
}

#[derive(Clone, Debug)]
pub enum Operator {
    LothianBuses,
    AllOperators,
}

impl Display for Operator {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let printable = match *self {
            Operator::LothianBuses => "LB",
            Operator::AllOperators => "0",
        };
        write!(f, "{}", printable)
    }
}

impl<'de> Deserialize<'de> for Operator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match s {
            "LB" => Ok(Operator::LothianBuses),
            "0" | "ALL" => Ok(Operator::AllOperators),
            e => Err(D::Error::custom(format!("Unknown Operator: {}", e))),
        }
    }
}

#[derive(Clone, Debug)]
pub enum JourneyIdentifier {
    JourneyId(String),
    BusId(String),
}

#[derive(Clone, Debug)]
pub enum JourneyTimeMode {
    All,
    NextReference,
}

impl Display for JourneyTimeMode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let printable = match *self {
            JourneyTimeMode::All => "0",
            JourneyTimeMode::NextReference => "1",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JourneyTimes {
    pub journey_times: Vec<JourneyTime>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JourneyTime {
    pub journey_id: String,
    pub bus_id: Option<String>,
    pub operator_id: Operator,
    #[serde(rename = "refService")]
    pub service_reference: String,
    #[serde(rename = "mnemoService")]
    pub service_mnemonic: String,
    #[serde(rename = "nameService")]
    pub service_name: String,
    #[serde(rename = "refDest")]
    pub destination_reference: String,
    #[serde(rename = "nameDest")]
    pub destination_name: String,
    #[serde(rename = "journeyTimeDatas")]
    pub journey_times: Vec<JourneyTimeData>,
    pub global_disruption: bool,
    pub service_disruption: bool,
    pub service_diversion: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JourneyTimeData {
    pub order: u32,
    pub stop_id: String,
    pub stop_name: String,
    pub day: u32,           //TODO - Date
    pub time: NaiveTimeExt, // TODO - Date
    pub minutes: i32,
    pub reliability: Reliability,
    #[serde(rename = "type")]
    pub stop_type: String,
    #[serde(rename = "busStopDisruption")]
    pub disruption: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TopoId {
    pub topo_id: String,
    pub operator_id: Operator,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Services {
    pub services: Vec<Service>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Service {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(rename = "operatorId")]
    pub operator_id: Operator,
    #[serde(rename = "mnemo")]
    pub mnemonic: String,
    pub name: String,
    #[serde(rename = "dests")]
    pub destinations: Vec<String>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServicePoints {
    #[serde(rename = "ref")]
    pub service_reference: String,
    pub operator_id: Operator,
    pub service_points: Vec<ServicePoint>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServicePoint {
    pub chainage: u32,
    pub order: u32,
    #[serde(rename = "x")]
    pub latitude: f32,
    #[serde(rename = "y")]
    pub longitude: f32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Destinations {
    #[serde(rename = "dests")]
    pub destinations: Vec<Destination>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Destination {
    #[serde(rename = "ref")]
    pub reference: String,
    pub operator_id: Operator,
    pub name: String,
    pub direction: Direction,
    pub service: String,
}

#[derive(Clone, Debug, Deserialize)]
pub enum Direction {
    #[serde(rename = "A")]
    Inbound,
    #[serde(rename = "R")]
    Outbound,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusStops {
    pub bus_stops: Vec<BusStop>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BusStop {
    pub operator_id: Operator,
    pub stop_id: String,
    pub name: String,
    #[serde(rename = "x")]
    pub latitude: f32,
    #[serde(rename = "y")]
    pub longitude: f32,
    #[serde(rename = "cap")]
    pub orientation: u16,
    pub services: Vec<String>,
    #[serde(rename = "dests")]
    pub destinations: Vec<String>,
}

#[derive(Clone, Debug)]
pub enum DisruptionType {
    All,
    Network,
    Service,
    BusStop,
}

impl<'de> Deserialize<'de> for DisruptionType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let i: u8 = Deserialize::deserialize(deserializer)?;
        match i {
            0 => Ok(DisruptionType::All),
            1 => Ok(DisruptionType::Network),
            2 => Ok(DisruptionType::Service),
            3 => Ok(DisruptionType::BusStop),
            e => Err(D::Error::custom(format!("Unknown Disruption Type: {}", e))),
        }
    }
}

impl Display for DisruptionType {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let printable = match *self {
            DisruptionType::All => "0",
            DisruptionType::Network => "1",
            DisruptionType::Service => "2",
            DisruptionType::BusStop => "3",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Disruptions {
    pub disruptions: Vec<Disruption>,
}

#[derive(Clone, Debug)]
pub enum DisruptionLevel {
    Informative,
    Minor,
    Major,
}

impl<'de> Deserialize<'de> for DisruptionLevel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let i: u8 = Deserialize::deserialize(deserializer)?;
        match i {
            1 => Ok(DisruptionLevel::Informative),
            2 => Ok(DisruptionLevel::Minor),
            3 => Ok(DisruptionLevel::Major),
            e => Err(D::Error::custom(format!("Unknown Disruption Level: {}", e))),
        }
    }
}

impl Display for DisruptionLevel {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let printable = match *self {
            DisruptionLevel::Informative => "1",
            DisruptionLevel::Minor => "2",
            DisruptionLevel::Major => "3",
        };
        write!(f, "{}", printable)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Disruption {
    pub id: String,
    pub operator_id: Operator,
    pub level: DisruptionLevel,
    #[serde(rename = "type")]
    pub disruption_type: DisruptionType,
    pub targets: Vec<String>,
    pub valid_until: Option<DateTime<Utc>>,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Diversions {
    pub diversions: Vec<Diversion>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Diversion {
    #[serde(rename = "ref")]
    pub diversion_reference: String,
    pub diversion_id: String,
    pub operator_id: Operator,
    #[serde(rename = "refService")]
    pub service_reference: String,
    pub start_stop_id: String,
    pub start_stop_name: String,
    pub start_date: DateTime<Utc>,
    pub end_stop_id: String,
    pub end_stop_name: String,
    pub end_date: DateTime<Utc>,
    pub days: String,
    pub length: u32,
    pub time_shift: i32,
    pub cancelled_bus_stops: Vec<CancelledBusStop>,
    pub temporary_bus_stops: Vec<TemporaryBusStop>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelledBusStop {
    pub stop_id: String,
    pub stop_name: String,
    pub replaced_stop_id: String,
    pub replaced_stop_name: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TemporaryBusStop {
    pub stop_id: String,
    pub stop_name: String,
    #[serde(rename = "num")]
    pub stop_number: u32,
    #[serde(rename = "type")]
    pub stop_type: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiversionPoints {
    //    pub diversion_id: String,
    //    pub operator_id: Operator,
    pub diversion_points: Vec<DiversionPoint>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DiversionPoint {
    pub order: u32,
    #[serde(rename = "x")]
    pub latitude: f32,
    #[serde(rename = "y")]
    pub longitude: f32,
}

#[derive(Clone, Debug)]
pub struct NaiveTimeExt(NaiveTime);

impl Deref for NaiveTimeExt {
    type Target = NaiveTime;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'de> Deserialize<'de> for NaiveTimeExt {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let time_string: String = Deserialize::deserialize(deserializer)?;
        NaiveTime::parse_from_str(&time_string, "%H:%M")
            .map_err(D::Error::custom)
            .map(NaiveTimeExt)
    }
}
