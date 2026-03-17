

use postcard_rpc::{endpoints, topics, TopicDirection};
use postcard_schema::Schema;
use serde::{Deserialize, Serialize};
use defmt::Format;
use crate::conversions::MagneticField;


#[derive(Debug, Format, Serialize, Deserialize, Schema, PartialEq)]
pub struct SensorField {
    pub field: MagneticField,
    pub board_id: u16,
    pub position: (f32, f32, f32),
    pub address: u8,
    pub time: u64,
}

endpoints! {
    list = ENDPOINT_LIST;
    | EndpointTy                | RequestTy     | ResponseTy            | Path              |
    | ----------                | ---------     | ----------            | ----              |
    | PingEndpoint              | u32           | u32                   | "ping"            |
    | SingleFieldValue          | (u32, u32)    | SensorField           | "bfield/single"   |
    | StartFieldStream          | ()            | ()                    | "bfield/start"    |
    | StopFieldStream           | ()            | ()                    | "bfield/stop"     |
}


topics! {
    list = TOPICS_IN_LIST;
    direction = TopicDirection::ToServer;
    | TopicTy                   | MessageTy     | Path              |
    | -------                   | ---------     | ----              |
}

topics! {
    list = TOPICS_OUT_LIST;
    direction = TopicDirection::ToClient;
    | TopicTy                   | MessageTy     | Path              | Cfg                           |
    | -------                   | ---------     | ----              | ---                           |
    | MagneticTopic             | SensorField   | "bfield/data"     |                               |
}
