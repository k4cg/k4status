use std::sync::Arc;

use axum::{body::Body, extract::State, http::StatusCode, http::header, response::Response};

use crate::server::router::AppState;

async fn query(appstate: &Arc<AppState>) -> String {
    let cfg = &appstate.config.sensors.door;

    match appstate
        .database
        .get_value(&cfg.entity, &cfg.unit, cfg.validity)
        .await
    {
        Some(value) => {
            if value.value > 0.5 {
                appstate.badges.open.clone()
            } else {
                appstate.badges.closed.clone()
            }
        }
        None => appstate.badges.unknown.clone(),
    }
}

pub async fn get_badge(State(state): State<Arc<AppState>>) -> Response {
    log::info!("GET /badge");

    let badge = query(&state).await;

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/svg+xml")
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(badge))
        .expect("Failed to build response")
}
