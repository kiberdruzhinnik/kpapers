use crate::extraction::MarkerSettings;
use axum::http::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Page-aware output contract returned by the Marker worker.
///
/// The worker strips Marker's HTML into plain text plus emphasis runs and
/// groups the result into page lines. The Rust pipeline applies the same
/// boundary, page-label, Sponsr and safe-HTML rules to these lines as it does
/// to Pdfium output.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarkerResponse {
    pub page_count: usize,
    pub pages: Vec<MarkerPage>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarkerPage {
    #[serde(default)]
    pub lines: Vec<MarkerLine>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarkerLine {
    pub runs: Vec<MarkerRun>,
    #[serde(default = "default_size")]
    pub size: f32,
    #[serde(default)]
    pub gap_before: f32,
    #[serde(default = "default_left")]
    pub left: f32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MarkerRun {
    pub text: String,
    #[serde(default)]
    pub bold: bool,
    #[serde(default)]
    pub italic: bool,
}

fn default_size() -> f32 {
    12.0
}

fn default_left() -> f32 {
    50.0
}

pub struct MarkerClient {
    endpoint: String,
    timeout: Duration,
    max_response_bytes: usize,
    permits: std::sync::Arc<tokio::sync::Semaphore>,
    client: reqwest::Client,
}

impl MarkerClient {
    pub fn from_settings(settings: &MarkerSettings) -> Result<Self, String> {
        let endpoint = settings
            .endpoint
            .clone()
            .ok_or_else(|| "Marker worker is not configured; set MARKER_WORKER_URL".to_owned())?;
        let client = reqwest::Client::builder()
            .timeout(settings.timeout)
            .build()
            .map_err(|error| format!("could not create the Marker worker client: {error}"))?;
        Ok(Self {
            endpoint,
            timeout: settings.timeout,
            max_response_bytes: settings.max_response_bytes,
            permits: settings.permits.clone(),
            client,
        })
    }

    pub async fn extract(
        &self,
        bytes: &[u8],
        maximum_bytes: usize,
    ) -> Result<MarkerResponse, String> {
        if bytes.len() > maximum_bytes {
            return Err("PDF exceeds configured size limit".into());
        }
        let _permit = self
            .permits
            .acquire()
            .await
            .map_err(|_| "Marker worker concurrency limit is closed".to_owned())?;
        let response = self
            .client
            .post(format!("{}/extract", self.endpoint))
            .header(CONTENT_TYPE, "application/pdf")
            .timeout(self.timeout)
            .body(bytes.to_vec())
            .send()
            .await
            .map_err(|error| format!("Marker worker request failed: {error}"))?;
        let status = response.status();
        if !status.is_success() {
            return Err(format!("Marker worker returned HTTP {status}"));
        }
        if response
            .content_length()
            .is_some_and(|length| length > self.max_response_bytes as u64)
        {
            return Err("Marker worker response exceeds the configured size limit".into());
        }
        let body = response
            .bytes()
            .await
            .map_err(|error| format!("Marker worker response could not be read: {error}"))?;
        if body.len() > self.max_response_bytes {
            return Err("Marker worker response exceeds the configured size limit".into());
        }
        let parsed = serde_json::from_slice::<MarkerResponse>(&body)
            .map_err(|error| format!("Marker worker returned malformed JSON: {error}"))?;
        validate_response(&parsed)?;
        Ok(parsed)
    }
}

fn validate_response(response: &MarkerResponse) -> Result<(), String> {
    if response.page_count == 0 {
        return Err("Marker worker returned no pages".into());
    }
    if response.pages.len() != response.page_count {
        return Err("Marker worker page count does not match its page list".into());
    }
    for page in &response.pages {
        for line in &page.lines {
            if !line.size.is_finite() || line.size <= 0.0 {
                return Err("Marker worker returned an invalid line size".into());
            }
            if !line.gap_before.is_finite() || line.gap_before < 0.0 {
                return Err("Marker worker returned an invalid line gap".into());
            }
            if !line.left.is_finite() || line.left < 0.0 {
                return Err("Marker worker returned an invalid line position".into());
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{MarkerClient, MarkerResponse, MarkerSettings};
    use axum::{Router, http::StatusCode, response::IntoResponse, routing::post};
    use std::{sync::Arc, time::Duration};

    async fn worker(app: Router) -> String {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });
        format!("http://{address}")
    }

    fn client(endpoint: String) -> MarkerClient {
        MarkerClient::from_settings(&MarkerSettings {
            endpoint: Some(endpoint),
            timeout: Duration::from_secs(2),
            max_response_bytes: 4096,
            permits: Arc::new(tokio::sync::Semaphore::new(1)),
        })
        .unwrap()
    }

    fn response_json() -> serde_json::Value {
        serde_json::json!({
            "page_count": 1,
            "pages": [{"lines": [
                {"runs": [{"text": "Title", "bold": true, "italic": false}], "size": 22.0, "gap_before": 0.0, "left": 50.0},
                {"runs": [{"text": "Body", "bold": false, "italic": false}], "size": 12.0, "gap_before": 32.0, "left": 50.0}
            ]}]
        })
    }

    #[tokio::test]
    async fn accepts_a_page_aware_worker_response() {
        let endpoint =
            worker(Router::new().route("/extract", post(|| async { axum::Json(response_json()) })))
                .await;
        let response = client(endpoint).extract(b"%PDF-1.4", 1024).await.unwrap();
        assert_eq!(response.page_count, 1);
        assert_eq!(response.pages[0].lines[0].runs[0].text, "Title");
    }

    #[tokio::test]
    async fn reports_worker_http_errors_without_falling_back() {
        let endpoint = worker(Router::new().route(
            "/extract",
            post(|| async { (StatusCode::BAD_GATEWAY, "worker failed") }),
        ))
        .await;
        let error = client(endpoint)
            .extract(b"%PDF-1.4", 1024)
            .await
            .unwrap_err();
        assert!(error.contains("HTTP 502"), "{error}");
        assert!(!error.to_lowercase().contains("pdfium"), "{error}");
    }

    #[tokio::test]
    async fn rejects_malformed_and_inconsistent_responses() {
        let malformed = worker(
            Router::new().route("/extract", post(|| async { (StatusCode::OK, "not json") })),
        )
        .await;
        assert!(
            client(malformed)
                .extract(b"%PDF-1.4", 1024)
                .await
                .unwrap_err()
                .contains("malformed JSON")
        );

        let inconsistent = worker(Router::new().route(
            "/extract",
            post(|| async {
                axum::Json(serde_json::json!({
                    "page_count": 2,
                    "pages": [{"lines": []}]
                }))
            }),
        ))
        .await;
        assert!(
            client(inconsistent)
                .extract(b"%PDF-1.4", 1024)
                .await
                .unwrap_err()
                .contains("page count")
        );
    }

    #[tokio::test]
    async fn enforces_request_and_response_size_limits() {
        let endpoint =
            worker(Router::new().route("/extract", post(|| async { axum::Json(response_json()) })))
                .await;
        let client = client(endpoint);
        assert!(
            client
                .extract(b"%PDF-1.4", 4)
                .await
                .unwrap_err()
                .contains("size limit")
        );

        let oversized = worker(Router::new().route(
            "/extract",
            post(|| async {
                axum::Json(serde_json::json!({
                    "page_count": 1,
                    "pages": [{"lines": [{"runs": [{"text": "x".repeat(5000)}]}]}]
                }))
            }),
        ))
        .await;
        let error = client_with_limit(oversized, 128)
            .extract(b"%PDF-1.4", 1024)
            .await
            .unwrap_err();
        assert!(error.contains("response exceeds"), "{error}");
    }

    #[tokio::test]
    async fn enforces_an_execution_timeout() {
        let endpoint = worker(Router::new().route(
            "/extract",
            post(|| async {
                tokio::time::sleep(Duration::from_millis(300)).await;
                StatusCode::OK.into_response()
            }),
        ))
        .await;
        let client = MarkerClient::from_settings(&MarkerSettings {
            endpoint: Some(endpoint),
            timeout: Duration::from_millis(50),
            max_response_bytes: 4096,
            permits: Arc::new(tokio::sync::Semaphore::new(1)),
        })
        .unwrap();
        let error = client.extract(b"%PDF-1.4", 1024).await.unwrap_err();
        assert!(error.contains("request failed"), "{error}");
    }

    fn client_with_limit(endpoint: String, limit: usize) -> MarkerClient {
        MarkerClient::from_settings(&MarkerSettings {
            endpoint: Some(endpoint),
            timeout: Duration::from_secs(2),
            max_response_bytes: limit,
            permits: Arc::new(tokio::sync::Semaphore::new(1)),
        })
        .unwrap()
    }

    #[test]
    fn a_missing_endpoint_is_reported_as_configuration_error() {
        let error = match MarkerClient::from_settings(&MarkerSettings::default()) {
            Ok(_) => panic!("a missing endpoint must fail"),
            Err(error) => error,
        };
        assert!(error.contains("MARKER_WORKER_URL"), "{error}");
    }

    #[test]
    fn response_contract_is_deserializable() {
        let parsed: MarkerResponse = serde_json::from_value(response_json()).unwrap();
        assert_eq!(parsed.pages[0].lines.len(), 2);
    }
}
