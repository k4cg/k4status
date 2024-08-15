use crate::{configuration, StatusError};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::vec::Vec;

pub type Temperature = f64;
pub type Humidity = u8;
pub type DoorStatus = bool;

#[derive(Clone)]
pub struct Database {
    client: influxdb::Client,
}

#[derive(Debug)]
pub struct TimeValue<T> {
    pub time: DateTime<Utc>,
    pub value: T,
}

#[derive(Deserialize, Clone)]
struct QueryResults {
    pub results: Vec<QueryResult>,
}

#[derive(Deserialize, Clone)]
struct QueryResult {
    #[allow(dead_code)]
    pub statement_id: u64,
    pub series: Vec<QuerySeries>,
}

#[derive(Deserialize, Clone)]
struct QuerySeries {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub columns: Vec<String>,
    pub values: Vec<(String, f64)>,
}

impl Database {
    pub fn new(config: &configuration::Database) -> Self {
        let client = influxdb::Client::new(&config.connection, &config.database)
            .with_auth(&config.username, &config.password);

        Database { client }
    }

    pub async fn check_connection(&self) -> Result<(), StatusError> {
        self.client
            .ping()
            .await
            .map(|_| ())
            .map_err(|e| StatusError::Database(format!("Failed to connect to database ({})", e)))
    }

    pub async fn get_temperature(
        &self,
        name: &str,
        unit: &str,
    ) -> Result<TimeValue<Temperature>, StatusError> {
        let result = self.query_and_extract(name, unit).await?;

        Ok(TimeValue {
            time: result.0,
            value: result.1,
        })
    }

    pub async fn get_humidity(
        &self,
        name: &str,
        unit: &str,
    ) -> Result<TimeValue<Humidity>, StatusError> {
        let result = self.query_and_extract(name, unit).await?;

        Ok(TimeValue {
            time: result.0,
            value: result.1 as Humidity,
        })
    }

    pub async fn get_door_status(
        &self,
        name: &str,
        unit: &str,
    ) -> Result<TimeValue<DoorStatus>, StatusError> {
        let result = self.query_and_extract(name, unit).await?;

        Ok(TimeValue {
            time: result.0,
            value: result.1 > 0.5,
        })
    }

    async fn query_and_extract(
        &self,
        name: &str,
        unit: &str,
    ) -> Result<(DateTime<Utc>, f64), StatusError> {
        let query = influxdb::ReadQuery::new(format!(
            r#"SELECT time, value FROM "{}" WHERE entity_id = '{}' ORDER BY time DESC LIMIT 1"#,
            unit, name
        ));

        let results_raw = self
            .client
            .query(query)
            .await
            .map_err(|e| StatusError::Database(e.to_string()))?;

        let results: QueryResults =
            serde_json::from_str(&results_raw).map_err(|e| StatusError::Database(e.to_string()))?;

        let values = results
            .results
            .first()
            .ok_or(StatusError::Database(
                "Unexpected response: no results".into(),
            ))?
            .series
            .first()
            .ok_or(StatusError::Database(
                "Unexpected response: no series".into(),
            ))?
            .values
            .first()
            .ok_or(StatusError::Database(
                "Unexpected response: no values".into(),
            ))?;

        let time: DateTime<Utc> = DateTime::parse_from_rfc3339(&values.0)
            .map(|e| e.into())
            .map_err(|e| StatusError::Database(e.to_string()))?;

        Ok((time, values.1))
    }
}
