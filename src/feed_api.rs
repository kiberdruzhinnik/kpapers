use axum::{extract::{Path, Query, State}, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, sqlx::FromRow)]
pub struct Feed { pub id: String, pub display_name: String, pub description: String, pub active: bool, pub refresh_interval_seconds: Option<i64> }
#[derive(Deserialize)]
pub struct FeedInput { pub display_name: String, #[serde(default)] pub description: String, pub refresh_interval_seconds: Option<i64> }

#[derive(Serialize, sqlx::FromRow)]
pub struct Year { pub feed_id: String, pub year: i64, pub active: bool }
#[derive(Deserialize)]
pub struct MonthInput { pub source_url: String }
#[derive(Serialize, sqlx::FromRow)]
pub struct Month { pub id: String, pub feed_id: String, pub year: i64, pub month: i64, pub provider: String, pub source_url: String, pub active: bool, pub last_attempt_at: Option<String>, pub next_refresh_at: Option<String>, pub last_result: Option<String>, pub last_error: Option<String> }

#[derive(Serialize, sqlx::FromRow)]
pub struct SourceDocument { pub id: String, pub filename: String, pub source_url: String, pub promotional_classification: String, pub promotional_override: Option<String>, pub extraction_status: String, pub last_error: Option<String> }
#[derive(Deserialize)]
pub struct PromotionOverride { pub override_value: Option<String> }

pub async fn list_documents(Path(month_id): Path<String>, State(pool): State<SqlitePool>) -> Result<Json<Vec<SourceDocument>>, StatusCode> {
    sqlx::query_as::<_, SourceDocument>("SELECT id, filename, source_url, promotional_classification, promotional_override, extraction_status, last_error FROM source_documents WHERE month_id = ? ORDER BY filename")
        .bind(month_id).fetch_all(&pool).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn set_promotion_override(Path(document_id): Path<String>, State(pool): State<SqlitePool>, Json(input): Json<PromotionOverride>) -> Result<StatusCode, StatusCode> {
    if input.override_value.as_deref().is_some_and(|v| !matches!(v, "include" | "exclude")) { return Err(StatusCode::BAD_REQUEST); }
    let result = sqlx::query("UPDATE source_documents SET promotional_override = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(input.override_value).bind(document_id).execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 { return Err(StatusCode::NOT_FOUND); }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct DiscoveryResult { pub discovered_pdfs: usize }
#[derive(Serialize)]
pub struct ExtractionResult { pub status: &'static str, pub page_count: usize, pub characters: usize, pub first_page_sample: String }
#[derive(Deserialize, Default)]
pub struct ForceQuery { pub force: Option<bool> }

#[derive(Serialize)]
pub struct QueuedRefresh { pub job_id: String, pub status: &'static str }

pub async fn force_feed_refresh(Path(feed_id): Path<String>, State(pool): State<SqlitePool>) -> Result<(StatusCode, Json<QueuedRefresh>), StatusCode> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM feeds WHERE id=? AND active=1)").bind(&feed_id).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !exists { return Err(StatusCode::NOT_FOUND); }
    queue_force(&pool, "feed", &feed_id, None, None).await
}

pub async fn force_year_refresh(Path((feed_id, year)): Path<(String, i64)>, State(pool): State<SqlitePool>) -> Result<(StatusCode, Json<QueuedRefresh>), StatusCode> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM feed_years y JOIN feeds f ON f.id=y.feed_id WHERE y.feed_id=? AND y.year=? AND y.active=1 AND f.active=1)").bind(&feed_id).bind(year).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !exists { return Err(StatusCode::NOT_FOUND); }
    queue_force(&pool, "year", &feed_id, Some(year), None).await
}

pub async fn force_month_refresh(Path(month_id): Path<String>, State(pool): State<SqlitePool>) -> Result<(StatusCode, Json<QueuedRefresh>), StatusCode> {
    let row = sqlx::query_as::<_, (String, i64)>("SELECT m.feed_id,m.year FROM feed_months m JOIN feeds f ON f.id=m.feed_id JOIN feed_years y ON y.feed_id=m.feed_id AND y.year=m.year WHERE m.id=? AND m.active=1 AND y.active=1 AND f.active=1").bind(&month_id).fetch_optional(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
    queue_force(&pool, "month", &row.0, Some(row.1), Some(&month_id)).await
}

async fn queue_force(pool: &SqlitePool, scope: &str, feed_id: &str, year: Option<i64>, month_id: Option<&str>) -> Result<(StatusCode, Json<QueuedRefresh>), StatusCode> {
    let job_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO refresh_jobs (id,scope,feed_id,year,month_id,force,status) VALUES (?,?,?,?,?,1,'queued')")
        .bind(&job_id).bind(scope).bind(feed_id).bind(year).bind(month_id).execute(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::ACCEPTED, Json(QueuedRefresh { job_id, status: "queued" })))
}

pub async fn extract_document(Path(document_id): Path<String>, State(pool): State<SqlitePool>, Query(query): Query<ForceQuery>) -> Result<Json<ExtractionResult>, StatusCode> {
    let row = sqlx::query_as::<_, (String, String, String, String, i64, String)>("SELECT fm.provider, fm.source_url, sd.source_path, sd.filename, sd.size_bytes, sd.content_fingerprint FROM source_documents sd JOIN feed_months fm ON fm.id = sd.month_id JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year JOIN feeds f ON f.id=fm.feed_id WHERE sd.id = ? AND fm.active=1 AND fy.active=1 AND f.active=1")
        .bind(&document_id).fetch_optional(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
    let cache_fingerprint = crate::pdf_processing::cache_fingerprint(&row.5);
    let cached_fingerprint = sqlx::query_scalar::<_, String>("SELECT content_fingerprint FROM cached_content WHERE document_id = ?")
        .bind(&document_id).fetch_optional(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !query.force.unwrap_or(false) && cached_fingerprint.as_deref() == Some(cache_fingerprint.as_str()) {
        let (characters, pages, first_page) = sqlx::query_as::<_, (i64, i64, String)>("SELECT length(extracted_text), page_count, first_page_text FROM cached_content WHERE document_id = ?").bind(&document_id).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Json(ExtractionResult { status: "cached", page_count: pages as usize, characters: characters as usize, first_page_sample: first_page.chars().take(500).collect() }));
    }
    let maximum = std::env::var("MAX_PDF_BYTES").ok().and_then(|v| v.parse::<usize>().ok()).filter(|n| *n > 0).unwrap_or(25 * 1024 * 1024);
    let result = async {
        let files = if row.1.starts_with("fixture://") {
            crate::providers::local_fixtures::list_root_pdfs(&row.1).await
        } else { match row.0.as_str() {
            "yandex_disk" => crate::providers::yandex_disk::list_root_pdfs(&row.1).await.map_err(|e| e.to_string()),
            "mail_cloud" => crate::providers::mail_cloud::list_root_pdfs(&row.1).await.map_err(|e| e.to_string()),
            _ => return Err("unsupported provider".to_owned()),
        }}?;
        let file = files.iter().find(|f| f.path == row.2).ok_or_else(|| "file is no longer present in the public folder".to_owned())?;
        let bytes = if row.1.starts_with("fixture://") {
            crate::providers::local_fixtures::download(file, maximum as u64).await
        } else { match row.0.as_str() {
            "yandex_disk" => crate::providers::yandex_disk::download(file, maximum as u64).await.map_err(|e| e.to_string()),
            "mail_cloud" => crate::providers::mail_cloud::download(file, maximum as u64).await.map_err(|e| e.to_string()),
            _ => unreachable!(),
        }}?;
        let extracted = crate::pdf_processing::extract_pdf(&bytes, maximum)?;
        let html = extracted.html.clone();
        let classification = crate::pdf_processing::classify_promotion(&row.3, &extracted.first_page);
        sqlx::query("INSERT INTO cached_content (document_id, content_fingerprint, extracted_text, first_page_text, sanitized_html, page_count) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(document_id) DO UPDATE SET content_fingerprint=excluded.content_fingerprint, extracted_text=excluded.extracted_text, first_page_text=excluded.first_page_text, sanitized_html=excluded.sanitized_html, page_count=excluded.page_count, processed_at=CURRENT_TIMESTAMP")
            .bind(&document_id).bind(&cache_fingerprint).bind(&extracted.full_text).bind(&extracted.first_page).bind(html).bind(extracted.page_count as i64).execute(&pool).await.map_err(|e| e.to_string())?;
        sqlx::query("UPDATE source_documents SET extraction_status='success', promotional_classification=?, last_error=NULL, updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(classification).bind(&document_id).execute(&pool).await.map_err(|e| e.to_string())?;
        Ok::<_, String>((extracted.page_count, extracted.full_text.chars().count(), extracted.first_page))
    }.await;
    match result {
        Ok((page_count, characters, first_page)) => Ok(Json(ExtractionResult { status: "success", page_count, characters, first_page_sample: first_page.chars().take(500).collect() })),
        Err(error) => {
            let _ = sqlx::query("UPDATE source_documents SET extraction_status='failure', last_error=?, updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(&error).bind(&document_id).execute(&pool).await;
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

pub async fn discover_month(Path(month_id): Path<String>, State(pool): State<SqlitePool>) -> Result<Json<DiscoveryResult>, StatusCode> {
    let row = sqlx::query_as::<_, (String, String)>("SELECT fm.provider, fm.source_url FROM feed_months fm JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year JOIN feeds f ON f.id=fm.feed_id WHERE fm.id = ? AND fm.active=1 AND fy.active=1 AND f.active=1")
        .bind(&month_id).fetch_optional(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
    let files = if row.1.starts_with("fixture://") {
        crate::providers::local_fixtures::list_root_pdfs(&row.1).await
    } else { match row.0.as_str() {
        "yandex_disk" => crate::providers::yandex_disk::list_root_pdfs(&row.1).await.map_err(|e| e.to_string()),
        "mail_cloud" => crate::providers::mail_cloud::list_root_pdfs(&row.1).await.map_err(|e| e.to_string()),
        _ => return Err(StatusCode::BAD_REQUEST),
    }}.map_err(|_| StatusCode::BAD_GATEWAY)?;
    for file in &files {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO source_documents (id, month_id, provider_file_id, source_path, filename, source_url, content_fingerprint, size_bytes, provider_modified_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(month_id, source_path) DO UPDATE SET provider_file_id=excluded.provider_file_id, filename=excluded.filename, source_url=excluded.source_url, content_fingerprint=excluded.content_fingerprint, size_bytes=excluded.size_bytes, provider_modified_at=excluded.provider_modified_at, updated_at=CURRENT_TIMESTAMP")
            .bind(id).bind(&month_id).bind(&file.provider_id).bind(&file.path).bind(&file.filename).bind(&file.public_url).bind(&file.fingerprint).bind(file.size as i64).bind(&file.modified)
            .execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(Json(DiscoveryResult { discovered_pdfs: files.len() }))
}

pub async fn create_year(Path(feed_id): Path<String>, State(pool): State<SqlitePool>, Json(input): Json<YearInput>) -> Result<(StatusCode, Json<Year>), StatusCode> {
    if !(1..=9999).contains(&input.year) { return Err(StatusCode::BAD_REQUEST); }
    let result = sqlx::query("INSERT INTO feed_years (feed_id, year) SELECT id, ? FROM feeds WHERE id = ? AND active = 1")
        .bind(input.year).bind(&feed_id).execute(&pool).await.map_err(|_| StatusCode::CONFLICT)?;
    if result.rows_affected() == 0 { return Err(StatusCode::NOT_FOUND); }
    Ok((StatusCode::CREATED, Json(Year { feed_id, year: input.year, active: true })))
}

#[derive(Deserialize)]
pub struct YearInput { pub year: i64 }

pub async fn create_month(Path((feed_id, year, month)): Path<(String, i64, i64)>, State(pool): State<SqlitePool>, Json(input): Json<MonthInput>) -> Result<(StatusCode, Json<Month>), StatusCode> {
    if !(1..=12).contains(&month) { return Err(StatusCode::BAD_REQUEST); }
    let provider = crate::providers::validate_source_url(&input.source_url).map_err(|_| StatusCode::BAD_REQUEST)?;
    let id = uuid::Uuid::new_v4().to_string();
    let result = sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url) SELECT ?, y.feed_id, y.year, ?, ?, ? FROM feed_years y JOIN feeds f ON f.id=y.feed_id WHERE y.feed_id = ? AND y.year = ? AND y.active=1 AND f.active=1")
        .bind(&id).bind(month).bind(provider).bind(input.source_url).bind(&feed_id).bind(year)
        .execute(&pool).await.map_err(|_| StatusCode::CONFLICT)?;
    if result.rows_affected() == 0 { return Err(StatusCode::NOT_FOUND); }
    let record = sqlx::query_as::<_, Month>("SELECT id, feed_id, year, month, provider, source_url, active, last_attempt_at, next_refresh_at, last_result, last_error FROM feed_months WHERE id = ?")
        .bind(id).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn list_years(Path(feed_id): Path<String>, State(pool): State<SqlitePool>) -> Result<Json<Vec<Year>>, StatusCode> {
    sqlx::query_as::<_, Year>("SELECT feed_id, year, active FROM feed_years WHERE feed_id = ? ORDER BY year DESC")
        .bind(feed_id).fetch_all(&pool).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn list_months(Path((feed_id, year)): Path<(String, i64)>, State(pool): State<SqlitePool>) -> Result<Json<Vec<Month>>, StatusCode> {
    sqlx::query_as::<_, Month>("SELECT id, feed_id, year, month, provider, source_url, active, last_attempt_at, next_refresh_at, last_result, last_error FROM feed_months WHERE feed_id = ? AND year = ? ORDER BY month")
        .bind(feed_id).bind(year).fetch_all(&pool).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Feed>>, StatusCode> {
    sqlx::query_as::<_, Feed>("SELECT id, display_name, description, active, refresh_interval_seconds FROM feeds ORDER BY display_name")
        .fetch_all(&pool).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn create(State(pool): State<SqlitePool>, Json(input): Json<FeedInput>) -> Result<(StatusCode, Json<Feed>), StatusCode> {
    if input.display_name.trim().is_empty() || input.refresh_interval_seconds.is_some_and(|v| v <= 0) { return Err(StatusCode::BAD_REQUEST); }
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO feeds (id, slug, display_name, description, refresh_interval_seconds) VALUES (?, ?, ?, ?, ?)")
        .bind(&id).bind(&id).bind(input.display_name.trim()).bind(input.description.trim()).bind(input.refresh_interval_seconds)
        .execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let feed = sqlx::query_as::<_, Feed>("SELECT id, display_name, description, active, refresh_interval_seconds FROM feeds WHERE id = ?")
        .bind(id).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(feed)))
}

pub async fn update(Path(id): Path<String>, State(pool): State<SqlitePool>, Json(input): Json<FeedInput>) -> Result<Json<Feed>, StatusCode> {
    if input.display_name.trim().is_empty() || input.refresh_interval_seconds.is_some_and(|v| v <= 0) { return Err(StatusCode::BAD_REQUEST); }
    let result = sqlx::query("UPDATE feeds SET display_name = ?, description = ?, refresh_interval_seconds = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND active = 1")
        .bind(input.display_name.trim()).bind(input.description.trim()).bind(input.refresh_interval_seconds).bind(&id)
        .execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 { return Err(StatusCode::NOT_FOUND); }
    let feed = sqlx::query_as::<_, Feed>("SELECT id, display_name, description, active, refresh_interval_seconds FROM feeds WHERE id = ?")
        .bind(id).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(feed))
}

pub async fn deactivate(Path(id): Path<String>, State(pool): State<SqlitePool>) -> impl IntoResponse {
    match sqlx::query("UPDATE feeds SET active = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND active = 1").bind(id).execute(&pool).await {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn archive_feed(Path(id): Path<String>, State(pool): State<SqlitePool>) -> StatusCode {
    set_feed_active(&pool, &id, false).await
}

pub async fn restore_feed(Path(id): Path<String>, State(pool): State<SqlitePool>) -> StatusCode {
    set_feed_active(&pool, &id, true).await
}

async fn set_feed_active(pool: &SqlitePool, id: &str, active: bool) -> StatusCode {
    match sqlx::query("UPDATE feeds SET active=?, updated_at=CURRENT_TIMESTAMP WHERE id=?")
        .bind(active).bind(id).execute(pool).await {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn permanently_delete_feed(Path(id): Path<String>, State(pool): State<SqlitePool>) -> StatusCode {
    delete_scope(&pool, "DELETE FROM feeds WHERE id=?", vec![id]).await
}

pub async fn archive_year(Path((feed_id, year)): Path<(String, i64)>, State(pool): State<SqlitePool>) -> StatusCode {
    set_year_active(&pool, &feed_id, year, false).await
}

pub async fn restore_year(Path((feed_id, year)): Path<(String, i64)>, State(pool): State<SqlitePool>) -> StatusCode {
    set_year_active(&pool, &feed_id, year, true).await
}

async fn set_year_active(pool: &SqlitePool, feed_id: &str, year: i64, active: bool) -> StatusCode {
    match sqlx::query("UPDATE feed_years SET active=? WHERE feed_id=? AND year=?")
        .bind(active).bind(feed_id).bind(year).execute(pool).await {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn permanently_delete_year(Path((feed_id, year)): Path<(String, i64)>, State(pool): State<SqlitePool>) -> StatusCode {
    delete_scope(&pool, "DELETE FROM feed_years WHERE feed_id=? AND year=?", vec![feed_id, year.to_string()]).await
}

pub async fn archive_month(Path(id): Path<String>, State(pool): State<SqlitePool>) -> StatusCode {
    set_month_active(&pool, &id, false).await
}

pub async fn restore_month(Path(id): Path<String>, State(pool): State<SqlitePool>) -> StatusCode {
    set_month_active(&pool, &id, true).await
}

async fn set_month_active(pool: &SqlitePool, id: &str, active: bool) -> StatusCode {
    match sqlx::query("UPDATE feed_months SET active=? WHERE id=?")
        .bind(active).bind(id).execute(pool).await {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn permanently_delete_month(Path(id): Path<String>, State(pool): State<SqlitePool>) -> StatusCode {
    delete_scope(&pool, "DELETE FROM feed_months WHERE id=?", vec![id]).await
}

async fn delete_scope(pool: &SqlitePool, query: &str, values: Vec<String>) -> StatusCode {
    let mut statement = sqlx::query(query);
    for value in values { statement = statement.bind(value); }
    match statement.execute(pool).await {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
