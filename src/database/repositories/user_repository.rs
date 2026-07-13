use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::User;
use crate::database::pool::DbPool;

#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error>;
    async fn find_by_stellar_public_key(&self, pk: &str) -> Result<Option<User>, sqlx::Error>;
    async fn create(&self, user: &User) -> Result<User, sqlx::Error>;
    async fn update(&self, user: &User) -> Result<User, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn list_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, sqlx::Error>;
}

pub struct UserRepositoryImpl {
    pool: DbPool,
}

impl UserRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for UserRepositoryImpl {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_stellar_public_key(&self, pk: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT * FROM users WHERE stellar_public_key = $1 AND deleted_at IS NULL",
        )
        .bind(pk)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, user: &User) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "INSERT INTO users (id, email, password_hash, display_name, avatar_url, stellar_public_key, role, last_login_at, is_active, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11) RETURNING *",
        )
        .bind(user.id)
        .bind(&user.email)
        .bind(&user.password_hash)
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(&user.stellar_public_key)
        .bind(&user.role)
        .bind(user.last_login_at)
        .bind(user.is_active)
        .bind(user.created_at)
        .bind(user.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(&self, user: &User) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "UPDATE users SET email = $1, display_name = $2, avatar_url = $3, stellar_public_key = $4, role = $5, last_login_at = $6, is_active = $7, updated_at = NOW() WHERE id = $8 AND deleted_at IS NULL RETURNING *",
        )
        .bind(&user.email)
        .bind(&user.display_name)
        .bind(&user.avatar_url)
        .bind(&user.stellar_public_key)
        .bind(&user.role)
        .bind(user.last_login_at)
        .bind(user.is_active)
        .bind(user.id)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_by_organization(&self, org_id: Uuid) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>(
            "SELECT u.* FROM users u JOIN organization_members m ON u.id = m.user_id WHERE m.organization_id = $1 AND u.deleted_at IS NULL",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
    }
}
