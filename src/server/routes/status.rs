use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
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
    let temp = get_generic_sensor(&appstate.database, &appstate.config.sensors.temperature).await;
    let humid = get_generic_sensor(&appstate.database, &appstate.config.sensors.humidity).await;
    let co2 = get_generic_sensor(&appstate.database, &appstate.config.sensors.carbondioxide).await;
    let door = get_door(&appstate.database, &appstate.config.sensors.door).await;

    let mut template = appstate.template.clone();
    template.state = door;

    if !temp.is_empty() || !humid.is_empty() || !co2.is_empty() {
        template.sensors = Some(sensors::Sensors {
            temperature: temp,
            humidity: humid,
            carbondioxide: co2,
            ..Default::default()
        })
    }

    template
}

async fn get_generic_sensor<T>(
    db: &database::Database,
    cfg: &configuration::SensorSettings,
) -> Vec<T>
where
    T: From<GenericSensor>,
{
    let mut sensors: Vec<T> = Vec::new();

    for sensor in cfg.id.iter() {
        if let Some(value) = get_value(db, &sensor.entity, &cfg.unit, cfg.validity).await {
            sensors.push(
                GenericSensor {
                    value: value.value,
                    unit: cfg.unit.clone(),
                    metadata: sensors::SensorMetadataWithLocation {
                        location: sensor.location.clone(),
                        ..Default::default()
                    },
                }
                .into(),
            );
        }
    }

    sensors
}

async fn get_door(
    db: &database::Database,
    cfg: &configuration::DoorSettings,
) -> Option<spaceapi::State> {
    get_value(db, &cfg.entity, &cfg.unit, cfg.validity)
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
    match database.query_and_extract(entity, unit, validity).await {
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

// There is no definition of a generic sensor in the spaceapi crate.
// The sensors we currently use share the same data fields but are implemented as seperate types.
// Therefore we define our own generic type and implement the From trait for the sensors we currently use.
#[derive(Clone)]
struct GenericSensor {
    value: f64,
    unit: String,
    metadata: sensors::SensorMetadataWithLocation,
}

impl From<GenericSensor> for sensors::TemperatureSensor {
    fn from(entry: GenericSensor) -> sensors::TemperatureSensor {
        sensors::TemperatureSensor {
            value: entry.value,
            unit: entry.unit.clone(),
            metadata: entry.metadata.clone(),
        }
    }
}

impl From<GenericSensor> for sensors::HumiditySensor {
    fn from(entry: GenericSensor) -> sensors::HumiditySensor {
        sensors::HumiditySensor {
            value: entry.value,
            unit: entry.unit.clone(),
            metadata: entry.metadata.clone(),
        }
    }
}

impl From<GenericSensor> for sensors::CarbondioxideSensor {
    fn from(entry: GenericSensor) -> sensors::CarbondioxideSensor {
        sensors::CarbondioxideSensor {
            value: entry.value as u64,
            unit: entry.unit.clone(),
            metadata: entry.metadata.clone(),
        }
    }
}
