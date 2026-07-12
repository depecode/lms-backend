use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Guarantor {
    pub id: Uuid,
    pub borrower_id: Uuid,
    pub loan_id: Option<Uuid>,
    pub name: String,
    pub relationship: String,
    pub email: String,
    pub phone: String,
    pub address: Option<String>,
    pub id_number: Option<String>,
    pub guarantee_amount: f64,
    pub liability_type: Option<String>,
    pub status: String,
    pub employment_status: Option<String>,
    pub income: Option<f64>,
    pub net_worth: Option<f64>,
    pub signature_date: Option<DateTime<Utc>>,
    pub signature_evidence: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateGuarantorRequest {
    pub borrower_id: Uuid,
    pub loan_id: Option<Uuid>,
    pub name: String,
    pub relationship: String,
    pub email: String,
    pub phone: String,
    pub address: Option<String>,
    pub id_number: Option<String>,
    pub guarantee_amount: f64,
    pub liability_type: Option<String>,
    pub employment_status: Option<String>,
    pub income: Option<f64>,
    pub net_worth: Option<f64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGuarantorRequest {
    pub name: Option<String>,
    pub relationship: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub id_number: Option<String>,
    pub guarantee_amount: Option<f64>,
    pub liability_type: Option<String>,
    pub employment_status: Option<String>,
    pub income: Option<f64>,
    pub net_worth: Option<f64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UploadSignatureRequest {
    pub signature_evidence: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/guarantors",
    responses(
        (status = 200, description = "List guarantors", body = ApiResponse<Vec<Guarantor>>)
    ),
    tag = "Guarantors"
)]
#[get("")]
pub async fn get_guarantors(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let guarantors = sqlx::query_as::<_, Guarantor>(
        r#"
        SELECT 
            id, borrower_id, loan_id, name, relationship, email, phone, 
            address, id_number, guarantee_amount::float8 as guarantee_amount, 
            liability_type, status, employment_status, income::float8 as income, 
            net_worth::float8 as net_worth, signature_date, signature_evidence
        FROM guarantors
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(guarantors, "Guarantors list retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/guarantors/{id}",
    responses(
        (status = 200, description = "Get guarantor by ID", body = ApiResponse<Guarantor>)
    ),
    tag = "Guarantors"
)]
#[get("/{id}")]
pub async fn get_guarantor(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid guarantor ID format".to_string()))?;

    let guarantor = sqlx::query_as::<_, Guarantor>(
        r#"
        SELECT 
            id, borrower_id, loan_id, name, relationship, email, phone, 
            address, id_number, guarantee_amount::float8 as guarantee_amount, 
            liability_type, status, employment_status, income::float8 as income, 
            net_worth::float8 as net_worth, signature_date, signature_evidence
        FROM guarantors 
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Guarantor not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(guarantor, "Guarantor retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/guarantors/loan/{loan_id}",
    responses(
        (status = 200, description = "Get guarantors for loan", body = ApiResponse<Vec<Guarantor>>)
    ),
    tag = "Guarantors"
)]
#[get("/loan/{loan_id}")]
pub async fn get_guarantors_for_loan(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let loan_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid loan ID format".to_string()))?;

    let guarantors = sqlx::query_as::<_, Guarantor>(
        r#"
        SELECT 
            id, borrower_id, loan_id, name, relationship, email, phone, 
            address, id_number, guarantee_amount::float8 as guarantee_amount, 
            liability_type, status, employment_status, income::float8 as income, 
            net_worth::float8 as net_worth, signature_date, signature_evidence
        FROM guarantors 
        WHERE loan_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(loan_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(guarantors, "Guarantors for loan retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/guarantors",
    responses(
        (status = 201, description = "Create guarantor", body = ApiResponse<Guarantor>)
    ),
    tag = "Guarantors"
)]
#[post("")]
pub async fn create_guarantor(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateGuarantorRequest>,
) -> Result<impl Responder, AppError> {
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

    // Insert guarantor
    let guarantor = sqlx::query_as::<_, Guarantor>(
        r#"
        INSERT INTO guarantors (
            borrower_id, loan_id, name, relationship, email, phone, address, 
            id_number, guarantee_amount, liability_type, status, 
            employment_status, income, net_worth
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, 
            $8, $9, $10, 'Invited', 
            $11, $12, $13
        ) RETURNING 
            id, borrower_id, loan_id, name, relationship, email, phone, 
            address, id_number, guarantee_amount::float8 as guarantee_amount, 
            liability_type, status, employment_status, income::float8 as income, 
            net_worth::float8 as net_worth, signature_date, signature_evidence
        "#
    )
    .bind(payload.borrower_id)
    .bind(payload.loan_id)
    .bind(&payload.name)
    .bind(&payload.relationship)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.address)
    .bind(&payload.id_number)
    .bind(payload.guarantee_amount)
    .bind(&payload.liability_type)
    .bind(&payload.employment_status)
    .bind(payload.income)
    .bind(payload.net_worth)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(guarantor, "Guarantor registered successfully")))
}

#[utoipa::path(
    put,
    path = "/api/v1/guarantors/{id}",
    responses(
        (status = 200, description = "Update guarantor", body = ApiResponse<Guarantor>)
    ),
    tag = "Guarantors"
)]
#[put("/{id}")]
pub async fn update_guarantor(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateGuarantorRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid guarantor ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, Guarantor>(
        r#"
        SELECT 
            id, borrower_id, loan_id, name, relationship, email, phone, 
            address, id_number, guarantee_amount::float8 as guarantee_amount, 
            liability_type, status, employment_status, income::float8 as income, 
            net_worth::float8 as net_worth, signature_date, signature_evidence
        FROM guarantors WHERE id = $1 FOR UPDATE
        "#
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Guarantor not found".to_string()))?;

    let name = payload.name.as_ref().unwrap_or(&existing.name);
    let relationship = payload.relationship.as_ref().unwrap_or(&existing.relationship);
    let email = payload.email.as_ref().unwrap_or(&existing.email);
    let phone = payload.phone.as_ref().unwrap_or(&existing.phone);
    let address = payload.address.as_ref().or(existing.address.as_ref());
    let id_number = payload.id_number.as_ref().or(existing.id_number.as_ref());
    let guarantee_amount = payload.guarantee_amount.unwrap_or(existing.guarantee_amount);
    let liability_type = payload.liability_type.as_ref().or(existing.liability_type.as_ref());
    let employment_status = payload.employment_status.as_ref().or(existing.employment_status.as_ref());
    let income = payload.income.or(existing.income);
    let net_worth = payload.net_worth.or(existing.net_worth);

    let updated = sqlx::query_as::<_, Guarantor>(
        r#"
        UPDATE guarantors SET
            name = $1, relationship = $2, email = $3, phone = $4, address = $5,
            id_number = $6, guarantee_amount = $7, liability_type = $8,
            employment_status = $9, income = $10, net_worth = $11, updated_at = NOW()
        WHERE id = $12
        RETURNING 
            id, borrower_id, loan_id, name, relationship, email, phone, 
            address, id_number, guarantee_amount::float8 as guarantee_amount, 
            liability_type, status, employment_status, income::float8 as income, 
            net_worth::float8 as net_worth, signature_date, signature_evidence
        "#
    )
    .bind(name)
    .bind(relationship)
    .bind(email)
    .bind(phone)
    .bind(address)
    .bind(id_number)
    .bind(guarantee_amount)
    .bind(liability_type)
    .bind(employment_status)
    .bind(income)
    .bind(net_worth)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Guarantor updated successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/guarantors/{id}/status",
    responses(
        (status = 200, description = "Update guarantor status", body = ApiResponse<Guarantor>)
    ),
    tag = "Guarantors"
)]
#[post("/{id}/status")]
pub async fn update_guarantor_status(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateStatusRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid guarantor ID format".to_string()))?;

    let updated = sqlx::query_as::<_, Guarantor>(
        r#"
        UPDATE guarantors 
        SET status = $1, updated_at = NOW() 
        WHERE id = $2
        RETURNING 
            id, borrower_id, loan_id, name, relationship, email, phone, 
            address, id_number, guarantee_amount::float8 as guarantee_amount, 
            liability_type, status, employment_status, income::float8 as income, 
            net_worth::float8 as net_worth, signature_date, signature_evidence
        "#,
    )
    .bind(&payload.status)
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Guarantor not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Guarantor status updated successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/guarantors/{id}/signature",
    responses(
        (status = 200, description = "Upload guarantor signature", body = ApiResponse<Guarantor>)
    ),
    tag = "Guarantors"
)]
#[post("/{id}/signature")]
pub async fn upload_signature(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UploadSignatureRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid guarantor ID format".to_string()))?;

    let updated = sqlx::query_as::<_, Guarantor>(
        r#"
        UPDATE guarantors 
        SET signature_evidence = $1, signature_date = NOW(), status = 'Signed', updated_at = NOW() 
        WHERE id = $2
        RETURNING 
            id, borrower_id, loan_id, name, relationship, email, phone, 
            address, id_number, guarantee_amount::float8 as guarantee_amount, 
            liability_type, status, employment_status, income::float8 as income, 
            net_worth::float8 as net_worth, signature_date, signature_evidence
        "#,
    )
    .bind(&payload.signature_evidence)
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Guarantor not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Guarantor signature uploaded successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/guarantors/{id}/liabilities",
    responses(
        (status = 200, description = "Get guarantor liabilities", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Guarantors"
)]
#[get("/{id}/liabilities")]
pub async fn get_liabilities(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid guarantor ID format".to_string()))?;

    // Check if guarantor exists to fetch email
    let email = sqlx::query_scalar::<_, String>(
        "SELECT email FROM guarantors WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Guarantor not found".to_string()))?;

    // Get aggregated liabilities across loans for this guarantor email
    let total_liability: f64 = sqlx::query_scalar::<_, f64>(
        r#"
        SELECT COALESCE(SUM(guarantee_amount)::float8, 0.0) as total_liability
        FROM guarantors 
        WHERE email = $1 AND status IN ('active', 'Signed', 'Accepted')
        "#
    )
    .bind(&email)
    .fetch_one(pool.get_ref())
    .await?;

    let loans = sqlx::query(
        r#"
        SELECT loan_id, guarantee_amount::float8 as amount
        FROM guarantors 
        WHERE email = $1 AND status IN ('active', 'Signed', 'Accepted') AND loan_id IS NOT NULL
        "#
    )
    .bind(&email)
    .fetch_all(pool.get_ref())
    .await?
    .into_iter()
    .map(|row| {
        serde_json::json!({
            "loanId": row.get::<Option<Uuid>, _>("loan_id"),
            "amount": row.get::<f64, _>("amount")
        })
    })
    .collect::<Vec<_>>();

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "totalLiability": total_liability,
            "loans": loans
        }),
        "Guarantor liabilities retrieved successfully"
    )))
}

#[utoipa::path(
    delete,
    path = "/api/v1/guarantors/{id}",
    responses(
        (status = 200, description = "Delete guarantor", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Guarantors"
)]
#[delete("/{id}")]
pub async fn delete_guarantor(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid guarantor ID format".to_string()))?;

    let result = sqlx::query("DELETE FROM guarantors WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Guarantor not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Guarantor deleted successfully")))
}
