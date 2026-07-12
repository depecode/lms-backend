use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::NaiveDate;

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Expense {
    pub id: Uuid,
    pub category_id: Uuid,
    pub category_name: String,
    pub description: String,
    pub amount: f64,
    pub date: NaiveDate,
    pub vendor: Option<String>,
    pub reference: Option<String>,
    pub status: String,
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ExpenseCategory {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub description: Option<String>,
    pub budget_limit: Option<f64>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateExpenseRequest {
    pub category_id: Uuid,
    pub description: String,
    pub amount: f64,
    pub date: NaiveDate,
    pub vendor: Option<String>,
    pub reference: Option<String>,
    pub attachments: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExpenseRequest {
    pub category_id: Option<Uuid>,
    pub description: Option<String>,
    pub amount: Option<f64>,
    pub date: Option<NaiveDate>,
    pub vendor: Option<String>,
    pub reference: Option<String>,
    pub status: Option<String>,
    pub attachments: Option<Vec<String>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/expenses",
    responses(
        (status = 200, description = "List expenses", body = ApiResponse<Vec<Expense>>)
    ),
    tag = "Expenses"
)]
#[get("")]
pub async fn get_expenses(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let expenses = sqlx::query_as::<_, Expense>(
        r#"
        SELECT 
            e.id, e.category_id, ec.name as category_name, e.description, 
            e.amount::float8 as amount, e.date, e.vendor, e.reference, 
            e.status, e.attachments
        FROM expenses e
        JOIN expense_categories ec ON e.category_id = ec.id
        ORDER BY e.date DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(expenses, "Expenses list retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/expenses/categories",
    responses(
        (status = 200, description = "List categories", body = ApiResponse<Vec<ExpenseCategory>>)
    ),
    tag = "Expenses"
)]
#[get("/categories")]
pub async fn get_categories(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let categories = sqlx::query_as::<_, ExpenseCategory>(
        r#"
        SELECT id, code, name, description, budget_limit::float8 as budget_limit
        FROM expense_categories
        ORDER BY code ASC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(categories, "Expense categories retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/expenses",
    responses(
        (status = 201, description = "Create expense", body = ApiResponse<Expense>)
    ),
    tag = "Expenses"
)]
#[post("")]
pub async fn create_expense(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateExpenseRequest>,
) -> Result<impl Responder, AppError> {
    // Check if category exists
    let cat_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM expense_categories WHERE id = $1)"
    )
    .bind(payload.category_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !cat_exists {
        return Err(AppError::NotFound("Expense category not found".to_string()));
    }

    let attachments = payload.attachments.clone().unwrap_or_default();
    let ref_no = payload.reference.clone()
        .unwrap_or_else(|| format!("EXP-{}", &Uuid::new_v4().to_string()[..8].to_uppercase()));

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO expenses (category_id, description, amount, date, vendor, reference, status, attachments)
        VALUES ($1, $2, $3, $4, $5, $6, 'Draft', $7)
        RETURNING id
        "#
    )
    .bind(payload.category_id)
    .bind(&payload.description)
    .bind(payload.amount)
    .bind(payload.date)
    .bind(&payload.vendor)
    .bind(ref_no)
    .bind(attachments)
    .fetch_one(pool.get_ref())
    .await?;

    let expense = sqlx::query_as::<_, Expense>(
        r#"
        SELECT 
            e.id, e.category_id, ec.name as category_name, e.description, 
            e.amount::float8 as amount, e.date, e.vendor, e.reference, 
            e.status, e.attachments
        FROM expenses e
        JOIN expense_categories ec ON e.category_id = ec.id
        WHERE e.id = $1
        "#,
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(expense, "Expense created successfully")))
}

#[utoipa::path(
    put,
    path = "/api/v1/expenses/{id}",
    responses(
        (status = 200, description = "Update expense", body = ApiResponse<Expense>)
    ),
    tag = "Expenses"
)]
#[put("/{id}")]
pub async fn update_expense(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateExpenseRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid expense ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    // Check existing
    let existing = sqlx::query_as::<_, Expense>(
        r#"
        SELECT 
            e.id, e.category_id, ec.name as category_name, e.description, 
            e.amount::float8 as amount, e.date, e.vendor, e.reference, 
            e.status, e.attachments
        FROM expenses e
        JOIN expense_categories ec ON e.category_id = ec.id
        WHERE e.id = $1 FOR UPDATE
        "#,
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Expense not found".to_string()))?;

    let category_id = payload.category_id.unwrap_or(existing.category_id);
    let description = payload.description.as_ref().unwrap_or(&existing.description);
    let amount = payload.amount.unwrap_or(existing.amount);
    let date = payload.date.unwrap_or(existing.date);
    let vendor = payload.vendor.as_ref().or(existing.vendor.as_ref());
    let reference = payload.reference.as_ref().or(existing.reference.as_ref());
    let status = payload.status.as_ref().unwrap_or(&existing.status);
    let attachments = payload.attachments.as_ref().or(existing.attachments.as_ref());

    let updated = sqlx::query_as::<_, Expense>(
        r#"
        UPDATE expenses SET
            category_id = $1, description = $2, amount = $3, date = $4,
            vendor = $5, reference = $6, status = $7, attachments = $8, updated_at = NOW()
        WHERE id = $9
        RETURNING id, category_id, 
            (SELECT name FROM expense_categories WHERE id = category_id) as category_name, 
            description, amount::float8 as amount, date, vendor, reference, status, attachments
        "#
    )
    .bind(category_id)
    .bind(description)
    .bind(amount)
    .bind(date)
    .bind(vendor)
    .bind(reference)
    .bind(status)
    .bind(attachments)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Expense updated successfully")))
}

#[utoipa::path(
    delete,
    path = "/api/v1/expenses/{id}",
    responses(
        (status = 200, description = "Delete expense", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Expenses"
)]
#[delete("/{id}")]
pub async fn delete_expense(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid expense ID format".to_string()))?;

    let result = sqlx::query("DELETE FROM expenses WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Expense not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Expense deleted successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/expenses/{id}/approve",
    responses(
        (status = 200, description = "Approve expense", body = ApiResponse<Expense>)
    ),
    tag = "Expenses"
)]
#[post("/{id}/approve")]
pub async fn approve_expense(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid expense ID format".to_string()))?;

    let updated = sqlx::query_as::<_, Expense>(
        r#"
        UPDATE expenses 
        SET status = 'Approved', updated_at = NOW() 
        WHERE id = $1
        RETURNING id, category_id, 
            (SELECT name FROM expense_categories WHERE id = category_id) as category_name, 
            description, amount::float8 as amount, date, vendor, reference, status, attachments
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Expense not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Expense approved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/expenses/kpi",
    responses(
        (status = 200, description = "Get expense KPI data", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Expenses"
)]
#[get("/kpi")]
pub async fn get_expense_kpi(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let row = sqlx::query(
        r#"
        SELECT 
            COALESCE(SUM(amount)::float8, 0.0) as total_spent,
            COALESCE(SUM(CASE WHEN status = 'Approved' OR status = 'Paid' THEN amount END)::float8, 0.0) as approved_spent,
            COALESCE(COUNT(id), 0)::int4 as total_count
        FROM expenses
        "#
    )
    .fetch_one(pool.get_ref())
    .await?;

    let total_spent: f64 = row.get("total_spent");
    let approved_spent: f64 = row.get("approved_spent");
    let total_count: i32 = row.get("total_count");

    // Fetch monthly budgets limit aggregate
    let active_budget: f64 = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(budget_limit)::float8, 0.0) FROM expense_categories"
    )
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "totalSpent": total_spent,
            "approvedSpent": approved_spent,
            "activeBudget": active_budget,
            "transactionsCount": total_count
        }),
        "Expense KPIs retrieved successfully"
    )))
}
