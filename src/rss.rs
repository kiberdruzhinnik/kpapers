use axum::{
    extract::{Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use sqlx::{FromRow, SqlitePool};

#[derive(FromRow)]
struct Feed {
    id: String,
    display_name: String,
    description: String,
}
#[derive(FromRow)]
struct Item {
    id: String,
    filename: String,
    publication_date: Option<String>,
    provider_modified_at: Option<String>,
    discovered_at: String,
    sanitized_html: String,
}

pub async fn render(Path(feed_id): Path<String>, State(pool): State<SqlitePool>) -> Response {
    let feed = match sqlx::query_as::<_, Feed>(
        "SELECT id, display_name, description FROM feeds WHERE id = ? AND active = 1",
    )
    .bind(&feed_id)
    .fetch_optional(&pool)
    .await
    {
        Ok(Some(feed)) => feed,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let items = match sqlx::query_as::<_, Item>("SELECT sd.id, sd.filename, sd.publication_date, sd.provider_modified_at, sd.discovered_at, cc.sanitized_html FROM source_documents sd JOIN feed_months fm ON fm.id=sd.month_id JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year JOIN cached_content cc ON cc.document_id=sd.id WHERE fm.feed_id=? AND fm.active=1 AND fy.active=1 AND CASE COALESCE(sd.promotional_override, sd.promotional_classification) WHEN 'include' THEN 'publish' WHEN 'exclude' THEN 'promotional' ELSE COALESCE(sd.promotional_override, sd.promotional_classification) END='publish' ORDER BY COALESCE(sd.publication_date, sd.provider_modified_at, sd.discovered_at) DESC")
        .bind(&feed.id).fetch_all(&pool).await {
            Ok(items) => items,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
    let base = std::env::var("PUBLIC_BASE_URL").unwrap_or_else(|_| "http://localhost".into());
    let self_url = format!("{}/feeds/{}", base.trim_end_matches('/'), feed.id);
    let now = Utc::now().to_rfc2822();
    let mut xml = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<rss version=\"2.0\" xmlns:atom=\"http://www.w3.org/2005/Atom\" xmlns:content=\"http://purl.org/rss/1.0/modules/content/\"><channel><title>{}</title><link>{}</link><description>{}</description><atom:link href=\"{}\" rel=\"self\" type=\"application/rss+xml\"/><lastBuildDate>{}</lastBuildDate>",
        xml_escape(&feed.display_name),
        xml_escape(&self_url),
        xml_escape(&feed.description),
        xml_escape(&self_url),
        now
    );
    for item in items {
        let title = item
            .filename
            .strip_suffix(".pdf")
            .or_else(|| item.filename.strip_suffix(".PDF"))
            .unwrap_or(&item.filename);
        let date = item_date(&item).to_rfc2822();
        let download_url = format!(
            "{}/documents/{}/download",
            base.trim_end_matches('/'),
            item.id
        );
        xml.push_str(&format!("<item><title>{}</title><link>{}</link><guid isPermaLink=\"false\">{}</guid><pubDate>{}</pubDate><description><![CDATA[{}]]></description><content:encoded><![CDATA[{}]]></content:encoded></item>", xml_escape(title), xml_escape(&download_url), xml_escape(&item.id), xml_escape(&date), item.sanitized_html, item.sanitized_html));
    }
    xml.push_str("</channel></rss>");
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/rss+xml; charset=utf-8")
        .body(xml.into())
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
}

pub async fn download(Path(document_id): Path<String>, State(pool): State<SqlitePool>) -> Response {
    document_redirect(pool, document_id, true).await
}

pub async fn review_download(
    Path(document_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Response {
    document_redirect(pool, document_id, false).await
}

async fn document_redirect(pool: SqlitePool, document_id: String, public_only: bool) -> Response {
    let row = if public_only {
        sqlx::query_as::<_, (String, String, String)>(
            "SELECT fm.provider, fm.source_url, sd.source_path FROM source_documents sd JOIN feed_months fm ON fm.id=sd.month_id JOIN feed_years fy ON fy.feed_id=fm.feed_id AND fy.year=fm.year JOIN feeds f ON f.id=fm.feed_id JOIN cached_content cc ON cc.document_id=sd.id WHERE sd.id=? AND f.active=1 AND fy.active=1 AND fm.active=1 AND CASE COALESCE(sd.promotional_override, sd.promotional_classification) WHEN 'include' THEN 'publish' WHEN 'exclude' THEN 'promotional' ELSE COALESCE(sd.promotional_override, sd.promotional_classification) END='publish'",
        ).bind(&document_id).fetch_optional(&pool).await
    } else {
        sqlx::query_as::<_, (String, String, String)>(
            "SELECT fm.provider, fm.source_url, sd.source_path FROM source_documents sd JOIN feed_months fm ON fm.id=sd.month_id WHERE sd.id=?",
        ).bind(&document_id).fetch_optional(&pool).await
    };
    let row = match row {
        Ok(Some(row)) => row,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let files = if row.1.starts_with("fixture://") {
        crate::providers::local_fixtures::list_root_pdfs(&row.1)
            .await
            .map_err(|e| e.to_string())
    } else {
        match row.0.as_str() {
            "yandex_disk" => crate::providers::yandex_disk::list_root_pdfs(&row.1)
                .await
                .map_err(|e| e.to_string()),
            "mail_cloud" => crate::providers::mail_cloud::list_root_pdfs(&row.1)
                .await
                .map_err(|e| e.to_string()),
            _ => return StatusCode::BAD_GATEWAY.into_response(),
        }
    };
    let files = match files {
        Ok(files) => files,
        Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
    };
    let Some(file) = files.iter().find(|file| file.path == row.2) else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let target = if row.1.starts_with("fixture://") {
        match url::Url::parse(&file.public_url) {
            Ok(url) if url.scheme() == "https" && url.host_str() == Some("fixture.invalid") => url,
            _ => return StatusCode::BAD_GATEWAY.into_response(),
        }
    } else {
        match crate::providers::validate_download_url(&row.0, &file.download_url) {
            Ok(url) => url,
            Err(_) => return StatusCode::BAD_GATEWAY.into_response(),
        }
    };
    Redirect::temporary(target.as_str()).into_response()
}

fn item_date(item: &Item) -> DateTime<Utc> {
    if let Some(date) = parse_publication_date(&item.filename)
        .or_else(|| item.publication_date.as_deref().and_then(parse_date))
    {
        return date;
    }
    if let Some(modified) = item.provider_modified_at.as_deref() {
        if let Ok(date) = DateTime::parse_from_rfc3339(modified) {
            return date.with_timezone(&Utc);
        }
        if let Ok(seconds) = modified.parse::<i64>()
            && let Some(date) = Utc.timestamp_opt(seconds, 0).single()
        {
            return date;
        }
    }
    NaiveDateTime::parse_from_str(&item.discovered_at, "%Y-%m-%d %H:%M:%S")
        .ok()
        .map(|date| Utc.from_utc_datetime(&date))
        .unwrap_or_else(Utc::now)
}

pub(crate) fn parse_publication_date(filename: &str) -> Option<DateTime<Utc>> {
    let bytes = filename.as_bytes();
    for start in 0..bytes.len() {
        for (len, fmt) in [(10, "%Y-%m-%d"), (10, "%Y.%m.%d"), (8, "%Y%m%d")] {
            if start + len > bytes.len()
                || !filename.is_char_boundary(start)
                || !filename.is_char_boundary(start + len)
            {
                continue;
            }
            if let Ok(date) = NaiveDate::parse_from_str(&filename[start..start + len], fmt) {
                return date.and_hms_opt(0, 0, 0).map(|d| Utc.from_utc_datetime(&d));
            }
        }
    }
    None
}

pub(crate) fn parse_date(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|d| d.with_timezone(&Utc))
        .or_else(|| {
            NaiveDate::parse_from_str(value, "%Y-%m-%d")
                .ok()
                .and_then(|d| d.and_hms_opt(0, 0, 0))
                .map(|d| Utc.from_utc_datetime(&d))
        })
}

fn xml_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn publication_date_parser_handles_supported_formats_and_invalid_dates() {
        assert_eq!(
            parse_publication_date("2026-11-06 article.pdf")
                .unwrap()
                .format("%Y-%m-%d")
                .to_string(),
            "2026-11-06"
        );
        assert_eq!(
            parse_publication_date("issue_2026.11.06.pdf")
                .unwrap()
                .format("%Y-%m-%d")
                .to_string(),
            "2026-11-06"
        );
        assert_eq!(
            parse_publication_date("20261106 update.pdf")
                .unwrap()
                .format("%Y-%m-%d")
                .to_string(),
            "2026-11-06"
        );
        assert!(parse_publication_date("2026-02-30.pdf").is_none());
        assert!(parse_publication_date("no-date.pdf").is_none());
        assert!(parse_publication_date("📄2026-11-06.pdf").is_some());
    }

    #[test]
    fn xml_escaping_covers_markup_quotes_and_ampersands() {
        assert_eq!(
            xml_escape("<tag a=\"x\">&'"),
            "&lt;tag a=&quot;x&quot;&gt;&amp;&apos;"
        );
    }
}
