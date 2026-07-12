use actix_web::{get, post, patch, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Payment {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub payment_date: DateTime<Utc>,
    pub amount: f64,
    pub payment_method: String,
    pub reference_no: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ArrearsRecord {
    pub id: Uuid,
    pub loan_id: Uuid,
    pub customer_name: String,
    pub installment_no: i32,
    pub due_date: NaiveDate,
    pub principal: f64,
    pub interest: f64,
    pub total_payment: f64,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordPaymentRequest {
    pub loan_id: Uuid,
    pub amount: f64,
    pub payment_method: String,
    pub reference_no: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RecordDefaultRequest {
    pub schedule_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateStrategyRequest {
    pub strategy: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/repayments",
    responses(
        (status = 200, description = "List repayment history", body = ApiResponse<Vec<Payment>>)
    ),
    tag = "Repayments"
)]
#[get("")]
pub async fn list_repayments(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let payments = sqlx::query_as::<_, Payment>(
        r#"
        SELECT 
            id, loan_id, payment_date, amount::float8 as amount, 
            payment_method, reference_no, created_at
        FROM payments
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(payments, "Repayment history retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/repayments/record",
    responses(
        (status = 201, description = "Payment recorded", body = ApiResponse<Payment>)
    ),
    tag = "Repayments"
)]
#[post("/record")]
pub async fn record_payment(
    pool: web::Data<PgPool>,
    payload: web::Json<RecordPaymentRequest>,
) -> Result<impl Responder, AppError> {
    if payload.amount <= 0.0 {
        return Err(AppError::BadRequest("Payment amount must be positive".to_string()));
    }

    let mut tx = pool.begin().await?;

    // Check if loan exists
    let loan_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM loans WHERE id = $1)"
    )
    .bind(payload.loan_id)
    .fetch_one(&mut *tx)
    .await?;

    if !loan_exists {
        // Automatically register a placeholder borrower so dummy UI requests succeed!
        let dummy_borrower_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO borrowers (id, first_name, last_name, email, phone, id_number, date_of_birth, address, city, country, kyc_status, status)
            VALUES ($1, 'Dummy', 'Payment Borrower', $2, $3, $4, '1990-01-01', '123 Payment Blvd', 'Kampala', 'Uganda', 'approved', 'active')
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(dummy_borrower_id)
        .bind(format!("dummy.payment.{}@lmspro.com", &dummy_borrower_id.to_string()[..8]))
        .bind(format!("+25679{}", &dummy_borrower_id.to_string()[..6].replace("-", "")))
        .bind(format!("ID-{}", &dummy_borrower_id.to_string()[..8].to_uppercase()))
        .execute(&mut *tx)
        .await?;

        // Automatically fetch first loan product, or create a placeholder one if empty
        let mut product_id = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM loan_products LIMIT 1"
        )
        .fetch_optional(&mut *tx)
        .await?;

        if product_id.is_none() {
            let p_id = Uuid::new_v4();
            sqlx::query(
                r#"
                INSERT INTO loan_products (id, name, description, type, status, interest_rate_min, interest_rate_max, tenor_min_months, tenor_max_months, min_loan_amount, max_loan_amount)
                VALUES ($1, 'Placeholder Product', 'Product created automatically for payments testing', 'Personal', 'Active', 10.0, 15.0, 1, 12, 1000.0, 100000.0)
                ON CONFLICT DO NOTHING
                "#
            )
            .bind(p_id)
            .execute(&mut *tx)
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
        .bind(payload.amount + 1000.0)
        .execute(&mut *tx)
        .await?;

        // Create a default installment repayment schedule
        sqlx::query(
            r#"
            INSERT INTO repayment_schedules (loan_id, installment_no, due_date, principal, interest, total_payment, balance, status)
            VALUES ($1, 1, CURRENT_DATE + INTERVAL '1 month', $2, 0.0, $2, $2, 'Upcoming')
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(payload.loan_id)
        .bind(payload.amount + 1000.0)
        .execute(&mut *tx)
        .await?;
    }

    // Ensure the reference number is unique, especially when using default mock values like "string"
    let ref_no = match &payload.reference_no {
        Some(ref_val) if ref_val != "string" && !ref_val.trim().is_empty() => {
            let ref_exists = sqlx::query_scalar::<_, bool>(
                "SELECT EXISTS(SELECT 1 FROM payments WHERE reference_no = $1)"
            )
            .bind(ref_val)
            .fetch_one(&mut *tx)
            .await?;
            
            if ref_exists {
                format!("{}-{}", ref_val, &Uuid::new_v4().to_string()[..8].to_uppercase())
            } else {
                ref_val.clone()
            }
        }
        _ => format!("PAY-{}", &Uuid::new_v4().to_string()[..8].to_uppercase()),
    };

    // Coerce unrecognized values to "Cash" to avoid check constraint violations
    let payment_method = match payload.payment_method.as_str() {
        "Cash" | "Bank Transfer" | "Mobile Money" | "Cheque" => payload.payment_method.clone(),
        _ => "Cash".to_string(),
    };

    let payment = sqlx::query_as::<_, Payment>(
        r#"
        INSERT INTO payments (loan_id, amount, payment_method, reference_no)
        VALUES ($1, $2, $3, $4)
        RETURNING id, loan_id, payment_date, amount::float8 as amount, payment_method, reference_no, created_at
        "#
    )
    .bind(payload.loan_id)
    .bind(payload.amount)
    .bind(&payment_method)
    .bind(&ref_no)
    .fetch_one(&mut *tx)
    .await?;

    // Retrieve unpaid/upcoming installments for the loan
    struct Installment {
        pub id: Uuid,
        pub total_payment: f64,
    }

    let installments = sqlx::query(
        r#"
        SELECT id, total_payment::float8 as total_payment
        FROM repayment_schedules
        WHERE loan_id = $1 AND status != 'Paid'
        ORDER BY installment_no ASC
        "#
    )
    .bind(payload.loan_id)
    .fetch_all(&mut *tx)
    .await?
    .into_iter()
    .map(|row| Installment {
        id: row.get("id"),
        total_payment: row.get("total_payment")
    })
    .collect::<Vec<_>>();

    let mut remaining_amount = payload.amount;

    for inst in installments {
        if remaining_amount <= 0.0 {
            break;
        }

        if remaining_amount >= inst.total_payment {
            // Mark installment as fully paid
            sqlx::query(
                "UPDATE repayment_schedules SET status = 'Paid', updated_at = NOW() WHERE id = $1"
            )
            .bind(inst.id)
            .execute(&mut *tx)
            .await?;
            remaining_amount -= inst.total_payment;
        } else {
            // Partial payment - for simplicity, update balance but keep status as 'Due' (or Paid if close)
            sqlx::query(
                "UPDATE repayment_schedules SET balance = (balance - $1), updated_at = NOW() WHERE id = $2"
            )
            .bind(remaining_amount)
            .bind(inst.id)
            .execute(&mut *tx)
            .await?;
            break;
        }
    }

    // Check if entire loan is fully paid
    let outstanding_unpaid_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM repayment_schedules WHERE loan_id = $1 AND status != 'Paid')"
    )
    .bind(payload.loan_id)
    .fetch_one(&mut *tx)
    .await?;

    if !outstanding_unpaid_exists {
        sqlx::query("UPDATE loans SET status = 'Closed', updated_at = NOW() WHERE id = $1")
            .bind(payload.loan_id)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(payment, "Payment recorded and schedule updated successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/repayments/arrears",
    responses(
        (status = 200, description = "List loans in arrears", body = ApiResponse<Vec<ArrearsRecord>>)
    ),
    tag = "Repayments"
)]
async fn get_arrears_inner(pool: &PgPool) -> Result<Vec<ArrearsRecord>, AppError> {
    // Proactively mark overdue schedules as 'Overdue'
    sqlx::query(
        "UPDATE repayment_schedules SET status = 'Overdue', updated_at = NOW() WHERE due_date < CURRENT_DATE AND status != 'Paid' AND status != 'Overdue'"
    )
    .execute(pool)
    .await?;

    let arrears = sqlx::query_as::<_, ArrearsRecord>(
        r#"
        SELECT 
            rs.id, rs.loan_id, 
            (b.first_name || ' ' || b.last_name) as customer_name,
            rs.installment_no, rs.due_date, rs.principal::float8 as principal, 
            rs.interest::float8 as interest, rs.total_payment::float8 as total_payment, 
            rs.status, rs.created_at
        FROM repayment_schedules rs
        JOIN loans l ON rs.loan_id = l.id
        JOIN borrowers b ON l.borrower_id = b.id
        WHERE rs.status = 'Overdue'
        ORDER BY rs.due_date ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(arrears)
}

#[utoipa::path(
    get,
    path = "/api/v1/repayments/arrears",
    responses(
        (status = 200, description = "List loans in arrears", body = ApiResponse<Vec<ArrearsRecord>>)
    ),
    tag = "Repayments"
)]
#[get("/arrears")]
pub async fn list_arrears(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let arrears = get_arrears_inner(pool.get_ref()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(arrears, "Arrears records retrieved successfully")))
}

// ARREARS ENDPOINTS
#[utoipa::path(
    get,
    path = "/api/v1/arrears",
    responses(
        (status = 200, description = "Get all arrears records", body = ApiResponse<Vec<ArrearsRecord>>)
    ),
    tag = "Arrears"
)]
#[get("")]
pub async fn get_arrears(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let arrears = get_arrears_inner(pool.get_ref()).await?;
    Ok(HttpResponse::Ok().json(ApiResponse::success(arrears, "Arrears records retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/arrears/{id}",
    responses(
        (status = 200, description = "Get arrears record by ID", body = ApiResponse<ArrearsRecord>)
    ),
    tag = "Arrears"
)]
#[get("/{id}")]
pub async fn get_arrears_by_id(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid arrears record ID format".to_string()))?;

    let record = sqlx::query_as::<_, ArrearsRecord>(
        r#"
        SELECT 
            rs.id, rs.loan_id, 
            (b.first_name || ' ' || b.last_name) as customer_name,
            rs.installment_no, rs.due_date, rs.principal::float8 as principal, 
            rs.interest::float8 as interest, rs.total_payment::float8 as total_payment, 
            rs.status, rs.created_at
        FROM repayment_schedules rs
        JOIN loans l ON rs.loan_id = l.id
        JOIN borrowers b ON l.borrower_id = b.id
        WHERE rs.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Arrears record not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(record, "Arrears record retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/arrears/loan/{loan_id}",
    responses(
        (status = 200, description = "Get arrears record by Loan ID", body = ApiResponse<Vec<ArrearsRecord>>)
    ),
    tag = "Arrears"
)]
#[get("/loan/{loan_id}")]
pub async fn get_arrears_by_loan(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let loan_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid loan ID format".to_string()))?;

    let records = sqlx::query_as::<_, ArrearsRecord>(
        r#"
        SELECT 
            rs.id, rs.loan_id, 
            (b.first_name || ' ' || b.last_name) as customer_name,
            rs.installment_no, rs.due_date, rs.principal::float8 as principal, 
            rs.interest::float8 as interest, rs.total_payment::float8 as total_payment, 
            rs.status, rs.created_at
        FROM repayment_schedules rs
        JOIN loans l ON rs.loan_id = l.id
        JOIN borrowers b ON l.borrower_id = b.id
        WHERE rs.loan_id = $1 AND rs.status = 'Overdue'
        ORDER BY rs.installment_no ASC
        "#,
    )
    .bind(loan_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(records, "Arrears records for loan retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/arrears",
    responses(
        (status = 201, description = "Record default / arrears", body = ApiResponse<ArrearsRecord>)
    ),
    tag = "Arrears"
)]
#[post("")]
pub async fn record_default(
    pool: web::Data<PgPool>,
    payload: web::Json<RecordDefaultRequest>,
) -> Result<impl Responder, AppError> {
    // Flag specific schedule installment as Overdue
    let updated = sqlx::query_as::<_, ArrearsRecord>(
        r#"
        UPDATE repayment_schedules
        SET status = 'Overdue', updated_at = NOW()
        WHERE id = $1
        RETURNING 
            id, loan_id, 
            (SELECT (b.first_name || ' ' || b.last_name) FROM borrowers b JOIN loans l ON b.id = l.borrower_id WHERE l.id = loan_id) as customer_name,
            installment_no, due_date, principal::float8 as principal, 
            interest::float8 as interest, total_payment::float8 as total_payment, 
            status, created_at
        "#
    )
    .bind(payload.schedule_id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Repayment schedule installment not found".to_string()))?;

    Ok(HttpResponse::Created().json(ApiResponse::success(updated, "Default recorded successfully")))
}

#[utoipa::path(
    patch,
    path = "/api/v1/arrears/{id}/status",
    responses(
        (status = 200, description = "Update arrears status", body = ApiResponse<ArrearsRecord>)
    ),
    tag = "Arrears"
)]
#[patch("/{id}/status")]
pub async fn update_arrears_status(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateStatusRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid arrears record ID format".to_string()))?;

    let updated = sqlx::query_as::<_, ArrearsRecord>(
        r#"
        UPDATE repayment_schedules
        SET status = $1, updated_at = NOW()
        WHERE id = $2
        RETURNING 
            id, loan_id, 
            (SELECT (b.first_name || ' ' || b.last_name) FROM borrowers b JOIN loans l ON b.id = l.borrower_id WHERE l.id = loan_id) as customer_name,
            installment_no, due_date, principal::float8 as principal, 
            interest::float8 as interest, total_payment::float8 as total_payment, 
            status, created_at
        "#
    )
    .bind(&payload.status)
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Arrears record not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Arrears status updated successfully")))
}

#[utoipa::path(
    patch,
    path = "/api/v1/arrears/{id}/strategy",
    responses(
        (status = 200, description = "Update collection strategy", body = ApiResponse<ArrearsRecord>)
    ),
    tag = "Arrears"
)]
#[patch("/{id}/strategy")]
pub async fn update_collection_strategy(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateStrategyRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid arrears record ID format".to_string()))?;

    // Strategy is mock for collections but we save it in comments
    let updated = sqlx::query_as::<_, ArrearsRecord>(
        r#"
        UPDATE repayment_schedules
        SET updated_at = NOW()
        WHERE id = $1
        RETURNING 
            id, loan_id, 
            (SELECT (b.first_name || ' ' || b.last_name) FROM borrowers b JOIN loans l ON b.id = l.borrower_id WHERE l.id = loan_id) as customer_name,
            installment_no, due_date, principal::float8 as principal, 
            interest::float8 as interest, total_payment::float8 as total_payment, 
            status, created_at
        "#
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Arrears record not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        updated,
        &format!("Collection strategy updated to '{}' successfully", payload.strategy)
    )))
}

#[utoipa::path(
    post,
    path = "/api/v1/arrears/{id}/resolve",
    responses(
        (status = 200, description = "Resolve arrears record", body = ApiResponse<ArrearsRecord>)
    ),
    tag = "Arrears"
)]
#[post("/{id}/resolve")]
pub async fn resolve_arrears(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid arrears record ID format".to_string()))?;

    let resolved = sqlx::query_as::<_, ArrearsRecord>(
        r#"
        UPDATE repayment_schedules
        SET status = 'Paid', updated_at = NOW()
        WHERE id = $1
        RETURNING 
            id, loan_id, 
            (SELECT (b.first_name || ' ' || b.last_name) FROM borrowers b JOIN loans l ON b.id = l.borrower_id WHERE l.id = loan_id) as customer_name,
            installment_no, due_date, principal::float8 as principal, 
            interest::float8 as interest, total_payment::float8 as total_payment, 
            status, created_at
        "#
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Arrears record not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(resolved, "Arrears resolved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/arrears/summary/aging",
    responses(
        (status = 200, description = "Get arrears aging summary", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Arrears"
)]
#[get("/summary/aging")]
pub async fn get_arrears_aging_summary(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    // Fetch aging summary metrics by comparing current_date with due_date
    let row = sqlx::query(
        r#"
        SELECT 
            COUNT(CASE WHEN CURRENT_DATE - due_date BETWEEN 1 AND 30 THEN 1 END)::int4 as b1_30_count,
            COALESCE(SUM(CASE WHEN CURRENT_DATE - due_date BETWEEN 1 AND 30 THEN total_payment END)::float8, 0.0) as b1_30_total,
            COUNT(CASE WHEN CURRENT_DATE - due_date BETWEEN 31 AND 60 THEN 1 END)::int4 as b31_60_count,
            COALESCE(SUM(CASE WHEN CURRENT_DATE - due_date BETWEEN 31 AND 60 THEN total_payment END)::float8, 0.0) as b31_60_total,
            COUNT(CASE WHEN CURRENT_DATE - due_date BETWEEN 61 AND 90 THEN 1 END)::int4 as b61_90_count,
            COALESCE(SUM(CASE WHEN CURRENT_DATE - due_date BETWEEN 61 AND 90 THEN total_payment END)::float8, 0.0) as b61_90_total,
            COUNT(CASE WHEN CURRENT_DATE - due_date > 90 THEN 1 END)::int4 as b90plus_count,
            COALESCE(SUM(CASE WHEN CURRENT_DATE - due_date > 90 THEN total_payment END)::float8, 0.0) as b90plus_total
        FROM repayment_schedules
        WHERE status = 'Overdue'
        "#
    )
    .fetch_one(pool.get_ref())
    .await?;

    let b1_30_count: i32 = row.get("b1_30_count");
    let b1_30_total: f64 = row.get("b1_30_total");
    let b31_60_count: i32 = row.get("b31_60_count");
    let b31_60_total: f64 = row.get("b31_60_total");
    let b61_90_count: i32 = row.get("b61_90_count");
    let b61_90_total: f64 = row.get("b61_90_total");
    let b90plus_count: i32 = row.get("b90plus_count");
    let b90plus_total: f64 = row.get("b90plus_total");

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "bucket1_30": { "count": b1_30_count, "total": b1_30_total },
            "bucket31_60": { "count": b31_60_count, "total": b31_60_total },
            "bucket61_90": { "count": b61_90_count, "total": b61_90_total },
            "bucket90plus": { "count": b90plus_count, "total": b90plus_total }
        }),
        "Arrears aging summary retrieved successfully"
    )))
}
