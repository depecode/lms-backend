use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct LoanProduct {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub r#type: String,
    pub status: String,
    pub interest_rate_min: f64,
    pub interest_rate_max: f64,
    pub processing_fee_percent: Option<f64>,
    pub insurance_percent: Option<f64>,
    pub tenor_min_months: i32,
    pub tenor_max_months: i32,
    pub min_loan_amount: f64,
    pub max_loan_amount: f64,
    pub allow_early_repayment: bool,
    pub allow_partial_repayment: bool,
    pub grace_period_months: i32,
    pub requires_collateral: bool,
    pub requires_guarantors: bool,
    pub min_guarantors: i32,
    pub min_credit_score: i32,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateProductRequest {
    pub name: String,
    pub description: Option<String>,
    pub r#type: String,
    pub interest_rate_min: f64,
    pub interest_rate_max: f64,
    pub processing_fee_percent: Option<f64>,
    pub insurance_percent: Option<f64>,
    pub tenor_min_months: i32,
    pub tenor_max_months: i32,
    pub min_loan_amount: f64,
    pub max_loan_amount: f64,
    pub allow_early_repayment: Option<bool>,
    pub allow_partial_repayment: Option<bool>,
    pub grace_period_months: Option<i32>,
    pub requires_collateral: Option<bool>,
    pub requires_guarantors: Option<bool>,
    pub min_guarantors: Option<i32>,
    pub min_credit_score: Option<i32>,
}

#[utoipa::path(
    get,
    path = "/api/v1/loan-products",
    responses(
        (status = 200, description = "List loan products", body = ApiResponse<Vec<LoanProduct>>)
    ),
    tag = "Loan Products"
)]
#[get("")]
pub async fn list_products(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let products = sqlx::query_as::<_, LoanProduct>(
        r#"
        SELECT 
            id, name, description, type, status,
            interest_rate_min::float8 as interest_rate_min,
            interest_rate_max::float8 as interest_rate_max,
            processing_fee_percent::float8 as processing_fee_percent,
            insurance_percent::float8 as insurance_percent,
            tenor_min_months, tenor_max_months,
            min_loan_amount::float8 as min_loan_amount,
            max_loan_amount::float8 as max_loan_amount,
            allow_early_repayment, allow_partial_repayment,
            grace_period_months, requires_collateral, requires_guarantors,
            min_guarantors, min_credit_score
        FROM loan_products
        ORDER BY name ASC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(products, "List loan products retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/loan-products",
    responses(
        (status = 201, description = "Loan product created", body = ApiResponse<LoanProduct>)
    ),
    tag = "Loan Products"
)]
#[post("")]
pub async fn create_product(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateProductRequest>,
) -> Result<impl Responder, AppError> {
    // Check if product with name already exists
    let name_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM loan_products WHERE name = $1)"
    )
    .bind(&payload.name)
    .fetch_one(pool.get_ref())
    .await?;

    if name_exists {
        return Err(AppError::BadRequest("Loan product with this name already exists".to_string()));
    }

    let allow_early_repayment = payload.allow_early_repayment.unwrap_or(true);
    let allow_partial_repayment = payload.allow_partial_repayment.unwrap_or(true);
    let grace_period_months = payload.grace_period_months.unwrap_or(0);
    let requires_collateral = payload.requires_collateral.unwrap_or(false);
    let requires_guarantors = payload.requires_guarantors.unwrap_or(false);
    let min_guarantors = payload.min_guarantors.unwrap_or(0);
    let min_credit_score = payload.min_credit_score.unwrap_or(0);

    // Coerce unrecognized product types to "Personal" to avoid check constraint violations
    let product_type = match payload.r#type.as_str() {
        "Personal" | "Business" | "Mortgage" | "Auto" | "Education" => payload.r#type.clone(),
        _ => "Personal".to_string(),
    };

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO loan_products (
            name, description, type, status, interest_rate_min, interest_rate_max,
            processing_fee_percent, insurance_percent, tenor_min_months, tenor_max_months,
            min_loan_amount, max_loan_amount, allow_early_repayment, allow_partial_repayment,
            grace_period_months, requires_collateral, requires_guarantors, min_guarantors,
            min_credit_score
        ) VALUES (
            $1, $2, $3, 'Active', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18
        ) RETURNING id
        "#
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&product_type)
    .bind(payload.interest_rate_min)
    .bind(payload.interest_rate_max)
    .bind(payload.processing_fee_percent)
    .bind(payload.insurance_percent)
    .bind(payload.tenor_min_months)
    .bind(payload.tenor_max_months)
    .bind(payload.min_loan_amount)
    .bind(payload.max_loan_amount)
    .bind(allow_early_repayment)
    .bind(allow_partial_repayment)
    .bind(grace_period_months)
    .bind(requires_collateral)
    .bind(requires_guarantors)
    .bind(min_guarantors)
    .bind(min_credit_score)
    .fetch_one(pool.get_ref())
    .await?;

    let product = sqlx::query_as::<_, LoanProduct>(
        r#"
        SELECT 
            id, name, description, type, status,
            interest_rate_min::float8 as interest_rate_min,
            interest_rate_max::float8 as interest_rate_max,
            processing_fee_percent::float8 as processing_fee_percent,
            insurance_percent::float8 as insurance_percent,
            tenor_min_months, tenor_max_months,
            min_loan_amount::float8 as min_loan_amount,
            max_loan_amount::float8 as max_loan_amount,
            allow_early_repayment, allow_partial_repayment,
            grace_period_months, requires_collateral, requires_guarantors,
            min_guarantors, min_credit_score
        FROM loan_products
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(product, "Loan product created successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/loan-products/{id}",
    params(
        ("id" = String, Path, description = "Product ID")
    ),
    responses(
        (status = 200, description = "Get product details", body = ApiResponse<LoanProduct>)
    ),
    tag = "Loan Products"
)]
#[get("/{id}")]
pub async fn get_product(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid product ID format".to_string()))?;

    let product = sqlx::query_as::<_, LoanProduct>(
        r#"
        SELECT 
            id, name, description, type, status,
            interest_rate_min::float8 as interest_rate_min,
            interest_rate_max::float8 as interest_rate_max,
            processing_fee_percent::float8 as processing_fee_percent,
            insurance_percent::float8 as insurance_percent,
            tenor_min_months, tenor_max_months,
            min_loan_amount::float8 as min_loan_amount,
            max_loan_amount::float8 as max_loan_amount,
            allow_early_repayment, allow_partial_repayment,
            grace_period_months, requires_collateral, requires_guarantors,
            min_guarantors, min_credit_score
        FROM loan_products
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Loan product not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(product, "Loan product details retrieved successfully")))
}
