use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/savings",
    responses(
        (status = 200, description = "List savings accounts", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Savings"
)]
#[get("/")]
pub async fn list_savings_accounts() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("List savings accounts"))
}

#[utoipa::path(
    post,
    path = "/api/v1/savings/bulk-upload",
    responses(
        (status = 200, description = "Bulk deposits processed", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Savings"
)]
#[post("/bulk-upload")]
pub async fn bulk_upload_deposits() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Bulk deposits processed"))
}

#[utoipa::path(
    get,
    path = "/api/v1/savings/{id}/history",
    params(
        ("id" = String, Path, description = "Account ID")
    ),
    responses(
        (status = 200, description = "Transaction history retrieved", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Savings"
)]
#[get("/{id}/history")]
pub async fn get_account_history(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message(&format!("Transaction history for account {}", id)))
}
