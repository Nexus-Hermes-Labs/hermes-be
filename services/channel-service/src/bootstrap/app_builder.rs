// src/bootstrap/app_builder.rs
use anyhow::Result;
use sqlx::PgPool;

/// Application builder - handles dependency composition
pub struct AppBuilder {
    _db_pool: Option<PgPool>,
}

impl AppBuilder {
    pub fn new() -> Self {
        Self {
            _db_pool: None,
        }
    }

    pub fn with_database(mut self, pool: PgPool) -> Self {
        self._db_pool = Some(pool);
        self
    }

    /// Build the complete application with all dependencies
    pub async fn build(
        self,
    ) -> Result<()> {
        Ok(())
    }
}
