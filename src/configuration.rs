use serde::{Deserialize, Deserializer};

use crate::StatusError;
use std::vec::Vec;

pub async fn read_config(fname: &str) -> Result<Configuration, StatusError> {
    let raw = tokio::fs::read_to_string(fname).await.map_err(|e| {
        StatusError::Configuration(format!(
            "Failed to read configuration file {} ({})",
            fname, e
        ))
    })?;

    let config: Configuration = serde_json::from_str(&raw).map_err(|e| {
        StatusError::Configuration(format!("Failed to parse configuration ({})", e))
    })?;

    Ok(config)
}

#[derive(Deserialize, Clone)]
pub struct Configuration {
    pub database: Database,
    pub server: Server,
    pub sensors: Sensors,
    pub cache_time: CacheTime,
}

#[derive(Deserialize, Clone)]
pub struct Database {
    pub connection: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize, Clone)]
pub struct Server {
    pub hostname: String,
    pub port: u16,
}

#[derive(Deserialize, Clone)]
pub struct Sensors {
    pub door: SensorDescription<String>,
    pub temperature: SensorDescription<Vec<SensorName>>,
    pub humidity: SensorDescription<Vec<SensorName>>,
}

#[derive(Deserialize, Clone)]
pub struct SensorDescription<T> {
    pub name: T,
    pub unit: String,
    #[serde(deserialize_with = "parse_timedelta")]
    pub validity: chrono::TimeDelta,
}

#[derive(Deserialize, Clone)]
pub struct SensorName {
    pub id: String,
    pub location: String,
}

#[derive(Deserialize, Clone)]
pub struct CacheTime {
    #[serde(rename = "status.json")]
    #[serde(deserialize_with = "parse_timedelta")]
    pub status_json: chrono::TimeDelta,
    #[serde(deserialize_with = "parse_timedelta")]
    pub health: chrono::TimeDelta,
}

fn parse_timedelta<'de, D>(deserializer: D) -> Result<chrono::TimeDelta, D::Error>
where
    D: Deserializer<'de>,
{
    let s = serde_json::Number::deserialize(deserializer)?;
    let secs = s
        .as_i64()
        .ok_or(serde::de::Error::custom("Value not an i64"))?;
    let td =
        chrono::TimeDelta::new(secs, 0).ok_or(serde::de::Error::custom("Value of out range"))?;
    Ok(td)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::FILE_CONFIG;
    use tokio::runtime::Runtime;

    #[test]
    fn parse() {
        Runtime::new().unwrap().block_on(async {
            read_config(FILE_CONFIG).await.unwrap();
        });
    }
}
