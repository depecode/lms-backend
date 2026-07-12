use serde::Serialize;
use chrono::{DateTime, Utc};
use utoipa::ToSchema;

#[derive(Serialize, ToSchema)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub message: String,
    pub meta: ResponseMeta,
}

#[derive(Serialize, ToSchema)]
pub struct ResponseMeta {
    pub timestamp: DateTime<Utc>,
    pub version: String,
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub data: Vec<T>,
    pub meta: PaginatedMeta,
    pub message: String,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedMeta {
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_pages: i64,
    pub has_next_page: bool,
    pub has_previous_page: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, ToSchema)]
pub struct ApiError {
    pub success: bool,
    pub error: ErrorDetail,
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ErrorDetail {
    pub code: String,
    pub message: String,
    pub details: Vec<String>,
    pub timestamp: DateTime<Utc>,
    pub request_id: String,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T, message: &str) -> Self {
        Self {
            success: true,
            data: Some(data),
            message: message.to_string(),
            meta: ResponseMeta {
                timestamp: Utc::now(),
                version: "v1".to_string(),
            },
        }
    }

    pub fn message(message: &str) -> Self {
        Self {
            success: true,
            data: None,
            message: message.to_string(),
            meta: ResponseMeta {
                timestamp: Utc::now(),
                version: "v1".to_string(),
            },
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            data: None,
            message: message.to_string(),
            meta: ResponseMeta {
                timestamp: Utc::now(),
                version: "v1".to_string(),
            },
        }
    }
}
