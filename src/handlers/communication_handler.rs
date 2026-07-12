use actix_web::{get, post, put, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::{PgPool, Row};
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommunicationTemplate {
    pub id: Uuid,
    pub name: String,
    pub subject: Option<String>,
    pub body: String,
    pub r#type: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CommunicationHistory {
    pub id: Uuid,
    pub borrower_id: Uuid,
    pub recipient: String,
    pub r#type: String,
    pub subject: Option<String>,
    pub body: String,
    pub status: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateTemplateRequest {
    pub name: String,
    pub subject: Option<String>,
    pub body: String,
    pub r#type: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTemplateRequest {
    pub name: Option<String>,
    pub subject: Option<String>,
    pub body: Option<String>,
    pub r#type: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendMessageRequest {
    pub borrower_id: Uuid,
    pub recipient: String,
    pub r#type: String,
    pub subject: Option<String>,
    pub body: String,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendBulkMessagesRequest {
    pub borrower_ids: Vec<Uuid>,
    pub r#type: String,
    pub subject: Option<String>,
    pub body: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/communication/templates",
    responses(
        (status = 200, description = "List templates", body = ApiResponse<Vec<CommunicationTemplate>>)
    ),
    tag = "Communication"
)]
#[get("/templates")]
pub async fn get_templates(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let templates = sqlx::query_as::<_, CommunicationTemplate>(
        r#"
        SELECT id, name, subject, body, type, created_at, updated_at
        FROM communication_templates
        ORDER BY name ASC
        "#
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(templates, "Communication templates retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/communication/templates/{id}",
    responses(
        (status = 200, description = "Get template", body = ApiResponse<CommunicationTemplate>)
    ),
    tag = "Communication"
)]
#[get("/templates/{id}")]
pub async fn get_template(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid template ID format".to_string()))?;

    let template = sqlx::query_as::<_, CommunicationTemplate>(
        r#"
        SELECT id, name, subject, body, type, created_at, updated_at
        FROM communication_templates
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Communication template not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(template, "Communication template retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/communication/templates",
    responses(
        (status = 201, description = "Create template", body = ApiResponse<CommunicationTemplate>)
    ),
    tag = "Communication"
)]
#[post("/templates")]
pub async fn create_template(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateTemplateRequest>,
) -> Result<impl Responder, AppError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO communication_templates (name, subject, body, type)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#
    )
    .bind(&payload.name)
    .bind(&payload.subject)
    .bind(&payload.body)
    .bind(&payload.r#type)
    .fetch_one(pool.get_ref())
    .await?;

    let template = sqlx::query_as::<_, CommunicationTemplate>(
        r#"
        SELECT id, name, subject, body, type, created_at, updated_at
        FROM communication_templates
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(template, "Communication template created successfully")))
}

#[utoipa::path(
    put,
    path = "/api/v1/communication/templates/{id}",
    responses(
        (status = 200, description = "Update template", body = ApiResponse<CommunicationTemplate>)
    ),
    tag = "Communication"
)]
#[put("/templates/{id}")]
pub async fn update_template(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateTemplateRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid template ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, CommunicationTemplate>(
        r#"
        SELECT id, name, subject, body, type, created_at, updated_at
        FROM communication_templates
        WHERE id = $1 FOR UPDATE
        "#
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Communication template not found".to_string()))?;

    let name = payload.name.as_ref().unwrap_or(&existing.name);
    let subject = payload.subject.as_ref().or(existing.subject.as_ref());
    let body = payload.body.as_ref().unwrap_or(&existing.body);
    let r#type = payload.r#type.as_ref().unwrap_or(&existing.r#type);

    let updated = sqlx::query_as::<_, CommunicationTemplate>(
        r#"
        UPDATE communication_templates SET
            name = $1, subject = $2, body = $3, type = $4, updated_at = NOW()
        WHERE id = $5
        RETURNING id, name, subject, body, type, created_at, updated_at
        "#
    )
    .bind(name)
    .bind(subject)
    .bind(body)
    .bind(r#type)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Communication template updated successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/communication/messages",
    responses(
        (status = 200, description = "Send message", body = ApiResponse<CommunicationHistory>)
    ),
    tag = "Communication"
)]
#[post("/messages")]
pub async fn send_message(
    pool: web::Data<PgPool>,
    payload: web::Json<SendMessageRequest>,
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

    let history = sqlx::query_as::<_, CommunicationHistory>(
        r#"
        INSERT INTO communications (borrower_id, recipient, type, subject, body, status, sent_at)
        VALUES ($1, $2, $3, $4, $5, 'Sent', NOW())
        RETURNING id, borrower_id, recipient, type, subject, body, status, sent_at, created_at
        "#
    )
    .bind(payload.borrower_id)
    .bind(&payload.recipient)
    .bind(&payload.r#type)
    .bind(&payload.subject)
    .bind(&payload.body)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(history, "Message sent and recorded successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/communication/messages/bulk",
    responses(
        (status = 200, description = "Send bulk messages", body = ApiResponse<Vec<CommunicationHistory>>)
    ),
    tag = "Communication"
)]
#[post("/messages/bulk")]
pub async fn send_bulk_messages(
    pool: web::Data<PgPool>,
    payload: web::Json<SendBulkMessagesRequest>,
) -> Result<impl Responder, AppError> {
    let mut tx = pool.begin().await?;
    let mut results = Vec::new();

    for borrower_id in &payload.borrower_ids {

        let info = sqlx::query(
            "SELECT phone, email FROM borrowers WHERE id = $1"
        )
        .bind(borrower_id)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(row) = info {
            let phone: String = row.get("phone");
            let email: String = row.get("email");
            let recipient = if payload.r#type == "Email" { email } else { phone };

            let hist = sqlx::query_as::<_, CommunicationHistory>(
                r#"
                INSERT INTO communications (borrower_id, recipient, type, subject, body, status, sent_at)
                VALUES ($1, $2, $3, $4, $5, 'Sent', NOW())
                RETURNING id, borrower_id, recipient, type, subject, body, status, sent_at, created_at
                "#
            )
            .bind(borrower_id)
            .bind(recipient)
            .bind(&payload.r#type)
            .bind(&payload.subject)
            .bind(&payload.body)
            .fetch_one(&mut *tx)
            .await?;

            results.push(hist);
        }
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(results, "Bulk messages sent successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/communication/messages/history/{borrower_id}",
    responses(
        (status = 200, description = "Get message history", body = ApiResponse<Vec<CommunicationHistory>>)
    ),
    tag = "Communication"
)]
#[get("/messages/history/{borrower_id}")]
pub async fn get_message_history(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let borrower_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid borrower ID format".to_string()))?;

    let history = sqlx::query_as::<_, CommunicationHistory>(
        r#"
        SELECT id, borrower_id, recipient, type, subject, body, status, sent_at, created_at
        FROM communications
        WHERE borrower_id = $1
        ORDER BY created_at DESC
        "#
    )
    .bind(borrower_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(history, "Message history retrieved successfully")))
}
