use actix_web::{get, post, web, HttpResponse, Responder};
use crate::models::response::ApiResponse;
use crate::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Workflow {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    pub status: String,
    pub steps: serde_json::Value,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowInstance {
    pub id: Uuid,
    pub workflow_id: Uuid,
    pub entity_type: String,
    pub entity_id: Uuid,
    pub status: String,
    pub current_step: i32,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, sqlx::FromRow, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: Uuid,
    pub instance_id: Uuid,
    pub step_number: i32,
    pub step_name: String,
    pub assigned_to: String,
    pub priority: String,
    pub status: String,
    pub due_date: DateTime<Utc>,
    pub comments: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateWorkflowRequest {
    pub name: String,
    pub description: Option<String>,
    pub entity_type: String,
    pub steps: serde_json::Value,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StartWorkflowRequest {
    pub entity_type: String,
    pub entity_id: Uuid,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CompleteTaskRequest {
    pub comments: Option<String>,
}

#[derive(Debug, Deserialize, utoipa::ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RejectTaskRequest {
    pub comments: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/tasks",
    responses(
        (status = 200, description = "Pending tasks retrieved successfully", body = ApiResponse<Vec<Task>>)
    ),
    tag = "Workflows"
)]
#[get("/tasks")]
pub async fn list_tasks(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let tasks = sqlx::query_as::<_, Task>(
        r#"
        SELECT 
            id, instance_id, step_number, step_name, assigned_to, priority, 
            status, due_date, comments, completed_at, created_at, updated_at
        FROM workflow_tasks
        WHERE status IN ('Pending', 'InProgress')
        ORDER BY due_date ASC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(tasks, "Pending tasks retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows",
    responses(
        (status = 200, description = "Get workflows list", body = ApiResponse<Vec<Workflow>>)
    ),
    tag = "Workflows"
)]
#[get("")]
pub async fn get_workflows(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let workflows = sqlx::query_as::<_, Workflow>(
        r#"
        SELECT 
            id, name, description, entity_type, status, steps, version, 
            created_at, updated_at
        FROM workflow_definitions
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(workflows, "Workflows retrieved successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/{id}",
    responses(
        (status = 200, description = "Get workflow by ID", body = ApiResponse<Workflow>)
    ),
    tag = "Workflows"
)]
#[get("/{id}")]
pub async fn get_workflow(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid workflow ID format".to_string()))?;

    let workflow = sqlx::query_as::<_, Workflow>(
        r#"
        SELECT 
            id, name, description, entity_type, status, steps, version, 
            created_at, updated_at
        FROM workflow_definitions
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool.get_ref())
    .await?
    .ok_or_else(|| AppError::NotFound("Workflow not found".to_string()))?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(workflow, "Workflow details retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows",
    responses(
        (status = 201, description = "Create workflow definition", body = ApiResponse<Workflow>)
    ),
    tag = "Workflows"
)]
#[post("")]
pub async fn create_workflow(
    pool: web::Data<PgPool>,
    payload: web::Json<CreateWorkflowRequest>,
) -> Result<impl Responder, AppError> {
    let workflow = sqlx::query_as::<_, Workflow>(
        r#"
        INSERT INTO workflow_definitions (name, description, entity_type, steps, status, version)
        VALUES ($1, $2, $3, $4, 'Active', 1)
        RETURNING id, name, description, entity_type, status, steps, version, created_at, updated_at
        "#
    )
    .bind(&payload.name)
    .bind(&payload.description)
    .bind(&payload.entity_type)
    .bind(&payload.steps)
    .fetch_one(pool.get_ref())
    .await?;

    Ok(HttpResponse::Created().json(ApiResponse::success(workflow, "Workflow created successfully")))
}

#[utoipa::path(
    get,
    path = "/api/v1/workflows/instances",
    responses(
        (status = 200, description = "Get active workflow instances", body = ApiResponse<Vec<WorkflowInstance>>)
    ),
    tag = "Workflows"
)]
#[get("/instances")]
pub async fn get_instances(
    pool: web::Data<PgPool>,
) -> Result<impl Responder, AppError> {
    let instances = sqlx::query_as::<_, WorkflowInstance>(
        r#"
        SELECT 
            id, workflow_id, entity_type, entity_id, status, current_step, 
            started_at, completed_at, created_at, updated_at
        FROM workflow_instances
        ORDER BY started_at DESC
        "#,
    )
    .fetch_all(pool.get_ref())
    .await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(instances, "Workflow instances retrieved successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows/{id}/start",
    responses(
        (status = 200, description = "Start workflow instance", body = ApiResponse<WorkflowInstance>)
    ),
    tag = "Workflows"
)]
#[post("/{id}/start")]
pub async fn start_workflow(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<StartWorkflowRequest>,
) -> Result<impl Responder, AppError> {
    let workflow_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid workflow ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    // 1. Fetch workflow definition
    let workflow = sqlx::query_as::<_, Workflow>(
        "SELECT * FROM workflow_definitions WHERE id = $1"
    )
    .bind(workflow_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Workflow definition not found".to_string()))?;

    // 2. Create instance
    let instance = sqlx::query_as::<_, WorkflowInstance>(
        r#"
        INSERT INTO workflow_instances (workflow_id, entity_type, entity_id, status, current_step)
        VALUES ($1, $2, $3, 'Active', 1)
        RETURNING id, workflow_id, entity_type, entity_id, status, current_step, started_at, completed_at, created_at, updated_at
        "#
    )
    .bind(workflow_id)
    .bind(&payload.entity_type)
    .bind(payload.entity_id)
    .fetch_one(&mut *tx)
    .await?;

    // 3. Extract first step from jsonb array if available
    let mut step_name = "Initial Verification".to_string();
    let mut assigned_to = "Loan Officer".to_string();
    let priority = "Medium".to_string();

    if let Some(steps_array) = workflow.steps.as_array() {
        if !steps_array.is_empty() {
            let first_step = &steps_array[0];
            if let Some(name) = first_step.get("name").and_then(|v| v.as_str()) {
                step_name = name.to_string();
            }
            if let Some(role) = first_step.get("assignedRole").and_then(|v| v.as_str()) {
                assigned_to = role.to_string();
            }
        }
    }

    // 4. Create first task
    let due_date = Utc::now() + chrono::Duration::hours(24);
    sqlx::query(
        r#"
        INSERT INTO workflow_tasks (instance_id, step_number, step_name, assigned_to, priority, status, due_date)
        VALUES ($1, 1, $2, $3, $4, 'Pending', $5)
        "#
    )
    .bind(instance.id)
    .bind(step_name)
    .bind(assigned_to)
    .bind(priority)
    .bind(due_date)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::success(instance, "Workflow instance started successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows/tasks/{id}/complete",
    responses(
        (status = 200, description = "Complete task", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Workflows"
)]
#[post("/tasks/{id}/complete")]
pub async fn complete_task(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<CompleteTaskRequest>,
) -> Result<impl Responder, AppError> {
    let task_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid task ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    // 1. Fetch task
    let task = sqlx::query_as::<_, Task>(
        "SELECT * FROM workflow_tasks WHERE id = $1 FOR UPDATE"
    )
    .bind(task_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    if task.status != "Pending" && task.status != "InProgress" {
        return Err(AppError::BadRequest("Task is already resolved".to_string()));
    }

    // 2. Complete task
    sqlx::query(
        "UPDATE workflow_tasks SET status = 'Completed', completed_at = NOW(), comments = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(&payload.comments)
    .bind(task_id)
    .execute(&mut *tx)
    .await?;

    // 3. Fetch workflow instance
    let instance = sqlx::query_as::<_, WorkflowInstance>(
        "SELECT * FROM workflow_instances WHERE id = $1 FOR UPDATE"
    )
    .bind(task.instance_id)
    .fetch_one(&mut *tx)
    .await?;

    // 4. Fetch workflow definition steps
    let workflow = sqlx::query_as::<_, Workflow>(
        "SELECT * FROM workflow_definitions WHERE id = $1"
    )
    .bind(instance.workflow_id)
    .fetch_one(&mut *tx)
    .await?;

    let mut next_step_opt = None;
    if let Some(steps_array) = workflow.steps.as_array() {
        let current_index = (task.step_number - 1) as usize;
        if current_index + 1 < steps_array.len() {
            next_step_opt = Some(&steps_array[current_index + 1]);
        }
    }

    if let Some(next_step) = next_step_opt {
        let next_step_number = task.step_number + 1;
        let mut next_step_name = format!("Step {}", next_step_number);
        let mut assigned_to = "Manager".to_string();

        if let Some(name) = next_step.get("name").and_then(|v| v.as_str()) {
            next_step_name = name.to_string();
        }
        if let Some(role) = next_step.get("assignedRole").and_then(|v| v.as_str()) {
            assigned_to = role.to_string();
        }

        // Update instance current step
        sqlx::query(
            "UPDATE workflow_instances SET current_step = $1, updated_at = NOW() WHERE id = $2"
        )
        .bind(next_step_number)
        .bind(instance.id)
        .execute(&mut *tx)
        .await?;

        // Create next task
        let due_date = Utc::now() + chrono::Duration::hours(24);
        sqlx::query(
            r#"
            INSERT INTO workflow_tasks (instance_id, step_number, step_name, assigned_to, priority, status, due_date)
            VALUES ($1, $2, $3, $4, 'Medium', 'Pending', $5)
            "#
        )
        .bind(instance.id)
        .bind(next_step_number)
        .bind(next_step_name)
        .bind(assigned_to)
        .bind(due_date)
        .execute(&mut *tx)
        .await?;
    } else {
        // Complete the entire workflow
        sqlx::query(
            "UPDATE workflow_instances SET status = 'Completed', completed_at = NOW(), updated_at = NOW() WHERE id = $1"
        )
        .bind(instance.id)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Task completed successfully")))
}

#[utoipa::path(
    post,
    path = "/api/v1/workflows/tasks/{id}/reject",
    responses(
        (status = 200, description = "Reject task", body = ApiResponse<serde_json::Value>)
    ),
    tag = "Workflows"
)]
#[post("/tasks/{id}/reject")]
pub async fn reject_task(
    pool: web::Data<PgPool>,
    path: web::Path<String>,
    payload: web::Json<RejectTaskRequest>,
) -> Result<impl Responder, AppError> {
    let task_id = Uuid::parse_str(&path.into_inner())
        .map_err(|_| AppError::BadRequest("Invalid task ID format".to_string()))?;

    let mut tx = pool.begin().await?;

    // 1. Fetch task
    let task = sqlx::query_as::<_, Task>(
        "SELECT * FROM workflow_tasks WHERE id = $1 FOR UPDATE"
    )
    .bind(task_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("Task not found".to_string()))?;

    if task.status != "Pending" && task.status != "InProgress" {
        return Err(AppError::BadRequest("Task is already resolved".to_string()));
    }

    // 2. Reject task
    sqlx::query(
        "UPDATE workflow_tasks SET status = 'Rejected', completed_at = NOW(), comments = $1, updated_at = NOW() WHERE id = $2"
    )
    .bind(&payload.comments)
    .bind(task_id)
    .execute(&mut *tx)
    .await?;

    // 3. Reject instance
    sqlx::query(
        "UPDATE workflow_instances SET status = 'Cancelled', completed_at = NOW(), updated_at = NOW() WHERE id = $1"
    )
    .bind(task.instance_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(HttpResponse::Ok().json(ApiResponse::<()>::message("Task rejected successfully")))
}
