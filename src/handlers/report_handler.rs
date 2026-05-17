use actix_web::{get, post, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/reports/portfolio",
    responses(
        (status = 200, description = "Portfolio analytics summary", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Reports"
)]
#[get("/portfolio")]
pub async fn get_portfolio_summary() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Portfolio analytics summary"))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/loans/stats",
    responses(
        (status = 200, description = "Loan disbursement/repayment stats", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Reports"
)]
#[get("/loans/stats")]
pub async fn get_loan_stats() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Loan disbursement/repayment stats"))
}

#[utoipa::path(
    post,
    path = "/api/v1/reports/export",
    responses(
        (status = 200, description = "Report exported", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Reports"
)]
#[post("/export")]
pub async fn export_report() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Report exported"))
}
