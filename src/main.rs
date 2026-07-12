use actix_web::{web, App, HttpServer};
use lms_api::config::AppConfig;
use lms_api::db::establish_connection;
use lms_api::routes;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 1. Load configuration
    let config = AppConfig::from_env();

    // 2. Initialize logger
    env_logger::init_from_env(env_logger::Env::new().default_filter_or(&config.rust_log));

    // 3. Initialize database connection
    let pool = establish_connection(&config).await;

    // Run database migrations on startup
    println!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");
    println!("Database migrations completed successfully.");

    println!("Starting server at http://{}:{}", config.app_host, config.app_port);

    let host = config.app_host.clone();
    let port = config.app_port;

    // 3. Start HttpServer
    HttpServer::new(move || {
        let cors = actix_cors::Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(web::Data::new(pool.clone()))
            .app_data(web::Data::new(config.clone()))
            .configure(routes::init)
    })
    .bind((host, port))?
    .run()
    .await
}
