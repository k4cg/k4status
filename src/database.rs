use crate::{configuration, StatusError};
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::vec::Vec;

#[derive(Clone)]
pub struct Database {
    client: influxdb::Client,
}

#[derive(Deserialize, Clone)]
struct QueryResults {
    pub results: Vec<QueryResult>,
}

#[derive(Deserialize, Clone)]
struct QueryResult {
    #[allow(dead_code)]
    pub statement_id: u64,
    pub series: Option<Vec<QuerySeries>>,
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

    pub async fn query_and_extract(
        &self,
        name: &str,
        unit: &str,
        validity: DateTime<Utc>,
    ) -> Result<f64, StatusError> {
        let time = validity
            .timestamp_nanos_opt()
            .ok_or(StatusError::Database("DateTime out of range".into()))?;

        let query = influxdb::ReadQuery::new(format!(
            r#"SELECT time, value FROM "{}" WHERE (entity_id = '{}' AND time > {}) ORDER BY time DESC LIMIT 1"#,
            unit, name, time
        ));

        let results_raw = self
            .client
            .query(query)
            .await
            .map_err(|e| StatusError::Database(e.to_string()))?;

        let results: QueryResults = serde_json::from_str(&results_raw)
            .map_err(|e| StatusError::Database(format!("serde '{}'", e)))?;

        let values = results
            .results
            .first()
            .ok_or(StatusError::Database(
                "unexpected response, no results".into(),
            ))?
            .series
            .as_ref()
            .ok_or(StatusError::Database(
                "unexpected response, statements result does not contain any data".into(),
            ))?
            .first()
            .ok_or(StatusError::Database(
                "unexpected response, no series".into(),
            ))?
            .values
            .first()
            .ok_or(StatusError::Database(
                "unexpected response, no values".into(),
            ))?;

        Ok(values.1)
    }
}
