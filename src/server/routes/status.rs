use axum::{extract::State, Json};
use chrono::{DateTime, TimeZone, Utc};
use spaceapi::{sensors, Status as SpaceStatus};
use std::sync::Arc;
use std::vec::Vec;

use crate::{configuration, database};

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

pub async fn get_status(State(state): State<Arc<AppState>>) -> Json<SpaceStatus> {
    log::info!("GET /status.json");

    Json(update_template(&state).await)
}

pub async fn get_status_cache(State(state): State<Arc<AppState>>) -> Json<SpaceStatus> {
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

    Json(status)
}

async fn update_template(appstate: &Arc<AppState>) -> SpaceStatus {
    log::debug!("Update template with new values");
    let temperature = get_temperature(&appstate.database, &appstate.config.sensors).await;
    let humidity = get_humidity(&appstate.database, &appstate.config.sensors).await;
    let co2 = get_carbondioxide(&appstate.database, &appstate.config.sensors).await;
    let door_status = get_door(&appstate.database, &appstate.config.sensors).await;

    let mut template = appstate.template.clone();

    if !temperature.is_empty() || !humidity.is_empty() || !co2.is_empty() {
        let mut sensors = sensors::Sensors::default();
        if !temperature.is_empty() {
            sensors.temperature = temperature;
        }
        if !humidity.is_empty() {
            sensors.humidity = humidity;
        }
        if !co2.is_empty() {
            sensors.carbondioxide = co2;
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
    let mut sensors: Vec<sensors::TemperatureSensor> = Vec::new();

    for sensor in config.temperature.id.iter() {
        if let Some(value) = get_value(
            database,
            &sensor.entity,
            &config.temperature.unit,
            config.temperature.validity,
        )
        .await
        {
            sensors.push(sensors::TemperatureSensor {
                value: value.value,
                unit: config.temperature.unit.clone(),
                metadata: sensors::SensorMetadataWithLocation {
                    location: sensor.location.clone(),
                    ..Default::default()
                },
            });
        }
    }

    sensors
}

async fn get_humidity(
    database: &database::Database,
    config: &configuration::Sensors,
) -> Vec<sensors::HumiditySensor> {
    let mut sensors: Vec<sensors::HumiditySensor> = Vec::new();

    for sensor in config.humidity.id.iter() {
        if let Some(value) = get_value(
            database,
            &sensor.entity,
            &config.humidity.unit,
            config.humidity.validity,
        )
        .await
        {
            sensors.push(sensors::HumiditySensor {
                value: value.value,
                unit: config.humidity.unit.clone(),
                metadata: sensors::SensorMetadataWithLocation {
                    location: sensor.location.clone(),
                    ..Default::default()
                },
            });
        }
    }

    sensors
}

async fn get_carbondioxide(
    database: &database::Database,
    config: &configuration::Sensors,
) -> Vec<sensors::CarbondioxideSensor> {
    let mut sensors: Vec<sensors::CarbondioxideSensor> = Vec::new();

    for sensor in config.carbondioxide.id.iter() {
        if let Some(value) = get_value(
            database,
            &sensor.entity,
            &config.carbondioxide.unit,
            config.carbondioxide.validity,
        )
        .await
        {
            sensors.push(sensors::CarbondioxideSensor {
                value: value.value as u64,
                unit: config.carbondioxide.unit.clone(),
                metadata: sensors::SensorMetadataWithLocation {
                    location: sensor.location.clone(),
                    ..Default::default()
                },
            });
        }
    }

    sensors
}

async fn get_door(
    database: &database::Database,
    config: &configuration::Sensors,
) -> Option<spaceapi::State> {
    get_value(
        database,
        &config.door.entity,
        &config.door.unit,
        config.door.validity,
    )
    .await
    .map(|value| spaceapi::State {
        open: Some(value.value > 0.5),
        lastchange: Some(value.time.timestamp() as u64),
        ..Default::default()
    })
}

async fn get_value(
    database: &database::Database,
    entity: &str,
    unit: &str,
    validity: chrono::TimeDelta,
) -> Option<database::TimeValue> {
    match database
        .query_and_extract(entity, unit, get_start_time(validity))
        .await
    {
        Ok(temp) => Some(temp),
        Err(err) => {
            log::warn!(
                "Failed to get measurement for entity='{}' unit='{}' validity='{}' ({})",
                entity,
                unit,
                validity,
                err
            );
            None
        }
    }
}

fn get_start_time(validity: chrono::TimeDelta) -> DateTime<Utc> {
    if validity.is_zero() {
        Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap()
    } else {
        Utc::now() - validity
    }
}
