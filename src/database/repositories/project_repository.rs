use async_trait::async_trait;
use uuid::Uuid;

use crate::database::models::Project;
use crate::database::pool::DbPool;

#[async_trait]
pub trait ProjectRepository: Send + Sync {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>, sqlx::Error>;
    async fn find_by_slug_and_org(
        &self,
        slug: &str,
        org_id: Uuid,
    ) -> Result<Option<Project>, sqlx::Error>;
    async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<Project>, sqlx::Error>;
    async fn create(&self, project: &Project) -> Result<Project, sqlx::Error>;
    async fn update(&self, project: &Project) -> Result<Project, sqlx::Error>;
    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error>;
}

pub struct ProjectRepositoryImpl {
    pool: DbPool,
}

impl ProjectRepositoryImpl {
    pub fn new(pool: DbPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProjectRepository for ProjectRepositoryImpl {
    async fn find_by_id(&self, id: Uuid) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>("SELECT * FROM projects WHERE id = $1 AND deleted_at IS NULL")
            .bind(id)
            .fetch_optional(&self.pool)
            .await
    }

    async fn find_by_slug_and_org(
        &self,
        slug: &str,
        org_id: Uuid,
    ) -> Result<Option<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE slug = $1 AND organization_id = $2 AND deleted_at IS NULL",
        )
        .bind(slug)
        .bind(org_id)
        .fetch_optional(&self.pool)
        .await
    }

    async fn find_by_organization(&self, org_id: Uuid) -> Result<Vec<Project>, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "SELECT * FROM projects WHERE organization_id = $1 AND deleted_at IS NULL ORDER BY created_at DESC",
        )
        .bind(org_id)
        .fetch_all(&self.pool)
        .await
    }

    async fn create(&self, project: &Project) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "INSERT INTO projects (id, organization_id, name, slug, description, repository_url, local_path, default_branch, language, settings, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12) RETURNING *",
        )
        .bind(project.id)
        .bind(project.organization_id)
        .bind(&project.name)
        .bind(&project.slug)
        .bind(&project.description)
        .bind(&project.repository_url)
        .bind(&project.local_path)
        .bind(&project.default_branch)
        .bind(&project.language)
        .bind(&project.settings)
        .bind(project.created_at)
        .bind(project.updated_at)
        .fetch_one(&self.pool)
        .await
    }

    async fn update(&self, project: &Project) -> Result<Project, sqlx::Error> {
        sqlx::query_as::<_, Project>(
            "UPDATE projects SET name = $1, slug = $2, description = $3, repository_url = $4, local_path = $5, default_branch = $6, language = $7, settings = $8, updated_at = NOW() WHERE id = $9 AND deleted_at IS NULL RETURNING *",
        )
        .bind(&project.name)
        .bind(&project.slug)
        .bind(&project.description)
        .bind(&project.repository_url)
        .bind(&project.local_path)
        .bind(&project.default_branch)
        .bind(&project.language)
        .bind(&project.settings)
        .bind(project.id)
        .fetch_one(&self.pool)
        .await
    }

    async fn delete(&self, id: Uuid) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE projects SET deleted_at = NOW() WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
