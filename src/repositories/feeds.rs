use sqlx::SqlitePool;

#[derive(Clone)]
pub struct FeedRepository {
    _pool: SqlitePool,
}

impl FeedRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { _pool: pool }
    }
}
