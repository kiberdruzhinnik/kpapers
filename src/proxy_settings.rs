use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use crate::proxy::ProxyUrl;

#[derive(Serialize)]
pub struct ProxySettingsView {
    pub configured: bool,
    pub endpoint: Option<String>,
}

#[derive(Serialize)]
pub struct FeedProxySettingsView {
    pub mode: String,
    pub configured: bool,
    pub endpoint: Option<String>,
}

#[derive(Deserialize)]
pub struct ProxyUrlInput {
    pub proxy_url: String,
}

#[derive(Deserialize)]
pub struct FeedProxyInput {
    pub mode: String,
    pub proxy_url: Option<String>,
}

fn view(raw: Option<String>) -> ProxySettingsView {
    let configured = raw.is_some();
    let endpoint = raw.and_then(|raw| ProxyUrl::parse(&raw).ok().map(|proxy| proxy.sanitized()));
    ProxySettingsView {
        configured,
        endpoint,
    }
}

pub async fn get_global(
    State(pool): State<SqlitePool>,
) -> Result<Json<ProxySettingsView>, StatusCode> {
    let raw = sqlx::query_scalar::<_, String>(
        "SELECT value FROM application_settings WHERE key='proxy_url'",
    )
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(view(raw)))
}

pub async fn set_global(
    State(pool): State<SqlitePool>,
    Json(input): Json<ProxyUrlInput>,
) -> Result<Json<ProxySettingsView>, StatusCode> {
    let proxy = ProxyUrl::parse(&input.proxy_url).map_err(|_| StatusCode::BAD_REQUEST)?;
    sqlx::query(
        "INSERT INTO application_settings (key, value) VALUES ('proxy_url', ?) \
         ON CONFLICT(key) DO UPDATE SET value=excluded.value, updated_at=CURRENT_TIMESTAMP",
    )
    .bind(proxy.as_url().as_str())
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(view(Some(proxy.as_url().to_string()))))
}

pub async fn clear_global(State(pool): State<SqlitePool>) -> Result<StatusCode, StatusCode> {
    sqlx::query("DELETE FROM application_settings WHERE key='proxy_url'")
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn get_feed(
    Path(feed_id): Path<String>,
    State(pool): State<SqlitePool>,
) -> Result<Json<FeedProxySettingsView>, StatusCode> {
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT proxy_mode, proxy_url FROM feeds WHERE id=?",
    )
    .bind(feed_id)
    .fetch_optional(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
    .ok_or(StatusCode::NOT_FOUND)?;
    let proxy = view(row.1);
    Ok(Json(FeedProxySettingsView {
        mode: row.0,
        configured: proxy.configured,
        endpoint: proxy.endpoint,
    }))
}

pub async fn set_feed(
    Path(feed_id): Path<String>,
    State(pool): State<SqlitePool>,
    Json(input): Json<FeedProxyInput>,
) -> Result<Json<FeedProxySettingsView>, StatusCode> {
    let proxy_url = match input.mode.as_str() {
        "inherit" | "direct" if input.proxy_url.is_none() => None,
        "inherit" | "direct" => return Err(StatusCode::BAD_REQUEST),
        "custom" => Some(
            ProxyUrl::parse(input.proxy_url.as_deref().ok_or(StatusCode::BAD_REQUEST)?)
                .map_err(|_| StatusCode::BAD_REQUEST)?,
        ),
        _ => return Err(StatusCode::BAD_REQUEST),
    };
    let raw = proxy_url.as_ref().map(|proxy| proxy.as_url().as_str());
    let result = sqlx::query(
        "UPDATE feeds SET proxy_mode=?, proxy_url=?, updated_at=CURRENT_TIMESTAMP WHERE id=?",
    )
    .bind(&input.mode)
    .bind(raw)
    .bind(&feed_id)
    .execute(&pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(Json(FeedProxySettingsView {
        mode: input.mode,
        configured: proxy_url.is_some(),
        endpoint: proxy_url.map(|proxy| proxy.sanitized()),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn global_proxy_is_write_only_and_invalid_replacement_preserves_saved_value() {
        let (pool, path) = crate::test_support::database().await;
        let state = || State(pool.clone());

        let Json(saved) = set_global(
            state(),
            Json(ProxyUrlInput {
                proxy_url: "http://alice:topsecret@proxy.example:8080".into(),
            }),
        )
        .await
        .unwrap();
        assert!(saved.configured);
        assert_eq!(
            saved.endpoint.as_deref(),
            Some("http://redacted:redacted@proxy.example:8080/")
        );
        assert!(!serde_json::to_string(&saved).unwrap().contains("topsecret"));

        let invalid = set_global(
            state(),
            Json(ProxyUrlInput {
                proxy_url: "ftp://proxy.example:21".into(),
            }),
        )
        .await;
        assert_eq!(invalid.err(), Some(StatusCode::BAD_REQUEST));

        let Json(read) = get_global(state()).await.unwrap();
        assert_eq!(read.endpoint, saved.endpoint);
        assert!(!serde_json::to_string(&read).unwrap().contains("topsecret"));
        assert_eq!(clear_global(state()).await.unwrap(), StatusCode::NO_CONTENT);
        assert!(!get_global(state()).await.unwrap().0.configured);
        crate::test_support::cleanup(pool, path).await;
    }

    #[tokio::test]
    async fn feed_proxy_modes_validate_and_return_only_sanitized_settings() {
        let (pool, path) = crate::test_support::database().await;
        sqlx::query("INSERT INTO feeds (id, slug, display_name) VALUES ('feed', 'feed', 'Feed')")
            .execute(&pool)
            .await
            .unwrap();
        let state = || State(pool.clone());

        let Json(custom) = set_feed(
            Path("feed".into()),
            state(),
            Json(FeedProxyInput {
                mode: "custom".into(),
                proxy_url: Some("socks5://alice:topsecret@proxy.example:1080".into()),
            }),
        )
        .await
        .unwrap();
        assert_eq!(custom.mode, "custom");
        assert!(
            !serde_json::to_string(&custom)
                .unwrap()
                .contains("topsecret")
        );

        let invalid = set_feed(
            Path("feed".into()),
            state(),
            Json(FeedProxyInput {
                mode: "custom".into(),
                proxy_url: Some("http://proxy.example".into()),
            }),
        )
        .await;
        assert_eq!(invalid.err(), Some(StatusCode::BAD_REQUEST));
        assert_eq!(
            get_feed(Path("feed".into()), state())
                .await
                .unwrap()
                .0
                .endpoint,
            custom.endpoint
        );

        let Json(direct) = set_feed(
            Path("feed".into()),
            state(),
            Json(FeedProxyInput {
                mode: "direct".into(),
                proxy_url: None,
            }),
        )
        .await
        .unwrap();
        assert_eq!(direct.mode, "direct");
        assert!(!direct.configured);
        assert_eq!(
            get_feed(Path("missing".into()), state()).await.err(),
            Some(StatusCode::NOT_FOUND)
        );
        crate::test_support::cleanup(pool, path).await;
    }
}
