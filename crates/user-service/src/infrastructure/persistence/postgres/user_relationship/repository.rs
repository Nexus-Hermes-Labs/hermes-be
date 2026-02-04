use sqlx::PgPool;

pub struct PostgresUserRelationshipRepository {
    pool: PgPool,
}

impl PostgresUserRelationshipRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}