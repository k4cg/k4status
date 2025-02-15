use std::sync::Arc;
use tokio::sync::Mutex;

use crate::badge::Badges;
use crate::spaceapi::SpaceApi;
use crate::{configuration::Configuration, database::Database, StatusError};

use crate::server::routes::*;

#[derive(Clone)]
pub struct AppState {
    pub config: Configuration,
    pub database: Database,
    pub template: SpaceApi,
    pub badges: Badges,
    pub state_status: Arc<Mutex<StateStatus>>,
    pub state_health: Arc<Mutex<StateHealth>>,
    pub state_badge: Arc<Mutex<StateBadge>>,
}

pub async fn run(
    config: &Configuration,
    database: &Database,
    status: &SpaceApi,
    badges: &Badges,
) -> Result<(), StatusError> {
    let state = Arc::new(AppState {
        config: config.clone(),
        database: database.clone(),
        template: status.clone(),
        badges: badges.clone(),
        state_status: Arc::new(Mutex::new(StateStatus::new(status.clone()))),
        state_health: Arc::new(Mutex::new(StateHealth::default())),
        state_badge: Arc::new(Mutex::new(StateBadge::new(&badges.unknown))),
    });

    let route_status = match config.cache_time.status.is_zero() {
        true => axum::routing::get(get_status),
        false => axum::routing::get(get_status_cache),
    };

    let route_health = match config.cache_time.health.is_zero() {
        true => axum::routing::get(get_health),
        false => axum::routing::get(get_health_cache),
    };

    let route_badge = match config.cache_time.badge.is_zero() {
        true => axum::routing::get(get_badge),
        false => axum::routing::get(get_badge_cache),
    };

    let app = axum::Router::new()
        .route("/status", route_status)
        .route("/health", route_health)
        .route("/badge", route_badge)
        .with_state(state);

    let serve_str = format!("{}:{}", config.server.hostname, config.server.port);

    let listener = tokio::net::TcpListener::bind(&serve_str)
        .await
        .map_err(|e| {
            StatusError::Server(format!(
                "Failed to bind server port for {} ({})",
                serve_str, e
            ))
        })?;

    axum::serve(listener, app)
        .await
        .map_err(|e| StatusError::Server(format!("Failed to start server ({})", e)))
}
