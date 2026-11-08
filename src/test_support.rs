use sqlx::SqlitePool;
use std::{fs, path::PathBuf};

pub async fn database() -> (SqlitePool, PathBuf) {
    let path = std::env::temp_dir().join(format!("kpapers-test-{}.sqlite", uuid::Uuid::new_v4()));
    let url = format!("sqlite://{}?mode=rwc", path.display());
    let pool = crate::db::connect_and_migrate(&url)
        .await
        .expect("create isolated test database");
    (pool, path)
}

pub async fn cleanup(pool: SqlitePool, path: PathBuf) {
    pool.close().await;
    for suffix in ["", "-wal", "-shm"] {
        let mut file = path.as_os_str().to_os_string();
        file.push(suffix);
        let _ = fs::remove_file(PathBuf::from(file));
    }
}
