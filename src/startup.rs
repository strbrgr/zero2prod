use crate::routes::{health_check, subscribe};
use axum::{
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}

pub fn app(pool: PgPool) -> Router {
    let state = AppState { db: pool };

    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(state)
}
