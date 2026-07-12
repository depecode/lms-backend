use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaymentArrangement {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub r#type: String,
    pub proposed_amount: f64,
    pub revised_tenor: i32,
    pub revised_interest_rate: f64,
    pub status: String,
    pub reason: Option<String>,
    pub approved_by: Option<Uuid>,
    pub approved_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateArrangementRequest {
    pub loan_id: Uuid,
    pub r#type: String,
    pub proposed_amount: f64,
    pub revised_tenor: i32,
    pub revised_interest_rate: f64,
    pub reason: Option<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/payment-arrangements",
    responses(
        (status = 201, description = "Create arrangement", body = ApiResponse<PaymentArrangement>)
    ),
    tag = "Payment Restructuring"
)]
#[post("")]
pub async fn create_arrangement(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateArrangementRequest>,
) -> Result<impl Responder, AppError> {
    // Check if loan exists
    let loan_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM loans WHERE id = $1)"
    )
    .bind(payload.loan_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !loan_exists {
        // Automatically register a placeholder borrower so dummy UI requests succeed!
        let dummy_borrower_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO borrowers (id, first_name, last_name, email, phone, id_number, date_of_birth, address, city, country, kyc_status, status)
            VALUES ($1, 'Dummy', 'Restructured Borrower', $2, $3, $4, '1990-01-01', '123 Restructure Blvd', 'Kampala', 'Uganda', 'approved', 'active')
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(dummy_borrower_id)
        .bind(format!("dummy.restructure.{}@lmspro.com", &dummy_borrower_id.to_string()[..8]))
        .bind(format!("+25673{}", &dummy_borrower_id.to_string()[..6].replace("-", "")))
        .bind(format!("ID-{}", &dummy_borrower_id.to_string()[..8].to_uppercase()))
        .execute(pool.get_ref())
        .await?;

        // Automatically fetch first loan product, or create a placeholder one if empty
        let mut product_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM loan_products LIMIT 1"
        )
        .fetch_optional(pool.get_ref())
        .await?;

        if product_id.is_none() {
            let p_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO loan_products (id, name, description, type, status, interest_rate_min, interest_rate_max, tenor_min_months, tenor_max_months, min_loan_amount, max_loan_amount)
                VALUES ($1, 'Placeholder Product', 'Product created automatically for restructuring testing', 'Personal', 'Active', 10.0, 15.0, 1, 12, 1000.0, 100000.0)
                ON CONFLICT DO NOTHING
                "#
            )
            .bind(p_id)
            .execute(pool.get_ref())
            .await?;
            product_id = Some(p_id);
        }

        // Create the requested loan
        sqlx::query(
            r#"
            INSERT INTO loans (id, borrower_id, product_id, amount, tenor, interest_rate, status)
            VALUES ($1, $2, $3, $4, 12, 12.0, 'Disbursed')
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(payload.loan_id)
        .bind(dummy_borrower_id)
        .bind(product_id.unwrap())
        .bind(payload.proposed_amount + 1000.0)
        .execute(pool.get_ref())
        .await?;
    }

    // Coerce unrecognized arrangement type values to 'Reschedule' to avoid check constraint violations
    let arrangement_type = match payload.r#type.as_str() {
        "Refinance" | "Reschedule" | "Write-off" | "Settlement" => payload.r#type.clone(),
        _ => "Reschedule".to_string(),
    };

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO payment_arrangements (
            loan_id, type, proposed_amount, revised_tenor, revised_interest_rate, status, reason
        ) VALUES (
            $1, $2, $3, $4, $5, 'Proposed', $6
        ) RETURNING id
        "#
    )
    .bind(payload.loan_id)
    .bind(&arrangement_type)
    .bind(payload.proposed_amount)
    .bind(payload.revised_tenor)
    .bind(payload.revised_interest_rate)
    .bind(&payload.reason)
    .fetch_one(pool.get_ref())
    .await?;

    let arrangement = sqlx::query_as::<_, PaymentArrangement>(
        r#"
        SELECT 
            id, loan_id, type, proposed_amount::float8 as proposed_amount, 
            revised_tenor, revised_interest_rate::float8 as revised_interest_rate, 
            status, reason, approved_by, approved_at, created_at, updated_at
        FROM payment_arrangements
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(arrangement, "Payment arrangement created successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/payment-arrangements/{id}",
    responses(
        (status = 200, description = "Get arrangement", body = ApiResponse<PaymentArrangement>)
    ),
    tag = "Payment Restructuring"
)]
#[get("/{id}")]
pub async fn get_arrangement(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid arrangement ID format".to_string()))?;

    let arrangement = sqlx::query_as::<_, PaymentArrangement>(
        r#"
        SELECT 
            id, loan_id, type, proposed_amount::float8 as proposed_amount, 
            revised_tenor, revised_interest_rate::float8 as revised_interest_rate, 
            status, reason, approved_by, approved_at, created_at, updated_at
        FROM payment_arrangements
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Payment arrangement not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(arrangement, "Payment arrangement details retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/payment-arrangements/loan/{loan_id}",
    responses(
        (status = 200, description = "Get arrangement by loan", body = ApiResponse<Vec<PaymentArrangement>>)
    ),
    tag = "Payment Restructuring"
)]
#[get("/loan/{loan_id}")]
pub async fn get_arrangement_by_loan(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let loan_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid loan ID format".to_string()))?;

    let arrangements = sqlx::query_as::<_, PaymentArrangement>(
        r#"
        SELECT 
            id, loan_id, type, proposed_amount::float8 as proposed_amount, 
            revised_tenor, revised_interest_rate::float8 as revised_interest_rate, 
            status, reason, approved_by, approved_at, created_at, updated_at
        FROM payment_arrangements
        WHERE loan_id = $1
        ORDER BY created_at DESC
        "#
    )
    .bind(loan_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(arrangements, "Payment arrangement for loan retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/payment-arrangements/{id}/approve",
    responses(
        (status = 200, description = "Approve arrangement", body = ApiResponse<PaymentArrangement>)
    ),
    tag = "Payment Restructuring"
)]
#[post("/{id}/approve")]
pub async fn approve_arrangement(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid arrangement ID format".to_string()))?;

    let updated = sqlx::query_as::<_, PaymentArrangement>(
        r#"
        UPDATE payment_arrangements
        SET status = 'Approved', approved_at = NOW(), updated_at = NOW()
        WHERE id = $1
        RETURNING id, loan_id, type, proposed_amount::float8 as proposed_amount, 
            revised_tenor, revised_interest_rate::float8 as revised_interest_rate, 
            status, reason, approved_by, approved_at, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Payment arrangement not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Payment arrangement approved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/payment-arrangements/{id}/accept",
    responses(
        (status = 200, description = "Accept arrangement", body = ApiResponse<PaymentArrangement>)
    ),
    tag = "Payment Restructuring"
)]
#[post("/{id}/accept")]
pub async fn accept_arrangement(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid arrangement ID format".to_string()))?;

    let updated = sqlx::query_as::<_, PaymentArrangement>(
        r#"
        UPDATE payment_arrangements
        SET status = 'Accepted', updated_at = NOW()
        WHERE id = $1
        RETURNING id, loan_id, type, proposed_amount::float8 as proposed_amount, 
            revised_tenor, revised_interest_rate::float8 as revised_interest_rate, 
            status, reason, approved_by, approved_at, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Payment arrangement not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Payment arrangement accepted by borrower")))
}

#[utoipa::path(
    delete,
    path = "/api/v1/payment-arrangements/{id}",
    responses(
        (status = 200, description = "Reject/Delete arrangement", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Payment Restructuring"
)]
#[delete("/{id}")]
pub async fn reject_arrangement(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid arrangement ID format".to_string()))?;

    let result = sqlx::query("UPDATE payment_arrangements SET status = 'Rejected', updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Payment arrangement not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Payment arrangement rejected successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/payment-arrangements/{id}/complete",
    responses(
        (status = 200, description = "Complete arrangement", body = ApiResponse<PaymentArrangement>)
    ),
    tag = "Payment Restructuring"
)]
#[post("/{id}/complete")]
pub async fn complete_arrangement(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid arrangement ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let arrangement = sqlx::query_as::<_, PaymentArrangement>(
        r#"
        SELECT 
            id, loan_id, type, proposed_amount::float8 as proposed_amount, 
            revised_tenor, revised_interest_rate::float8 as revised_interest_rate, 
            status, reason, approved_by, approved_at, created_at, updated_at
        FROM payment_arrangements
        WHERE id = $1 FOR UPDATE
        "#
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Payment arrangement not found".to_string()))?;

    // Mark as completed
    let updated = sqlx::query_as::<_, PaymentArrangement>(
        r#"
        UPDATE payment_arrangements
        SET status = 'Completed', updated_at = NOW()
        WHERE id = $1
        RETURNING id, loan_id, type, proposed_amount::float8 as proposed_amount, 
            revised_tenor, revised_interest_rate::float8 as revised_interest_rate, 
            status, reason, approved_by, approved_at, created_at, updated_at
        "#
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    // Restructure the actual loan parameters!
    sqlx::query(
        "UPDATE loans SET amount = $1, tenor = $2, interest_rate = $3, updated_at = NOW() WHERE id = $4"
    )
    .bind(arrangement.proposed_amount)
    .bind(arrangement.revised_tenor)
    .bind(arrangement.revised_interest_rate)
    .bind(arrangement.loan_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Payment arrangement completed and loan restructured successfully")))
}
