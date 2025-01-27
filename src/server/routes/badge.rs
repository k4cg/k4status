use std::sync::Arc;

use axum::{body::Body, extract::State, http::header, http::StatusCode, response::Response};

use crate::server::router::AppState;

pub async fn get_badge(State(state): State<Arc<AppState>>) -> Response {
    log::info!("GET /badge");

    let cfg = &state.config.sensors.door;

    let badge = match state
        .database
        .get_value(&cfg.entity, &cfg.unit, cfg.validity)
        .await
    {
        Some(value) => {
            if value.value > 0.5 {
                &state.badges.open
            } else {
                &state.badges.closed
            }
        }
        None => &state.badges.unknown,
    };

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/svg+xml")
        .body(Body::from(badge.clone()))
        .unwrap()
}
