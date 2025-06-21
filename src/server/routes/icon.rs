use std::sync::Arc;

use axum::{body::Body, extract::State, http::StatusCode, http::header, response::Response};

use crate::icon::Image;
use crate::server::router::AppState;

fn build_response(badge: Image) -> Response {
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "image/png")
        .body(Body::from(badge))
        .unwrap()
}

pub async fn get_icon_open(State(state): State<Arc<AppState>>) -> Response {
    log::info!("GET /icon/open");
    build_response(state.icons.open.clone())
}

pub async fn get_icon_closed(State(state): State<Arc<AppState>>) -> Response {
    log::info!("GET /icon/closed");
    build_response(state.icons.closed.clone())
}
