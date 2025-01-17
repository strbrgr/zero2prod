use std::net::SocketAddr;

use crate::{
    configuration::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{health_check, subscribe},
};
use axum::{
    extract::Request,
    routing::{get, post},
    Router,
};
use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::Level;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub email_client: EmailClient,
}

pub struct Application {
    port: u16,
    router: Router,
    listener: TcpListener,
}

pub fn router(pool: PgPool, email_client: EmailClient) -> Router {
    let state = AppState {
        db: pool,
        email_client,
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

impl Application {
    pub async fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let connection_pool = get_connection_pool(&configuration.database);
        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let port = configuration.application.port;

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address.");
        let timeout = configuration.email_client.timeout();
        let base_url =
            reqwest::Url::parse(&configuration.email_client.base_url).expect("Could not parse url");
        let email_client = EmailClient::new(
            base_url,
            sender_email,
            configuration.email_client.authorization_token,
            timeout,
        );
        let listener = tokio::net::TcpListener::bind(address).await.unwrap();

        Ok(Application {
            port,
            router: router(connection_pool, email_client),
            listener,
        })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub fn address(&self) -> SocketAddr {
        self.listener.local_addr().unwrap()
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        axum::serve(self.listener, self.router).await
    }
}

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.with_db())
}
