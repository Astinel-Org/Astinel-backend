use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::Organization;
use crate::database::pool::DbPool;

#[async_trait]
pub trait OrganizationRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>, sqlx::Error>;
    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, sqlx::Error>;
    async fn create(&self, org: &Organization) -> Result<Organization, sqlx::Error>;
    async fn update(&self, org: &Organization) -> Result<Organization, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Organization>, sqlx::Error>;
}

pub struct OrganizationRepositoryImpl {
    pool: DbPool,
}

impl OrganizationRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl OrganizationRepository for OrganizationRepositoryImpl {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            "SELECT * FROM organizations WHERE id = $1 AND deleted_at IS NULL",
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_slug(&self, slug: &str) -> Result<Option<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            "SELECT * FROM organizations WHERE slug = $1 AND deleted_at IS NULL",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await
    }

    async fn create(&self, org: &Organization) -> Result<Organization, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            "INSERT INTO organizations (id, name, slug, description, owner_user_id, billing_plan, is_active, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10) RETURNING *",
        )
        .bind(org.id)
        .bind(&org.name)
        .bind(&org.slug)
        .bind(&org.description)
        .bind(org.owner_user_id)
        .bind(&org.billing_plan)
        .bind(org.is_active)
        .bind(&org.settings)
        .bind(org.created_at)
        .bind(org.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(&self, org: &Organization) -> Result<Organization, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            "UPDATE organizations SET name = $1, slug = $2, description = $3, billing_plan = $4, is_active = $5, settings = $6, updated_at = NOW() WHERE id = $7 AND deleted_at IS NULL RETURNING *",
        )
        .bind(&org.name)
        .bind(&org.slug)
        .bind(&org.description)
        .bind(&org.billing_plan)
        .bind(org.is_active)
        .bind(&org.settings)
        .bind(org.id)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE organizations SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn list_for_user(&self, user_id: Uuid) -> Result<Vec<Organization>, sqlx::Error> {
        sqlx::query_as::<_, Organization>(
            "SELECT o.* FROM organizations o JOIN organization_members m ON o.id = m.organization_id WHERE m.user_id = $1 AND o.deleted_at IS NULL",
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
    }
}
