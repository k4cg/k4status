use serde::{Deserialize, Deserializer};
use std::vec::Vec;

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Configuration {
    pub database: Database,
    pub server: Server,
    pub sensors: Sensors,
    pub cache_time: CacheTime,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Database {
    pub connection: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Server {
    pub hostname: String,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Sensors {
    pub door: DoorSettings,
    pub temperature: SensorSettings,
    pub humidity: SensorSettings,
    pub carbondioxide: SensorSettings,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DoorSettings {
    pub entity: String,
    pub unit: String,
    #[serde(deserialize_with = "parse_timedelta")]
    pub validity: chrono::TimeDelta,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SensorSettings {
    pub id: Vec<SensorIdentification>,
    pub unit: String,
    #[serde(deserialize_with = "parse_timedelta")]
    pub validity: chrono::TimeDelta,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct SensorIdentification {
    pub entity: String,
    pub location: String,
}

#[derive(Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct CacheTime {
    #[serde(deserialize_with = "parse_timedelta")]
    pub status: chrono::TimeDelta,
    #[serde(deserialize_with = "parse_timedelta")]
    pub health: chrono::TimeDelta,
    #[serde(deserialize_with = "parse_timedelta")]
    pub badge: chrono::TimeDelta,
}

fn parse_timedelta<'de, D>(deserializer: D) -> Result<chrono::TimeDelta, D::Error>
where
    D: Deserializer<'de>,
{
    serde_json::Number::deserialize(deserializer)?
        .as_i64()
        .map(|i| chrono::TimeDelta::new(i, 0))
        .ok_or(serde::de::Error::custom("Value not an i64"))?
        .ok_or(serde::de::Error::custom("Value of out range"))
}
