use actix_web::{get, post, web, HttpResponse, Responder};
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

// FRONTEND COMPATIBLE ENDPOINTS UNDER /api/reports
#[utoipa::path(
    get,
    path = "/api/v1/reports",
    responses(
        (status = 200, description = "Get list of reports", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Reports"
)]
#[get("")]
pub async fn get_reports() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!([
            { "id": "1", "name": "Portfolio Quality Report", "description": "Analysis of loan portfolio performance", "type": "Portfolio", "category": "Credit", "frequency": "Monthly", "format": "PDF" },
            { "id": "2", "name": "Collection Efficiency", "description": "Tracking repayment rates", "type": "Collection", "category": "Operations", "frequency": "Daily", "format": "Excel" }
        ]),
        "Reports list retrieved successfully"
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/reports/{id}/generate",
    responses(
        (status = 200, description = "Generate report file")
    ),
    tag = "Reports"
)]
#[post("/{id}/generate")]
pub async fn generate_report(path: web::Path<String>) -> impl Responder {
    let _id = path.into_inner();
    HttpResponse::Ok()
        .content_type("application/pdf")
        .body("mock_pdf_report_content")
}

#[utoipa::path(
    post,
    path = "/api/v1/reports/{id}/schedule",
    responses(
        (status = 200, description = "Schedule report execution", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Reports"
)]
#[post("/{id}/schedule")]
pub async fn schedule_report(path: web::Path<String>) -> impl Responder {
    let _id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message("Report scheduled successfully"))
}

#[utoipa::path(
    get,
    path = "/api/v1/reports/kpi",
    responses(
        (status = 200, description = "Get reports KPIs summary", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Reports"
)]
#[get("/kpi")]
pub async fn get_report_kpi() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "totalGenerated": 15,
            "failedRuns": 0,
            "activeSchedules": 3
        }),
        "Reports KPI summary retrieved successfully"
    ))
}
