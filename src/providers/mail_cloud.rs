use super::RemotePdf;
use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use serde::Deserialize;

const API: &str = "https://cloud.mail.ru/api/v2";

#[derive(Deserialize)]
pub(crate) struct ApiResponse<T> {
    body: T,
}
#[derive(Deserialize)]
pub(crate) struct Folder {
    list: Vec<Entry>,
}
#[derive(Deserialize)]
pub(crate) struct Entry {
    #[serde(rename = "type")]
    kind: String,
    name: String,
    weblink: String,
    size: Option<u64>,
    mtime: Option<i64>,
    hash: Option<String>,
}
#[derive(Deserialize)]
pub(crate) struct Dispatcher {
    weblink_get: Vec<DispatcherUrl>,
}
#[derive(Deserialize)]
pub(crate) struct DispatcherUrl {
    url: String,
}

pub async fn list_root_pdfs(
    public_url: &str,
) -> Result<Vec<RemotePdf>, Box<dyn std::error::Error + Send + Sync>> {
    let parsed = url::Url::parse(public_url)?;
    if parsed.scheme() != "https"
        || parsed.host_str() != Some("cloud.mail.ru")
        || parsed.port_or_known_default() != Some(443)
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err("unsupported Mail Cloud URL".into());
    }
    let token = parsed.path().trim_start_matches("/public/");
    if token.split('/').count() != 2
        || token
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("invalid Mail Cloud public folder path".into());
    }
    let client = super::safe_client("mail", std::time::Duration::from_secs(20))?;
    let response: ApiResponse<Folder> = client
        .get(format!("{API}/folder"))
        .query(&[("weblink", token), ("folder", "/")])
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let dispatcher: ApiResponse<Dispatcher> = client
        .get(format!("{API}/dispatcher"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;
    let base = dispatcher
        .body
        .weblink_get
        .first()
        .ok_or("Mail Cloud returned no download dispatcher")?
        .url
        .trim_end_matches('/')
        .to_owned();
    Ok(response
        .body
        .list
        .into_iter()
        .filter(|entry| entry.kind == "file" && entry.name.to_ascii_lowercase().ends_with(".pdf"))
        .map(|entry| {
            let download_url = format!(
                "{}/{}",
                base,
                utf8_percent_encode(&entry.weblink, NON_ALPHANUMERIC)
            );
            let fingerprint = entry
                .hash
                .clone()
                .unwrap_or_else(|| format!("{}:{}", entry.name, entry.size.unwrap_or_default()));
            RemotePdf {
                provider_id: entry.weblink.clone(),
                path: entry.weblink.clone(),
                filename: entry.name,
                size: entry.size.unwrap_or_default(),
                modified: entry.mtime.map(|t| t.to_string()),
                fingerprint,
                download_url,
                public_url: entry.weblink,
            }
        })
        .collect())
}

pub async fn download(
    pdf: &RemotePdf,
    maximum_bytes: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    if pdf.size > maximum_bytes {
        return Err("file exceeds configured size limit".into());
    }
    super::validate_download_url("mail_cloud", &pdf.download_url)?;
    let client = super::safe_client("mail", std::time::Duration::from_secs(60))?;
    let response = client
        .get(&pdf.download_url)
        .send()
        .await?
        .error_for_status()?;
    if response.status().is_redirection()
        || response
            .content_length()
            .is_some_and(|size| size > maximum_bytes)
    {
        return Err("download destination rejected or exceeds size limit".into());
    }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() as u64 + chunk.len() as u64 > maximum_bytes {
            return Err("download exceeds configured size limit".into());
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn download_rejects_disallowed_url_before_request() {
        let pdf = RemotePdf {
            provider_id: "id".into(),
            path: "article.pdf".into(),
            filename: "article.pdf".into(),
            size: 10,
            modified: None,
            fingerprint: "fingerprint".into(),
            download_url: "http://127.0.0.1:1/internal".into(),
            public_url: "https://cloud.mail.ru/public/owner/token".into(),
        };

        let error = download(&pdf, 1024).await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "provider returned a download URL outside its HTTPS host allowlist"
        );
    }

    #[test]
    fn folder_payload_accepts_optional_metadata_and_ignores_unknown_fields() {
        let response: ApiResponse<Folder> = serde_json::from_str(
            r#"{"body":{"list":[{"type":"file","name":"article.pdf","weblink":"owner/file.pdf","size":25,"mtime":1793400000,"hash":"sha256"}],"next_cursor":"ignored"},"request_id":"123"}"#,
        ).unwrap();
        assert_eq!(response.body.list.len(), 1);
        let entry = &response.body.list[0];
        assert_eq!(entry.kind, "file");
        assert_eq!(entry.name, "article.pdf");
        assert_eq!(entry.size, Some(25));
        assert_eq!(entry.hash.as_deref(), Some("sha256"));

        let minimal: ApiResponse<Folder> = serde_json::from_str(
            r#"{"body":{"list":[{"type":"file","name":"x.pdf","weblink":"x"}]}}"#,
        )
        .unwrap();
        assert_eq!(minimal.body.list[0].size, None);
        assert_eq!(minimal.body.list[0].mtime, None);
        assert_eq!(minimal.body.list[0].hash, None);
    }

    #[test]
    fn folder_and_dispatcher_payloads_reject_malformed_or_missing_required_fields() {
        assert!(serde_json::from_str::<ApiResponse<Folder>>(r#"{"body":{}}"#).is_err());
        assert!(
            serde_json::from_str::<ApiResponse<Folder>>(
                r#"{"body":{"list":[{"type":"file","name":"x.pdf"}]}}"#
            )
            .is_err()
        );
        assert!(serde_json::from_str::<ApiResponse<Folder>>("{bad json").is_err());
        assert!(
            serde_json::from_str::<ApiResponse<Dispatcher>>(r#"{"body":{"weblink_get":[]}}"#)
                .is_ok()
        );
        assert!(
            serde_json::from_str::<ApiResponse<Dispatcher>>(
                r#"{"body":{"weblink_get":[{"url":false}]}}"#
            )
            .is_err()
        );
        assert!(serde_json::from_str::<ApiResponse<Dispatcher>>(r#"{"body":{}}"#).is_err());
    }
}
