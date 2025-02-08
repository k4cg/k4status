use chrono::{DateTime, Utc};
use std::sync::Arc;

use axum::{body::Body, extract::State, http::header, http::StatusCode, response::Response};

use crate::server::router::AppState;

pub struct StateBadge {
    pub badge: String,
    pub last_update: DateTime<Utc>,
}

impl StateBadge {
    pub fn new(badge: &str) -> Self {
        Self {
            badge: String::from(badge),
            last_update: DateTime::<Utc>::MIN_UTC,
        }
    }
}

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

fn build_response(badge: String) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/svg+xml")
        .body(Body::from(badge))
        .unwrap()
}

pub async fn get_badge(State(state): State<Arc<AppState>>) -> Response {
    log::info!("GET /badge");

    build_response(query(&state).await)
}

pub async fn get_badge_cache(State(state): State<Arc<AppState>>) -> Response {
    log::info!("GET /badge");

    let mut current = state.state_badge.lock().await;
    let now = Utc::now();
    if (now - state.config.cache_time.badge) > current.last_update {
        current.last_update = now;
        current.badge = query(&state).await;
    }
    build_response(current.badge.clone())
}
