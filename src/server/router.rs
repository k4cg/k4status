use std::sync::Arc;

use crate::badge::Badges;
use crate::icon::Icons;
use crate::spaceapi::SpaceApi;
use crate::{StatusError, configuration::Configuration, database::Database};

use crate::server::routes::*;

#[derive(Clone)]
pub struct AppState {
    pub config: Configuration,
    pub database: Database,
    pub template: SpaceApi,
    pub badges: Badges,
    pub icons: Icons,
}

pub async fn run(
    config: Configuration,
    database: Database,
    template: SpaceApi,
    badges: Badges,
    icons: Icons,
) -> Result<(), StatusError> {
    let state = Arc::new(AppState {
        config,
        database,
        template,
        badges,
        icons,
    });

    let serve_str = format!(
        "{}:{}",
        state.config.server.hostname, state.config.server.port
    );

    let listener = tokio::net::TcpListener::bind(&serve_str)
        .await
        .map_err(|e| {
            StatusError::Server(format!(
                "Failed to bind server port for {} ({})",
                serve_str, e
            ))
        })?;

    let app = axum::Router::new()
        .route("/status", axum::routing::get(get_status))
        .route("/health", axum::routing::get(get_health))
        .route("/badge", axum::routing::get(get_badge))
        .route("/icon/open", axum::routing::get(get_icon_open))
        .route("/icon/closed", axum::routing::get(get_icon_closed))
        .with_state(state);

    axum::serve(listener, app)
        .await
        .map_err(|e| StatusError::Server(format!("Failed to start server ({})", e)))
}
