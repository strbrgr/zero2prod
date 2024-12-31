use axum::{extract::State, response::IntoResponse, Form};
use chrono::Utc;
use hyper::StatusCode;
use serde::Deserialize;
use uuid::Uuid;

use crate::startup::AppState;

#[derive(Deserialize)]
pub struct SignUp {
    email: String,
    name: String,
}

pub async fn subscribe(
    State(state): State<AppState>,
    Form(sign_up): Form<SignUp>,
) -> impl IntoResponse {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions (id, email, name, subscriptions_at)
        VALUES ($1, $2, $3, $4)
    "#,
        Uuid::new_v4(),
        sign_up.email,
        sign_up.name,
        Utc::now()
    )
    .execute(&state.db)
    .await
    {
        Ok(_) => (StatusCode::OK, "subscription successful"),
        Err(e) => {
            println!("Failed to execute query: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to execute query.",
            )
        }
    }
}
