use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::Response,
};
use sqlx::{FromRow, SqlitePool};
use std::{
    collections::HashSet,
    io::{Cursor, Write},
};
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

#[derive(Clone, Copy)]
enum ExportScope<'a> {
    Feed(&'a str),
    Year(&'a str, i64),
    Month(&'a str),
}

#[derive(FromRow)]
struct ExportEntry {
    filename: String,
    extracted_text: String,
    sanitized_html: String,
    year: i64,
    month: i64,
}

async fn entries_for_scope(
    pool: &SqlitePool,
    scope: ExportScope<'_>,
) -> Result<Vec<ExportEntry>, sqlx::Error> {
    let query = "SELECT sd.filename, cc.extracted_text, cc.sanitized_html, fy.year, fm.month \
        FROM source_documents sd \
        JOIN cached_content cc ON cc.document_id = sd.id \
        JOIN feed_months fm ON fm.id = sd.month_id \
        JOIN feed_years fy ON fy.feed_id = fm.feed_id AND fy.year = fm.year \
        JOIN feeds f ON f.id = fm.feed_id \
        WHERE f.active = 1 AND fy.active = 1 AND fm.active = 1 \
          AND sd.extraction_status = 'success' \
          AND CASE COALESCE(sd.promotional_override, sd.promotional_classification) \
              WHEN 'include' THEN 'publish' WHEN 'exclude' THEN 'promotional' \
              ELSE COALESCE(sd.promotional_override, sd.promotional_classification) END = 'publish' \
          AND (?1 IS NULL OR f.id = ?1) \
          AND (?2 IS NULL OR fy.year = ?2) \
          AND (?3 IS NULL OR fm.id = ?3) \
        ORDER BY f.display_name, fy.year, fm.month, sd.filename, sd.source_path";
    let (feed_id, year, month_id) = match scope {
        ExportScope::Feed(feed_id) => (Some(feed_id), None, None),
        ExportScope::Year(feed_id, year) => (Some(feed_id), Some(year), None),
        ExportScope::Month(month_id) => (None, None, Some(month_id)),
    };
    sqlx::query_as::<_, ExportEntry>(query)
        .bind(feed_id)
        .bind(year)
        .bind(month_id)
        .fetch_all(pool)
        .await
}

pub async fn export_feed(
    Path(feed_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Response, StatusCode> {
    export_scope(&pool, ExportScope::Feed(&feed_id)).await
}

pub async fn export_year(
    Path((feed_id, year)): Path<(String, i64)>,
    State(pool): State<SqlitePool>,
) -> Result<Response, StatusCode> {
    export_scope(&pool, ExportScope::Year(&feed_id, year)).await
}

pub async fn export_month(
    Path(month_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Response, StatusCode> {
    export_scope(&pool, ExportScope::Month(&month_id)).await
}

async fn export_scope(pool: &SqlitePool, scope: ExportScope<'_>) -> Result<Response, StatusCode> {
    let (feed_name, year, month) = match scope {
        ExportScope::Feed(feed_id) => sqlx::query_scalar::<_, String>(
            "SELECT display_name FROM feeds WHERE id = ? AND active = 1",
        )
        .bind(feed_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|name| (name, None, None))
        .ok_or(StatusCode::NOT_FOUND)?,
        ExportScope::Year(feed_id, year) => sqlx::query_as::<_, (String, i64)>(
            "SELECT f.display_name, fy.year FROM feeds f JOIN feed_years fy ON fy.feed_id = f.id \
             WHERE f.id = ? AND fy.year = ? AND f.active = 1 AND fy.active = 1",
        )
        .bind(feed_id)
        .bind(year)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|(name, year)| (name, Some(year), None))
        .ok_or(StatusCode::NOT_FOUND)?,
        ExportScope::Month(month_id) => sqlx::query_as::<_, (String, i64, i64)>(
            "SELECT f.display_name, fy.year, fm.month FROM feed_months fm \
             JOIN feed_years fy ON fy.feed_id = fm.feed_id AND fy.year = fm.year \
             JOIN feeds f ON f.id = fm.feed_id \
             WHERE fm.id = ? AND fm.active = 1 AND fy.active = 1 AND f.active = 1",
        )
        .bind(month_id)
        .fetch_optional(pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map(|(name, year, month)| (name, Some(year), Some(month)))
        .ok_or(StatusCode::NOT_FOUND)?,
    };

    let entries = entries_for_scope(pool, scope)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let mut writer = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Deflated);
    let feed_folder = safe_component(&feed_name);
    let mut used_paths = HashSet::new();
    for entry in entries {
        let entry_year = entry.year;
        let entry_month = entry.month;
        let folder = format!(
            "{}/{}/{}/",
            feed_folder,
            entry_year,
            month_folder(entry_month)
        );
        let filename = unique_markdown_filename(&entry.filename, &folder, &mut used_paths);
        let path = format!("{folder}{filename}");
        writer
            .start_file(path, options)
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let title = markdown_title(&entry.filename)
            .replace('\\', "\\\\")
            .replace('#', "\\#")
            .replace('[', "\\[")
            .replace(']', "\\]");
        let body = if entry.sanitized_html.trim().is_empty() {
            entry.extracted_text
        } else {
            crate::pdf_processing::html_to_markdown(&entry.sanitized_html)
        };
        write!(writer, "# {title}\n\n{body}\n").map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    let archive = writer
        .finish()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .into_inner();

    let download_name = match (year, month) {
        (None, _) => format!("{}.zip", download_component(&feed_name)),
        (Some(year), None) => format!("{}-{year}.zip", download_component(&feed_name)),
        (Some(year), Some(month)) => {
            format!("{}-{year}-{:02}.zip", download_component(&feed_name), month)
        }
    };
    let disposition = format!("attachment; filename=\"{download_name}\"");
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/zip")
        .header(header::CONTENT_DISPOSITION, disposition)
        .header(header::CACHE_CONTROL, "no-store")
        .body(archive.into())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn safe_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|character| {
            if character.is_alphanumeric() || matches!(character, '-' | '_' | '.') {
                character
            } else if character.is_whitespace() {
                '-'
            } else {
                '_'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches(['.', '-']).to_owned();
    if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
        "entry".to_owned()
    } else {
        sanitized
    }
}

fn download_component(value: &str) -> String {
    let sanitized: String = value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '-'
            }
        })
        .collect();
    let sanitized = sanitized.trim_matches('-');
    if sanitized.is_empty() {
        "kpapers".to_owned()
    } else {
        sanitized.to_owned()
    }
}

fn markdown_title(filename: &str) -> String {
    let basename = filename.rsplit(['/', '\\']).next().unwrap_or(filename);
    basename
        .strip_suffix(".pdf")
        .or_else(|| basename.strip_suffix(".PDF"))
        .unwrap_or(basename)
        .replace(['\r', '\n'], " ")
}

fn month_folder(month: i64) -> String {
    format!("{month:02}")
}

fn unique_markdown_filename(filename: &str, folder: &str, used: &mut HashSet<String>) -> String {
    let title = safe_component(&markdown_title(filename));
    let base = format!("{title}.md");
    if used.insert(format!("{folder}{base}")) {
        return base;
    }
    let mut suffix = 2;
    loop {
        let candidate = format!("{title}-{suffix}.md");
        if used.insert(format!("{folder}{candidate}")) {
            return candidate;
        }
        suffix += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    async fn insert_entry(
        pool: &SqlitePool,
        id: &str,
        month_id: &str,
        filename: &str,
        classification: &str,
        override_value: Option<&str>,
        extraction_status: &str,
    ) {
        sqlx::query("INSERT INTO source_documents (id, month_id, source_path, filename, source_url, content_fingerprint, size_bytes, promotional_classification, promotional_override, extraction_status) VALUES (?, ?, ?, ?, 'https://example.test/source.pdf', ?, 1, ?, ?, ?)")
            .bind(id)
            .bind(month_id)
            .bind(format!("/{id}.pdf"))
            .bind(filename)
            .bind(id)
            .bind(classification)
            .bind(override_value)
            .bind(extraction_status)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO cached_content (document_id, content_fingerprint, extracted_text, first_page_text, sanitized_html, page_count) VALUES (?, ?, ?, '', '', 1)")
            .bind(id)
            .bind(id)
            .bind(format!("text for {filename}"))
            .execute(pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn scoped_entry_query_matches_publication_readiness_and_promotion_overrides() {
        let (pool, path) = crate::test_support::database().await;
        sqlx::query(
            "INSERT INTO feeds (id, slug, display_name) VALUES ('feed', 'feed', 'Ada Feed')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO feed_years (feed_id, year) VALUES ('feed', 2025), ('feed', 2026)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url) VALUES ('m1', 'feed', 2026, 1, 'yandex_disk', 'https://example.test/1'), ('m2', 'feed', 2026, 2, 'yandex_disk', 'https://example.test/2'), ('m3', 'feed', 2025, 3, 'yandex_disk', 'https://example.test/3')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_years (feed_id, year, active) VALUES ('feed', 2024, 0)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url, active) VALUES ('archived-month', 'feed', 2026, 4, 'yandex_disk', 'https://example.test/4', 0), ('archived-year-month', 'feed', 2024, 5, 'yandex_disk', 'https://example.test/5', 1)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feeds (id, slug, display_name, active) VALUES ('archived-feed', 'archived-feed', 'Archived Feed', 0)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_years (feed_id, year) VALUES ('archived-feed', 2026)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url) VALUES ('archived-feed-month', 'archived-feed', 2026, 6, 'yandex_disk', 'https://example.test/6')")
            .execute(&pool)
            .await
            .unwrap();

        insert_entry(
            &pool,
            "ready",
            "m1",
            "ready.pdf",
            "publish",
            None,
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "included",
            "m1",
            "included.pdf",
            "promotional",
            Some("include"),
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "excluded",
            "m1",
            "excluded.pdf",
            "publish",
            Some("exclude"),
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "promotional",
            "m1",
            "promotional.pdf",
            "promotional",
            None,
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "failed",
            "m1",
            "failed.pdf",
            "publish",
            None,
            "failure",
        )
        .await;
        insert_entry(
            &pool,
            "pending",
            "m1",
            "pending.pdf",
            "publish",
            None,
            "pending",
        )
        .await;
        insert_entry(
            &pool,
            "other-month",
            "m2",
            "month-two.pdf",
            "publish",
            None,
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "other-year",
            "m3",
            "year-2025.pdf",
            "publish",
            None,
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "archived-month-doc",
            "archived-month",
            "archived-month.pdf",
            "publish",
            None,
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "archived-year-doc",
            "archived-year-month",
            "archived-year.pdf",
            "publish",
            None,
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "archived-feed-doc",
            "archived-feed-month",
            "archived-feed.pdf",
            "publish",
            None,
            "success",
        )
        .await;

        let feed = entries_for_scope(&pool, ExportScope::Feed("feed"))
            .await
            .unwrap();
        let filenames: HashSet<_> = feed.iter().map(|entry| entry.filename.as_str()).collect();
        assert_eq!(
            filenames,
            HashSet::from([
                "ready.pdf",
                "included.pdf",
                "month-two.pdf",
                "year-2025.pdf"
            ])
        );

        let year = entries_for_scope(&pool, ExportScope::Year("feed", 2026))
            .await
            .unwrap();
        let filenames: HashSet<_> = year.iter().map(|entry| entry.filename.as_str()).collect();
        assert_eq!(
            filenames,
            HashSet::from(["ready.pdf", "included.pdf", "month-two.pdf"])
        );

        let month = entries_for_scope(&pool, ExportScope::Month("m1"))
            .await
            .unwrap();
        let filenames: HashSet<_> = month.iter().map(|entry| entry.filename.as_str()).collect();
        assert_eq!(filenames, HashSet::from(["ready.pdf", "included.pdf"]));
        crate::test_support::cleanup(pool, path).await;
    }

    #[test]
    fn archive_components_and_markdown_names_are_safe_and_unique() {
        assert_eq!(safe_component("../Ada / Feed"), "_Ada-_-Feed");
        assert_eq!(markdown_title(r"nested\\A #1.pdf"), "A #1");
        let mut used = HashSet::new();
        assert_eq!(
            unique_markdown_filename("entry.pdf", "feed/2026/01/", &mut used),
            "entry.md"
        );
        assert_eq!(
            unique_markdown_filename("entry.pdf", "feed/2026/01/", &mut used),
            "entry-2.md"
        );
        assert_eq!(
            unique_markdown_filename("entry.pdf", "feed/2026/02/", &mut used),
            "entry.md"
        );
    }

    #[tokio::test]
    async fn export_builds_preserved_markdown_paths_and_valid_empty_archives() {
        use axum::body::to_bytes;
        use std::io::Read;

        let (pool, path) = crate::test_support::database().await;
        sqlx::query(
            "INSERT INTO feeds (id, slug, display_name) VALUES ('feed', 'feed', 'Ada Feed')",
        )
        .execute(&pool)
        .await
        .unwrap();
        sqlx::query("INSERT INTO feed_years (feed_id, year) VALUES ('feed', 2026)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_years (feed_id, year) VALUES ('feed', 2025)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url) VALUES ('m1', 'feed', 2026, 1, 'yandex_disk', 'https://example.test/1'), ('m2', 'feed', 2026, 2, 'yandex_disk', 'https://example.test/2'), ('empty', 'feed', 2026, 3, 'yandex_disk', 'https://example.test/3')")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url) VALUES ('other-year', 'feed', 2025, 4, 'yandex_disk', 'https://example.test/4')")
            .execute(&pool)
            .await
            .unwrap();
        insert_entry(
            &pool,
            "doc",
            "m1",
            "nested/entry.pdf",
            "publish",
            None,
            "success",
        )
        .await;
        let inline_html = "<p>посвоему, the teh</p>\n<p>Before image</p>\n<img src=\"data:image/jpeg;base64,AQID\" alt=\"\" width=\"3\" height=\"2\" style=\"max-width:100%;height:auto\">\n<p>After image</p>";
        sqlx::query("UPDATE cached_content SET sanitized_html=? WHERE document_id='doc'")
            .bind(inline_html)
            .execute(&pool)
            .await
            .unwrap();
        insert_entry(
            &pool,
            "duplicate",
            "m1",
            "entry.pdf",
            "publish",
            None,
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "second-month",
            "m2",
            "second.pdf",
            "publish",
            None,
            "success",
        )
        .await;
        insert_entry(
            &pool,
            "other-year",
            "other-year",
            "older-entry.pdf",
            "publish",
            None,
            "success",
        )
        .await;

        let response = export_scope(&pool, ExportScope::Feed("feed"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(response.headers()[header::CONTENT_TYPE], "application/zip");
        assert_eq!(
            response.headers()[header::CONTENT_DISPOSITION],
            "attachment; filename=\"Ada-Feed.zip\""
        );
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let mut archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 4);
        assert!(
            archive
                .file_names()
                .any(|name| name.unwrap() == "Ada-Feed/2026/01/entry.md")
        );
        assert!(
            archive
                .file_names()
                .any(|name| name.unwrap() == "Ada-Feed/2026/01/entry-2.md")
        );
        assert!(
            archive
                .file_names()
                .any(|name| name.unwrap() == "Ada-Feed/2026/02/second.md")
        );
        assert!(
            archive
                .file_names()
                .any(|name| name.unwrap() == "Ada-Feed/2025/04/older-entry.md")
        );
        let mut markdown = String::new();
        archive
            .by_name("Ada-Feed/2026/01/entry-2.md")
            .unwrap()
            .read_to_string(&mut markdown)
            .unwrap();
        assert_eq!(
            markdown,
            "# entry\n\nпосвоему, the teh\n\nBefore image\n\n<img src=\"data:image/jpeg;base64,AQID\" alt=\"\" width=\"3\" height=\"2\" style=\"max-width:100%;height:auto\">\n\nAfter image\n"
        );
        assert!(!markdown.contains("[исправлено:"));
        assert!(!markdown.contains("[corrected from:"));

        let response = export_scope(&pool, ExportScope::Year("feed", 2026))
            .await
            .unwrap();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 3);
        assert!(
            archive
                .file_names()
                .all(|name| name.unwrap().starts_with("Ada-Feed/2026/"))
        );

        let response = export_scope(&pool, ExportScope::Month("m1")).await.unwrap();
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 2);
        assert!(
            archive
                .file_names()
                .all(|name| name.unwrap().starts_with("Ada-Feed/2026/01/"))
        );

        let response = export_scope(&pool, ExportScope::Month("empty"))
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let archive = zip::ZipArchive::new(Cursor::new(bytes)).unwrap();
        assert_eq!(archive.len(), 0);
        crate::test_support::cleanup(pool, path).await;
    }
}
