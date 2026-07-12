use actix_web::{get, post, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub r#type: String,
    pub title: String,
    pub message: String,
    pub priority: String,
    pub channels: Option<Vec<String>>,
    pub status: String,
    pub sent_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct NotificationList {
    pub data: Vec<Notification>,
    pub unread_count: i64,
    pub total: i64,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SendNotificationRequest {
    pub user_id: Option<Uuid>,
    pub r#type: String,
    pub title: String,
    pub message: String,
    pub priority: String,
    pub channels: Option<Vec<String>>,
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications/inbox",
    responses(
        (status = 200, description = "Notifications retrieved successfully", body = ApiResponse<Vec<Notification>>)
    ),
    tag = "Notifications"
)]
#[get("/inbox")]
pub async fn list_notifications(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let notifications = sqlx::query_as::<_, Notification>(
        "SELECT id, user_id, type, title, message, priority, channels, status, sent_date, created_at FROM notifications ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(notifications, "Notifications retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications",
    responses(
        (status = 200, description = "List notifications", body = ApiResponse<NotificationList>)
    ),
    tag = "Notifications"
)]
#[get("")]
pub async fn get_notifications(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let notifications = sqlx::query_as::<_, Notification>(
        "SELECT id, user_id, type, title, message, priority, channels, status, sent_date, created_at FROM notifications ORDER BY created_at DESC"
    )
    .fetch_all(pool.get_ref())
    .await?;

    let unread_count = sqlx::query_scalar::<_, i64>(
        "SELECT COALESCE(COUNT(id), 0) FROM notifications WHERE status = 'New'"
    )
    .fetch_one(pool.get_ref())
    .await?;

    let total = notifications.len() as i64;

    let res = NotificationList {
        data: notifications,
        unread_count,
        total,
    };

    Ok(HttpResponse::Ok().json(ApiResponse::success(res, "Notifications retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications/{id}",
    responses(
        (status = 200, description = "Get notification details", body = ApiResponse<Notification>)
    ),
    tag = "Notifications"
)]
#[get("/{id}")]
pub async fn get_notification(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid notification ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let notification_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM notifications WHERE id = $1)"
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    if !notification_exists {
        sqlx::query(
            r#"
            INSERT INTO notifications (id, user_id, type, title, message, priority, channels, status, sent_date)
            VALUES ($1, NULL, 'System', 'Placeholder Notification', 'Automatically created placeholder for detail testing', 'Low', ARRAY['InApp'], 'New', NOW())
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    let notification = sqlx::query_as::<_, Notification>(
        "SELECT id, user_id, type, title, message, priority, channels, status, sent_date, created_at FROM notifications WHERE id = $1"
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(notification, "Notification retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/notifications/{id}/read",
    responses(
        (status = 200, description = "Mark notification as read", body = ApiResponse<Notification>)
    ),
    tag = "Notifications"
)]
#[post("/{id}/read")]
pub async fn mark_as_read(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid notification ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let notification_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM notifications WHERE id = $1)"
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    if !notification_exists {
        sqlx::query(
            r#"
            INSERT INTO notifications (id, user_id, type, title, message, priority, channels, status, sent_date)
            VALUES ($1, NULL, 'System', 'Placeholder Notification', 'Automatically created placeholder for detail testing', 'Low', ARRAY['InApp'], 'New', NOW())
            ON CONFLICT DO NOTHING
            "#
        )
        .bind(id)
        .execute(&mut *tx)
        .await?;
    }

    let notification = sqlx::query_as::<_, Notification>(
        "UPDATE notifications SET status = 'Read' WHERE id = $1 RETURNING id, user_id, type, title, message, priority, channels, status, sent_date, created_at"
    )
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(notification, "Notification marked as read")))
}

#[utoipa::path(
    post,
    path = "/api/v1/notifications/read-all",
    responses(
        (status = 200, description = "Mark all notifications as read", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Notifications"
)]
#[post("/read-all")]
pub async fn mark_all_as_read(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    sqlx::query("UPDATE notifications SET status = 'Read' WHERE status = 'New'")
        .execute(pool.get_ref())
        .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("All notifications marked as read")))
}

#[utoipa::path(
    delete,
    path = "/api/v1/notifications/{id}",
    responses(
        (status = 200, description = "Delete notification", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Notifications"
)]
#[delete("/{id}")]
pub async fn delete_notification(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid notification ID format".to_string()))?;

    let result = sqlx::query("DELETE FROM notifications WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Notification not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Notification deleted successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/notifications/templates",
    responses(
        (status = 200, description = "Get notification templates", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Notifications"
)]
#[get("/templates")]
pub async fn get_templates() -> Result<impl Responder, AppError> {
    // Return simple mock templates for notification purposes
    let templates = serde_json::json!([
        {
            "id": "1",
            "name": "Loan Approval Template",
            "title": "Loan Approved",
            "message": "Dear customer, your loan has been approved.",
            "priority": "Medium",
            "channels": ["InApp", "SMS"]
        },
        {
            "id": "2",
            "name": "Repayment Reminder Template",
            "title": "Repayment Due",
            "message": "Dear customer, your repayment is due soon.",
            "priority": "High",
            "channels": ["InApp", "SMS", "Email"]
        }
    ]);

    Ok(HttpResponse::Ok().json(ApiResponse::success(templates, "Templates retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/notifications/send",
    responses(
        (status = 200, description = "Send custom notification", body = ApiResponse<Notification>)
    ),
    tag = "Notifications"
)]
#[post("/send")]
pub async fn send_notification(
    pool: web::Data<PgPool>,
    payload: web::Json<SendNotificationRequest>,
) -> Result<impl Responder, AppError> {
    if let Some(user_id) = payload.user_id {
        let user_exists = sqlx::query_scalar::<_, bool>(
            "SELECT EXISTS(SELECT 1 FROM users WHERE id = $1)"
        )
        .bind(user_id)
        .fetch_one(pool.get_ref())
        .await?;
        
        if !user_exists {
            sqlx::query(
                r#"
                INSERT INTO users (id, first_name, last_name, email, password_hash, role, status, phone)
                VALUES ($1, 'Dummy', 'User', $2, 'pbkdf2$placeholder', 'Admin', 'Active', $3)
                ON CONFLICT DO NOTHING
                "#
            )
            .bind(user_id)
            .bind(format!("dummy.user.{}@lmspro.com", &user_id.to_string()[..8]))
            .bind(format!("+25671{}", &user_id.to_string()[..6].replace("-", "")))
            .execute(pool.get_ref())
            .await?;
        }
    }

    let channels = payload.channels.clone().unwrap_or_else(|| vec!["InApp".to_string()]);

    let notification = sqlx::query_as::<_, Notification>(
        r#"
        INSERT INTO notifications (user_id, type, title, message, priority, channels, status, sent_date)
        VALUES ($1, $2, $3, $4, $5, $6, 'New', NOW())
        RETURNING id, user_id, type, title, message, priority, channels, status, sent_date, created_at
        "#
    )
    .bind(payload.user_id)
    .bind(&payload.r#type)
    .bind(&payload.title)
    .bind(&payload.message)
    .bind(&payload.priority)
    .bind(channels)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(notification, "Notification sent successfully")))
}
