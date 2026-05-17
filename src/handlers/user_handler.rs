use actix_web::{get, post, patch, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;

#[utoipa::path(
    post,
    path = "/api/v1/users",
    responses(
        (status = 201, description = "User created", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Users"
)]
#[post("/")]
pub async fn create_user() -> impl Responder {
    HttpResponse::Created().json(ApiResponse::<()>::message("User created"))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    responses(
        (status = 200, description = "List users", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Users"
)]
#[get("/")]
pub async fn get_users() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("List users"))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Get user", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Users"
)]
#[get("/{id}")]
pub async fn get_user(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message(&format!("Get user {}", id)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Update user", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Users"
)]
#[patch("/{id}")]
pub async fn update_user(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message(&format!("Update user {}", id)))
}

#[utoipa::path(
    delete,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Delete user", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Users"
)]
#[delete("/{id}")]
pub async fn delete_user(path: web::Path<String>) -> impl Responder {
    let id = path.into_inner();
    HttpResponse::Ok().json(ApiResponse::<()>::message(&format!("Delete user {}", id)))
}
