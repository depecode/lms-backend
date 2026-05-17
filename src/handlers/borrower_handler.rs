use actix_web::{get, post, patch, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    get,
    path = "/api/v1/borrowers",
    responses(
        (status = 200, description = "List borrowers", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Borrowers"
)]
#[get("/")]
pub async fn list_borrowers() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Borrowers retrieved successfully"))
}

#[utoipa::path(
    post,
    path = "/api/v1/borrowers",
    responses(
        (status = 201, description = "Borrower created successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Borrowers"
)]
#[post("/")]
pub async fn create_borrower() -> impl Responder {
    HttpResponse::Created().json(ApiResponse::<()>::message("Borrower created successfully"))
}

#[utoipa::path(
    get,
    path = "/api/v1/borrowers/{id}",
    params(
        ("id" = String, Path, description = "Borrower ID")
    ),
    responses(
        (status = 200, description = "Borrower details retrieved successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Borrowers"
)]
#[get("/{id}")]
pub async fn get_borrower(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message("Borrower details retrieved successfully"))
}

#[utoipa::path(
    patch,
    path = "/api/v1/borrowers/{id}",
    params(
        ("id" = String, Path, description = "Borrower ID")
    ),
    responses(
        (status = 200, description = "Borrower updated successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Borrowers"
)]
#[patch("/{id}")]
pub async fn update_borrower(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message("Borrower updated successfully"))
}

#[utoipa::path(
    get,
    path = "/api/v1/borrowers/groups",
    responses(
        (status = 200, description = "Borrower groups retrieved successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Borrowers"
)]
#[get("/groups")]
pub async fn list_groups() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Borrower groups retrieved successfully"))
}

#[utoipa::path(
    get,
    path = "/api/v1/borrowers/guarantors",
    responses(
        (status = 200, description = "Guarantors retrieved successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Borrowers"
)]
#[get("/guarantors")]
pub async fn list_guarantors() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Guarantors retrieved successfully"))
}
