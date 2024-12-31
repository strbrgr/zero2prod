use sqlx::PgPool;
use zero2prod::{configuration::get_configuration, startup::app};

#[tokio::main]
async fn main() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    axum::serve(listener, app(connection_pool)).await.unwrap();
}
