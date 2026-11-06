mod cache;
mod documents;
mod feeds;
mod hierarchy;
mod refresh_jobs;

use sqlx::SqlitePool;

pub struct Repositories {
    pub feeds: feeds::FeedRepository,
    pub hierarchy: hierarchy::HierarchyRepository,
    pub documents: documents::DocumentRepository,
    pub cache: cache::CacheRepository,
    pub refresh_jobs: refresh_jobs::RefreshJobRepository,
}

impl Repositories {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            feeds: feeds::FeedRepository::new(pool.clone()),
            hierarchy: hierarchy::HierarchyRepository::new(pool.clone()),
            documents: documents::DocumentRepository::new(pool.clone()),
            cache: cache::CacheRepository::new(pool.clone()),
            refresh_jobs: refresh_jobs::RefreshJobRepository::new(pool),
        }
    }
}
