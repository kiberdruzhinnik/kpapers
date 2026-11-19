use axum::extract::{Path, Query, State};
use chrono::{DateTime, Datelike, Utc};
use sqlx::{FromRow, SqlitePool};
use std::time::Duration;

#[derive(FromRow)]
struct DueMonth {
    id: String,
    feed_id: String,
    year: i64,
    month: i64,
    override_seconds: Option<i64>,
    next_refresh_at: Option<String>,
}
#[derive(FromRow)]
struct Job {
    id: String,
    scope: String,
    feed_id: String,
    year: Option<i64>,
    month_id: Option<String>,
    force: i64,
}

pub async fn run(pool: SqlitePool) {
    let mut ticker = tokio::time::interval(Duration::from_secs(5));
    loop {
        ticker.tick().await;
        run_tick(&pool).await;
    }
}

async fn run_tick(pool: &SqlitePool) {
    if let Err(error) = schedule_due(pool).await {
        tracing::error!(
            event = "scheduler_enqueue_failed",
            operation = "enqueue_due_refreshes",
            error = %error,
            "scheduler enqueue failed"
        );
    }
    loop {
        match process_one(pool).await {
            Ok(true) => continue,
            Ok(false) => break,
            Err(error) => {
                tracing::error!(
                    event = "scheduler_job_failed",
                    operation = "process_refresh_job",
                    error = %error,
                    "scheduler job failed"
                );
                break;
            }
        }
    }
}

async fn schedule_due(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    schedule_due_at(
        pool,
        Utc::now(),
        env_seconds("CURRENT_MONTH_REFRESH_SECONDS", 3600),
    )
    .await
}

async fn schedule_due_at(
    pool: &SqlitePool,
    now: DateTime<Utc>,
    current_month_interval: i64,
) -> Result<(), sqlx::Error> {
    let months = sqlx::query_as::<_, DueMonth>("SELECT fm.id, fm.feed_id, fm.year, fm.month, f.refresh_interval_seconds AS override_seconds, fm.next_refresh_at FROM feed_months fm JOIN feeds f ON f.id=fm.feed_id JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year WHERE f.active=1 AND fy.active=1 AND fm.active=1")
        .fetch_all(pool).await?;
    for month in months {
        let current = month.year == now.year() as i64 && month.month == now.month() as i64;
        if !current {
            if month.next_refresh_at.is_some() {
                sqlx::query("UPDATE feed_months SET next_refresh_at=NULL WHERE id=?")
                    .bind(&month.id)
                    .execute(pool)
                    .await?;
            }
            continue;
        }
        let next_due = month
            .next_refresh_at
            .as_deref()
            .and_then(|v| DateTime::parse_from_rfc3339(v).ok())
            .map(|t| t.with_timezone(&Utc));
        if next_due.is_some_and(|due| due > now) {
            continue;
        }
        let has_pending: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM refresh_jobs WHERE month_id=? AND status IN ('queued','running'))")
            .bind(&month.id).fetch_one(pool).await?;
        if has_pending {
            continue;
        }
        let interval = month.override_seconds.unwrap_or(current_month_interval);
        let next = (now + chrono::Duration::seconds(interval)).to_rfc3339();
        sqlx::query("INSERT INTO refresh_jobs (id,scope,feed_id,year,month_id,force,status) VALUES (?, 'month', ?, ?, ?, 0, 'queued')")
            .bind(uuid::Uuid::new_v4().to_string()).bind(&month.feed_id).bind(month.year).bind(&month.id).execute(pool).await?;
        sqlx::query("UPDATE feed_months SET next_refresh_at=? WHERE id=?")
            .bind(next)
            .bind(month.id)
            .execute(pool)
            .await?;
    }
    Ok(())
}

async fn process_one(pool: &SqlitePool) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let Some(job) = sqlx::query_as::<_, Job>("SELECT id,scope,feed_id,year,month_id,force FROM refresh_jobs WHERE status='queued' ORDER BY requested_at LIMIT 1")
        .fetch_optional(pool).await? else { return Ok(false); };
    sqlx::query("UPDATE refresh_jobs SET status='running',started_at=CURRENT_TIMESTAMP WHERE id=? AND status='queued'").bind(&job.id).execute(pool).await?;
    let month_ids = match job.scope.as_str() {
        "month" => sqlx::query_scalar::<_, String>("SELECT fm.id FROM feed_months fm JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year JOIN feeds f ON f.id=fm.feed_id WHERE fm.id=? AND fm.active=1 AND fy.active=1 AND f.active=1").bind(job.month_id).fetch_all(pool).await?,
        "year" => sqlx::query_scalar::<_, String>("SELECT fm.id FROM feed_months fm JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year JOIN feeds f ON f.id=fm.feed_id WHERE fm.feed_id=? AND fm.year=? AND fm.active=1 AND fy.active=1 AND f.active=1 ORDER BY fm.month").bind(&job.feed_id).bind(job.year).fetch_all(pool).await?,
        _ => sqlx::query_scalar::<_, String>("SELECT fm.id FROM feed_months fm JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year JOIN feeds f ON f.id=fm.feed_id WHERE fm.feed_id=? AND fm.active=1 AND fy.active=1 AND f.active=1 ORDER BY fm.year,fm.month").bind(&job.feed_id).fetch_all(pool).await?,
    };
    let mut errors = Vec::new();
    for month_id in month_ids {
        let now = Utc::now().to_rfc3339();
        let month_result = async {
            let _ = feed_api::discover_month(Path(month_id.clone()), State(pool.clone()))
                .await
                .map_err(|_| "provider listing failed".to_owned())?;
            let docs = feed_api::list_documents(Path(month_id.clone()), State(pool.clone()))
                .await
                .map_err(|_| "could not load discovered files".to_owned())?;
            let mut document_errors = Vec::new();
            for document in docs.0 {
                let document_id = document.id.clone();
                if feed_api::extract_document(
                    Path(document_id.clone()),
                    State(pool.clone()),
                    Query(feed_api::ForceQuery {
                        force: Some(job.force != 0),
                    }),
                )
                .await
                .is_err()
                {
                    let detail = sqlx::query_scalar::<_, Option<String>>(
                        "SELECT last_error FROM source_documents WHERE id=?",
                    )
                    .bind(document_id)
                    .fetch_one(pool)
                    .await
                    .map_err(|_| "could not load PDF failure details".to_owned())?
                    .unwrap_or_else(|| {
                        "PDF processing failed without a recorded detail".to_owned()
                    });
                    document_errors.push((document.filename, detail));
                }
            }
            if !document_errors.is_empty() {
                return Err(document_error_summary(&document_errors));
            }
            Ok::<(), String>(())
        }
        .await;
        let (result, error) = match month_result {
            Ok(()) => ("success", None),
            Err(e) => {
                crate::logging::refresh_failed(&job.id, &job.scope, &job.feed_id, &month_id, &e);
                errors.push(e.clone());
                ("failure", Some(e))
            }
        };
        sqlx::query(
            "UPDATE feed_months SET last_attempt_at=?,last_result=?,last_error=? WHERE id=?",
        )
        .bind(now)
        .bind(result)
        .bind(error)
        .bind(month_id)
        .execute(pool)
        .await?;
    }
    let (status, error) = if errors.is_empty() {
        ("completed", None)
    } else {
        ("failed", Some(errors.join("; ")))
    };
    sqlx::query(
        "UPDATE refresh_jobs SET status=?,finished_at=CURRENT_TIMESTAMP,error=? WHERE id=?",
    )
    .bind(status)
    .bind(error)
    .bind(job.id)
    .execute(pool)
    .await?;
    let _ = job.force;
    Ok(true)
}

fn document_error_summary(errors: &[(String, String)]) -> String {
    truncate_error_summary(
        errors
            .iter()
            .map(|(filename, detail)| format!("{filename}: {detail}"))
            .collect::<Vec<_>>()
            .join("; "),
    )
}

fn truncate_error_summary(mut error: String) -> String {
    const MAX_SUMMARY_CHARS: usize = 4000;
    if error.chars().count() > MAX_SUMMARY_CHARS {
        error = error.chars().take(MAX_SUMMARY_CHARS - 3).collect();
        error.push_str("...");
    }
    error
}

#[cfg(test)]
mod error_summary_tests {
    use super::document_error_summary;

    #[test]
    fn scheduled_refresh_summary_keeps_each_filename_and_failure_reason() {
        let summary = document_error_summary(&[
            ("first.pdf".into(), "download failed: timeout".into()),
            (
                "second.pdf".into(),
                "PDF extraction failed: invalid header".into(),
            ),
        ]);
        assert_eq!(
            summary,
            "first.pdf: download failed: timeout; second.pdf: PDF extraction failed: invalid header"
        );
    }
}

fn env_seconds(name: &str, default: i64) -> i64 {
    positive_seconds(std::env::var(name).ok().as_deref(), default)
}

fn positive_seconds(value: Option<&str>, default: i64) -> i64 {
    value
        .and_then(|v| v.parse::<i64>().ok())
        .filter(|seconds| *seconds > 0)
        .unwrap_or(default)
}

mod feed_api {
    pub use crate::feed_api::{ForceQuery, discover_month, extract_document, list_documents};
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        extract::{Path, State},
        http::StatusCode,
    };
    use chrono::TimeZone;

    async fn insert_feed(pool: &SqlitePool, id: &str, interval: Option<i64>) {
        sqlx::query("INSERT INTO feeds (id, slug, display_name, refresh_interval_seconds) VALUES (?, ?, ?, ?)")
            .bind(id).bind(id).bind(id).bind(interval).execute(pool).await.unwrap();
    }

    async fn insert_month(
        pool: &SqlitePool,
        feed: &str,
        year: i64,
        month: i64,
        id: &str,
        source: &str,
    ) {
        sqlx::query("INSERT INTO feed_years (feed_id, year) VALUES (?, ?) ON CONFLICT DO NOTHING")
            .bind(feed)
            .bind(year)
            .execute(pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url) VALUES (?, ?, ?, ?, 'yandex_disk', ?)")
            .bind(id).bind(feed).bind(year).bind(month).bind(source).execute(pool).await.unwrap();
    }

    #[test]
    fn refresh_interval_parser_uses_only_positive_integer_values() {
        assert_eq!(positive_seconds(Some("1"), 3600), 1);
        assert_eq!(positive_seconds(Some("86400"), 3600), 86400);
        for invalid in [
            None,
            Some(""),
            Some("0"),
            Some("-1"),
            Some("1.5"),
            Some("abc"),
            Some("9223372036854775808"),
        ] {
            assert_eq!(positive_seconds(invalid, 3600), 3600, "input {invalid:?}");
        }
    }

    #[tokio::test]
    async fn scheduler_only_refreshes_current_month_and_deduplicates_pending_jobs() {
        let (pool, path) = crate::test_support::database().await;
        let now = Utc.with_ymd_and_hms(2026, 11, 15, 12, 0, 0).unwrap();
        insert_feed(&pool, "default", None).await;
        insert_month(&pool, "default", 2026, 11, "current", "fixture://current/").await;
        insert_month(&pool, "default", 2026, 10, "older", "fixture://older/").await;
        insert_feed(&pool, "override", Some(1800)).await;
        insert_month(
            &pool,
            "override",
            2026,
            11,
            "override-current",
            "fixture://override-current/",
        )
        .await;
        insert_month(
            &pool,
            "override",
            2026,
            10,
            "override-month",
            "fixture://override/",
        )
        .await;
        sqlx::query(
            "UPDATE feed_months SET next_refresh_at='2026-11-14T12:00:00Z' WHERE id='older'",
        )
        .execute(&pool)
        .await
        .unwrap();

        schedule_due_at(&pool, now, 3600).await.unwrap();

        for (month_id, expected_seconds) in [("current", 3600), ("override-current", 1800)] {
            let next: String =
                sqlx::query_scalar("SELECT next_refresh_at FROM feed_months WHERE id=?")
                    .bind(month_id)
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            let next = DateTime::parse_from_rfc3339(&next).unwrap().timestamp();
            assert_eq!(next - now.timestamp(), expected_seconds, "{month_id}");
        }
        let older_next: Option<String> =
            sqlx::query_scalar("SELECT next_refresh_at FROM feed_months WHERE id='older'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(older_next, None);
        let jobs: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM refresh_jobs WHERE status='queued'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(jobs, 2);
        schedule_due_at(&pool, now, 3600).await.unwrap();
        let jobs_after_second_run: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM refresh_jobs WHERE status='queued'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(jobs_after_second_run, jobs);
        crate::test_support::cleanup(pool, path).await;
    }

    #[tokio::test]
    async fn scheduler_moves_automatic_refresh_to_new_current_month() {
        let (pool, path) = crate::test_support::database().await;
        let november = Utc.with_ymd_and_hms(2026, 11, 30, 23, 59, 0).unwrap();
        let december = Utc.with_ymd_and_hms(2026, 12, 1, 0, 1, 0).unwrap();
        insert_feed(&pool, "feed", None).await;
        insert_month(&pool, "feed", 2026, 11, "november", "fixture://november/").await;
        insert_month(&pool, "feed", 2026, 12, "december", "fixture://december/").await;

        schedule_due_at(&pool, november, 3600).await.unwrap();
        schedule_due_at(&pool, december, 3600).await.unwrap();

        let december_jobs: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM refresh_jobs WHERE month_id='december' AND status='queued'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(december_jobs, 1);
        let november_next: Option<String> =
            sqlx::query_scalar("SELECT next_refresh_at FROM feed_months WHERE id='november'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(november_next, None);

        crate::test_support::cleanup(pool, path).await;
    }

    #[tokio::test]
    async fn force_refresh_queues_feed_year_and_month_scopes() {
        let (pool, path) = crate::test_support::database().await;
        insert_feed(&pool, "feed", None).await;
        insert_month(&pool, "feed", 2026, 11, "month", "fixture://month/").await;

        assert_eq!(
            crate::feed_api::force_feed_refresh(Path("feed".into()), State(pool.clone()))
                .await
                .unwrap()
                .0,
            StatusCode::ACCEPTED
        );
        assert_eq!(
            crate::feed_api::force_year_refresh(Path(("feed".into(), 2026)), State(pool.clone()))
                .await
                .unwrap()
                .0,
            StatusCode::ACCEPTED
        );
        assert_eq!(
            crate::feed_api::force_month_refresh(Path("month".into()), State(pool.clone()))
                .await
                .unwrap()
                .0,
            StatusCode::ACCEPTED
        );

        type JobRow = (String, String, Option<i64>, Option<String>, i64);
        let jobs: Vec<JobRow> = sqlx::query_as(
            "SELECT scope, feed_id, year, month_id, force FROM refresh_jobs ORDER BY scope",
        )
        .fetch_all(&pool)
        .await
        .unwrap();
        assert_eq!(
            jobs,
            vec![
                ("feed".into(), "feed".into(), None, None, 1),
                (
                    "month".into(),
                    "feed".into(),
                    Some(2026),
                    Some("month".into()),
                    1
                ),
                ("year".into(), "feed".into(), Some(2026), None, 1),
            ]
        );
        crate::test_support::cleanup(pool, path).await;
    }

    #[tokio::test]
    async fn failed_refresh_keeps_the_last_successful_cached_content() {
        let (pool, path) = crate::test_support::database().await;
        insert_feed(&pool, "feed", None).await;
        insert_month(&pool, "feed", 2026, 11, "month", "fixture://missing/").await;
        sqlx::query("INSERT INTO source_documents (id, month_id, source_path, filename, source_url, content_fingerprint, size_bytes, extraction_status) VALUES ('doc', 'month', 'article.pdf', 'article.pdf', 'https://fixture.invalid/article.pdf', 'source-v1', 10, 'success')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO cached_content (document_id, content_fingerprint, extracted_text, first_page_text, sanitized_html, page_count) VALUES ('doc', 'cached-v1', 'last good text', 'last good page', '<p>last good text</p>', 1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO refresh_jobs (id, scope, feed_id, year, month_id, force, status) VALUES ('job', 'month', 'feed', 2026, 'month', 0, 'queued')")
            .execute(&pool).await.unwrap();

        let (processed, logs) = crate::test_support::capture_logs(process_one(&pool)).await;
        assert!(processed.unwrap());
        let events = crate::test_support::json_events(&logs);
        let failure = events
            .iter()
            .find(|event| event["fields"]["event"] == "refresh_month_failed")
            .expect("refresh failure event");
        assert_eq!(failure["fields"]["job_id"], "job");
        assert_eq!(failure["fields"]["scope"], "month");
        assert_eq!(failure["fields"]["feed_id"], "feed");
        assert_eq!(failure["fields"]["month_id"], "month");
        assert_eq!(failure["fields"]["error"], "provider listing failed");
        assert!(!logs.contains("fixture://missing"));
        let (status, error): (String, Option<String>) =
            sqlx::query_as("SELECT status, error FROM refresh_jobs WHERE id='job'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(status, "failed");
        assert!(error.is_some());
        let (last_result, _last_error): (Option<String>, Option<String>) =
            sqlx::query_as("SELECT last_result, last_error FROM feed_months WHERE id='month'")
                .fetch_one(&pool)
                .await
                .unwrap();
        assert_eq!(last_result.as_deref(), Some("failure"));
        let (fingerprint, text, html): (String, String, String) = sqlx::query_as("SELECT content_fingerprint, extracted_text, sanitized_html FROM cached_content WHERE document_id='doc'")
            .fetch_one(&pool).await.unwrap();
        assert_eq!(fingerprint, "cached-v1");
        assert_eq!(text, "last good text");
        assert_eq!(html, "<p>last good text</p>");
        crate::test_support::cleanup(pool, path).await;
    }

    #[tokio::test]
    async fn scheduler_errors_are_structured_and_exclude_sensitive_values() {
        let (pool, path) = crate::test_support::database().await;
        pool.close().await;

        let ((), logs) = crate::test_support::capture_logs(run_tick(&pool)).await;
        let events = crate::test_support::json_events(&logs);
        for name in ["scheduler_enqueue_failed", "scheduler_job_failed"] {
            let event = events
                .iter()
                .find(|event| event["fields"]["event"] == name)
                .unwrap_or_else(|| panic!("missing {name}: {logs}"));
            assert!(
                event["fields"]["operation"]
                    .as_str()
                    .is_some_and(|v| !v.is_empty())
            );
            assert!(event["fields"]["error"].as_str().is_some());
        }
        assert!(!logs.contains("sqlite://"));
        crate::test_support::cleanup(pool, path).await;
    }
}
