use async_trait::async_trait;

/// Generic repository trait for common CRUD operations
#[async_trait]
pub trait Repository<T, ID> {
    type Error: std::error::Error + Send + Sync + 'static;

    async fn find_by_id(&self, id: ID) -> Result<Option<T>, Self::Error>;
    async fn find_all(&self) -> Result<Vec<T>, Self::Error>;
    async fn save(&self, entity: &T) -> Result<(), Self::Error>;
    async fn update(&self, entity: &T) -> Result<(), Self::Error>;
    async fn delete(&self, id: ID) -> Result<(), Self::Error>;
    async fn exists(&self, id: ID) -> Result<bool, Self::Error>;
    async fn count(&self) -> Result<i64, Self::Error>;
}
