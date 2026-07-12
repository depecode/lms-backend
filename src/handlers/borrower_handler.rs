use actix_web::{get, post, patch, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Borrower {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub date_of_birth: NaiveDate,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BorrowerGroupMinimal {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GuarantorMinimal {
    pub id: Uuid,
    pub name: String,
    pub phone: String,
    pub email: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateBorrowerRequest {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub date_of_birth: NaiveDate,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateBorrowerRequest {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub date_of_birth: Option<NaiveDate>,
    pub address: Option<String>,
    pub city: Option<String>,
    pub country: Option<String>,
    pub status: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/borrowers",
    responses(
        (status = 200, description = "List borrowers", body = ApiResponse<Vec<Borrower>>)
    ),
    tag = "Borrowers"
)]
#[get("")]
pub async fn list_borrowers(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let borrowers = sqlx::query_as::<_, Borrower>(
        r#"
        SELECT 
            id, first_name, last_name, email, phone, date_of_birth, 
            address, city, country, status, created_at, updated_at
        FROM borrowers
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(borrowers, "Borrowers retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/borrowers",
    responses(
        (status = 201, description = "Borrower created successfully", body = ApiResponse<Borrower>)
    ),
    tag = "Borrowers"
)]
#[post("")]
pub async fn create_borrower(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateBorrowerRequest>,
) -> Result<impl Responder, AppError> {
    // Check if email already registered
    let email_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM borrowers WHERE email = $1)"
    )
    .bind(&payload.email)
    .fetch_one(pool.get_ref())
    .await?;

    if email_exists {
        return Err(AppError::BadRequest("Email already registered".to_string()));
    }

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO borrowers (
            first_name, last_name, email, phone, date_of_birth, address, city, country, status
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, 'Active'
        ) RETURNING id
        "#
    )
    .bind(&payload.first_name)
    .bind(&payload.last_name)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(payload.date_of_birth)
    .bind(&payload.address)
    .bind(&payload.city)
    .bind(&payload.country)
    .fetch_one(pool.get_ref())
    .await?;

    let borrower = sqlx::query_as::<_, Borrower>(
        r#"
        SELECT 
            id, first_name, last_name, email, phone, date_of_birth, 
            address, city, country, status, created_at, updated_at
        FROM borrowers
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(borrower, "Borrower created successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/borrowers/{id}",
    params(
        ("id" = String, Path, description = "Borrower ID")
    ),
    responses(
        (status = 200, description = "Borrower details retrieved successfully", body = ApiResponse<Borrower>)
    ),
    tag = "Borrowers"
)]
#[get("/{id}")]
pub async fn get_borrower(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid borrower ID format".to_string()))?;

    let borrower = sqlx::query_as::<_, Borrower>(
        r#"
        SELECT 
            id, first_name, last_name, email, phone, date_of_birth, 
            address, city, country, status, created_at, updated_at
        FROM borrowers
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Borrower not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(borrower, "Borrower details retrieved successfully")))
}

#[utoipa::path(
    patch,
    path = "/api/v1/borrowers/{id}",
    params(
        ("id" = String, Path, description = "Borrower ID")
    ),
    responses(
        (status = 200, description = "Borrower updated successfully", body = ApiResponse<Borrower>)
    ),
    tag = "Borrowers"
)]
#[patch("/{id}")]
pub async fn update_borrower(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateBorrowerRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid borrower ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, Borrower>(
        r#"
        SELECT 
            id, first_name, last_name, email, phone, date_of_birth, 
            address, city, country, status, created_at, updated_at
        FROM borrowers
        WHERE id = $1 FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Borrower not found".to_string()))?;

    let first_name = payload.first_name.as_ref().unwrap_or(&existing.first_name);
    let last_name = payload.last_name.as_ref().unwrap_or(&existing.last_name);
    let email = payload.email.as_ref().unwrap_or(&existing.email);
    let phone = payload.phone.as_ref().unwrap_or(&existing.phone);
    let date_of_birth = payload.date_of_birth.unwrap_or(existing.date_of_birth);
    let address = payload.address.as_ref().or(existing.address.as_ref());
    let city = payload.city.as_ref().or(existing.city.as_ref());
    let country = payload.country.as_ref().or(existing.country.as_ref());
    let status = payload.status.as_ref().unwrap_or(&existing.status);

    let updated = sqlx::query_as::<_, Borrower>(
        r#"
        UPDATE borrowers SET
            first_name = $1, last_name = $2, email = $3, phone = $4,
            date_of_birth = $5, address = $6, city = $7, country = $8, status = $9, updated_at = NOW()
        WHERE id = $10
        RETURNING id, first_name, last_name, email, phone, date_of_birth, 
            address, city, country, status, created_at, updated_at
        "#
    )
    .bind(first_name)
    .bind(last_name)
    .bind(email)
    .bind(phone)
    .bind(date_of_birth)
    .bind(address)
    .bind(city)
    .bind(country)
    .bind(status)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Borrower updated successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/borrowers/groups",
    responses(
        (status = 200, description = "Borrower groups retrieved successfully", body = ApiResponse<Vec<BorrowerGroupMinimal>>)
    ),
    tag = "Borrowers"
)]
#[get("/groups")]
pub async fn list_groups(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let groups = sqlx::query_as::<_, BorrowerGroupMinimal>(
        r#"
        SELECT id, name, description, status, created_at
        FROM borrower_groups
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(groups, "Borrower groups retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/borrowers/guarantors",
    responses(
        (status = 200, description = "Guarantors retrieved successfully", body = ApiResponse<Vec<GuarantorMinimal>>)
    ),
    tag = "Borrowers"
)]
#[get("/guarantors")]
pub async fn list_guarantors(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let guarantors = sqlx::query_as::<_, GuarantorMinimal>(
        r#"
        SELECT id, name, phone, email, status, created_at
        FROM guarantors
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(guarantors, "Guarantors retrieved successfully")))
}
