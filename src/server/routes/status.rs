use axum::{extract::State, Json};
use chrono::{DateTime, Utc};
use num_traits::AsPrimitive;
use std::sync::Arc;
use std::vec::Vec;

use crate::{configuration, database};
use crate::{spaceapi, spaceapi::Sensor, spaceapi::Sensors, spaceapi::SpaceApi};

use crate::server::router::AppState;

pub struct StateStatus {
    pub status: SpaceApi,
    pub last_update: DateTime<Utc>,
}

impl StateStatus {
    pub fn new(status: SpaceApi) -> Self {
        Self {
            status,
            last_update: DateTime::<Utc>::MIN_UTC,
        }
    }
}

pub async fn get_status(State(state): State<Arc<AppState>>) -> Json<SpaceApi> {
    log::info!("GET /status");

    Json(update_template(&state).await)
}

pub async fn get_status_cache(State(state): State<Arc<AppState>>) -> Json<SpaceApi> {
    log::info!("GET /status");

    let status = {
        let mut current = state.state_status.lock().await;
        let now = Utc::now();
        if (now - state.config.cache_time.status) > current.last_update {
            current.last_update = now;
            current.status = update_template(&state).await;
        }
        current.status.clone()
    };

    Json(status)
}

async fn update_template(appstate: &Arc<AppState>) -> SpaceApi {
    let temp = get_generic_sensor(&appstate.database, &appstate.config.sensors.temperature).await;
    let humid = get_generic_sensor(&appstate.database, &appstate.config.sensors.humidity).await;
    let co2 = get_generic_sensor(&appstate.database, &appstate.config.sensors.carbondioxide).await;
    let door = get_door(&appstate.database, &appstate.config.sensors.door).await;

    let mut template = appstate.template.clone();

    if let Some(ref mut state) = template.state {
        if let Some(d) = door {
            state.open = d.open;
            state.lastchange = d.lastchange;
        }
    } else {
        template.state = door;
    }

    if !temp.is_empty() || !humid.is_empty() || !co2.is_empty() {
        template.sensors = Some(Sensors {
            temperature: temp,
            humidity: humid,
            carbondioxide: co2,
        })
    }

    template
}

async fn get_generic_sensor<T>(
    db: &database::Database,
    cfg: &configuration::SensorSettings,
) -> Vec<Sensor<T>>
where
    T: AsPrimitive<T>,
    f64: AsPrimitive<T>,
{
    let mut sensors: Vec<Sensor<T>> = Vec::new();

    for sensor in cfg.id.iter() {
        if let Some(value) = db.get_value(&sensor.entity, &cfg.unit, cfg.validity).await {
            sensors.push(Sensor {
                value: value.value.as_(),
                unit: cfg.unit.clone(),
                location: sensor.location.clone(),
                lastchange: Some(value.time.timestamp() as u64),
            });
        }
    }

    sensors
}

async fn get_door(
    db: &database::Database,
    cfg: &configuration::DoorSettings,
) -> Option<spaceapi::State> {
    db.get_value(&cfg.entity, &cfg.unit, cfg.validity)
        .await
        .map(|value| spaceapi::State {
            open: Some(value.value > 0.5),
            lastchange: Some(value.time.timestamp() as u64),
            ..Default::default()
        })
}
