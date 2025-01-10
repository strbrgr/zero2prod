use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};
use axum::{
    extract::Request,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use tracing::Level;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub connection_pool: EmailClient,
}

pub fn app(pool: PgPool, email_client: EmailClient) -> Router {
    let state = AppState {
        db: pool,
        connection_pool: email_client,
    };

    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .with_state(state)
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                let request_id = uuid::Uuid::new_v4().to_string();

                tracing::span!(
                    Level::INFO,
                    "request",
                    %request_id,
                    method = ?request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                )
            }),
        )
}
