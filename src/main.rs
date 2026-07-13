use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use lms_api::config::AppConfig;
use lms_api::db::establish_connection;
use lms_api::routes;

// Direct root health check to satisfy Render's healthCheckPath: /
#[get("/")]
async fn root_health_check() -> impl Responder {
    HttpResponse::Ok().json(serde_json::json!({ "status": "healthy", "service": "lms_api" }))
}

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

    // Dynamic Port Handling: Look for Render's PORT environment variable first
    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| config.app_port.to_string())
        .parse()
        .expect("PORT environment variable must be a valid number");

    // Force the host to listen globally on all interfaces for production routing
    let host = std::env::var("APP_HOST")
        .unwrap_or_else(|_| "0.0.0.0".to_string());

    println!("Starting server at http://{}:{}", host, port);

    // 4. Start HttpServer
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
            .service(root_health_check) // Registers public root URL http://0.0.0.0:PORT/
            .configure(routes::init)    // Registers all your regular api routes
    })
    .bind((host, port))?
    .run()
    .await
}