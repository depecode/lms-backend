use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub database_url: String,
    pub rust_log: String,
    pub app_host: String,
    pub app_port: u16,
    pub jwt_secret: String,
    pub jwt_exp_hours: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();

        let app_port = env::var("PORT")
            .or_else(|_| env::var("APP_PORT"))
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .expect("PORT must be a number");

        let app_host = env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        Self {
            database_url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
            app_host,
            app_port,
            jwt_secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            jwt_exp_hours: env::var("JWT_EXP_HOURS")
                .unwrap_or_else(|_| "24".to_string())
                .parse()
                .expect("JWT_EXP_HOURS must be a number"),
        }
    }
}
