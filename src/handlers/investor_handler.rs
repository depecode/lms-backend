use actix_web::{get, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/investors",
    responses(
        (status = 200, description = "List investors", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Investors"
)]
#[get("/")]
pub async fn list_investors() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("List investors"))
}
