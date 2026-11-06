use sqlx::SqlitePool;

#[derive(Clone)]
pub struct DocumentRepository {
    _pool: SqlitePool,
}

impl DocumentRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { _pool: pool }
    }
}
