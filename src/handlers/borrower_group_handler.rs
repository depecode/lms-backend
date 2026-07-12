use actix_web::{get, post, put, delete, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc, NaiveDate};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BorrowerGroup {
    pub id: Uuid,
    pub name: String,
    pub group_code: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<String>,
    pub primary_borrower_id: Option<Uuid>,
    pub primary_contact: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub leader_name: Option<String>,
    pub leader_phone: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub formation_date: Option<NaiveDate>,
    pub guarantee_percentage: f64,
    pub expected_member_count: i32,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BorrowerGroupMember {
    pub borrower_id: Uuid,
    pub member_name: String,
    pub role: String,
    pub join_date: NaiveDate,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BorrowerGroupDetail {
    #[serde(flatten)]
    pub group: BorrowerGroup,
    pub members: Vec<BorrowerGroupMember>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateGroupRequest {
    pub name: String,
    pub group_code: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<String>,
    pub primary_borrower_id: Option<Uuid>,
    pub primary_contact: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub leader_name: Option<String>,
    pub leader_phone: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub formation_date: Option<NaiveDate>,
    pub guarantee_percentage: Option<f64>,
    pub expected_member_count: Option<i32>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateGroupRequest {
    pub name: Option<String>,
    pub group_code: Option<String>,
    pub description: Option<String>,
    pub r#type: Option<String>,
    pub primary_borrower_id: Option<Uuid>,
    pub primary_contact: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub leader_name: Option<String>,
    pub leader_phone: Option<String>,
    pub location: Option<String>,
    pub country: Option<String>,
    pub formation_date: Option<NaiveDate>,
    pub guarantee_percentage: Option<f64>,
    pub expected_member_count: Option<i32>,
    pub status: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct AddMemberRequest {
    pub borrower_id: Uuid,
    pub role: String,
}

#[utoipa::path(
    get,
    path = "/api/v1/borrower-groups",
    responses(
        (status = 200, description = "List borrower groups", body = ApiResponse<Vec<BorrowerGroup>>)
    ),
    tag = "Borrower Groups"
)]
#[get("")]
pub async fn get_groups(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let groups = sqlx::query_as::<_, BorrowerGroup>(
        r#"
        SELECT 
            id, name, group_code, description, type, primary_borrower_id, 
            primary_contact, contact_email, contact_phone, leader_name, 
            leader_phone, location, country, formation_date, 
            guarantee_percentage::float8 as guarantee_percentage, 
            expected_member_count, status, created_at, updated_at
        FROM borrower_groups
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(groups, "Borrower groups list retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/borrower-groups/{id}",
    responses(
        (status = 200, description = "Get group details", body = ApiResponse<BorrowerGroupDetail>)
    ),
    tag = "Borrower Groups"
)]
#[get("/{id}")]
pub async fn get_group(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid group ID format".to_string()))?;

    let group = sqlx::query_as::<_, BorrowerGroup>(
        r#"
        SELECT 
            id, name, group_code, description, type, primary_borrower_id, 
            primary_contact, contact_email, contact_phone, leader_name, 
            leader_phone, location, country, formation_date, 
            guarantee_percentage::float8 as guarantee_percentage, 
            expected_member_count, status, created_at, updated_at
        FROM borrower_groups
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Borrower group not found".to_string()))?;

    let members = sqlx::query_as::<_, BorrowerGroupMember>(
        r#"
        SELECT 
            bgm.borrower_id, 
            (b.first_name || ' ' || b.last_name) as member_name, 
            bgm.role, bgm.join_date
        FROM borrower_group_members bgm
        JOIN borrowers b ON bgm.borrower_id = b.id
        WHERE bgm.group_id = $1
        "#,
    )
    .bind(id)
    .fetch_all(pool.get_ref())
    .await?;

    let detail = BorrowerGroupDetail { group, members };
    Ok(HttpResponse::Ok().json(ApiResponse::success(detail, "Borrower group details retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/borrower-groups/borrower/{borrower_id}",
    responses(
        (status = 200, description = "Get groups for borrower", body = ApiResponse<Vec<BorrowerGroup>>)
    ),
    tag = "Borrower Groups"
)]
#[get("/borrower/{borrower_id}")]
pub async fn get_groups_by_borrower(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let borrower_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid borrower ID format".to_string()))?;

    let groups = sqlx::query_as::<_, BorrowerGroup>(
        r#"
        SELECT 
            bg.id, bg.name, bg.group_code, bg.description, bg.type, 
            bg.primary_borrower_id, bg.primary_contact, bg.contact_email, 
            bg.contact_phone, bg.leader_name, bg.leader_phone, bg.location, 
            bg.country, bg.formation_date, 
            bg.guarantee_percentage::float8 as guarantee_percentage, 
            bg.expected_member_count, bg.status, bg.created_at, bg.updated_at
        FROM borrower_groups bg
        JOIN borrower_group_members bgm ON bg.id = bgm.group_id
        WHERE bgm.borrower_id = $1
        ORDER BY bg.created_at DESC
        "#,
    )
    .bind(borrower_id)
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(groups, "Groups for borrower retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/borrower-groups",
    responses(
        (status = 201, description = "Create group", body = ApiResponse<BorrowerGroupDetail>)
    ),
    tag = "Borrower Groups"
)]
#[post("")]
pub async fn create_group(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateGroupRequest>,
) -> Result<impl Responder, AppError> {
    let guarantee_percentage = payload.guarantee_percentage.unwrap_or(0.0);
    let expected_member_count = payload.expected_member_count.unwrap_or(0);

    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO borrower_groups (
            name, group_code, description, type, primary_borrower_id,
            primary_contact, contact_email, contact_phone, leader_name,
            leader_phone, location, country, formation_date, guarantee_percentage,
            expected_member_count
        ) VALUES (
            $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15
        ) RETURNING id
        "#
    )
    .bind(&payload.name)
    .bind(&payload.group_code)
    .bind(&payload.description)
    .bind(&payload.r#type)
    .bind(payload.primary_borrower_id)
    .bind(&payload.primary_contact)
    .bind(&payload.contact_email)
    .bind(&payload.contact_phone)
    .bind(&payload.leader_name)
    .bind(&payload.leader_phone)
    .bind(&payload.location)
    .bind(&payload.country)
    .bind(payload.formation_date)
    .bind(guarantee_percentage)
    .bind(expected_member_count)
    .fetch_one(pool.get_ref())
    .await?;

    let group = sqlx::query_as::<_, BorrowerGroup>(
        r#"
        SELECT 
            id, name, group_code, description, type, primary_borrower_id, 
            primary_contact, contact_email, contact_phone, leader_name, 
            leader_phone, location, country, formation_date, 
            guarantee_percentage::float8 as guarantee_percentage, 
            expected_member_count, status, created_at, updated_at
        FROM borrower_groups
        WHERE id = $1
        "#
    )
    .bind(id)
    .fetch_one(pool.get_ref())
    .await?;

    let detail = BorrowerGroupDetail { group, members: vec![] };
    Ok(HttpResponse::Created().json(ApiResponse::success(detail, "Borrower group created successfully")))
}

#[utoipa::path(
    put,
    path = "/api/v1/borrower-groups/{id}",
    responses(
        (status = 200, description = "Update group", body = ApiResponse<BorrowerGroup>)
    ),
    tag = "Borrower Groups"
)]
#[put("/{id}")]
pub async fn update_group(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<UpdateGroupRequest>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid group ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    let existing = sqlx::query_as::<_, BorrowerGroup>(
        r#"
        SELECT 
            id, name, group_code, description, type, primary_borrower_id, 
            primary_contact, contact_email, contact_phone, leader_name, 
            leader_phone, location, country, formation_date, 
            guarantee_percentage::float8 as guarantee_percentage, 
            expected_member_count, status, created_at, updated_at
        FROM borrower_groups WHERE id = $1 FOR UPDATE
        "#
    )
    .bind(id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Borrower group not found".to_string()))?;

    let name = payload.name.as_ref().unwrap_or(&existing.name);
    let group_code = payload.group_code.as_ref().or(existing.group_code.as_ref());
    let description = payload.description.as_ref().or(existing.description.as_ref());
    let r#type = payload.r#type.as_ref().or(existing.r#type.as_ref());
    let primary_borrower_id = payload.primary_borrower_id.or(existing.primary_borrower_id);
    let primary_contact = payload.primary_contact.as_ref().or(existing.primary_contact.as_ref());
    let contact_email = payload.contact_email.as_ref().or(existing.contact_email.as_ref());
    let contact_phone = payload.contact_phone.as_ref().or(existing.contact_phone.as_ref());
    let leader_name = payload.leader_name.as_ref().or(existing.leader_name.as_ref());
    let leader_phone = payload.leader_phone.as_ref().or(existing.leader_phone.as_ref());
    let location = payload.location.as_ref().or(existing.location.as_ref());
    let country = payload.country.as_ref().or(existing.country.as_ref());
    let formation_date = payload.formation_date.or(existing.formation_date);
    let guarantee_percentage = payload.guarantee_percentage.unwrap_or(existing.guarantee_percentage);
    let expected_member_count = payload.expected_member_count.unwrap_or(existing.expected_member_count);
    let status = payload.status.as_ref().unwrap_or(&existing.status);

    let updated = sqlx::query_as::<_, BorrowerGroup>(
        r#"
        UPDATE borrower_groups SET
            name = $1, group_code = $2, description = $3, type = $4,
            primary_borrower_id = $5, primary_contact = $6, contact_email = $7,
            contact_phone = $8, leader_name = $9, leader_phone = $10,
            location = $11, country = $12, formation_date = $13,
            guarantee_percentage = $14, expected_member_count = $15,
            status = $16, updated_at = NOW()
        WHERE id = $17
        RETURNING 
            id, name, group_code, description, type, primary_borrower_id, 
            primary_contact, contact_email, contact_phone, leader_name, 
            leader_phone, location, country, formation_date, 
            guarantee_percentage::float8 as guarantee_percentage, 
            expected_member_count, status, created_at, updated_at
        "#
    )
    .bind(name)
    .bind(group_code)
    .bind(description)
    .bind(r#type)
    .bind(primary_borrower_id)
    .bind(primary_contact)
    .bind(contact_email)
    .bind(contact_phone)
    .bind(leader_name)
    .bind(leader_phone)
    .bind(location)
    .bind(country)
    .bind(formation_date)
    .bind(guarantee_percentage)
    .bind(expected_member_count)
    .bind(status)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(updated, "Borrower group updated successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/borrower-groups/{group_id}/members",
    responses(
        (status = 200, description = "Add member to group", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Borrower Groups"
)]
#[post("/{group_id}/members")]
pub async fn add_member(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<AddMemberRequest>,
) -> Result<impl Responder, AppError> {
    let group_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid group ID format".to_string()))?;

    // Check if group exists
    let group_exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM borrower_groups WHERE id = $1)"
    )
    .bind(group_id)
    .fetch_one(pool.get_ref())
    .await?;

    if !group_exists {
        return Err(AppError::NotFound("Borrower group not found".to_string()));
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

    // Check if already a member
    let is_member = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS(SELECT 1 FROM borrower_group_members WHERE group_id = $1 AND borrower_id = $2)"
    )
    .bind(group_id)
    .bind(payload.borrower_id)
    .fetch_one(pool.get_ref())
    .await?;

    if is_member {
        return Err(AppError::BadRequest("Borrower is already a member of this group".to_string()));
    }

    sqlx::query(
        r#"
        INSERT INTO borrower_group_members (group_id, borrower_id, role)
        VALUES ($1, $2, $3)
        "#
    )
    .bind(group_id)
    .bind(payload.borrower_id)
    .bind(&payload.role)
    .execute(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Member added to group successfully")))
}

#[utoipa::path(
    delete,
    path = "/api/v1/borrower-groups/{group_id}/members/{borrower_id}",
    responses(
        (status = 200, description = "Remove member from group", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Borrower Groups"
)]
#[delete("/{group_id}/members/{borrower_id}")]
pub async fn remove_member(
    pool: web::Data<PgPool>,
    path: web::Path<(String, String)>,
) -> Result<impl Responder, AppError> {
    let (group_id_str, borrower_id_str) = path.into_inner();

    let group_id = Uuid::parse_str(&group_id_str)
        .map_err(|_| AppError::BadRequest("Invalid group ID format".to_string()))?;
    let borrower_id = Uuid::parse_str(&borrower_id_str)
        .map_err(|_| AppError::BadRequest("Invalid borrower ID format".to_string()))?;

    let result = sqlx::query(
        "DELETE FROM borrower_group_members WHERE group_id = $1 AND borrower_id = $2"
    )
    .bind(group_id)
    .bind(borrower_id)
    .execute(pool.get_ref())
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Member not found in this group".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Member removed from group successfully")))
}

#[utoipa::path(
    delete,
    path = "/api/v1/borrower-groups/{id}",
    responses(
        (status = 200, description = "Delete group", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Borrower Groups"
)]
#[delete("/{id}")]
pub async fn delete_group(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid group ID format".to_string()))?;

    let result = sqlx::query("DELETE FROM borrower_groups WHERE id = $1")
        .bind(id)
        .execute(pool.get_ref())
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Borrower group not found".to_string()));
    }

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Borrower group deleted successfully")))
}
