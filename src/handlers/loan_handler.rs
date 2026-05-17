use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/loans",
    responses(
        (status = 200, description = "List loans", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loans"
)]
#[get("/")]
pub async fn list_loans() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Loans retrieved successfully"))
}

#[utoipa::path(
    post,
    path = "/api/v1/loans",
    responses(
        (status = 201, description = "Loan application created successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loans"
)]
#[post("/")]
pub async fn submit_loan() -> impl Responder {
    HttpResponse::Created().json(ApiResponse::<()>::message("Loan application created successfully"))
}

#[utoipa::path(
    get,
    path = "/api/v1/loans/{id}",
    params(
        ("id" = String, Path, description = "Loan ID")
    ),
    responses(
        (status = 200, description = "Loan details retrieved successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loans"
)]
#[get("/{id}")]
pub async fn get_loan(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message("Loan details retrieved successfully"))
}

#[utoipa::path(
    post,
    path = "/api/v1/loans/{id}/approve",
    params(
        ("id" = String, Path, description = "Loan ID")
    ),
    responses(
        (status = 200, description = "Loan approved successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loans"
)]
#[post("/{id}/approve")]
pub async fn approve_loan(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message("Loan approved successfully"))
}

#[utoipa::path(
    post,
    path = "/api/v1/loans/{id}/disburse",
    params(
        ("id" = String, Path, description = "Loan ID")
    ),
    responses(
        (status = 200, description = "Loan disbursed successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loans"
)]
#[post("/{id}/disburse")]
pub async fn disburse_loan(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message("Loan disbursed successfully"))
}

#[utoipa::path(
    get,
    path = "/api/v1/loans/scoring/{id}",
    params(
        ("id" = String, Path, description = "Loan ID")
    ),
    responses(
        (status = 200, description = "Credit score analysis retrieved successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loans"
)]
#[get("/scoring/{id}")]
pub async fn get_loan_scoring(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message("Credit score analysis retrieved successfully"))
}
