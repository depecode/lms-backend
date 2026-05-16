use actix_web::{post, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;
use sqlx::PgPool;
use crate::auth::jwt::create_jwt;
// use bcrypt::verify; // if you store hashed passwords

#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[post("/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    payload: web::Json<LoginRequest>,
) -> impl Responder {
    // --- STUB: replace with real DB lookup & password verify ---
    // Example: SELECT id, password_hash FROM users WHERE email = $1
    // then verify with bcrypt::verify(&payload.password, &password_hash)

    // For demonstration: find user id by email (simple query)
    let user = sqlx::query!("SELECT id, email FROM users WHERE email = $1", payload.email)
        .fetch_optional(pool.get_ref())
        .await;

    match user {
        Ok(Some(row)) => {
            // TODO: verify password hash here
            // if !verify(&payload.password, &row.password_hash).unwrap_or(false) { ... }

            let user_id: Uuid = row.id;
            let token = create_jwt(user_id, &row.email);
            match token {
                Ok(t) => HttpResponse::Ok().json(serde_json::json!({ "token": t })),
                Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().body("Invalid credentials"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
