use axum::{
    Extension, Json, Router, middleware,
    routing::{delete, get, post, put},
};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    if args.any(|arg| arg == "--hash-password") {
        use argon2::password_hash::{PasswordHasher, SaltString};
        let mut password = String::new();
        std::io::stdin().read_line(&mut password)?;
        let salt = SaltString::generate(&mut rand::rngs::OsRng);
        let hash = argon2::Argon2::default()
            .hash_password(password.trim_end().as_bytes(), &salt)
            .map_err(|error| error.to_string())?
            .to_string();
        println!("{hash}");
        return Ok(());
    }
    if std::env::args().any(|arg| arg == "--healthcheck") {
        let response = reqwest::get("http://127.0.0.1:8080/healthz").await?;
        if !response.status().is_success() {
            return Err("API health check failed".into());
        }
        return Ok(());
    }
    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://kpapers.db?mode=rwc".to_owned());
    let pool = db::connect_and_migrate(&database_url).await?;
    tokio::spawn(scheduler::run(pool.clone()));
    let auth = auth::AuthConfig::from_env()?;

    let app = app(pool, auth);
    let address = SocketAddr::from(([0, 0, 0, 0], 8080));
    let listener = tokio::net::TcpListener::bind(address)
        .await
        .expect("bind API listener");

    axum::serve(listener, app).await?;
    Ok(())
}

fn app(pool: sqlx::SqlitePool, auth: auth::AuthConfig) -> Router {
    let management = Router::new()
        .route(
            "/api/management/session",
            get(management_session).delete(auth::logout),
        )
        .route("/api/management/overview", get(feed_api::overview_summary))
        .route(
            "/api/management/feeds",
            get(feed_api::list).post(feed_api::create),
        )
        .route(
            "/api/management/feeds/{id}",
            put(feed_api::update).delete(feed_api::deactivate),
        )
        .route(
            "/api/management/feeds/{id}/archive",
            post(feed_api::archive_feed),
        )
        .route(
            "/api/management/feeds/{id}/restore",
            post(feed_api::restore_feed),
        )
        .route(
            "/api/management/feeds/{id}/permanent",
            delete(feed_api::permanently_delete_feed),
        )
        .route(
            "/api/management/feeds/{feed_id}/refresh",
            post(feed_api::force_feed_refresh),
        )
        .route(
            "/api/management/feeds/{feed_id}/years",
            get(feed_api::list_years).post(feed_api::create_year),
        )
        .route(
            "/api/management/feeds/{feed_id}/years/{year}",
            delete(feed_api::permanently_delete_year),
        )
        .route(
            "/api/management/feeds/{feed_id}/years/{year}/archive",
            post(feed_api::archive_year),
        )
        .route(
            "/api/management/feeds/{feed_id}/years/{year}/restore",
            post(feed_api::restore_year),
        )
        .route(
            "/api/management/feeds/{feed_id}/years/{year}/refresh",
            post(feed_api::force_year_refresh),
        )
        .route(
            "/api/management/feeds/{feed_id}/years/{year}/months",
            get(feed_api::list_months),
        )
        .route(
            "/api/management/feeds/{feed_id}/years/{year}/months/{month}",
            put(feed_api::create_month),
        )
        .route(
            "/api/management/months/{month_id}/documents",
            get(feed_api::list_documents),
        )
        .route(
            "/api/management/months/{month_id}",
            delete(feed_api::permanently_delete_month),
        )
        .route(
            "/api/management/months/{month_id}/archive",
            post(feed_api::archive_month),
        )
        .route(
            "/api/management/months/{month_id}/restore",
            post(feed_api::restore_month),
        )
        .route(
            "/api/management/months/{month_id}/discover",
            post(feed_api::discover_month),
        )
        .route(
            "/api/management/months/{month_id}/refresh",
            post(feed_api::force_month_refresh),
        )
        .route(
            "/api/management/documents/{document_id}/promotion",
            put(feed_api::set_promotion_override),
        )
        .route(
            "/api/management/documents/{document_id}/extract",
            post(feed_api::extract_document),
        )
        .route(
            "/api/management/documents/{document_id}/source",
            get(rss::review_download),
        )
        .route_layer(middleware::from_fn(auth::require_session));
    Router::new()
        .route("/healthz", get(health))
        .route("/api/login", post(auth::login))
        .route("/api/management/login", post(auth::login))
        .route("/feeds/{feed_id}", get(rss::render))
        .route("/documents/{document_id}/download", get(rss::download))
        .merge(management)
        .layer(Extension(auth))
        .with_state(pool)
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { status: "ok" })
}

async fn management_session() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "authenticated",
    })
}

#[derive(serde::Serialize)]
struct HealthResponse {
    status: &'static str,
}

mod auth;
mod db;
mod feed_api;
mod pdf_processing;
mod providers;
mod rss;
mod scheduler;
#[cfg(test)]
mod test_support;

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{
        body::{Body, to_bytes},
        http::{Request, StatusCode, header},
    };
    use tower::ServiceExt;

    async fn response_body(response: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), 1024 * 1024).await.unwrap();
        serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null)
    }

    #[tokio::test]
    async fn routes_enforce_sessions_and_validate_requests_at_the_http_boundary() {
        let (pool, path) = test_support::database().await;
        let app = app(
            pool.clone(),
            auth::test_auth_config("operator", "correct horse"),
        );

        let response = app
            .clone()
            .oneshot(Request::get("/healthz").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(
                Request::get("/api/management/feeds")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response = app
            .clone()
            .oneshot(
                Request::get("/api/management/overview")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response = app
            .clone()
            .oneshot(
                Request::post("/api/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{broken"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(matches!(
            response.status(),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
        ));

        let response = app
            .clone()
            .oneshot(
                Request::post("/api/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"username":"operator","password":"wrong"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response = app
            .clone()
            .oneshot(
                Request::post("/api/login")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"username":"operator","password":"correct horse"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let cookie = response
            .headers()
            .get(header::SET_COOKIE)
            .unwrap()
            .to_str()
            .unwrap()
            .split(';')
            .next()
            .unwrap()
            .to_owned();

        let response = app
            .clone()
            .oneshot(
                Request::get("/api/management/overview")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response_body(response).await,
            serde_json::json!({
                "source_folder_count": 0,
                "needs_review_count": 0,
            })
        );

        let response = app
            .clone()
            .oneshot(
                Request::post("/api/management/feeds")
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(
                        r#"{"display_name":"Route test","description":"HTTP coverage"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
        let feed = response_body(response).await;
        let feed_id = feed["id"].as_str().unwrap();

        let response = app
            .clone()
            .oneshot(
                Request::post(format!("/api/management/feeds/{feed_id}/years"))
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from(r#"{"year":10000}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);

        let response = app
            .clone()
            .oneshot(
                Request::post(format!("/api/management/feeds/{feed_id}/years"))
                    .header(header::COOKIE, &cookie)
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{broken"))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(matches!(
            response.status(),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
        ));

        let response = app
            .clone()
            .oneshot(
                Request::get(format!(
                    "/api/management/feeds/{feed_id}/years/not-a-number/months"
                ))
                .header(header::COOKIE, &cookie)
                .body(Body::empty())
                .unwrap(),
            )
            .await
            .unwrap();
        assert!(matches!(
            response.status(),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
        ));

        let response = app
            .oneshot(
                Request::post("/api/management/documents/doc/extract?force=maybe")
                    .header(header::COOKIE, &cookie)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(matches!(
            response.status(),
            StatusCode::BAD_REQUEST | StatusCode::UNPROCESSABLE_ENTITY
        ));

        test_support::cleanup(pool, path).await;
    }
}
