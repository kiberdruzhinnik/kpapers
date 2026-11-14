use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
};
use serde::{Deserialize, Deserializer, Serialize};
use sqlx::SqlitePool;

#[derive(Serialize, sqlx::FromRow)]
pub struct Feed {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub active: bool,
    pub refresh_interval_seconds: Option<i64>,
    pub extraction_engine: Option<String>,
}
#[derive(Deserialize)]
pub struct FeedInput {
    pub display_name: String,
    #[serde(default)]
    pub description: String,
    pub refresh_interval_seconds: Option<i64>,
    /// Absent means "leave the current override unchanged" on updates and
    /// "inherit" on creation. `null` explicitly selects inheritance.
    #[serde(default, deserialize_with = "deserialize_engine_field")]
    pub extraction_engine: Option<Option<String>>,
}

fn deserialize_engine_field<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}

fn valid_engine(value: Option<&str>) -> bool {
    matches!(value, None | Some("pdfium") | Some("marker"))
}

fn valid_feed_input(input: &FeedInput) -> bool {
    !input.display_name.trim().is_empty()
        && input
            .refresh_interval_seconds
            .is_none_or(|seconds| seconds > 0)
        && input
            .extraction_engine
            .as_ref()
            .is_none_or(|value| valid_engine(value.as_deref()))
}

fn valid_year(year: i64) -> bool {
    (1..=9999).contains(&year)
}

fn valid_month(month: i64) -> bool {
    (1..=12).contains(&month)
}

#[derive(Serialize, sqlx::FromRow)]
pub struct Year {
    pub feed_id: String,
    pub year: i64,
    pub active: bool,
}
#[derive(Deserialize)]
pub struct MonthInput {
    pub source_url: String,
}
#[derive(Serialize, sqlx::FromRow)]
pub struct Month {
    pub id: String,
    pub feed_id: String,
    pub year: i64,
    pub month: i64,
    pub provider: String,
    pub source_url: String,
    pub active: bool,
    pub last_attempt_at: Option<String>,
    pub next_refresh_at: Option<String>,
    pub last_result: Option<String>,
    pub last_error: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct SourceDocument {
    pub id: String,
    pub filename: String,
    pub source_url: String,
    pub promotional_classification: String,
    pub promotional_override: Option<String>,
    pub extraction_status: String,
    pub last_error: Option<String>,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct OverviewSummary {
    pub source_folder_count: i64,
    pub needs_review_count: i64,
}

const NEEDS_REVIEW_PREDICATE: &str = "sd.extraction_status IN ('pending', 'failure') OR sd.promotional_classification NOT IN ('publish', 'promotional')";

pub async fn overview_summary(
    State(pool): State<SqlitePool>,
) -> Result<Json<OverviewSummary>, StatusCode> {
    let query = format!(
        "SELECT \
            (SELECT COUNT(*) FROM feed_months fm \
             JOIN feed_years fy ON fy.feed_id = fm.feed_id AND fy.year = fm.year \
             JOIN feeds f ON f.id = fm.feed_id \
             WHERE fm.active = 1 AND fy.active = 1 AND f.active = 1) AS source_folder_count, \
            (SELECT COUNT(*) FROM source_documents sd \
             JOIN feed_months fm ON fm.id = sd.month_id \
             JOIN feed_years fy ON fy.feed_id = fm.feed_id AND fy.year = fm.year \
             JOIN feeds f ON f.id = fm.feed_id \
             WHERE fm.active = 1 AND fy.active = 1 AND f.active = 1 \
               AND ({NEEDS_REVIEW_PREDICATE})) \
             AS needs_review_count",
    );
    sqlx::query_as::<_, OverviewSummary>(&query)
        .fetch_one(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Deserialize, Default)]
pub struct ReviewQueueFilter {
    #[serde(default)]
    pub archived: bool,
}

#[derive(Serialize, sqlx::FromRow)]
pub struct ReviewQueueDocument {
    pub id: String,
    pub filename: String,
    pub source_url: String,
    pub promotional_classification: String,
    pub promotional_override: Option<String>,
    pub extraction_status: String,
    pub last_error: Option<String>,
    pub feed_id: String,
    pub feed_name: String,
    pub year: i64,
    pub month: i64,
    pub month_id: String,
    pub archived: bool,
}

pub async fn list_review_queue(
    State(pool): State<SqlitePool>,
    Query(filter): Query<ReviewQueueFilter>,
) -> Result<Json<Vec<ReviewQueueDocument>>, StatusCode> {
    let query = format!(
        "SELECT sd.id, sd.filename, sd.source_url, sd.promotional_classification, \
                sd.promotional_override, sd.extraction_status, sd.last_error, \
                f.id AS feed_id, f.display_name AS feed_name, fy.year, fm.month, \
                fm.id AS month_id, \
                CASE WHEN f.active = 0 OR fy.active = 0 OR fm.active = 0 \
                     THEN 1 ELSE 0 END AS archived \
         FROM source_documents sd \
         JOIN feed_months fm ON fm.id = sd.month_id \
         JOIN feed_years fy ON fy.feed_id = fm.feed_id AND fy.year = fm.year \
         JOIN feeds f ON f.id = fm.feed_id \
         WHERE ({NEEDS_REVIEW_PREDICATE}) \
           AND CASE WHEN f.active = 0 OR fy.active = 0 OR fm.active = 0 \
                    THEN 1 ELSE 0 END = ? \
         ORDER BY f.display_name, fy.year DESC, fm.month, sd.filename",
    );
    sqlx::query_as::<_, ReviewQueueDocument>(&query)
        .bind(filter.archived)
        .fetch_all(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Deserialize)]
pub struct PromotionOverride {
    pub override_value: Option<String>,
}

pub async fn list_documents(
    Path(month_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<SourceDocument>>, StatusCode> {
    sqlx::query_as::<_, SourceDocument>("SELECT id, filename, source_url, promotional_classification, promotional_override, extraction_status, last_error FROM source_documents WHERE month_id = ? ORDER BY filename")
        .bind(month_id).fetch_all(&pool).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn set_promotion_override(
    Path(document_id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(input): Json<PromotionOverride>,
) -> Result<StatusCode, StatusCode> {
    if input
        .override_value
        .as_deref()
        .is_some_and(|v| !matches!(v, "include" | "exclude"))
    {
        return Err(StatusCode::BAD_REQUEST);
    }
    let result = sqlx::query("UPDATE source_documents SET promotional_override = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(input.override_value).bind(document_id).execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Serialize)]
pub struct DiscoveryResult {
    pub discovered_pdfs: usize,
}
#[derive(Serialize)]
pub struct ExtractionResult {
    pub status: &'static str,
    pub page_count: usize,
    pub characters: usize,
    pub first_page_sample: String,
}
#[derive(Deserialize, Default)]
pub struct ForceQuery {
    pub force: Option<bool>,
}

#[derive(Serialize)]
pub struct QueuedRefresh {
    pub job_id: String,
    pub status: &'static str,
}

pub async fn force_feed_refresh(
    Path(feed_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<(StatusCode, Json<QueuedRefresh>), StatusCode> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM feeds WHERE id=? AND active=1)")
            .bind(&feed_id)
            .fetch_one(&pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }
    queue_force(&pool, "feed", &feed_id, None, None).await
}

pub async fn force_year_refresh(
    Path((feed_id, year)): Path<(String, i64)>,
    State(pool): State<SqlitePool>,
) -> Result<(StatusCode, Json<QueuedRefresh>), StatusCode> {
    let exists: bool = sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM feed_years y JOIN feeds f ON f.id=y.feed_id WHERE y.feed_id=? AND y.year=? AND y.active=1 AND f.active=1)").bind(&feed_id).bind(year).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !exists {
        return Err(StatusCode::NOT_FOUND);
    }
    queue_force(&pool, "year", &feed_id, Some(year), None).await
}

pub async fn force_month_refresh(
    Path(month_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<(StatusCode, Json<QueuedRefresh>), StatusCode> {
    let row = sqlx::query_as::<_, (String, i64)>("SELECT m.feed_id,m.year FROM feed_months m JOIN feeds f ON f.id=m.feed_id JOIN feed_years y ON y.feed_id=m.feed_id AND y.year=m.year WHERE m.id=? AND m.active=1 AND y.active=1 AND f.active=1").bind(&month_id).fetch_optional(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
    queue_force(&pool, "month", &row.0, Some(row.1), Some(&month_id)).await
}

async fn queue_force(
    pool: &SqlitePool,
    scope: &str,
    feed_id: &str,
    year: Option<i64>,
    month_id: Option<&str>,
) -> Result<(StatusCode, Json<QueuedRefresh>), StatusCode> {
    let job_id = uuid::Uuid::new_v4().to_string();
    sqlx::query("INSERT INTO refresh_jobs (id,scope,feed_id,year,month_id,force,status) VALUES (?,?,?,?,?,1,'queued')")
        .bind(&job_id).bind(scope).bind(feed_id).bind(year).bind(month_id).execute(pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((
        StatusCode::ACCEPTED,
        Json(QueuedRefresh {
            job_id,
            status: "queued",
        }),
    ))
}

pub async fn extract_document(
    Path(document_id): Path<String>,
    State(pool): State<SqlitePool>,
    Extension(settings): Extension<crate::extraction::ExtractionSettings>,
    Query(query): Query<ForceQuery>,
) -> Result<Json<ExtractionResult>, StatusCode> {
    let row = sqlx::query_as::<_, (String, String, String, String, i64, String, Option<String>)>("SELECT fm.provider, fm.source_url, sd.source_path, sd.filename, sd.size_bytes, sd.content_fingerprint, f.extraction_engine FROM source_documents sd JOIN feed_months fm ON fm.id = sd.month_id JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year JOIN feeds f ON f.id=fm.feed_id WHERE sd.id = ? AND fm.active=1 AND fy.active=1 AND f.active=1")
        .bind(&document_id).fetch_optional(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
    let engine = settings.effective_engine(row.6.as_deref());
    let cache_fingerprint = crate::pdf_processing::cache_fingerprint_for(engine, &row.5);
    let cached_fingerprint = sqlx::query_scalar::<_, String>(
        "SELECT content_fingerprint FROM cached_content WHERE document_id = ?",
    )
    .bind(&document_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if !query.force.unwrap_or(false)
        && cached_fingerprint.as_deref() == Some(cache_fingerprint.as_str())
    {
        // A matching cache is the last known-good extraction. Restore the
        // document's publication/recovery state after a failed reprocess.
        sqlx::query("UPDATE source_documents SET extraction_status='success', last_error=NULL, updated_at=CURRENT_TIMESTAMP WHERE id=?")
            .bind(&document_id).execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        let (characters, pages, first_page) = sqlx::query_as::<_, (i64, i64, String)>("SELECT length(extracted_text), page_count, first_page_text FROM cached_content WHERE document_id = ?").bind(&document_id).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        return Ok(Json(ExtractionResult {
            status: "cached",
            page_count: pages as usize,
            characters: characters as usize,
            first_page_sample: first_page.chars().take(500).collect(),
        }));
    }
    let maximum = std::env::var("MAX_PDF_BYTES")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or(25 * 1024 * 1024);
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
        let mut extracted = crate::pdf_processing::extract_with_engine(
            engine,
            &bytes,
            maximum,
            &settings.marker,
        )
        .await?;
        crate::pdf_processing::repair_sponsr_title_from_filename(&row.3, &mut extracted);
        let html = extracted.html.clone();
        let classification = crate::pdf_processing::classify_promotion(&row.3, &extracted.first_page);
        sqlx::query("INSERT INTO cached_content (document_id, content_fingerprint, extracted_text, first_page_text, sanitized_html, page_count) VALUES (?, ?, ?, ?, ?, ?) ON CONFLICT(document_id) DO UPDATE SET content_fingerprint=excluded.content_fingerprint, extracted_text=excluded.extracted_text, first_page_text=excluded.first_page_text, sanitized_html=excluded.sanitized_html, page_count=excluded.page_count, processed_at=CURRENT_TIMESTAMP")
            .bind(&document_id).bind(&cache_fingerprint).bind(&extracted.full_text).bind(&extracted.first_page).bind(html).bind(extracted.page_count as i64).execute(&pool).await.map_err(|e| e.to_string())?;
        sqlx::query("UPDATE source_documents SET extraction_status='success', promotional_classification=?, last_error=NULL, updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(classification).bind(&document_id).execute(&pool).await.map_err(|e| e.to_string())?;
        Ok::<_, String>((extracted.page_count, extracted.full_text.chars().count(), extracted.first_page))
    }.await;
    match result {
        Ok((page_count, characters, first_page)) => Ok(Json(ExtractionResult {
            status: "success",
            page_count,
            characters,
            first_page_sample: first_page.chars().take(500).collect(),
        })),
        Err(error) => {
            let _ = sqlx::query("UPDATE source_documents SET extraction_status='failure', last_error=?, updated_at=CURRENT_TIMESTAMP WHERE id=?").bind(&error).bind(&document_id).execute(&pool).await;
            Err(StatusCode::BAD_GATEWAY)
        }
    }
}

pub async fn discover_month(
    Path(month_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<DiscoveryResult>, StatusCode> {
    let row = sqlx::query_as::<_, (String, String)>("SELECT fm.provider, fm.source_url FROM feed_months fm JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year JOIN feeds f ON f.id=fm.feed_id WHERE fm.id = ? AND fm.active=1 AND fy.active=1 AND f.active=1")
        .bind(&month_id).fetch_optional(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?.ok_or(StatusCode::NOT_FOUND)?;
    let files = if row.1.starts_with("fixture://") {
        crate::providers::local_fixtures::list_root_pdfs(&row.1).await
    } else {
        match row.0.as_str() {
            "yandex_disk" => crate::providers::yandex_disk::list_root_pdfs(&row.1)
                .await
                .map_err(|e| e.to_string()),
            "mail_cloud" => crate::providers::mail_cloud::list_root_pdfs(&row.1)
                .await
                .map_err(|e| e.to_string()),
            _ => return Err(StatusCode::BAD_REQUEST),
        }
    }
    .map_err(|_| StatusCode::BAD_GATEWAY)?;
    for file in &files {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO source_documents (id, month_id, provider_file_id, source_path, filename, source_url, content_fingerprint, size_bytes, provider_modified_at) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?) ON CONFLICT(month_id, source_path) DO UPDATE SET provider_file_id=excluded.provider_file_id, filename=excluded.filename, source_url=excluded.source_url, content_fingerprint=excluded.content_fingerprint, size_bytes=excluded.size_bytes, provider_modified_at=excluded.provider_modified_at, updated_at=CURRENT_TIMESTAMP")
            .bind(id).bind(&month_id).bind(&file.provider_id).bind(&file.path).bind(&file.filename).bind(&file.public_url).bind(&file.fingerprint).bind(file.size as i64).bind(&file.modified)
            .execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }
    Ok(Json(DiscoveryResult {
        discovered_pdfs: files.len(),
    }))
}

pub async fn create_year(
    Path(feed_id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(input): Json<YearInput>,
) -> Result<(StatusCode, Json<Year>), StatusCode> {
    if !valid_year(input.year) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let result = sqlx::query("INSERT INTO feed_years (feed_id, year) SELECT id, ? FROM feeds WHERE id = ? AND active = 1")
        .bind(input.year).bind(&feed_id).execute(&pool).await.map_err(|_| StatusCode::CONFLICT)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok((
        StatusCode::CREATED,
        Json(Year {
            feed_id,
            year: input.year,
            active: true,
        }),
    ))
}

#[derive(Deserialize)]
pub struct YearInput {
    pub year: i64,
}

pub async fn create_month(
    Path((feed_id, year, month)): Path<(String, i64, i64)>,
    State(pool): State<SqlitePool>,
    Json(input): Json<MonthInput>,
) -> Result<(StatusCode, Json<Month>), StatusCode> {
    if !valid_month(month) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let provider = crate::providers::validate_source_url(&input.source_url)
        .map_err(|_| StatusCode::BAD_REQUEST)?;
    let id = uuid::Uuid::new_v4().to_string();
    let result = sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url) SELECT ?, y.feed_id, y.year, ?, ?, ? FROM feed_years y JOIN feeds f ON f.id=y.feed_id WHERE y.feed_id = ? AND y.year = ? AND y.active=1 AND f.active=1")
        .bind(&id).bind(month).bind(provider).bind(input.source_url).bind(&feed_id).bind(year)
        .execute(&pool).await.map_err(|_| StatusCode::CONFLICT)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    let record = sqlx::query_as::<_, Month>("SELECT id, feed_id, year, month, provider, source_url, active, last_attempt_at, next_refresh_at, last_result, last_error FROM feed_months WHERE id = ?")
        .bind(id).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(record)))
}

pub async fn list_years(
    Path(feed_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Year>>, StatusCode> {
    sqlx::query_as::<_, Year>(
        "SELECT feed_id, year, active FROM feed_years WHERE feed_id = ? ORDER BY year DESC",
    )
    .bind(feed_id)
    .fetch_all(&pool)
    .await
    .map(Json)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn list_months(
    Path((feed_id, year)): Path<(String, i64)>,
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<Month>>, StatusCode> {
    sqlx::query_as::<_, Month>("SELECT id, feed_id, year, month, provider, source_url, active, last_attempt_at, next_refresh_at, last_result, last_error FROM feed_months WHERE feed_id = ? AND year = ? ORDER BY month")
        .bind(feed_id).bind(year).fetch_all(&pool).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Feed>>, StatusCode> {
    sqlx::query_as::<_, Feed>("SELECT id, display_name, description, active, refresh_interval_seconds, extraction_engine FROM feeds ORDER BY display_name")
        .fetch_all(&pool).await.map(Json).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(input): Json<FeedInput>,
) -> Result<(StatusCode, Json<Feed>), StatusCode> {
    if !valid_feed_input(&input) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let id = uuid::Uuid::new_v4().to_string();
    let engine = input
        .extraction_engine
        .as_ref()
        .and_then(|value| value.as_deref());
    sqlx::query("INSERT INTO feeds (id, slug, display_name, description, refresh_interval_seconds, extraction_engine) VALUES (?, ?, ?, ?, ?, ?)")
        .bind(&id).bind(&id).bind(input.display_name.trim()).bind(input.description.trim()).bind(input.refresh_interval_seconds).bind(engine)
        .execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    let feed = sqlx::query_as::<_, Feed>("SELECT id, display_name, description, active, refresh_interval_seconds, extraction_engine FROM feeds WHERE id = ?")
        .bind(id).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok((StatusCode::CREATED, Json(feed)))
}

pub async fn update(
    Path(id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(input): Json<FeedInput>,
) -> Result<Json<Feed>, StatusCode> {
    if !valid_feed_input(&input) {
        return Err(StatusCode::BAD_REQUEST);
    }
    let result = if let Some(engine) = input.extraction_engine.as_ref() {
        sqlx::query("UPDATE feeds SET display_name = ?, description = ?, refresh_interval_seconds = ?, extraction_engine = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND active = 1")
            .bind(input.display_name.trim()).bind(input.description.trim()).bind(input.refresh_interval_seconds).bind(engine.as_deref()).bind(&id)
            .execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    } else {
        // Older clients omit the field; keep any existing override intact.
        sqlx::query("UPDATE feeds SET display_name = ?, description = ?, refresh_interval_seconds = ?, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND active = 1")
            .bind(input.display_name.trim()).bind(input.description.trim()).bind(input.refresh_interval_seconds).bind(&id)
            .execute(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    };
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    let feed = sqlx::query_as::<_, Feed>("SELECT id, display_name, description, active, refresh_interval_seconds, extraction_engine FROM feeds WHERE id = ?")
        .bind(id).fetch_one(&pool).await.map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(feed))
}

pub async fn deactivate(
    Path(id): Path<String>,
    State(pool): State<SqlitePool>,
) -> impl IntoResponse {
    match sqlx::query(
        "UPDATE feeds SET active = 0, updated_at = CURRENT_TIMESTAMP WHERE id = ? AND active = 1",
    )
    .bind(id)
    .execute(&pool)
    .await
    {
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
        .bind(active)
        .bind(id)
        .execute(pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn permanently_delete_feed(
    Path(id): Path<String>,
    State(pool): State<SqlitePool>,
) -> StatusCode {
    delete_scope(&pool, "DELETE FROM feeds WHERE id=?", vec![id]).await
}

pub async fn archive_year(
    Path((feed_id, year)): Path<(String, i64)>,
    State(pool): State<SqlitePool>,
) -> StatusCode {
    set_year_active(&pool, &feed_id, year, false).await
}

pub async fn restore_year(
    Path((feed_id, year)): Path<(String, i64)>,
    State(pool): State<SqlitePool>,
) -> StatusCode {
    set_year_active(&pool, &feed_id, year, true).await
}

async fn set_year_active(pool: &SqlitePool, feed_id: &str, year: i64, active: bool) -> StatusCode {
    match sqlx::query("UPDATE feed_years SET active=? WHERE feed_id=? AND year=?")
        .bind(active)
        .bind(feed_id)
        .bind(year)
        .execute(pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn permanently_delete_year(
    Path((feed_id, year)): Path<(String, i64)>,
    State(pool): State<SqlitePool>,
) -> StatusCode {
    delete_scope(
        &pool,
        "DELETE FROM feed_years WHERE feed_id=? AND year=?",
        vec![feed_id, year.to_string()],
    )
    .await
}

pub async fn archive_month(Path(id): Path<String>, State(pool): State<SqlitePool>) -> StatusCode {
    set_month_active(&pool, &id, false).await
}

pub async fn restore_month(Path(id): Path<String>, State(pool): State<SqlitePool>) -> StatusCode {
    set_month_active(&pool, &id, true).await
}

async fn set_month_active(pool: &SqlitePool, id: &str, active: bool) -> StatusCode {
    match sqlx::query("UPDATE feed_months SET active=? WHERE id=?")
        .bind(active)
        .bind(id)
        .execute(pool)
        .await
    {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

pub async fn permanently_delete_month(
    Path(id): Path<String>,
    State(pool): State<SqlitePool>,
) -> StatusCode {
    delete_scope(&pool, "DELETE FROM feed_months WHERE id=?", vec![id]).await
}

async fn delete_scope(pool: &SqlitePool, query: &str, values: Vec<String>) -> StatusCode {
    let mut statement = sqlx::query(query);
    for value in values {
        statement = statement.bind(value);
    }
    match statement.execute(pool).await {
        Ok(result) if result.rows_affected() > 0 => StatusCode::NO_CONTENT,
        Ok(_) => StatusCode::NOT_FOUND,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn simple_pdf() -> Vec<u8> {
        let content = "BT /F1 12 Tf 50 700 Td (Hello Marker) Tj ET";
        let objects = [
            "<< /Type /Catalog /Pages 2 0 R >>".to_owned(),
            "<< /Type /Pages /Kids [3 0 R] /Count 1 >>".to_owned(),
            "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 612 792] /Resources << /Font << /F1 4 0 R >> >> /Contents 5 0 R >>".to_owned(),
            "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica >>".to_owned(),
            format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len() + 1),
        ];
        let mut output = b"%PDF-1.4\n".to_vec();
        let mut offsets = Vec::new();
        for (index, object) in objects.iter().enumerate() {
            offsets.push(output.len());
            output.extend(format!("{} 0 obj\n{object}\nendobj\n", index + 1).as_bytes());
        }
        let xref = output.len();
        output.extend(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
        for offset in offsets {
            output.extend(format!("{offset:010} 00000 n \n").as_bytes());
        }
        output.extend(
            format!(
                "trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n",
                objects.len() + 1
            )
            .as_bytes(),
        );
        output
    }

    async fn marker_worker() -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let response = serde_json::json!({
            "page_count": 1,
            "pages": [{"lines": [{
                "runs": [{"text": "Marker article", "bold": false, "italic": false}],
                "size": 12.0,
                "gap_before": 0.0,
                "left": 50.0
            }]}]
        });
        tokio::spawn(async move {
            axum::serve(
                listener,
                axum::Router::new().route(
                    "/extract",
                    axum::routing::post(move || {
                        let response = response.clone();
                        async move { axum::Json(response) }
                    }),
                ),
            )
            .await
            .unwrap();
        });
        format!("http://{address}")
    }

    #[test]
    fn feed_input_requires_a_nonblank_name_and_positive_interval() {
        let input = |display_name: &str, refresh_interval_seconds| FeedInput {
            display_name: display_name.into(),
            description: String::new(),
            refresh_interval_seconds,
            extraction_engine: None,
        };
        assert!(valid_feed_input(&input(" Ada ", None)));
        assert!(valid_feed_input(&input("Ada", Some(1))));
        assert!(!valid_feed_input(&input(" \t\n ", None)));
        assert!(!valid_feed_input(&input("Ada", Some(0))));
        assert!(!valid_feed_input(&input("Ada", Some(-1))));
    }

    #[test]
    fn feed_input_accepts_only_inherit_pdfium_or_marker_engines() {
        let input = |engine: Option<Option<&str>>| FeedInput {
            display_name: "Ada".into(),
            description: String::new(),
            refresh_interval_seconds: None,
            extraction_engine: engine.map(|value| value.map(str::to_owned)),
        };
        assert!(valid_feed_input(&input(None)));
        assert!(valid_feed_input(&input(Some(None))));
        assert!(valid_feed_input(&input(Some(Some("pdfium")))));
        assert!(valid_feed_input(&input(Some(Some("marker")))));
        assert!(!valid_feed_input(&input(Some(Some("marker-pdf")))));
        assert!(!valid_feed_input(&input(Some(Some("")))));
    }

    #[test]
    fn omitted_engine_field_is_distinct_from_explicit_inheritance() {
        let omitted: FeedInput = serde_json::from_str(r#"{"display_name":"Ada"}"#).unwrap();
        let inherited: FeedInput =
            serde_json::from_str(r#"{"display_name":"Ada","extraction_engine":null}"#).unwrap();
        let marker: FeedInput =
            serde_json::from_str(r#"{"display_name":"Ada","extraction_engine":"marker"}"#).unwrap();
        assert_eq!(omitted.extraction_engine, None);
        assert_eq!(inherited.extraction_engine, Some(None));
        assert_eq!(marker.extraction_engine, Some(Some("marker".into())));
    }

    #[test]
    fn hierarchy_input_accepts_only_supported_calendar_ranges() {
        for year in [1, 2026, 9999] {
            assert!(valid_year(year));
        }
        for year in [i64::MIN, 0, 10000, i64::MAX] {
            assert!(!valid_year(year));
        }
        for month in [1, 6, 12] {
            assert!(valid_month(month));
        }
        for month in [i64::MIN, 0, 13, i64::MAX] {
            assert!(!valid_month(month));
        }
    }

    #[tokio::test]
    async fn management_lifecycle_persists_updates_and_cascades_deletions() {
        let (pool, path) = crate::test_support::database().await;
        let state = || State(pool.clone());
        let (status, Json(feed)) = create(
            state(),
            Json(FeedInput {
                display_name: "  Test feed  ".into(),
                description: "  description  ".into(),
                refresh_interval_seconds: Some(1800),
                extraction_engine: Some(Some("marker".into())),
            }),
        )
        .await
        .unwrap();
        assert_eq!(status, StatusCode::CREATED);
        assert_eq!(feed.display_name, "Test feed");
        assert_eq!(feed.description, "description");
        assert_eq!(feed.extraction_engine.as_deref(), Some("marker"));

        let Json(updated) = update(
            Path(feed.id.clone()),
            state(),
            Json(FeedInput {
                display_name: "Test feed renamed".into(),
                description: "description".into(),
                refresh_interval_seconds: Some(1800),
                extraction_engine: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(
            updated.extraction_engine.as_deref(),
            Some("marker"),
            "older clients that omit the field must not clear the override"
        );

        let invalid = update(
            Path(feed.id.clone()),
            state(),
            Json(FeedInput {
                display_name: "Test feed".into(),
                description: String::new(),
                refresh_interval_seconds: None,
                extraction_engine: Some(Some("unsupported".into())),
            }),
        )
        .await;
        assert_eq!(invalid.err(), Some(StatusCode::BAD_REQUEST));

        let (_, Json(year)) = create_year(
            Path(feed.id.clone()),
            state(),
            Json(YearInput { year: 2026 }),
        )
        .await
        .unwrap();
        assert_eq!(year.year, 2026);

        let (month_status, Json(month)) = create_month(
            Path((feed.id.clone(), 2026, 11)),
            state(),
            Json(MonthInput {
                source_url: "https://disk.yandex.ru/d/test-token".into(),
            }),
        )
        .await
        .unwrap();
        assert_eq!(month_status, StatusCode::CREATED);
        assert_eq!(month.provider, "yandex_disk");

        let duplicate_month = create_month(
            Path((feed.id.clone(), 2026, 11)),
            state(),
            Json(MonthInput {
                source_url: "https://disk.yandex.ru/d/test-token".into(),
            }),
        )
        .await;
        assert!(matches!(duplicate_month, Err(StatusCode::CONFLICT)));

        sqlx::query("INSERT INTO source_documents (id, month_id, source_path, filename, source_url, content_fingerprint, size_bytes) VALUES ('doc-1', ?, '/article.pdf', 'article.pdf', 'https://disk.yandex.ru/file.pdf', 'fp-1', 10)")
            .bind(&month.id).execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO cached_content (document_id, content_fingerprint, extracted_text, first_page_text, sanitized_html, page_count) VALUES ('doc-1', 'fp-1', 'text', 'text', '<p>text</p>', 1)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO refresh_jobs (id, scope, feed_id, year, month_id, force, status) VALUES ('job-1', 'month', ?, 2026, ?, 0, 'queued')")
            .bind(&feed.id).bind(&month.id).execute(&pool).await.unwrap();

        assert_eq!(
            set_promotion_override(
                Path("doc-1".into()),
                state(),
                Json(PromotionOverride {
                    override_value: Some("include".into())
                }),
            )
            .await
            .unwrap(),
            StatusCode::NO_CONTENT,
        );
        let override_value: Option<String> = sqlx::query_scalar(
            "SELECT promotional_override FROM source_documents WHERE id='doc-1'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(override_value.as_deref(), Some("include"));
        assert_eq!(
            set_promotion_override(
                Path("doc-1".into()),
                state(),
                Json(PromotionOverride {
                    override_value: Some("invalid".into())
                }),
            )
            .await
            .unwrap_err(),
            StatusCode::BAD_REQUEST,
        );
        assert_eq!(
            set_promotion_override(
                Path("missing".into()),
                state(),
                Json(PromotionOverride {
                    override_value: None
                }),
            )
            .await
            .unwrap_err(),
            StatusCode::NOT_FOUND,
        );

        assert_eq!(
            archive_month(Path(month.id.clone()), state()).await,
            StatusCode::NO_CONTENT
        );
        let active: i64 = sqlx::query_scalar("SELECT active FROM feed_months WHERE id=?")
            .bind(&month.id)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(active, 0);
        assert_eq!(
            restore_month(Path(month.id.clone()), state()).await,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            archive_year(Path((feed.id.clone(), 2026)), state()).await,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            restore_year(Path((feed.id.clone(), 2026)), state()).await,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            archive_feed(Path(feed.id.clone()), state()).await,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            restore_feed(Path(feed.id.clone()), state()).await,
            StatusCode::NO_CONTENT
        );

        assert_eq!(
            permanently_delete_month(Path(month.id), state()).await,
            StatusCode::NO_CONTENT
        );
        let documents: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM source_documents WHERE id='doc-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let cached: i64 =
            sqlx::query_scalar("SELECT COUNT(*) FROM cached_content WHERE document_id='doc-1'")
                .fetch_one(&pool)
                .await
                .unwrap();
        let jobs: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM refresh_jobs WHERE id='job-1'")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!((documents, cached, jobs), (0, 0, 0));
        assert_eq!(
            permanently_delete_year(Path((feed.id.clone(), 2026)), state()).await,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            permanently_delete_feed(Path(feed.id.clone()), state()).await,
            StatusCode::NO_CONTENT
        );
        assert_eq!(
            permanently_delete_feed(Path(feed.id), state()).await,
            StatusCode::NOT_FOUND
        );

        crate::test_support::cleanup(pool, path).await;
    }

    #[tokio::test]
    async fn engine_selection_drives_cache_identity_and_failures_keep_last_success() {
        use crate::extraction::{ExtractionEngine, ExtractionSettings, MarkerSettings};
        use sha2::{Digest, Sha256};
        use std::sync::Arc;

        let (pool, path) = crate::test_support::database().await;
        let fixture_root =
            std::env::temp_dir().join(format!("kpapers-engine-fixtures-{}", uuid::Uuid::new_v4()));
        let folder = fixture_root.join("engine");
        std::fs::create_dir_all(&folder).unwrap();
        let pdf = simple_pdf();
        std::fs::write(folder.join("article.pdf"), &pdf).unwrap();
        // SAFETY: this test is the only user of the fixture environment in the
        // unit suite; E2E sets the same variables in its own process.
        unsafe {
            std::env::set_var("E2E_FIXTURE_MODE", "true");
            std::env::set_var("E2E_FIXTURE_DIR", &fixture_root);
        }

        sqlx::query("INSERT INTO feeds (id, slug, display_name, extraction_engine) VALUES ('feed', 'feed', 'Feed', 'marker')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO feed_years (feed_id, year) VALUES ('feed', 2026)")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url) VALUES ('month', 'feed', 2026, 11, 'yandex_disk', 'fixture://engine/')")
            .execute(&pool).await.unwrap();
        let fingerprint = format!("{:x}", Sha256::digest(&pdf));
        sqlx::query("INSERT INTO source_documents (id, month_id, source_path, filename, source_url, content_fingerprint, size_bytes) VALUES ('doc', 'month', 'article.pdf', 'article.pdf', 'https://fixture.invalid/engine/article.pdf', ?, ?)")
            .bind(&fingerprint).bind(pdf.len() as i64).execute(&pool).await.unwrap();

        let endpoint = marker_worker().await;
        let mut settings = ExtractionSettings {
            default_engine: ExtractionEngine::Pdfium,
            marker: MarkerSettings {
                endpoint: Some(endpoint.clone()),
                timeout: std::time::Duration::from_secs(2),
                max_response_bytes: 64 * 1024,
                permits: Arc::new(tokio::sync::Semaphore::new(1)),
            },
        };
        macro_rules! extract {
            ($force:expr) => {
                extract_document(
                    Path("doc".into()),
                    State(pool.clone()),
                    Extension(settings.clone()),
                    Query(ForceQuery {
                        force: Some($force),
                    }),
                )
            };
        }

        let first = extract!(false).await;
        if let Err(status) = first {
            let last_error: Option<String> =
                sqlx::query_scalar("SELECT last_error FROM source_documents WHERE id='doc'")
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            panic!("first extraction failed with {status}: {last_error:?}");
        }
        let Json(result) = first.unwrap();
        assert_eq!(result.status, "success");
        assert_eq!(
            sqlx::query_scalar::<_, String>(
                "SELECT content_fingerprint FROM cached_content WHERE document_id='doc'"
            )
            .fetch_one(&pool)
            .await
            .unwrap(),
            crate::pdf_processing::cache_fingerprint_for(ExtractionEngine::Marker, &fingerprint)
        );

        let Json(result) = extract!(false).await.unwrap();
        assert_eq!(
            result.status, "cached",
            "unchanged source and engine reuse the cache"
        );

        sqlx::query("UPDATE feeds SET extraction_engine='pdfium' WHERE id='feed'")
            .execute(&pool)
            .await
            .unwrap();
        let pdfium_engine = settings.effective_engine(Some("pdfium"));
        assert_eq!(pdfium_engine, ExtractionEngine::Pdfium);
        assert_ne!(
            crate::pdf_processing::cache_fingerprint_for(pdfium_engine, &fingerprint),
            crate::pdf_processing::cache_fingerprint_for(ExtractionEngine::Marker, &fingerprint),
            "a feed override must not reuse the other engine's cache"
        );

        sqlx::query("UPDATE feeds SET extraction_engine=NULL WHERE id='feed'")
            .execute(&pool)
            .await
            .unwrap();
        assert_eq!(settings.effective_engine(None), ExtractionEngine::Pdfium);
        assert_ne!(
            crate::pdf_processing::cache_fingerprint_for(
                settings.effective_engine(None),
                &fingerprint
            ),
            crate::pdf_processing::cache_fingerprint_for(ExtractionEngine::Marker, &fingerprint),
            "the global default must participate in cache identity"
        );
        settings.default_engine = ExtractionEngine::Marker;
        assert_eq!(extract!(false).await.unwrap().0.status, "cached");
        assert_eq!(
            extract!(true).await.unwrap().0.status,
            "success",
            "a forced refresh bypasses the matching cache"
        );

        let cached_text: String =
            sqlx::query_scalar("SELECT extracted_text FROM cached_content WHERE document_id='doc'")
                .fetch_one(&pool)
                .await
                .unwrap();
        settings.marker.endpoint = Some("http://127.0.0.1:1".into());
        settings.marker.timeout = std::time::Duration::from_millis(100);
        let error = extract!(true)
            .await
            .err()
            .expect("unavailable worker must fail extraction");
        assert_eq!(error, StatusCode::BAD_GATEWAY);
        let (status, text, cache_fingerprint): (String, String, String) = sqlx::query_as(
            "SELECT extraction_status, (SELECT extracted_text FROM cached_content WHERE document_id='doc'), (SELECT content_fingerprint FROM cached_content WHERE document_id='doc') FROM source_documents WHERE id='doc'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(status, "failure");
        assert_eq!(text, cached_text);
        assert_eq!(
            cache_fingerprint,
            crate::pdf_processing::cache_fingerprint_for(ExtractionEngine::Marker, &fingerprint)
        );

        let rss = crate::rss::render(Path("feed".into()), State(pool.clone())).await;
        let rss_body = axum::body::to_bytes(rss.into_body(), 1024 * 1024)
            .await
            .unwrap();
        assert!(String::from_utf8_lossy(&rss_body).contains(&cached_text));
        let public_download = crate::rss::download(Path("doc".into()), State(pool.clone())).await;
        assert_eq!(public_download.status(), StatusCode::TEMPORARY_REDIRECT);

        // Reusing the same good cache clears the failure state without
        // repeating the unavailable Marker request.
        settings.marker.endpoint = Some(endpoint);
        let Json(recovered) = extract!(false).await.unwrap();
        assert_eq!(recovered.status, "cached");
        let (status, last_error): (String, Option<String>) = sqlx::query_as(
            "SELECT extraction_status, last_error FROM source_documents WHERE id='doc'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(status, "success");
        assert_eq!(last_error, None);

        let _ = std::fs::remove_dir_all(&fixture_root);
        crate::test_support::cleanup(pool, path).await;
    }

    #[tokio::test]
    async fn overview_summary_counts_active_folders_and_documents_needing_processing() {
        let (pool, path) = crate::test_support::database().await;
        sqlx::query("INSERT INTO feeds (id, slug, display_name) VALUES ('active-feed', 'active-feed', 'Active')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO feeds (id, slug, display_name, active) VALUES ('archived-feed', 'archived-feed', 'Archived', 0)")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO feed_years (feed_id, year) VALUES ('active-feed', 2026), ('active-feed', 2025), ('archived-feed', 2026)")
            .execute(&pool).await.unwrap();
        sqlx::query("UPDATE feed_years SET active=0 WHERE feed_id='active-feed' AND year=2025")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO feed_months (id, feed_id, year, month, provider, source_url) VALUES \
            ('active-month', 'active-feed', 2026, 1, 'yandex_disk', 'https://disk.yandex.ru/d/active'), \
            ('archived-month', 'active-feed', 2026, 2, 'yandex_disk', 'https://disk.yandex.ru/d/archived'), \
            ('archived-year-month', 'active-feed', 2025, 3, 'yandex_disk', 'https://disk.yandex.ru/d/year'), \
            ('inactive-feed-month', 'archived-feed', 2026, 4, 'yandex_disk', 'https://disk.yandex.ru/d/feed')")
            .execute(&pool).await.unwrap();
        sqlx::query("UPDATE feed_months SET active=0 WHERE id='archived-month'")
            .execute(&pool)
            .await
            .unwrap();
        sqlx::query("INSERT INTO source_documents (id, month_id, source_path, filename, source_url, content_fingerprint, size_bytes, extraction_status) VALUES \
            ('pending-doc', 'active-month', '/pending.pdf', 'pending.pdf', 'https://example.test/pending.pdf', 'pending', 10, 'pending'), \
            ('failed-doc', 'active-month', '/failed.pdf', 'failed.pdf', 'https://example.test/failed.pdf', 'failed', 10, 'failure'), \
            ('success-doc', 'active-month', '/success.pdf', 'success.pdf', 'https://example.test/success.pdf', 'success', 10, 'success'), \
            ('archived-doc', 'archived-month', '/archived.pdf', 'archived.pdf', 'https://example.test/archived.pdf', 'archived', 10, 'failure')")
            .execute(&pool).await.unwrap();
        sqlx::query("INSERT INTO source_documents (id, month_id, source_path, filename, source_url, content_fingerprint, size_bytes, extraction_status) VALUES \
            ('archived-year-doc', 'archived-year-month', '/year.pdf', 'year.pdf', 'https://example.test/year.pdf', 'year', 10, 'failure'), \
            ('archived-feed-doc', 'inactive-feed-month', '/feed.pdf', 'feed.pdf', 'https://example.test/feed.pdf', 'feed', 10, 'pending')")
            .execute(&pool).await.unwrap();
        sqlx::query(
            "UPDATE source_documents SET promotional_override='include' WHERE id='failed-doc'",
        )
        .execute(&pool)
        .await
        .unwrap();

        let Json(summary) = overview_summary(State(pool.clone())).await.unwrap();
        assert_eq!(summary.source_folder_count, 1);
        assert_eq!(summary.needs_review_count, 2);

        let Json(active_queue) =
            list_review_queue(State(pool.clone()), Query(ReviewQueueFilter::default()))
                .await
                .unwrap();
        assert_eq!(
            active_queue
                .iter()
                .map(|document| document.id.as_str())
                .collect::<Vec<_>>(),
            vec!["failed-doc", "pending-doc"]
        );
        assert!(active_queue.iter().all(|document| !document.archived));
        assert_eq!(active_queue[0].feed_name, "Active");
        assert_eq!(active_queue[0].year, 2026);
        assert_eq!(active_queue[0].month, 1);
        assert_eq!(active_queue[0].month_id, "active-month");

        let Json(archived_queue) = list_review_queue(
            State(pool.clone()),
            Query(ReviewQueueFilter { archived: true }),
        )
        .await
        .unwrap();
        assert_eq!(
            archived_queue
                .iter()
                .map(|document| document.id.as_str())
                .collect::<Vec<_>>(),
            vec!["archived-doc", "archived-year-doc", "archived-feed-doc"]
        );
        assert!(archived_queue.iter().all(|document| document.archived));

        assert_eq!(
            restore_month(Path("archived-month".into()), State(pool.clone())).await,
            StatusCode::NO_CONTENT
        );
        let Json(restored_queue) =
            list_review_queue(State(pool.clone()), Query(ReviewQueueFilter::default()))
                .await
                .unwrap();
        assert!(
            restored_queue
                .iter()
                .any(|document| document.id == "archived-doc")
        );
        let Json(remaining_archived) = list_review_queue(
            State(pool.clone()),
            Query(ReviewQueueFilter { archived: true }),
        )
        .await
        .unwrap();
        assert!(
            !remaining_archived
                .iter()
                .any(|document| document.id == "archived-doc")
        );
        let Json(restored_summary) = overview_summary(State(pool.clone())).await.unwrap();
        assert_eq!(
            restored_summary.needs_review_count,
            restored_queue.len() as i64
        );
        assert_eq!(restored_summary.needs_review_count, 3);

        crate::test_support::cleanup(pool, path).await;
    }
}
