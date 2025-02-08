use axum::{extract::State, http};
use chrono::{DateTime, Utc};
use std::sync::Arc;

use crate::server::router::AppState;

const STATUS_OK: http::StatusCode = http::StatusCode::OK;
const STATUS_ERR: http::StatusCode = http::StatusCode::IM_A_TEAPOT;

pub struct StateHealth {
    pub status: http::StatusCode,
    pub last_update: DateTime<Utc>,
}

impl Default for StateHealth {
    fn default() -> Self {
        Self {
            status: STATUS_ERR,
            last_update: chrono::DateTime::<Utc>::MIN_UTC,
        }
    }
}

pub async fn get_health(State(state): State<Arc<AppState>>) -> http::StatusCode {
    log::info!("GET /health");

    match state.database.check_connection().await {
        Ok(_) => STATUS_OK,
        Err(_) => STATUS_ERR,
    }
}

pub async fn get_health_cache(State(state): State<Arc<AppState>>) -> http::StatusCode {
    log::info!("GET /health");

    let mut current = state.state_health.lock().await;
    let now = Utc::now();
    if (now - state.config.cache_time.health) > current.last_update {
        current.last_update = now;
        current.status = match state.database.check_connection().await {
            Ok(_) => STATUS_OK,
            Err(_) => STATUS_ERR,
        };
    }
    current.status
}
