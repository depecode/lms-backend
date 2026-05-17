use actix_web::{get, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/workflows/tasks",
    responses(
        (status = 200, description = "Pending tasks retrieved successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Workflows"
)]
#[get("/tasks")]
pub async fn list_tasks() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Pending tasks retrieved successfully"))
}
