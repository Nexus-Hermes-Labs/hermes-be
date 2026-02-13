use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;
use common::infrastructure::persistence::error::RepositoryError;
use crate::domain::guild::Guild;
use crate::domain::guild_repository::GuildRepository;

pub struct PostgresGuildRepository {
    pool: PgPool,
}

impl PostgresGuildRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl GuildRepository for PostgresGuildRepository {}

#[async_trait]
impl common::infrastructure::persistence::repository::Repository<Guild, Uuid> for PostgresGuildRepository {
    type Error = RepositoryError;

    async fn find_by_id(&self, _id: Uuid) -> Result<Option<Guild>, Self::Error> {
        unimplemented!()
    }

    async fn find_all(&self) -> Result<Vec<Guild>, Self::Error> {
        unimplemented!()
    }

    async fn save(&self, _entity: &Guild) -> Result<(), Self::Error> {
        unimplemented!()
    }

    async fn update(&self, _entity: &Guild) -> Result<(), Self::Error> {
        unimplemented!()
    }

    async fn delete(&self, _id: Uuid) -> Result<(), Self::Error> {
        unimplemented!()
    }

    async fn exists(&self, _id: Uuid) -> Result<bool, Self::Error> {
        unimplemented!()
    }

    async fn count(&self) -> Result<i64, Self::Error> {
        unimplemented!()
    }
}
