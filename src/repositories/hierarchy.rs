use sqlx::SqlitePool;

#[derive(Clone)]
pub struct HierarchyRepository {
    _pool: SqlitePool,
}

impl HierarchyRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { _pool: pool }
    }
}
