use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, Duration};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Loan {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,
    pub amount: f64,
    pub tenor: i32,
    pub interest_rate: f64,
    pub status: String,
    pub r#type: String,
    pub application_date: DateTime<Utc>,
    pub approval_date: Option<DateTime<Utc>>,
    pub disbursement_date: Option<DateTime<Utc>>,
    pub purpose: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SubmitLoanRequest {
    pub borrower_id: Uuid,
    pub product_id: Uuid,
    pub amount: f64,
    pub tenor: i32,
    pub interest_rate: f64,
    pub purpose: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/loans",
    responses(
        (status = 200, description = "List loans", body = ApiResponse<Vec<Loan>>)
    ),
    tag = "Loans"
)]
#[get("")]
pub async fn list_loans(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let loans = sqlx::query_as::<_, Loan>(
        r#"
        SELECT 
            l.id, l.borrower_id as customer_id, 
            (b.first_name || ' ' || b.last_name) as customer_name,
            b.email as customer_email, b.phone as customer_phone,
            l.amount::float8 as amount, l.tenor, l.interest_rate::float8 as interest_rate, 
            l.status, lp.type as type, l.application_date, l.approval_date, 
            l.disbursement_date, l.purpose
        FROM loans l
        JOIN borrowers b ON l.borrower_id = b.id
        JOIN loan_products lp ON l.product_id = lp.id
        ORDER BY l.created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(loans, "Loans retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/loans",
    responses(
        (status = 201, description = "Loan application created successfully", body = ApiResponse<Loan>)
    ),
    tag = "Loans"
)]
#[post("")]
pub async fn submit_loan(
    pool: web::Data<PgPool>,
    payload: web::Json<SubmitLoanRequest>,
) -> Result<impl Responder, AppError> {
    // Verify borrower exists
    let borrower_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM borrowers WHERE id = $1)"
    )
    .bind(payload.borrower_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !borrower_exists {
        return Err(AppError::NotFound("Borrower not found".to_string()));
    }

    // Verify product exists
    let product_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM loan_products WHERE id = $1)"
    )
    .bind(payload.product_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !product_exists {
        return Err(AppError::NotFound("Loan product not found".to_string()));
    }

    let loan_id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO loans (
            borrower_id, product_id, amount, tenor, interest_rate, status, purpose
        ) VALUES (
            $1, $2, $3, $4, $5, 'Pending', $6
        ) RETURNING id
        "#
    )
    .bind(payload.borrower_id)
    .bind(payload.product_id)
    .bind(payload.amount)
    .bind(payload.tenor)
    .bind(payload.interest_rate)
    .bind(&payload.purpose)
    .fetch_one(pool.get_ref())
    .await?;

    let loan = sqlx::query_as::<_, Loan>(
        r#"
        SELECT 
            l.id, l.borrower_id as customer_id, 
            (b.first_name || ' ' || b.last_name) as customer_name,
            b.email as customer_email, b.phone as customer_phone,
            l.amount::float8 as amount, l.tenor, l.interest_rate::float8 as interest_rate, 
            l.status, lp.type as type, l.application_date, l.approval_date, 
            l.disbursement_date, l.purpose
        FROM loans l
        JOIN borrowers b ON l.borrower_id = b.id
        JOIN loan_products lp ON l.product_id = lp.id
        WHERE l.id = $1
        "#,
    )
    .bind(loan_id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(loan, "Loan application created successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/loans/{id}",
    params(
        ("id" = String, Path, description = "Loan ID")
    ),
    responses(
        (status = 200, description = "Loan details retrieved successfully", body = ApiResponse<Loan>)
    ),
    tag = "Loans"
)]
#[get("/{id}")]
pub async fn get_loan(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid loan ID format".to_string()))?;

    let loan = sqlx::query_as::<_, Loan>(
        r#"
        SELECT 
            l.id, l.borrower_id as customer_id, 
            (b.first_name || ' ' || b.last_name) as customer_name,
            b.email as customer_email, b.phone as customer_phone,
            l.amount::float8 as amount, l.tenor, l.interest_rate::float8 as interest_rate, 
            l.status, lp.type as type, l.application_date, l.approval_date, 
            l.disbursement_date, l.purpose
        FROM loans l
        JOIN borrowers b ON l.borrower_id = b.id
        JOIN loan_products lp ON l.product_id = lp.id
        WHERE l.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Loan not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(loan, "Loan details retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/loans/{id}/approve",
    params(
        ("id" = String, Path, description = "Loan ID")
    ),
    responses(
        (status = 200, description = "Loan approved successfully", body = ApiResponse<Loan>)
    ),
    tag = "Loans"
)]
#[post("/{id}/approve")]
pub async fn approve_loan(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid loan ID format".to_string()))?;

    sqlx::query(
        "UPDATE loans SET status = 'Approved', approval_date = NOW(), updated_at = NOW() WHERE id = $1"
    )
    .bind(id)
    .execute(pool.get_ref())
    .await?;

    let loan = sqlx::query_as::<_, Loan>(
        r#"
        SELECT 
            l.id, l.borrower_id as customer_id, 
            (b.first_name || ' ' || b.last_name) as customer_name,
            b.email as customer_email, b.phone as customer_phone,
            l.amount::float8 as amount, l.tenor, l.interest_rate::float8 as interest_rate, 
            l.status, lp.type as type, l.application_date, l.approval_date, 
            l.disbursement_date, l.purpose
        FROM loans l
        JOIN borrowers b ON l.borrower_id = b.id
        JOIN loan_products lp ON l.product_id = lp.id
        WHERE l.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(loan, "Loan approved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/loans/{id}/disburse",
    params(
        ("id" = String, Path, description = "Loan ID")
    ),
    responses(
        (status = 200, description = "Loan disbursed successfully", body = ApiResponse<Loan>)
    ),
    tag = "Loans"
)]
#[post("/{id}/disburse")]
pub async fn disburse_loan(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid loan ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    // Check existing
    let existing_loan = sqlx::query_as::<_, Loan>(
        r#"
        SELECT 
            l.id, l.borrower_id as customer_id, 
            (b.first_name || ' ' || b.last_name) as customer_name,
            b.email as customer_email, b.phone as customer_phone,
            l.amount::float8 as amount, l.tenor, l.interest_rate::float8 as interest_rate, 
            l.status, lp.type as type, l.application_date, l.approval_date, 
            l.disbursement_date, l.purpose
        FROM loans l
        JOIN borrowers b ON l.borrower_id = b.id
        JOIN loan_products lp ON l.product_id = lp.id
        WHERE l.id = $1 FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Loan not found".to_string()))?;

    if existing_loan.status != "Approved" && existing_loan.status != "Pending" {
        return Err(AppError::BadRequest("Loan must be in Approved or Pending status to disburse".to_string()));
    }

    // Update status
    sqlx::query(
        "UPDATE loans SET status = 'Disbursed', disbursement_date = NOW(), updated_at = NOW() WHERE id = $1"
    )
    .bind(id)
    .execute(&mut *tx)
    .await?;

    // Generate Repayment Schedule Installments (Equal Monthly Installments)
    let p = existing_loan.amount;
    let t = existing_loan.tenor;
    let r_annual = existing_loan.interest_rate;
    let r_monthly = r_annual / 12.0 / 100.0;

    let emi = if r_monthly > 0.0 {
        p * (r_monthly * (1.0 + r_monthly).powi(t)) / ((1.0 + r_monthly).powi(t) - 1.0)
    } else {
        p / (t as f64)
    };

    let start_date = Utc::now().naive_utc().date();
    let mut balance = p;

    for i in 1..=t {
        let installment_due_date = start_date + Duration::days(30 * i as i64);
        let interest = if r_monthly > 0.0 { balance * r_monthly } else { 0.0 };
        let principal = emi - interest;
        balance -= principal;
        
        let final_balance = if i == t { 0.0 } else { balance.max(0.0) };

        sqlx::query(
            r#"
            INSERT INTO repayment_schedules (loan_id, installment_no, due_date, principal, interest, total_payment, balance, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'Upcoming')
            "#
        )
        .bind(id)
        .bind(i)
        .bind(installment_due_date)
        .bind(principal)
        .bind(interest)
        .bind(emi)
        .bind(final_balance)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    // Refetch updated loan info
    let loan = sqlx::query_as::<_, Loan>(
        r#"
        SELECT 
            l.id, l.borrower_id as customer_id, 
            (b.first_name || ' ' || b.last_name) as customer_name,
            b.email as customer_email, b.phone as customer_phone,
            l.amount::float8 as amount, l.tenor, l.interest_rate::float8 as interest_rate, 
            l.status, lp.type as type, l.application_date, l.approval_date, 
            l.disbursement_date, l.purpose
        FROM loans l
        JOIN borrowers b ON l.borrower_id = b.id
        JOIN loan_products lp ON l.product_id = lp.id
        WHERE l.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(loan, "Loan disbursed and repayment schedule generated successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/loans/scoring/{id}",
    params(
        ("id" = String, Path, description = "Loan ID")
    ),
    responses(
        (status = 200, description = "Credit score analysis retrieved successfully", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Loans"
)]
#[get("/scoring/{id}")]
pub async fn get_loan_scoring(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid loan ID format".to_string()))?;

    // Verify loan exists
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM loans WHERE id = $1)"
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    if !exists {
        return Err(AppError::NotFound("Loan not found".to_string()));
    }

    // Return a mock credit scoring based deterministically on the loan ID's bytes
    let score = 300 + (id.as_bytes()[0] as i32) * 2;
    let status = if score >= 700 {
        "Excellent"
    } else if score >= 600 {
        "Good"
    } else if score >= 500 {
        "Fair"
    } else {
        "Poor"
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "loanId": id,
            "creditScore": score,
            "grade": status,
            "riskAssessment": if score >= 600 { "Low Risk" } else if score >= 500 { "Moderate Risk" } else { "High Risk" },
            "verifiedAttributes": {
                "identityMatched": true,
                "monthlyIncomeVerified": true,
                "collateralAppraised": true
            }
        }),
        "Credit score analysis retrieved successfully"
    )))
}
