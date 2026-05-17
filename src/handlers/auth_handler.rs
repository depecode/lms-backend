use actix_web::{get, post, web, HttpResponse, Responder};
use serde::Deserialize;
use uuid::Uuid;
use sqlx::{PgPool, Row};
use crate::auth::jwt::create_jwt;
use crate::models::response::ApiResponse;
use utoipa::ToSchema;

#[derive(Deserialize, ToSchema)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = ApiResponse<serde_json::Value>),
        (status = 401, description = "Invalid credentials", body = ApiResponse<serde_json::Value>)
    )
)]
#[post("/login")]
pub async fn login(
    pool: web::Data<PgPool>,
    payload: web::Json<LoginRequest>,
) -> impl Responder {
    let user = sqlx::query("SELECT id, email FROM users WHERE email = $1")
        .bind(&payload.email)
        .fetch_optional(pool.get_ref())
        .await;

    match user {
        Ok(Some(row)) => {
            let user_id: Uuid = row.get("id");
            let email: String = row.get("email");
            let token = create_jwt(user_id, &email);
            match token {
                Ok(t) => HttpResponse::Ok().json(ApiResponse::success(
                    serde_json::json!({
                        "accessToken": t,
                        "refreshToken": "refresh_token_stub",
                        "expiresIn": 3600,
                        "user": {
                            "id": user_id,
                            "name": "John Doe",
                            "email": email,
                            "role": "credit_officer",
                            "branch": "Main Branch",
                            "permissions": [
                                "view_borrowers",
                                "create_loans",
                                "approve_loans",
                                "view_reports"
                            ]
                        }
                    }),
                    "Login successful"
                )),
                Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::message(&e.to_string())),
            }
        }
        Ok(None) => HttpResponse::Unauthorized().json(ApiResponse::<()>::message("Invalid credentials")),
        Err(e) => HttpResponse::InternalServerError().json(ApiResponse::<()>::message(&e.to_string())),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/refresh",
    responses(
        (status = 200, description = "Token refreshed successfully", body = ApiResponse<serde_json::Value>)
    )
)]
#[post("/refresh")]
pub async fn refresh() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "accessToken": "new_access_token_stub",
            "expiresIn": 3600
        }),
        "Token refreshed successfully"
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/auth/profile",
    responses(
        (status = 200, description = "Profile retrieved successfully", body = ApiResponse<serde_json::Value>)
    )
)]
#[get("/profile")]
pub async fn profile() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::success(
        serde_json::json!({
            "id": "usr_001",
            "name": "John Doe",
            "email": "john.doe@example.com",
            "phone": "+256701234567",
            "role": "credit_officer",
            "branch": "Main Branch",
            "joinDate": "2024-01-15",
            "permissions": [
                "view_borrowers",
                "create_loans",
                "approve_loans",
                "view_reports",
                "export_data"
            ]
        }),
        "Profile retrieved successfully"
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/auth/logout",
    responses(
        (status = 200, description = "Logged out successfully", body = ApiResponse<serde_json::Value>)
    )
)]
#[post("/logout")]
pub async fn logout() -> impl Responder {
    HttpResponse::Ok().json(ApiResponse::<()>::message("Logged out successfully"))
}
