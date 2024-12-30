use axum::{
    routing::{get, post},
    Router,
};

use crate::routes::{health_check, subscribe};

pub fn app() -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}
