use sqlx::postgres::{PgPool, PgPoolOptions};
use crate::config::AppConfig;

pub async fn establish_connection(config: &AppConfig) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database_url)
        .await
        .expect("Failed to connect to Postgres")
}
