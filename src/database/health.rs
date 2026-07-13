use crate::database::pool::DbPool;

pub async fn check_database_health(pool: &DbPool) -> Result<(), sqlx::Error> {
    sqlx::query("SELECT 1").execute(pool).await?;
    Ok(())
}
