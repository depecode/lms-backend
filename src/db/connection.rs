use sqlx::postgres::{PgPool, PgPoolOptions};
use crate::config::AppConfig;
use std::time::Duration;
use log::{info, warn};

pub async fn establish_connection(config: &AppConfig) -> PgPool {
    let mut attempts = 0;
    let max_attempts = 5;
    let delay = Duration::from_secs(3);

    loop {
        attempts += 1;
        match PgPoolOptions::new()
            .max_connections(5)
            .connect(&config.database_url)
            .await
        {
            Ok(pool) => {
                info!("Successfully connected to the database after {} attempt(s).", attempts);
                return pool;
            }
            Err(err) => {
                if attempts >= max_attempts {
                    panic!("Failed to connect to Postgres after {} attempts: {}", max_attempts, err);
                }
                warn!(
                    "Database connection attempt {} failed ({}). Retrying in {:?}...",
                    attempts, err, delay
                );
                tokio::time::sleep(delay).await;
            }
        }
    }
}
