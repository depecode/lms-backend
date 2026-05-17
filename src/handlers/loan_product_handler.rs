use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/loan-products",
    responses(
        (status = 200, description = "List loan products", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loan Products"
)]
#[get("/")]
pub async fn list_products() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("List loan products"))
}

#[utoipa::path(
    post,
    path = "/api/v1/loan-products",
    responses(
        (status = 201, description = "Loan product created", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loan Products"
)]
#[post("/")]
pub async fn create_product() -> impl Responder {
    HttpResponse::Created().json(ApiResponse::<()>::message("Loan product created"))
}

#[utoipa::path(
    get,
    path = "/api/v1/loan-products/{id}",
    params(
        ("id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Get product details", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loan Products"
)]
#[get("/{id}")]
pub async fn get_product(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message(&format!("Get product details {}", id)))
}
