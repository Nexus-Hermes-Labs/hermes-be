/// A type alias for an active Postgres transaction.
///
/// Passed to repository methods that participate in a cross-aggregate unit of
/// work so that multiple operations across different repositories can be
/// committed or rolled back atomically.
pub type DbTransaction<'a> = sqlx::Transaction<'a, sqlx::Postgres>;
