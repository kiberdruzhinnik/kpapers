use axum::{extract::Request, middleware::Next, response::Response};
use std::{net::SocketAddr, time::Instant};

/// Install a JSON subscriber that writes one structured event per line to stdout.
pub fn init() {
    let subscriber = tracing_subscriber::fmt()
        .json()
        .with_target(false)
        .with_writer(std::io::stdout)
        .finish();
    let _ = tracing::subscriber::set_global_default(subscriber);
}

pub fn started(address: SocketAddr) {
    tracing::info!(
        event = "api_started",
        address = %address,
        "API started"
    );
}

pub fn initialization_failed(error: &(dyn std::error::Error + 'static)) {
    tracing::error!(
        event = "api_initialization_failed",
        error = %error,
        "API initialization failed"
    );
}

pub fn refresh_failed(job_id: &str, scope: &str, feed_id: &str, month_id: &str, error: &str) {
    tracing::error!(
        event = "refresh_month_failed",
        operation = "refresh_month",
        job_id = %job_id,
        scope = %scope,
        feed_id = %feed_id,
        month_id = %month_id,
        error = %safe_error_summary(error),
        "scheduled refresh failed"
    );
}

/// Keep a bounded summary and drop tokens that look like URLs or credentials.
pub fn safe_error_summary(error: &str) -> String {
    let mut kept = Vec::new();
    let mut skip = 0usize;
    let mut drop_rest = false;
    for token in error.split_whitespace() {
        if skip > 0 {
            skip -= 1;
            continue;
        }
        if drop_rest {
            continue;
        }

        let lower = token.to_ascii_lowercase();
        let label = lower.trim_end_matches([':', ';', ',']);
        if matches!(label, "cookie" | "set-cookie") {
            drop_rest = true;
            continue;
        }
        if matches!(label, "authorization" | "proxy-authorization") {
            // Authorization is followed by a scheme and its credential value.
            skip = 2;
            continue;
        }
        if matches!(
            label,
            "password"
                | "passwd"
                | "secret"
                | "token"
                | "api-key"
                | "api_key"
                | "key"
                | "bearer"
                | "basic"
        ) {
            skip = 1;
            continue;
        }
        if looks_sensitive(&lower) {
            continue;
        }

        kept.push(token);
        if kept.len() == 16 {
            break;
        }
    }

    let mut summary = kept.join(" ");
    if summary.is_empty() {
        return "provider operation failed".to_owned();
    }
    if summary.chars().count() > 200 {
        summary = summary.chars().take(200).collect::<String>() + "...";
    }
    summary
}

fn looks_sensitive(token: &str) -> bool {
    let lower = token.to_ascii_lowercase();
    ["://", "@", "token=", "password=", "secret=", "key="]
        .iter()
        .any(|marker| lower.contains(marker))
}

/// Log one completion event per HTTP request without headers, cookies, bodies, or query strings.
pub async fn request_log(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let path = request.uri().path().to_owned();
    let started = Instant::now();
    let response = next.run(request).await;
    let status = response.status().as_u16();
    let duration_ms = started.elapsed().as_millis() as u64;
    tracing::info!(
        event = "http_request",
        method = %method,
        path = %path,
        status,
        duration_ms,
        "request completed"
    );
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn startup_and_initialization_failure_events_are_parseable_json() {
        let ((), logs) = crate::test_support::capture_logs(async {
            started(SocketAddr::from(([0, 0, 0, 0], 8080)));
            initialization_failed(&std::io::Error::other("database unavailable"));
        })
        .await;
        let events = crate::test_support::json_events(&logs);
        assert_eq!(events.len(), 2);
        assert_eq!(events[0]["fields"]["event"], "api_started");
        assert_eq!(events[0]["fields"]["address"], "0.0.0.0:8080");
        assert_eq!(events[1]["fields"]["event"], "api_initialization_failed");
        assert_eq!(events[1]["fields"]["error"], "database unavailable");
    }

    #[tokio::test]
    async fn provider_failure_events_omit_credentials_and_secret_urls() {
        let ((), logs) = crate::test_support::capture_logs(async {
            refresh_failed(
                "job-1",
                "month",
                "feed-1",
                "month-1",
                "provider token=plain-secret rejected https://user:pass@example.com/file.pdf?token=url-secret Authorization: Bearer header-secret password: spaced-secret",
            );
        })
        .await;
        let events = crate::test_support::json_events(&logs);
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["fields"]["event"], "refresh_month_failed");
        assert_eq!(events[0]["fields"]["error"], "provider rejected");
        for secret in [
            "plain-secret",
            "url-secret",
            "user:pass",
            "example.com",
            "https://",
            "Authorization",
            "Bearer",
            "header-secret",
            "spaced-secret",
        ] {
            assert!(!logs.contains(secret), "log leaked {secret}: {logs}");
        }
    }

    #[test]
    fn provider_failure_summary_falls_back_when_all_tokens_are_sensitive() {
        assert_eq!(
            safe_error_summary("token=secret"),
            "provider operation failed"
        );
        assert_eq!(
            safe_error_summary("https://user:pass@example.com"),
            "provider operation failed"
        );
        assert_eq!(
            safe_error_summary("Cookie: session=abc; other=def"),
            "provider operation failed"
        );
    }
}
