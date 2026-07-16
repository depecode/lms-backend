use sqlx::PgPool;
use uuid::Uuid;
use serde_json::Value;

pub async fn create_audit_log(
    pool: &PgPool,
    user_id: Option<Uuid>,
    action: &str,
    entity_type: Option<&str>,
    entity_id: Option<Uuid>,
    old_values: Option<Value>,
    new_values: Option<Value>,
    ip_address: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO audit_logs (user_id, action, entity_type, entity_id, old_values, new_values, ip_address)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        "#
    )
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_id)
    .bind(old_values)
    .bind(new_values)
    .bind(ip_address)
    .execute(pool)
    .await?;

    Ok(())
}
