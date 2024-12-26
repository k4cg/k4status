use crate::{configuration, StatusError};
use chrono::{DateTime, Utc};
use serde::Deserialize;

#[derive(Clone)]
pub struct Database {
    client: influxdb::Client,
}

#[derive(Deserialize, Clone)]
pub struct TimeValue {
    pub time: DateTime<Utc>,
    pub value: f64,
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
    ) -> Result<TimeValue, StatusError> {
        let query = influxdb::ReadQuery::new(format!(
            r#"SELECT time, value FROM "{}" WHERE (entity_id = '{}' AND time > '{}') ORDER BY time DESC LIMIT 1"#,
            unit,
            name,
            validity.to_rfc3339()
        ));

        let mut result = self
            .client
            .json_query(query)
            .await
            .map_err(|e| StatusError::Database(e.to_string()))?;

        result
            .deserialize_next::<TimeValue>()
            .map_err(|e| StatusError::Database(format!("unexpected response: {:?}", e)))?
            .series
            .first()
            .ok_or(StatusError::Database(
                "unexpected response: no series".into(),
            ))?
            .values
            .first()
            .ok_or(StatusError::Database(
                "unexpected response: no values".into(),
            ))
            .cloned()
    }
}
