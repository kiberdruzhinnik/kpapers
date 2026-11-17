use sqlx::{SqlitePool, sqlite::SqliteConnectOptions};
use std::str::FromStr;

pub async fn connect_and_migrate(database_url: &str) -> Result<SqlitePool, sqlx::Error> {
    let options = SqliteConnectOptions::from_str(database_url)?
        .create_if_missing(true)
        .foreign_keys(true);
    let pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(options)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;
    Ok(pool)
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn proxy_migration_preserves_existing_feeds_and_defaults_to_inherit() {
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(1)
            .connect("sqlite::memory:")
            .await
            .unwrap();

        for migration in [
            include_str!("../migrations/0001_initial.sql"),
            include_str!("../migrations/0002_archive_hierarchy.sql"),
            include_str!("../migrations/0003_feed_extraction_engine.sql"),
            include_str!("../migrations/0004_remove_feed_extraction_engine.sql"),
        ] {
            sqlx::raw_sql(migration).execute(&pool).await.unwrap();
        }

        sqlx::query("INSERT INTO feeds (id, slug, display_name, description) VALUES ('existing', 'existing', 'Existing feed', 'kept')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::raw_sql(include_str!("../migrations/0005_proxy_settings.sql"))
            .execute(&pool)
            .await
            .unwrap();

        let feed = sqlx::query_as::<_, (String, String, String, String, Option<String>)>(
            "SELECT id, display_name, description, proxy_mode, proxy_url FROM feeds WHERE id='existing'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(
            feed,
            (
                "existing".into(),
                "Existing feed".into(),
                "kept".into(),
                "inherit".into(),
                None
            )
        );
        let settings_table_exists: bool = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='application_settings')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(settings_table_exists);
    }
}
