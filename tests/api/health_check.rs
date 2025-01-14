use crate::helpers::{configure_database, spawn_app};

use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use reqwest::Client;
use tower::ServiceExt;
use uuid::Uuid;
use zero2prod::{configuration::get_configuration, email_client::EmailClient};

#[tokio::test]
async fn health_check_oneshot() {
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    let connection_pool = configure_database(&configuration.database).await;

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address.");

    let base_url =
        reqwest::Url::parse(&configuration.email_client.base_url).expect("Could not parse url");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );
    let app = zero2prod::startup::app(connection_pool, email_client);

    let resp = app
        .oneshot(
            Request::builder()
                .uri("/health_check")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(resp.status(), StatusCode::OK);

    let body = resp.into_body().collect().await.unwrap().to_bytes();
    assert_eq!(&body[..], b"");
}

#[tokio::test]
async fn health_check_works() {
    let (app_address, _) = spawn_app().await;

    let client = Client::new();
    let resp = client
        .get(format!("http://{app_address}/health_check"))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(resp.status().is_success());
    assert_eq!(resp.content_length(), Some(0));
}
