use actix_web::{get, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/notifications/inbox",
    responses(
        (status = 200, description = "Notifications retrieved successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Notifications"
)]
#[get("/inbox")]
pub async fn list_notifications() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Notifications retrieved successfully"))
}
