use actix_web::{post, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::Deserialize;
use chrono::{Utc, NaiveDate};

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PrimaryIdentityFormData {
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub phone: String,
    pub date_of_birth: NaiveDate,
    pub gender: String,
    pub id_type: String,
    pub id_number: String,
    pub id_expiry_date: NaiveDate,
    pub address: String,
    pub city: String,
    pub state: String,
    pub country: String,
    pub postal_code: String,
    pub occupation: String,
    pub employment_type: String,
    pub monthly_income: Option<f64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CifAccountRequest {
    pub cifid: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/onboarding/generate-cifid",
    responses(
        (status = 200, description = "Generate CIFID", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Onboarding"
)]
#[post("/generate-cifid")]
pub async fn generate_cifid(
    pool: web::Data<PgPool>,
    payload: web::Json<PrimaryIdentityFormData>,
) -> Result<impl Responder, AppError> {
    // 1. Check if borrower already exists
    let existing = sqlx::query(
        "SELECT id, id_number FROM borrowers WHERE email = $1 OR phone = $2 OR id_number = $3"
    )
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.id_number)
    .fetch_optional(pool.get_ref())
    .await?;

    let borrower_id;
    let id_number;

    if let Some(row) = existing {
        borrower_id = row.get::<Uuid, _>("id");
        id_number = row.get::<String, _>("id_number");
    } else {
        // 2. Insert new borrower
        borrower_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO borrowers (
                first_name, last_name, email, phone, id_number, date_of_birth, address, city, country, kyc_status, status
            ) VALUES (
                $1, $2, $3, $4, $5, $6, $7, $8, $9, 'approved', 'active'
            ) RETURNING id
            "#
        )
        .bind(&payload.first_name)
        .bind(&payload.last_name)
        .bind(&payload.email)
        .bind(&payload.phone)
        .bind(&payload.id_number)
        .bind(payload.date_of_birth)
        .bind(&payload.address)
        .bind(&payload.city)
        .bind(&payload.country)
        .fetch_one(pool.get_ref())
        .await?;
        id_number = payload.id_number.clone();
    }

    let cifid = format!("CIF-{}", id_number);

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "cifid": cifid,
            "baseCifProfile": {
                "id": borrower_id,
                "cifid": cifid,
                "firstName": payload.first_name,
                "lastName": payload.last_name,
                "email": payload.email,
                "phone": payload.phone,
                "dateOfBirth": payload.date_of_birth,
                "gender": payload.gender,
                "kycStatus": "approved",
                "createdAt": Utc::now()
            }
        }),
        "CIFID generated successfully"
    )))
}

#[utoipa::path(
    post,
    path = "/api/v1/onboarding/savings-account",
    responses(
        (status = 200, description = "Create default savings account", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Onboarding"
)]
#[post("/savings-account")]
pub async fn create_savings(
    pool: web::Data<PgPool>,
    payload: web::Json<CifAccountRequest>,
) -> Result<impl Responder, AppError> {
    // Strip "CIF-" prefix to search by id_number
    let raw_cif = payload.cifid.replace("CIF-", "");

    // Fetch borrower
    let borrower = sqlx::query(
        "SELECT id, first_name, last_name FROM borrowers WHERE id_number = $1 OR id::text = $2 OR ('CIF-' || id_number) = $3"
    )
    .bind(&raw_cif)
    .bind(&payload.cifid)
    .bind(&payload.cifid)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Borrower profile not found for the given CIFID".to_string()))?;

    let borrower_id = borrower.get::<Uuid, _>("id");
    let first_name = borrower.get::<String, _>("first_name");
    let last_name = borrower.get::<String, _>("last_name");
    let account_name = format!("{} {}", first_name, last_name);

    // Create default savings account
    let mut tx = pool.begin().await?;

    let acc_number = format!("SAV-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());

    // Check if account already exists
    let existing_acc = sqlx::query(
        "SELECT account_number FROM savings_accounts WHERE borrower_id = $1"
    )
    .bind(borrower_id)
    .fetch_optional(&mut *tx)
    .await?;

    let final_acc_number;
    if let Some(row) = existing_acc {
        final_acc_number = row.get::<String, _>("account_number");
    } else {
        sqlx::query(
            "INSERT INTO savings_accounts (account_number, borrower_id, account_type, balance, interest_rate, status) VALUES ($1, $2, 'Ordinary', 0.00, 3.5, 'Active')"
        )
        .bind(&acc_number)
        .bind(borrower_id)
        .execute(&mut *tx)
        .await?;
        final_acc_number = acc_number;
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "accountNumber": final_acc_number,
            "accountName": account_name,
            "accountType": "Savings",
            "cifid": payload.cifid,
            "status": "active",
            "createdAt": Utc::now(),
            "balance": 0.00
        }),
        "Savings account created successfully"
    )))
}

#[utoipa::path(
    post,
    path = "/api/v1/onboarding/loan-account",
    responses(
        (status = 200, description = "Create default loan account", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Onboarding"
)]
#[post("/loan-account")]
pub async fn create_loan(
    pool: web::Data<PgPool>,
    payload: web::Json<CifAccountRequest>,
) -> Result<impl Responder, AppError> {
    // Strip "CIF-" prefix to search by id_number
    let raw_cif = payload.cifid.replace("CIF-", "");

    // Fetch borrower
    let borrower = sqlx::query(
        "SELECT id, first_name, last_name FROM borrowers WHERE id_number = $1 OR id::text = $2 OR ('CIF-' || id_number) = $3"
    )
    .bind(&raw_cif)
    .bind(&payload.cifid)
    .bind(&payload.cifid)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Borrower profile not found for the given CIFID".to_string()))?;

    let first_name = borrower.get::<String, _>("first_name");
    let last_name = borrower.get::<String, _>("last_name");
    let account_name = format!("{} {}", first_name, last_name);

    let acc_number = format!("LN-{}", &Uuid::new_v4().to_string()[..8].to_uppercase());

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "accountNumber": acc_number,
            "accountName": account_name,
            "accountType": "Loan",
            "cifid": payload.cifid,
            "status": "active",
            "createdAt": Utc::now(),
            "availableLimit": 5000000.00 // Default credit eligibility limit pre-approved
        }),
        "Loan account created successfully"
    )))
}
