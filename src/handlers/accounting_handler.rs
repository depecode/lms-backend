use actix_web::{get, post, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/accounting/ledger",
    responses(
        (status = 200, description = "General ledger entries", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Accounting"
)]
#[get("/ledger")]
pub async fn get_ledger() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("General ledger entries"))
}

#[utoipa::path(
    get,
    path = "/api/v1/accounting/statements",
    responses(
        (status = 200, description = "Financial statements", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Accounting"
)]
#[get("/statements")]
pub async fn get_statements() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Financial statements"))
}

#[utoipa::path(
    post,
    path = "/api/v1/accounting/other-income",
    responses(
        (status = 201, description = "Non-loan revenue recorded", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Accounting"
)]
#[post("/other-income")]
pub async fn record_other_income() -> impl Responder {
    HttpResponse::Created().json(ApiResponse::<()>::message("Non-loan revenue recorded"))
}
