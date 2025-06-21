use axum::{extract::State, http};
use std::sync::Arc;

use crate::server::router::AppState;

pub async fn get_health(State(state): State<Arc<AppState>>) -> http::StatusCode {
    log::info!("GET /health");

    match state.database.check_connection().await {
        Ok(_) => http::StatusCode::OK,
        Err(_) => http::StatusCode::IM_A_TEAPOT,
    }
}
