use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDate, Duration};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Investor {
    pub id: Uuid,
    pub name: String,
    pub r#type: String,
    pub contact_person: Option<String>,
    pub email: String,
    pub phone: String,
    pub country: String,
    pub website: Option<String>,
    pub registration_number: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Investment {
    pub id: Uuid,
    pub investor_id: Uuid,
    pub r#type: String,
    pub amount: f64,
    pub currency: String,
    pub term: i32,
    pub interest_rate: Option<f64>,
    pub start_date: NaiveDate,
    pub maturity_date: Option<NaiveDate>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateInvestorRequest {
    pub name: String,
    pub r#type: String,
    pub contact_person: Option<String>,
    pub email: String,
    pub phone: String,
    pub country: String,
    pub website: Option<String>,
    pub registration_number: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInvestorRequest {
    pub name: Option<String>,
    pub r#type: Option<String>,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub country: Option<String>,
    pub website: Option<String>,
    pub registration_number: Option<String>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordInvestmentRequest {
    pub r#type: String,
    pub amount: f64,
    pub currency: Option<String>,
    pub term: i32,
    pub interest_rate: Option<f64>,
    pub start_date: NaiveDate,
    pub notes: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/investors",
    responses(
        (status = 200, description = "List investors", body = ApiResponse<Vec<Investor>>)
    ),
    tag = "Investors"
)]
#[get("")]
pub async fn list_investors(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let investors = sqlx::query_as::<_, Investor>(
        r#"
        SELECT 
            id, name, type, contact_person, email, phone, country, 
            website, registration_number, status, created_at
        FROM investors
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(investors, "List investors retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/investors/{id}",
    responses(
        (status = 200, description = "Get investor by ID", body = ApiResponse<Investor>)
    ),
    tag = "Investors"
)]
#[get("/{id}")]
pub async fn get_investor(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid investor ID format".to_string()))?;

    let investor = sqlx::query_as::<_, Investor>(
        r#"
        SELECT 
            id, name, type, contact_person, email, phone, country, 
            website, registration_number, status, created_at
        FROM investors
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Investor not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(investor, "Investor retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/investors",
    responses(
        (status = 201, description = "Create investor", body = ApiResponse<Investor>)
    ),
    tag = "Investors"
)]
#[post("")]
pub async fn create_investor(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateInvestorRequest>,
) -> Result<impl Responder, AppError> {
    let investor_type = match payload.r#type.as_str() {
        "Individual" | "Institution" | "Bank" | "Government" | "NGO" => payload.r#type.clone(),
        _ => "Individual".to_string(),
    };

    let investor = sqlx::query_as::<_, Investor>(
        r#"
        INSERT INTO investors (
            name, type, contact_person, email, phone, country, website, 
            registration_number, status
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, 'Active'
        ) RETURNING id, name, type, contact_person, email, phone, country, website, registration_number, status, created_at
        "#
    )
    .bind(&payload.name)
    .bind(&investor_type)
    .bind(&payload.contact_person)
    .bind(&payload.email)
    .bind(&payload.phone)
    .bind(&payload.country)
    .bind(&payload.website)
    .bind(&payload.registration_number)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(investor, "Investor created successfully")))
}

#[utoipa::path(
    put,
    path = "/api/v1/investors/{id}",
    responses(
        (status = 200, description = "Update investor details", body = ApiResponse<Investor>)
    ),
    tag = "Investors"
)]
#[put("/{id}")]
pub async fn update_investor(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateInvestorRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid investor ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, Investor>(
        r#"
        SELECT 
            id, name, type, contact_person, email, phone, country, 
            website, registration_number, status, created_at
        FROM investors
        WHERE id = $1 FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Investor not found".to_string()))?;

    let name = payload.name.as_ref().unwrap_or(&existing.name);
    let r#type = payload.r#type.as_ref().unwrap_or(&existing.r#type);
    let coerced_type = match r#type.as_str() {
        "Individual" | "Institution" | "Bank" | "Government" | "NGO" => r#type.clone(),
        _ => "Individual".to_string(),
    };
    let contact_person = payload.contact_person.as_ref().or(existing.contact_person.as_ref());
    let email = payload.email.as_ref().unwrap_or(&existing.email);
    let phone = payload.phone.as_ref().unwrap_or(&existing.phone);
    let country = payload.country.as_ref().unwrap_or(&existing.country);
    let website = payload.website.as_ref().or(existing.website.as_ref());
    let registration_number = payload.registration_number.as_ref().or(existing.registration_number.as_ref());
    let status = payload.status.as_ref().unwrap_or(&existing.status);

    let updated = sqlx::query_as::<_, Investor>(
        r#"
        UPDATE investors SET
            name = $1, type = $2, contact_person = $3, email = $4, phone = $5,
            country = $6, website = $7, registration_number = $8, status = $9, updated_at = NOW()
        WHERE id = $10
        RETURNING id, name, type, contact_person, email, phone, country, website, registration_number, status, created_at
        "#
    )
    .bind(name)
    .bind(&coerced_type)
    .bind(contact_person)
    .bind(email)
    .bind(phone)
    .bind(country)
    .bind(website)
    .bind(registration_number)
    .bind(status)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Investor details updated successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/investors/{id}/investments",
    responses(
        (status = 200, description = "Get investments list", body = ApiResponse<Vec<Investment>>)
    ),
    tag = "Investors"
)]
#[get("/{id}/investments")]
pub async fn get_investments(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let investor_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid investor ID format".to_string()))?;

    let investments = sqlx::query_as::<_, Investment>(
        r#"
        SELECT 
            id, investor_id, type, amount::float8 as amount, currency, 
            term, interest_rate::float8 as interest_rate, start_date, maturity_date, 
            status, notes
        FROM investments
        WHERE investor_id = $1
        ORDER BY start_date DESC
        "#,
    )
    .bind(investor_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(investments, "Investments retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/investors/{id}/investments",
    responses(
        (status = 201, description = "Record investment", body = ApiResponse<Investment>)
    ),
    tag = "Investors"
)]
#[post("/{id}/investments")]
pub async fn record_investment(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<RecordInvestmentRequest>,
) -> Result<impl Responder, AppError> {
    let investor_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid investor ID format".to_string()))?;

    // Verify investor exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM investors WHERE id = $1)"
    )
    .bind(investor_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        return Err(AppError::NotFound("Investor not found".to_string()));
    }

    if payload.amount <= 0.0 {
        return Err(AppError::BadRequest("Amount must be positive".to_string()));
    }

    let currency = payload.currency.clone().unwrap_or_else(|| "UGX".to_string());
    let maturity_date = payload.start_date + Duration::days(30 * payload.term as i64);

    let investment_type = match payload.r#type.as_str() {
        "Equity" | "Debt" | "Grant" | "Guarantee" => payload.r#type.clone(),
        _ => "Equity".to_string(),
    };

    let investment = sqlx::query_as::<_, Investment>(
        r#"
        INSERT INTO investments (
            investor_id, type, amount, currency, term, interest_rate, start_date, 
            maturity_date, status, notes
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, 'Active', $9
        ) RETURNING id, investor_id, type, amount::float8 as amount, currency, term, interest_rate::float8 as interest_rate, start_date, maturity_date, status, notes
        "#
    )
    .bind(investor_id)
    .bind(&investment_type)
    .bind(payload.amount)
    .bind(currency)
    .bind(payload.term)
    .bind(payload.interest_rate)
    .bind(payload.start_date)
    .bind(maturity_date)
    .bind(&payload.notes)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(investment, "Investment recorded successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/investors/metrics",
    responses(
        (status = 200, description = "Get investor metrics", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Investors"
)]
#[get("/metrics")]
pub async fn get_metrics(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            COALESCE(SUM(amount)::float8, 0.0) as total_invested,
            COALESCE(COUNT(DISTINCT investor_id), 0)::int4 as active_investors,
            COALESCE(AVG(interest_rate)::float8, 0.0) as average_return_rate,
            COALESCE(SUM(CASE WHEN status = 'Matured' THEN amount END)::float8, 0.0) as total_payouts
        FROM investments
        "#
    )
    .fetch_one(pool.get_ref())
    .await?;

    let total_invested: f64 = row.get("total_invested");
    let active_investors: i32 = row.get("active_investors");
    let average_return_rate: f64 = row.get("average_return_rate");
    let total_payouts: f64 = row.get("total_payouts");

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "totalInvested": total_invested,
            "activeInvestors": active_investors,
            "averageReturnRate": average_return_rate,
            "totalPayouts": total_payouts
        }),
        "Investor metrics retrieved successfully"
    )))
}

#[utoipa::path(
    delete,
    path = "/api/v1/investors/{id}",
    responses(
        (status = 200, description = "Delete investor", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Investors"
)]
#[delete("/{id}")]
pub async fn delete_investor(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid investor ID format".to_string()))?;

    let result = sqlx::query("DELETE FROM investors WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Investor not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Investor deleted successfully")))
}
