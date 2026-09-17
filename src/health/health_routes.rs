use axum::{Router, routing::get};

use crate::app::AppState;

use super::health_handler::health;

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(health))
}
