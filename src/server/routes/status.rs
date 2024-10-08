use axum::{extract::State, http, Json};
use chrono::{DateTime, Utc};
use spaceapi::{sensors, Status as SpaceStatus};
use std::sync::Arc;
use std::vec::Vec;

use crate::{
    configuration,
    database::{self, TimeValue},
};

use crate::server::router::AppState;

#[derive(Clone)]
pub struct StateStatus {
    pub status: SpaceStatus,
    pub last_update: DateTime<Utc>,
}

impl StateStatus {
    pub fn new(status: SpaceStatus) -> Self {
        Self {
            status,
            last_update: chrono::DateTime::<Utc>::MIN_UTC,
        }
    }
}

pub async fn get_status(
    State(state): State<Arc<AppState>>,
) -> (http::StatusCode, Json<SpaceStatus>) {
    log::info!("GET /status.json");

    (http::StatusCode::OK, Json(update_template(&state).await))
}

pub async fn get_status_cache(
    State(state): State<Arc<AppState>>,
) -> (http::StatusCode, Json<SpaceStatus>) {
    log::info!("GET /status.json");

    let status = {
        let mut current = state.state_status.lock().await;
        let now = Utc::now();
        if (now - state.config.cache_time.status_json) > current.last_update {
            current.last_update = now;
            current.status = update_template(&state).await;
        }
        current.status.clone()
    };

    (http::StatusCode::OK, Json(status))
}

async fn update_template(appstate: &Arc<AppState>) -> SpaceStatus {
    log::debug!("Update template with new values");
    let temperature = get_temperature(&appstate.database, &appstate.config.sensors).await;
    let humidity = get_humidity(&appstate.database, &appstate.config.sensors).await;
    let door_status = get_door(&appstate.database, &appstate.config.sensors).await;

    let mut template = appstate.template.clone();

    if !temperature.is_empty() || !humidity.is_empty() {
        let mut sensors = sensors::Sensors::default();
        if !temperature.is_empty() {
            sensors.temperature = temperature;
        }
        if !humidity.is_empty() {
            sensors.humidity = humidity;
        }
        template.sensors = Some(sensors);
    }

    template.state = door_status;

    template
}

async fn get_temperature(
    database: &database::Database,
    config: &configuration::Sensors,
) -> Vec<sensors::TemperatureSensor> {
    log::debug!("Query temperature");

    let mut sensors: Vec<sensors::TemperatureSensor> = Vec::new();

    for sensor in config.temperature.id.iter() {
        match database
            .get_temperature(&sensor.entity, &config.temperature.unit)
            .await
        {
            Ok(temp) => match validate_time(&temp, config.temperature.validity) {
                Some(value) => sensors.push(sensors::TemperatureSensor {
                    value: *value,
                    unit: config.temperature.unit.clone(),
                    metadata: sensors::SensorMetadataWithLocation {
                        location: sensor.location.clone(),
                        ..Default::default()
                    },
                }),
                None => log::warn!(
                    "Latest sensor measurement too old: id='{}' unit='{}' sample_time='{}' validity='{}'",
                    sensor.entity,
                    config.temperature.unit,
                    temp.time,
                    config.temperature.validity
                ),
            },
            Err(err) => log::warn!(
                "Failed to query temperature: id='{}' unit='{}' ({})",
                sensor.entity,
                config.temperature.unit,
                err
            ),
        }
    }

    sensors
}

async fn get_humidity(
    database: &database::Database,
    config: &configuration::Sensors,
) -> Vec<sensors::HumiditySensor> {
    log::debug!("Query humidity");

    let mut sensors: Vec<sensors::HumiditySensor> = Vec::new();

    for sensor in config.humidity.id.iter() {
        match database
            .get_humidity(&sensor.entity, &config.humidity.unit)
            .await
        {
            Ok(temp) => match validate_time(&temp, config.humidity.validity) {
                Some(value) => sensors.push(sensors::HumiditySensor {
                    value: *value as f64,
                    unit: config.humidity.unit.clone(),
                    metadata: sensors::SensorMetadataWithLocation {
                        location: sensor.location.clone(),
                        ..Default::default()
                    },
                }),
                None => log::warn!(
                    "Latest sensor measurement too old: id='{}' unit='{}' sample_time='{}' validity='{}'",
                    sensor.entity,
                    config.humidity.unit,
                    temp.time,
                    config.humidity.validity
                ),
            },
            Err(err) => log::warn!(
                "Failed to query humidity: id='{}' unit='{}' ({})",
                sensor.entity,
                config.humidity.unit,
                err
            ),
        }
    }

    sensors
}

async fn get_door(
    database: &database::Database,
    config: &configuration::Sensors,
) -> Option<spaceapi::State> {
    log::debug!("Query door status");

    match database
        .get_door_status(&config.door.entity, &config.door.unit)
        .await
    {
        Ok(door) => validate_time(&door, config.door.validity).map(|val| spaceapi::State {
            open: Some(*val),
            ..Default::default()
        }),
        Err(_) => None,
    }
}

fn validate_time<T>(tv: &TimeValue<T>, validity: chrono::TimeDelta) -> Option<&T> {
    let start = chrono::Utc::now() - validity;
    if tv.time > start || validity.is_zero() {
        Some(&tv.value)
    } else {
        None
    }
}
