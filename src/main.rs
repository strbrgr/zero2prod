use sqlx::PgPool;
use tokio::net::TcpListener;
use zero2prod::{configuration::get_configuration, startup::app};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres");
    let address = format!("127.0.0.1:{}", &configuration.application_port);
    // let listener = tokio::net::TcpListener::bind("127.0.0.1:{}").await.unwrap();
    let listener = TcpListener::bind(address).await?;
    axum::serve(listener, app(connection_pool)).await.unwrap();

    Ok(())
}
