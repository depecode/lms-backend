use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use sqlx::PgPool;
use uuid::Uuid;

use crate::models::user::{User, CreateUser, UpdateUser};

#[post("/users")]
pub async fn create_user(pool: web::Data<PgPool>, data: web::Json<CreateUser>) -> impl Responder {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (name, email)
        VALUES ($1, $2)
        RETURNING *
        "#
    )
    .bind(&data.name)
    .bind(&data.email)
    .fetch_one(pool.get_ref())
    .await;

    match user {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/users")]
pub async fn get_users(pool: web::Data<PgPool>) -> impl Responder {
    let users = sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(pool.get_ref())
        .await;

    match users {
        Ok(users) => HttpResponse::Ok().json(users),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[get("/users/{id}")]
pub async fn get_user(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(*id)
        .fetch_one(pool.get_ref())
        .await;

    match user {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[put("/users/{id}")]
pub async fn update_user(
    pool: web::Data<PgPool>,
    id: web::Path<Uuid>,
    data: web::Json<UpdateUser>,
) -> impl Responder {
    let user = sqlx::query_as::<_, User>(
        r#"
        UPDATE users
        SET name = COALESCE($1, name),
            email = COALESCE($2, email),
            updated_at = NOW()
        WHERE id = $3
        RETURNING *
        "#
    )
    .bind(&data.name)
    .bind(&data.email)
    .bind(*id)
    .fetch_one(pool.get_ref())
    .await;

    match user {
        Ok(user) => HttpResponse::Ok().json(user),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}

#[delete("/users/{id}")]
pub async fn delete_user(pool: web::Data<PgPool>, id: web::Path<Uuid>) -> impl Responder {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(*id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("User deleted"),
        Err(e) => HttpResponse::InternalServerError().body(e.to_string()),
    }
}
