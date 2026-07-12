use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use std::fmt;
use crate::models::response::ApiResponse;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Jwt(jsonwebtoken::errors::Error),
    Unauthorized(String),
    NotFound(String),
    BadRequest(String),
    Internal(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(err) => write!(f, "Database error: {}", err),
            AppError::Jwt(err) => write!(f, "Authentication token error: {}", err),
            AppError::Unauthorized(msg) => write!(f, "Unauthorized access: {}", msg),
            AppError::NotFound(msg) => write!(f, "Resource not found: {}", msg),
            AppError::BadRequest(msg) => write!(f, "Invalid request: {}", msg),
            AppError::Internal(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl ResponseError for AppError {
    fn status_code(&self) -> StatusCode {
        match self {
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Jwt(_) => StatusCode::UNAUTHORIZED,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::BadRequest(_) => StatusCode::BAD_REQUEST,
            AppError::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        let status = self.status_code();
        let message = self.to_string();
        HttpResponse::build(status).json(ApiResponse::<()>::error(&message))
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Database(err)
    }
}

impl From<jsonwebtoken::errors::Error> for AppError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        AppError::Jwt(err)
    }
}
