use crate::models::User;
use anyhow::Result;
use sqlx::MySqlPool;
use uuid::Uuid;

pub async fn create_user(
    pool: &MySqlPool,
    email: &str,
    username: &str,
    password_hash: &str,
) -> Result<User> {
    let id = Uuid::new_v4();
    let id_str = id.to_string();

    sqlx::query!(
        r#"
        INSERT INTO users (id, email, username, password_hash, created_at, updated_at)
        VALUES (?, ?, ?, ?, NOW(), NOW())
        "#,
        id_str,
        email,
        username,
        password_hash
    )
    .execute(pool)
    .await?;

    let user = find_user_by_id(pool, id)
        .await?
        .ok_or_else(|| anyhow::anyhow!("User not found after insert"))?;

    Ok(user)
}

pub async fn find_user_by_email(pool: &MySqlPool, email: &str) -> Result<Option<User>> {
    let row = sqlx::query!(
        "SELECT id, email, username, password_hash, created_at, updated_at FROM users WHERE email = ?",
        email
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        id: Uuid::parse_str(&r.id).unwrap_or_default(),
        email: r.email,
        username: r.username,
        password_hash: r.password_hash,
        created_at: r.created_at.and_utc(),
        updated_at: r.updated_at.and_utc(),
    }))
}

pub async fn find_user_by_id(pool: &MySqlPool, id: Uuid) -> Result<Option<User>> {
    let id_str = id.to_string();

    let row = sqlx::query!(
        "SELECT id, email, username, password_hash, created_at, updated_at FROM users WHERE id = ?",
        id_str
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        id: Uuid::parse_str(&r.id).unwrap_or_default(),
        email: r.email,
        username: r.username,
        password_hash: r.password_hash,
        created_at: r.created_at.and_utc(),
        updated_at: r.updated_at.and_utc(),
    }))
}

pub async fn list_users(pool: &MySqlPool, limit: i64, offset: i64) -> Result<Vec<User>> {
    let rows = sqlx::query!(
        "SELECT id, email, username, password_hash, created_at, updated_at FROM users ORDER BY created_at DESC LIMIT ? OFFSET ?",
        limit,
        offset
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| User {
            id: Uuid::parse_str(&r.id).unwrap_or_default(),
            email: r.email,
            username: r.username,
            password_hash: r.password_hash,
            created_at: r.created_at.and_utc(),
            updated_at: r.updated_at.and_utc(),
        })
        .collect())
}

pub async fn update_user(
    pool: &MySqlPool,
    id: Uuid,
    username: Option<&str>,
    email: Option<&str>,
) -> Result<Option<User>> {
    let id_str = id.to_string();

    let affected = sqlx::query!(
        r#"
        UPDATE users SET
            username   = COALESCE(?, username),
            email      = COALESCE(?, email),
            updated_at = NOW()
        WHERE id = ?
        "#,
        username,
        email,
        id_str
    )
    .execute(pool)
    .await?
    .rows_affected();

    if affected == 0 {
        return Ok(None);
    }

    find_user_by_id(pool, id).await
}

pub async fn delete_user(pool: &MySqlPool, id: Uuid) -> Result<bool> {
    let id_str = id.to_string();

    let result = sqlx::query!("DELETE FROM users WHERE id = ?", id_str)
        .execute(pool)
        .await?;

    Ok(result.rows_affected() > 0)
}
