use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Collateral {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub borrower_id: Uuid,
    pub r#type: String,
    pub description: String,
    pub location: Option<String>,
    pub appraised_value: f64,
    pub registration_number: Option<String>,
    pub registration_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
    pub status: String,
    pub insured: bool,
    pub insurance_policy: Option<String>,
    pub insurance_value: Option<f64>,
    pub lien: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateCollateralRequest {
    pub loan_id: Uuid,
    pub borrower_id: Uuid,
    pub r#type: String,
    pub description: String,
    pub location: Option<String>,
    pub appraised_value: f64,
    pub registration_number: Option<String>,
    pub registration_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
    pub insured: Option<bool>,
    pub insurance_policy: Option<String>,
    pub insurance_value: Option<f64>,
    pub lien: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCollateralRequest {
    pub r#type: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub appraised_value: Option<f64>,
    pub registration_number: Option<String>,
    pub registration_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub status: Option<String>,
    pub insured: Option<bool>,
    pub insurance_policy: Option<String>,
    pub insurance_value: Option<f64>,
    pub lien: Option<bool>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AppraiseRequest {
    pub appraiser_name: String,
    pub appraisal_date: NaiveDate,
    pub value: f64,
    pub condition: String,
    pub notes: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/collateral",
    responses(
        (status = 200, description = "List collaterals", body = ApiResponse<Vec<Collateral>>)
    ),
    tag = "Collateral"
)]
#[get("")]
pub async fn get_collaterals(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let collaterals = sqlx::query_as::<_, Collateral>(
        r#"
        SELECT 
            id, loan_id, borrower_id, type, description, location, 
            appraised_value::float8 as appraised_value, registration_number, 
            registration_date, expiry_date, status, insured, insurance_policy, 
            insurance_value::float8 as insurance_value, lien, created_at, updated_at
        FROM collateral
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(collaterals, "Collateral list retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/collateral/{id}",
    responses(
        (status = 200, description = "Get collateral", body = ApiResponse<Collateral>)
    ),
    tag = "Collateral"
)]
#[get("/{id}")]
pub async fn get_collateral(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid collateral ID format".to_string()))?;

    let collateral = sqlx::query_as::<_, Collateral>(
        r#"
        SELECT 
            id, loan_id, borrower_id, type, description, location, 
            appraised_value::float8 as appraised_value, registration_number, 
            registration_date, expiry_date, status, insured, insurance_policy, 
            insurance_value::float8 as insurance_value, lien, created_at, updated_at
        FROM collateral
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Collateral not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(collateral, "Collateral details retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/collateral",
    responses(
        (status = 201, description = "Create collateral", body = ApiResponse<Collateral>)
    ),
    tag = "Collateral"
)]
#[post("")]
pub async fn create_collateral(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateCollateralRequest>,
) -> Result<impl Responder, AppError> {
    // Check if loan exists
    let loan_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM loans WHERE id = $1)"
    )
    .bind(payload.loan_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !loan_exists {
        return Err(AppError::NotFound("Loan not found".to_string()));
    }

    // Check if borrower exists
    let borrower_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM borrowers WHERE id = $1)"
    )
    .bind(payload.borrower_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !borrower_exists {
        return Err(AppError::NotFound("Borrower not found".to_string()));
    }

    let insured = payload.insured.unwrap_or(false);
    let lien = payload.lien.unwrap_or(false);

    let collateral_type = match payload.r#type.as_str() {
        "Land" | "Building" | "Vehicle" | "Equipment" | "Jewelry" | "Securities" | "Other" => payload.r#type.clone(),
        _ => "Other".to_string(),
    };

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO collateral (
            loan_id, borrower_id, type, description, location, appraised_value, 
            registration_number, registration_date, expiry_date, insured, 
            insurance_policy, insurance_value, lien, status
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, 'Registered'
        ) RETURNING id
        "#
    )
    .bind(payload.loan_id)
    .bind(payload.borrower_id)
    .bind(&collateral_type)
    .bind(&payload.description)
    .bind(&payload.location)
    .bind(payload.appraised_value)
    .bind(&payload.registration_number)
    .bind(payload.registration_date)
    .bind(payload.expiry_date)
    .bind(insured)
    .bind(&payload.insurance_policy)
    .bind(payload.insurance_value)
    .bind(lien)
    .fetch_one(pool.get_ref())
    .await?;

    let collateral = sqlx::query_as::<_, Collateral>(
        r#"
        SELECT 
            id, loan_id, borrower_id, type, description, location, 
            appraised_value::float8 as appraised_value, registration_number, 
            registration_date, expiry_date, status, insured, insurance_policy, 
            insurance_value::float8 as insurance_value, lien, created_at, updated_at
        FROM collateral
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(collateral, "Collateral registered successfully")))
}

#[utoipa::path(
    put,
    path = "/api/v1/collateral/{id}",
    responses(
        (status = 200, description = "Update collateral", body = ApiResponse<Collateral>)
    ),
    tag = "Collateral"
)]
#[put("/{id}")]
pub async fn update_collateral(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateCollateralRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid collateral ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, Collateral>(
        r#"
        SELECT 
            id, loan_id, borrower_id, type, description, location, 
            appraised_value::float8 as appraised_value, registration_number, 
            registration_date, expiry_date, status, insured, insurance_policy, 
            insurance_value::float8 as insurance_value, lien, created_at, updated_at
        FROM collateral
        WHERE id = $1 FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Collateral not found".to_string()))?;

    let r#type = payload.r#type.as_ref().unwrap_or(&existing.r#type);
    let coerced_type = match r#type.as_str() {
        "Land" | "Building" | "Vehicle" | "Equipment" | "Jewelry" | "Securities" | "Other" => r#type.clone(),
        _ => "Other".to_string(),
    };
    let description = payload.description.as_ref().unwrap_or(&existing.description);
    let location = payload.location.as_ref().or(existing.location.as_ref());
    let appraised_value = payload.appraised_value.unwrap_or(existing.appraised_value);
    let registration_number = payload.registration_number.as_ref().or(existing.registration_number.as_ref());
    let registration_date = payload.registration_date.unwrap_or(existing.registration_date);
    let expiry_date = payload.expiry_date.or(existing.expiry_date);
    let status = payload.status.as_ref().unwrap_or(&existing.status);
    let insured = payload.insured.unwrap_or(existing.insured);
    let insurance_policy = payload.insurance_policy.as_ref().or(existing.insurance_policy.as_ref());
    let insurance_value = payload.insurance_value.or(existing.insurance_value);
    let lien = payload.lien.unwrap_or(existing.lien);

    let updated = sqlx::query_as::<_, Collateral>(
        r#"
        UPDATE collateral SET
            type = $1, description = $2, location = $3, appraised_value = $4,
            registration_number = $5, registration_date = $6, expiry_date = $7,
            status = $8, insured = $9, insurance_policy = $10, insurance_value = $11,
            lien = $12, updated_at = NOW()
        WHERE id = $13
        RETURNING id, loan_id, borrower_id, type, description, location, 
            appraised_value::float8 as appraised_value, registration_number, 
            registration_date, expiry_date, status, insured, insurance_policy, 
            insurance_value::float8 as insurance_value, lien, created_at, updated_at
        "#
    )
    .bind(&coerced_type)
    .bind(description)
    .bind(location)
    .bind(appraised_value)
    .bind(registration_number)
    .bind(registration_date)
    .bind(expiry_date)
    .bind(status)
    .bind(insured)
    .bind(insurance_policy)
    .bind(insurance_value)
    .bind(lien)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Collateral updated successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/collateral/{id}/appraise",
    responses(
        (status = 200, description = "Appraise collateral", body = ApiResponse<Collateral>)
    ),
    tag = "Collateral"
)]
#[post("/{id}/appraise")]
pub async fn appraise(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<AppraiseRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid collateral ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    // Check existing
    let _existing = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM collateral WHERE id = $1 FOR UPDATE)"
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    // Insert appraisal record
    sqlx::query(
        r#"
        INSERT INTO collateral_appraisals (collateral_id, appraiser_name, appraisal_date, value, condition, notes)
        VALUES ($1, $2, $3, $4, $5, $6)
        "#
    )
    .bind(id)
    .bind(&payload.appraiser_name)
    .bind(payload.appraisal_date)
    .bind(payload.value)
    .bind(&payload.condition)
    .bind(&payload.notes)
    .execute(&mut *tx)
    .await?;

    // Update collateral value & status
    let updated = sqlx::query_as::<_, Collateral>(
        r#"
        UPDATE collateral 
        SET appraised_value = $1, status = 'Appraised', updated_at = NOW() 
        WHERE id = $2
        RETURNING id, loan_id, borrower_id, type, description, location, 
            appraised_value::float8 as appraised_value, registration_number, 
            registration_date, expiry_date, status, insured, insurance_policy, 
            insurance_value::float8 as insurance_value, lien, created_at, updated_at
        "#
    )
    .bind(payload.value)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Collateral appraisal registered successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/collateral/{id}/release",
    responses(
        (status = 200, description = "Release collateral", body = ApiResponse<Collateral>)
    ),
    tag = "Collateral"
)]
#[post("/{id}/release")]
pub async fn release(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid collateral ID format".to_string()))?;

    let updated = sqlx::query_as::<_, Collateral>(
        r#"
        UPDATE collateral 
        SET status = 'Released', updated_at = NOW() 
        WHERE id = $1
        RETURNING id, loan_id, borrower_id, type, description, location, 
            appraised_value::float8 as appraised_value, registration_number, 
            registration_date, expiry_date, status, insured, insurance_policy, 
            insurance_value::float8 as insurance_value, lien, created_at, updated_at
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Collateral not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Collateral released successfully")))
}

#[utoipa::path(
    delete,
    path = "/api/v1/collateral/{id}",
    responses(
        (status = 200, description = "Delete collateral", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Collateral"
)]
#[delete("/{id}")]
pub async fn delete_collateral(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid collateral ID format".to_string()))?;

    let result = sqlx::query("DELETE FROM collateral WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Collateral not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Collateral deleted successfully")))
}
