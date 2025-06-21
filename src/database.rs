use crate::{StatusError, configuration};
use chrono::{DateTime, TimeDelta, Utc};
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

    pub async fn get_value(
        &self,
        entity: &str,
        unit: &str,
        validity: TimeDelta,
    ) -> Option<TimeValue> {
        match self.query_and_extract(entity, unit, validity).await {
            Ok(temp) => Some(temp),
            Err(err) => {
                log::warn!(
                    "Failed to get measurement for entity='{}' unit='{}' validity='{:?}' ({})",
                    entity,
                    unit,
                    validity,
                    err
                );
                None
            }
        }
    }

    async fn query_and_extract(
        &self,
        name: &str,
        unit: &str,
        validity: TimeDelta,
    ) -> Result<TimeValue, StatusError> {
        let time = match validity.is_zero() {
            true => "".into(),
            false => format!(r#"AND time > now() - {}s"#, validity.num_seconds()),
        };

        let query = influxdb::ReadQuery::new(format!(
            r#"SELECT time, value FROM "{}" WHERE (entity_id = '{}' {}) ORDER BY time DESC LIMIT 1"#,
            unit, name, time
        ));

        self.client
            .json_query(query)
            .await
            .map_err(|e| StatusError::Database(e.to_string()))?
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
