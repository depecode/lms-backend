use actix_web::{get, HttpResponse, Responder};
use serde_json::json;

#[utoipa::path(
    get,
    path = "/api/v1/health/health",
    responses(
        (status = 200, description = "Health check", body = serde_json::Value)
    ),
    tag = "Health"
)]
// 🛑 FIX: Change the routing path to match the full API path explicitly
#[get("/api/v1/health/health")]
pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json(json!({
        "status": "ok"
    }))
}