use axum::{extract::State, response::IntoResponse, Form};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::startup::AppState;

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
    match insert_subscriber(&state.db, &sign_up).await {
        Ok(_) => (StatusCode::OK, "subscription successful"),
        Err(_) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to execute query.",
        ),
    }
}

#[tracing::instrument(
    name = "Saving new subscriber details in the database",
    skip(form, pool)
)]
pub async fn insert_subscriber(pool: &PgPool, form: &SignUp) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscriptions_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        form.email,
        form.name,
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
