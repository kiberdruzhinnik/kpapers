use axum::extract::{Path, Query, State};
use chrono::{Datelike, DateTime, Utc};
use sqlx::{FromRow, SqlitePool};
use std::time::Duration;

#[derive(FromRow)]
struct DueMonth { id: String, feed_id: String, year: i64, month: i64, override_seconds: Option<i64>, next_refresh_at: Option<String> }
#[derive(FromRow)]
struct Job { id: String, scope: String, feed_id: String, year: Option<i64>, month_id: Option<String>, force: i64 }

pub async fn run(pool: SqlitePool) {
    let mut ticker = tokio::time::interval(Duration::from_secs(5));
    loop {
        ticker.tick().await;
        if let Err(error) = schedule_due(&pool).await { eprintln!("scheduler enqueue failed: {error}"); }
        loop {
            match process_one(&pool).await {
                Ok(true) => continue,
                Ok(false) => break,
                Err(error) => { eprintln!("scheduler job failed: {error}"); break; }
            }
        }
    }
}

async fn schedule_due(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    let months = sqlx::query_as::<_, DueMonth>("SELECT fm.id, fm.feed_id, fm.year, fm.month, f.refresh_interval_seconds AS override_seconds, fm.next_refresh_at FROM feed_months fm JOIN feeds f ON f.id=fm.feed_id JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year WHERE f.active=1 AND fy.active=1 AND fm.active=1")
        .fetch_all(pool).await?;
    let now = Utc::now();
    let one_hour = env_seconds("CURRENT_MONTH_REFRESH_SECONDS", 3600);
    let older_interval = env_seconds("OLDER_MONTH_REFRESH_SECONDS", 86400);
    for month in months {
        let next_due = month.next_refresh_at.as_deref().and_then(|v| DateTime::parse_from_rfc3339(v).ok()).map(|t| t.with_timezone(&Utc));
        if next_due.is_some_and(|due| due > now) { continue; }
        let has_pending: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM refresh_jobs WHERE month_id=? AND status IN ('queued','running'))")
            .bind(&month.id).fetch_one(pool).await?;
        if has_pending { continue; }
        let current = month.year == now.year() as i64 && month.month == now.month() as i64;
        let interval = month.override_seconds.unwrap_or(if current { one_hour } else { older_interval });
        let next = (now + chrono::Duration::seconds(interval)).to_rfc3339();
        sqlx::query("INSERT INTO refresh_jobs (id,scope,feed_id,year,month_id,force,status) VALUES (?, 'month', ?, ?, ?, 0, 'queued')")
            .bind(uuid::Uuid::new_v4().to_string()).bind(&month.feed_id).bind(month.year).bind(&month.id).execute(pool).await?;
        sqlx::query("UPDATE feed_months SET next_refresh_at=? WHERE id=?").bind(next).bind(month.id).execute(pool).await?;
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
            feed_api::discover_month(Path(month_id.clone()), State(pool.clone())).await.map_err(|_| "provider listing failed".to_owned())?;
            let docs = feed_api::list_documents(Path(month_id.clone()), State(pool.clone())).await.map_err(|_| "could not load discovered files".to_owned())?;
            for document in docs.0 {
                feed_api::extract_document(Path(document.id), State(pool.clone()), Query(feed_api::ForceQuery { force: Some(job.force != 0) })).await.map_err(|_| "one or more PDFs could not be extracted".to_owned())?;
            }
            Ok::<(), String>(())
        }.await;
        let (result, error) = match month_result { Ok(()) => ("success", None), Err(e) => { errors.push(e.clone()); ("failure", Some(e)) } };
        sqlx::query("UPDATE feed_months SET last_attempt_at=?,last_result=?,last_error=? WHERE id=?")
            .bind(now).bind(result).bind(error).bind(month_id).execute(pool).await?;
    }
    let (status, error) = if errors.is_empty() { ("completed", None) } else { ("failed", Some(errors.join("; "))) };
    sqlx::query("UPDATE refresh_jobs SET status=?,finished_at=CURRENT_TIMESTAMP,error=? WHERE id=?")
        .bind(status).bind(error).bind(job.id).execute(pool).await?;
    let _ = job.force;
    Ok(true)
}

fn env_seconds(name: &str, default: i64) -> i64 {
    std::env::var(name).ok().and_then(|v| v.parse::<i64>().ok()).filter(|v| *v > 0).unwrap_or(default)
}

mod feed_api { pub use crate::feed_api::{discover_month, extract_document, list_documents, ForceQuery}; }
