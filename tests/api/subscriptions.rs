use crate::helpers::spawn_app;
use axum::http::StatusCode;
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    // Arrange
    let test_app = spawn_app().await;

    // Act
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = test_app.post_subscriptions(body.into()).await;

    // Assert
    assert_eq!(StatusCode::OK, response.status().as_u16());
    let saved = sqlx::query!("SELECT email, name FROM subscriptions")
        .fetch_one(&test_app.connection_pool)
        .await
        .expect("Failed to fetch saved subscription.");
    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");

    //TODO: Delete created database
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    // Arrange
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        // Act
        let response = test_app.post_subscriptions(invalid_body.into()).await;

        // Assert
        assert_eq!(
            StatusCode::UNPROCESSABLE_ENTITY,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_for_invalid_data() {
    // Arrange
    let test_app = spawn_app().await;

    let test_cases = vec![
        ("name=&email=james@x.com", "this should have a name"),
        ("name=Jenny&email=", "missing the email"),
        ("name=Bob&email=yes@%", "corrupted email"),
    ];

    for (invalid_body, error_message) in test_cases {
        // Act
        let response = test_app.post_subscriptions(invalid_body.into()).await;
        // Assert
        assert_eq!(
            StatusCode::BAD_REQUEST,
            response.status().as_u16(),
            // Additional customised error message on test failure
            "The API did not fail with 400 Bad Request when the payload was {}.",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    // Arrange
    let test_app = spawn_app().await;
    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Act
    test_app.post_subscriptions(body.into()).await;
    // Assert
}
