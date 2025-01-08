use axum::{extract::State, response::IntoResponse, Form};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    domain::{NewSubscriber, SubscriberEmail, SubscriberName},
    startup::AppState,
};

#[derive(Deserialize)]
pub struct SignUp {
    email: String,
    name: String,
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
    let name = match SubscriberName::parse(sign_up.name) {
        Ok(name) => name,
        Err(_) => return (StatusCode::BAD_REQUEST, "Error parsing name"),
    };

    let email = match SubscriberEmail::parse(sign_up.email) {
        Ok(email) => email,
        Err(_) => return (StatusCode::BAD_REQUEST, "Error parsing email"),
    };

    let new_subscriber = NewSubscriber { email, name };
    match insert_subscriber(&state.db, &new_subscriber).await {
        Ok(_) => (StatusCode::OK, "subscription successful"),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to execute query.",
        ),
    }
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
        INSERT INTO subscriptions (id, email, name, subscriptions_at)
        VALUES ($1, $2, $3, $4)
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
