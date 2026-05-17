use actix_web::{get, post, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/settings/branches",
    responses(
        (status = 200, description = "List branches", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Settings"
)]
#[get("/branches")]
pub async fn list_branches() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("List branches"))
}

#[utoipa::path(
    post,
    path = "/api/v1/settings/branches",
    responses(
        (status = 201, description = "Branch created", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Settings"
)]
#[post("/branches")]
pub async fn create_branch() -> impl Responder {
    HttpResponse::Created().json(ApiResponse::<()>::message("Branch created"))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/staff",
    responses(
        (status = 200, description = "List staff", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Settings"
)]
#[get("/staff")]
pub async fn list_staff() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("List staff"))
}

#[utoipa::path(
    post,
    path = "/api/v1/settings/staff",
    responses(
        (status = 201, description = "Staff member created", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Settings"
)]
#[post("/staff")]
pub async fn create_staff() -> impl Responder {
    HttpResponse::Created().json(ApiResponse::<()>::message("Staff member created"))
}

#[utoipa::path(
    get,
    path = "/api/v1/settings/audit-logs",
    responses(
        (status = 200, description = "System audit logs", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Settings"
)]
#[get("/audit-logs")]
pub async fn get_audit_logs() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("System audit logs"))
}
