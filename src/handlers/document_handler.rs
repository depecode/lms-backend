use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Document {
    pub id: Uuid,
    pub name: String,
    pub file_type: String,
    pub file_size: i32,
    pub file_url: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub uploaded_by: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UploadDocumentRequest {
    pub name: String,
    pub file_type: String,
    pub file_size: i32,
    pub file_url: String,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub uploaded_by: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/documents",
    responses(
        (status = 201, description = "Document uploaded", body = ApiResponse<Document>)
    ),
    tag = "Documents"
)]
#[post("")]
pub async fn upload_doc(
    pool: web::Data<PgPool>,
    payload: web::Json<UploadDocumentRequest>,
) -> Result<impl Responder, AppError> {
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO documents (name, file_type, file_size, file_url, entity_type, entity_id, uploaded_by, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, 'Pending')
        RETURNING id
        "#
    )
    .bind(&payload.name)
    .bind(&payload.file_type)
    .bind(payload.file_size)
    .bind(&payload.file_url)
    .bind(&payload.entity_type)
    .bind(payload.entity_id)
    .bind(&payload.uploaded_by)
    .fetch_one(pool.get_ref())
    .await?;

    let doc = sqlx::query_as::<_, Document>(
        "SELECT id, name, file_type, file_size, file_url, entity_type, entity_id, uploaded_by, status, created_at, updated_at FROM documents WHERE id = $1"
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(doc, "Document uploaded successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents",
    responses(
        (status = 200, description = "Get documents", body = ApiResponse<Vec<Document>>)
    ),
    tag = "Documents"
)]
#[get("")]
pub async fn get_docs(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let docs = sqlx::query_as::<_, Document>(
        "SELECT id, name, file_type, file_size, file_url, entity_type, entity_id, uploaded_by, status, created_at, updated_at FROM documents ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(docs, "Documents retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{entity_type}/{entity_id}",
    responses(
        (status = 200, description = "Get entity documents", body = ApiResponse<Vec<Document>>)
    ),
    tag = "Documents"
)]
#[get("/{entity_type}/{entity_id}")]
pub async fn get_entity_docs(
    pool: web::Data<PgPool>,
    path: web::Path<(String, String)>,
) -> Result<impl Responder, AppError> {
    let (entity_type, entity_id_str) = path.into_inner();
    let entity_id = Uuid::parse_str(&entity_id_str)
        .map_err(|_| AppError::BadRequest("Invalid entity ID format".to_string()))?;

    let docs = sqlx::query_as::<_, Document>(
        "SELECT id, name, file_type, file_size, file_url, entity_type, entity_id, uploaded_by, status, created_at, updated_at FROM documents WHERE entity_type = $1 AND entity_id = $2 ORDER BY created_at DESC"
    )
    .bind(&entity_type)
    .bind(entity_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(docs, "Entity documents retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{id}",
    responses(
        (status = 200, description = "Get document by id", body = ApiResponse<Document>)
    ),
    tag = "Documents"
)]
#[get("/{id}")]
pub async fn get_doc(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid document ID format".to_string()))?;

    let doc = sqlx::query_as::<_, Document>(
        "SELECT id, name, file_type, file_size, file_url, entity_type, entity_id, uploaded_by, status, created_at, updated_at FROM documents WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(doc, "Document details retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/documents/{id}/verify",
    responses(
        (status = 200, description = "Verify document", body = ApiResponse<Document>)
    ),
    tag = "Documents"
)]
#[post("/{id}/verify")]
pub async fn verify_doc(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid document ID format".to_string()))?;

    let doc = sqlx::query_as::<_, Document>(
        "UPDATE documents SET status = 'Verified', updated_at = NOW() WHERE id = $1 RETURNING id, name, file_type, file_size, file_url, entity_type, entity_id, uploaded_by, status, created_at, updated_at"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(doc, "Document verified successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{id}/download",
    responses(
        (status = 200, description = "Download document")
    ),
    tag = "Documents"
)]
#[get("/{id}/download")]
pub async fn download_doc(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid document ID format".to_string()))?;

    let doc = sqlx::query_as::<_, Document>(
        "SELECT id, name, file_type, file_size, file_url, entity_type, entity_id, uploaded_by, status, created_at, updated_at FROM documents WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    // Mock binary contents returned for download
    let contents = format!("Binary content of file {} ({})", doc.name, doc.file_url);

    Ok(HttpResponse::Ok()
        .content_type("application/octet-stream")
        .insert_header(("Content-Disposition", format!("attachment; filename=\"{}\"", doc.name)))
        .body(contents))
}

#[utoipa::path(
    get,
    path = "/api/v1/documents/{id}/versions",
    responses(
        (status = 200, description = "Get document versions", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Documents"
)]
#[get("/{id}/versions")]
pub async fn get_doc_versions(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid document ID format".to_string()))?;

    let doc = sqlx::query_as::<_, Document>(
        "SELECT id, name, file_type, file_size, file_url, entity_type, entity_id, uploaded_by, status, created_at, updated_at FROM documents WHERE id = $1"
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Document not found".to_string()))?;

    // Since we don't track multiple version entities, return version 1.0 based on current record
    let version_history = serde_json::json!([
        {
            "version": "1.0",
            "name": doc.name,
            "fileSize": doc.file_size,
            "uploadedBy": doc.uploaded_by,
            "uploadedAt": doc.created_at,
            "status": doc.status
        }
    ]);

    Ok(HttpResponse::Ok().json(ApiResponse::success(version_history, "Document versions retrieved successfully")))
}

#[utoipa::path(
    delete,
    path = "/api/v1/documents/{id}",
    responses(
        (status = 200, description = "Delete document", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Documents"
)]
#[delete("/{id}")]
pub async fn delete_doc(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid document ID format".to_string()))?;

    let result = sqlx::query("DELETE FROM documents WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Document not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Document deleted successfully")))
}
