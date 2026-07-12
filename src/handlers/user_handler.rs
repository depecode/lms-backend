use actix_web::{get, post, patch, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateUserRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub role: String,
    pub password: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateUserRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/users",
    responses(
        (status = 201, description = "User created", body = ApiResponse<User>)
    ),
    tag = "Users"
)]
#[post("")]
pub async fn create_user(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateUserRequest>,
) -> Result<impl Responder, AppError> {
    // Check if email already registered
    let email_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM users WHERE email = $1)"
    )
    .bind(&payload.email)
    .fetch_one(pool.get_ref())
    .await?;

    if email_exists {
        return Err(AppError::BadRequest("User email already exists".to_string()));
    }

    let password = payload.password.clone().unwrap_or_else(|| "TempPass123!".to_string());
    // Simple hash for seed/mock logic (in real apps we'd bcrypt or argon2)
    let password_hash = format!("hashed_{}", password);

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO users (first_name, last_name, email, password_hash, role, status)
        VALUES ($1, $2, $3, $4, $5, 'Active')
        RETURNING id
        "#
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.email)
    .bind(password_hash)
    .bind(&payload.role)
    .fetch_one(pool.get_ref())
    .await?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, first_name, last_name, email, role, status, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(user, "User created successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/users",
    responses(
        (status = 200, description = "List users", body = ApiResponse<Vec<User>>)
    ),
    tag = "Users"
)]
#[get("")]
pub async fn get_users(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let users = sqlx::query_as::<_, User>(
        "SELECT id, first_name, last_name, email, role, status, created_at, updated_at FROM users ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(users, "List users retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Get user", body = ApiResponse<User>)
    ),
    tag = "Users"
)]
#[get("/{id}")]
pub async fn get_user(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?;

    let user = sqlx::query_as::<_, User>(
        "SELECT id, first_name, last_name, email, role, status, created_at, updated_at FROM users WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(user, "User details retrieved successfully")))
}

#[utoipa::path(
    patch,
    path = "/api/v1/users/{id}",
    params(
        ("id" = String, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "Update user", body = ApiResponse<User>)
    ),
    tag = "Users"
)]
#[patch("/{id}")]
pub async fn update_user(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateUserRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, User>(
        "SELECT id, first_name, last_name, email, role, status, created_at, updated_at FROM users WHERE id = $1 FOR UPDATE"
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("User not found".to_string()))?;

    let first_name = payload.first_name.as_ref().unwrap_or(&existing.first_name);
    let last_name = payload.last_name.as_ref().unwrap_or(&existing.last_name);
    let email = payload.email.as_ref().unwrap_or(&existing.email);
    let role = payload.role.as_ref().unwrap_or(&existing.role);
    let status = payload.status.as_ref().unwrap_or(&existing.status);

    let updated = sqlx::query_as::<_, User>(
        r#"
        UPDATE users SET
            first_name = $1, last_name = $2, email = $3, role = $4, status = $5, updated_at = NOW()
        WHERE id = $6
        RETURNING id, first_name, last_name, email, role, status, created_at, updated_at
        "#
    )
    .bind(first_name)
    .bind(last_name)
    .bind(email)
    .bind(role)
    .bind(status)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "User updated successfully")))
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
pub async fn delete_user(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid user ID format".to_string()))?;

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("User not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("User deleted successfully")))
}
