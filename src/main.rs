use sqlx::postgres::PgPoolOptions;
use zero2prod::{
    configuration::get_configuration,
    startup::app,
    telemetry::{get_subscriber, init_subscriber},
};

#[tokio::main]
async fn main() {
    let subscriber = get_subscriber("zero2prod".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configuration().expect("Failed to read configuration.");
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = tokio::net::TcpListener::bind(address).await.unwrap();
    let connection_pool = PgPoolOptions::new().connect_lazy_with(configuration.database.with_db());
    axum::serve(listener, app(connection_pool)).await.unwrap();
}
