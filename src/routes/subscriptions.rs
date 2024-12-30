use axum::{response::IntoResponse, Form};
use hyper::StatusCode;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SignUp {
    email: String,
    name: String,
}

pub async fn subscribe(Form(sign_up): Form<SignUp>) -> impl IntoResponse {
    println!("{}", sign_up.email);
    (StatusCode::OK, format!("{}{}", sign_up.email, sign_up.name))
}
