use actix_web::{App, HttpServer, web};
use handlers::auth_handler;
use sqlx::postgres::PgPoolOptions;
use dotenvy::dotenv;
use std::env;

mod auth;
mod routes;
mod handlers;
mod models;
mod middleware;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))  // ✅ Register here
            .service(handlers::auth_handler::login)
            .configure(routes::init)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}