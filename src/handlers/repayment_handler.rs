use actix_web::{get, post, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/repayments",
    responses(
        (status = 200, description = "List repayment history", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Repayments"
)]
#[get("/")]
pub async fn list_repayments() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("List repayment history"))
}

#[utoipa::path(
    post,
    path = "/api/v1/repayments/record",
    responses(
        (status = 201, description = "Payment recorded", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Repayments"
)]
#[post("/record")]
pub async fn record_payment() -> impl Responder {
    HttpResponse::Created().json(ApiResponse::<()>::message("Payment recorded"))
}

#[utoipa::path(
    get,
    path = "/api/v1/repayments/arrears",
    responses(
        (status = 200, description = "List loans in arrears", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Repayments"
)]
#[get("/arrears")]
pub async fn list_arrears() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("List loans in arrears"))
}
