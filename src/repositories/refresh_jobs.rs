use sqlx::SqlitePool;

#[derive(Clone)]
pub struct RefreshJobRepository {
    _pool: SqlitePool,
}

impl RefreshJobRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { _pool: pool }
    }
}
