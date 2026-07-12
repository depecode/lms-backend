use actix_web::{get, post, delete, put, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    pub id: Uuid,
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub entity_type: String,
    pub required: bool,
    pub options: Option<Vec<String>>,
    pub default_value: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CustomFieldValue {
    pub id: Uuid,
    pub field_id: Uuid,
    pub entity_id: Uuid,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateFieldRequest {
    pub name: String,
    pub label: String,
    pub field_type: String,
    pub entity_type: String,
    pub required: bool,
    pub options: Option<Vec<String>>,
    pub default_value: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFieldRequest {
    pub name: Option<String>,
    pub label: Option<String>,
    pub field_type: Option<String>,
    pub entity_type: Option<String>,
    pub required: Option<bool>,
    pub options: Option<Vec<String>>,
    pub default_value: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SaveBorrowerDataRequest {
    pub field_id: Uuid,
    pub borrower_id: Uuid,
    pub value: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/custom-fields",
    responses(
        (status = 200, description = "List custom fields", body = ApiResponse<Vec<CustomField>>)
    ),
    tag = "Custom Fields"
)]
#[get("")]
pub async fn list_fields(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let fields = sqlx::query_as::<_, CustomField>(
        "SELECT id, name, label, field_type, entity_type, required, options, default_value, created_at, updated_at FROM custom_fields ORDER BY label ASC"
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(fields, "Custom fields retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/custom-fields/{id}",
    responses(
        (status = 200, description = "Get custom field", body = ApiResponse<CustomField>)
    ),
    tag = "Custom Fields"
)]
#[get("/{id}")]
pub async fn get_field(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid custom field ID format".to_string()))?;

    let field = sqlx::query_as::<_, CustomField>(
        "SELECT id, name, label, field_type, entity_type, required, options, default_value, created_at, updated_at FROM custom_fields WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Custom field not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(field, "Custom field retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/custom-fields",
    responses(
        (status = 201, description = "Create custom field", body = ApiResponse<CustomField>)
    ),
    tag = "Custom Fields"
)]
#[post("")]
pub async fn create_field(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateFieldRequest>,
) -> Result<impl Responder, AppError> {
    // Check if name already exists for the entity type
    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM custom_fields WHERE name = $1 AND entity_type = $2)"
    )
    .bind(&payload.name)
    .bind(&payload.entity_type)
    .fetch_one(pool.get_ref())
    .await?;

    if exists {
        return Err(AppError::BadRequest("Custom field name already exists for this entity type".to_string()));
    }

    let options = payload.options.clone().unwrap_or_default();

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO custom_fields (name, label, field_type, entity_type, required, options, default_value)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#
    )
    .bind(&payload.name)
    .bind(&payload.label)
    .bind(&payload.field_type)
    .bind(&payload.entity_type)
    .bind(payload.required)
    .bind(options)
    .bind(&payload.default_value)
    .fetch_one(pool.get_ref())
    .await?;

    let field = sqlx::query_as::<_, CustomField>(
        "SELECT id, name, label, field_type, entity_type, required, options, default_value, created_at, updated_at FROM custom_fields WHERE id = $1"
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(field, "Custom field created successfully")))
}

#[utoipa::path(
    put,
    path = "/api/v1/custom-fields/{id}",
    responses(
        (status = 200, description = "Update custom field", body = ApiResponse<CustomField>)
    ),
    tag = "Custom Fields"
)]
#[put("/{id}")]
pub async fn update_field(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateFieldRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid custom field ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, CustomField>(
        "SELECT id, name, label, field_type, entity_type, required, options, default_value, created_at, updated_at FROM custom_fields WHERE id = $1 FOR UPDATE"
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Custom field not found".to_string()))?;

    let name = payload.name.as_ref().unwrap_or(&existing.name);
    let label = payload.label.as_ref().unwrap_or(&existing.label);
    let field_type = payload.field_type.as_ref().unwrap_or(&existing.field_type);
    let entity_type = payload.entity_type.as_ref().unwrap_or(&existing.entity_type);
    let required = payload.required.unwrap_or(existing.required);
    let options = payload.options.as_ref().or(existing.options.as_ref());
    let default_value = payload.default_value.as_ref().or(existing.default_value.as_ref());

    let updated = sqlx::query_as::<_, CustomField>(
        r#"
        UPDATE custom_fields SET
            name = $1, label = $2, field_type = $3, entity_type = $4, required = $5, options = $6, default_value = $7, updated_at = NOW()
        WHERE id = $8
        RETURNING id, name, label, field_type, entity_type, required, options, default_value, created_at, updated_at
        "#
    )
    .bind(name)
    .bind(label)
    .bind(field_type)
    .bind(entity_type)
    .bind(required)
    .bind(options)
    .bind(default_value)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Custom field updated successfully")))
}

#[utoipa::path(
    delete,
    path = "/api/v1/custom-fields/{id}",
    responses(
        (status = 200, description = "Delete custom field", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Custom Fields"
)]
#[delete("/{id}")]
pub async fn delete_field(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid custom field ID format".to_string()))?;

    let result = sqlx::query("DELETE FROM custom_fields WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Custom field not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Custom field deleted successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/custom-fields/data/borrower/{borrower_id}",
    responses(
        (status = 200, description = "Get borrower custom data", body = ApiResponse<Vec<CustomFieldValue>>)
    ),
    tag = "Custom Fields"
)]
#[get("/data/borrower/{borrower_id}")]
pub async fn get_borrower_data(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let borrower_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid borrower ID format".to_string()))?;

    let values = sqlx::query_as::<_, CustomFieldValue>(
        r#"
        SELECT id, field_id, entity_id, value, created_at, updated_at
        FROM custom_field_values
        WHERE entity_id = $1
        ORDER BY created_at ASC
        "#
    )
    .bind(borrower_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(values, "Borrower custom data retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/custom-fields/data/borrower",
    responses(
        (status = 200, description = "Save borrower custom data", body = ApiResponse<CustomFieldValue>)
    ),
    tag = "Custom Fields"
)]
#[post("/data/borrower")]
pub async fn save_borrower_data(
    pool: web::Data<PgPool>,
    payload: web::Json<SaveBorrowerDataRequest>,
) -> Result<impl Responder, AppError> {
    // Check if field exists
    let field_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM custom_fields WHERE id = $1)"
    )
    .bind(payload.field_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !field_exists {
        return Err(AppError::NotFound("Custom field not found".to_string()));
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

    // Insert or update
    let value = sqlx::query_as::<_, CustomFieldValue>(
        r#"
        INSERT INTO custom_field_values (field_id, entity_id, value)
        VALUES ($1, $2, $3)
        ON CONFLICT (field_id, entity_id) 
        DO UPDATE SET value = EXCLUDED.value, updated_at = NOW()
        RETURNING id, field_id, entity_id, value, created_at, updated_at
        "#
    )
    .bind(payload.field_id)
    .bind(payload.borrower_id)
    .bind(&payload.value)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(value, "Borrower custom data saved successfully")))
}
