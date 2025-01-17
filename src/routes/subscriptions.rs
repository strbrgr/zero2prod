use axum::{extract::State, response::IntoResponse, Form};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    email_client::EmailClient,
    startup::AppState,
};

#[derive(Deserialize)]
pub struct SignUp {
    email: String,
    name: String,
}

impl TryFrom<SignUp> for NewSubscriber {
    type Error = String;

    fn try_from(value: SignUp) -> Result<Self, Self::Error> {
        let name = SubscriberName::parse(value.name)?;
        let email = SubscriberEmail::parse(value.email)?;

        Ok(NewSubscriber { email, name })
    }
}

#[tracing::instrument(
    name = "Adding a subscriber",
    skip(state, sign_up),
    fields(
        subscriber_email = %sign_up.email,
        subscriber_name = %sign_up.name
    )
)]
pub async fn subscribe(
    State(state): State<AppState>,
    Form(sign_up): Form<SignUp>,
) -> impl IntoResponse {
    let new_subscriber = match sign_up.try_into() {
        Ok(sign_up) => sign_up,
        Err(_) => return (StatusCode::BAD_REQUEST, "error parsing subscriber"),
    };
    match insert_subscriber(&state.db, &new_subscriber).await {
        Ok(_) => (StatusCode::OK, "subscription successful"),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to execute query.",
        ),
    };

    if send_confirmation_email(&state.email_client, new_subscriber)
        .await
        .is_err()
    {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to execute query.",
        );
    }
    (StatusCode::OK, "subscription successful")
}

#[tracing::instrument(
    name = "Sending confirmation link to new subscriber",
    skip(email_client, new_subscriber)
)]
pub async fn send_confirmation_email(
    email_client: &EmailClient,
    new_subscriber: NewSubscriber,
) -> Result<(), reqwest::Error> {
    let confirmation_link = "https://no-domain.com/subscriptions/confirm";

    let html_body = format!(
        "Welcome to our newsletter!<br />\
                Click <a href=\"{}\">here</a> to confirm your subscription.",
        confirmation_link
    );
    let plain_body = format!(
        "Welcome to our newsletter!\nVisit {} to confirm your subscription.",
        confirmation_link
    );
    email_client
        .send_email(new_subscriber.email, "Welcome!", &html_body, &plain_body)
        .await
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(new_subscriber, pool)
)]
pub async fn insert_subscriber(
    pool: &PgPool,
    new_subscriber: &NewSubscriber,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscriptions_at, status)
        VALUES ($1, $2, $3, $4, 'confirmed')
    "#,
        Uuid::new_v4(),
        new_subscriber.email.as_ref(),
        new_subscriber.name.as_ref(),
        Utc::now()
    )
    .execute(pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query: {:?}", e);
        e
    })?;
    Ok(())
}
