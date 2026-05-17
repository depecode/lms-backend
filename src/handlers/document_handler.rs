use actix_web::{post, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    post,
    path = "/api/v1/docs",
    responses(
        (status = 200, description = "Document uploaded", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Documents"
)]
#[post("/")]
pub async fn upload_doc() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Document uploaded"))
}
