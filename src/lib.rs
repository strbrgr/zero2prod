use axum::{
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Form, Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignUp {
    email: String,
    name: String,
}

pub async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, "")
}

async fn subscribe(Form(sign_up): Form<SignUp>) -> impl IntoResponse {
    println!("{}", sign_up.email);
    (StatusCode::OK, format!("{}{}", sign_up.email, sign_up.name))
}

pub fn app() -> Router {
    Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
}
