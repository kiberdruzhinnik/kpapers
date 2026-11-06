use sqlx::SqlitePool;

#[derive(Clone)]
pub struct CacheRepository {
    _pool: SqlitePool,
}

impl CacheRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { _pool: pool }
    }
}
