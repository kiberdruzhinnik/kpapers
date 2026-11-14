use super::RemotePdf;
use serde::Deserialize;
use url::Url;

const API: &str = "https://cloud-api.yandex.net/v1/disk/public/resources";

#[derive(Deserialize)]
pub(crate) struct ResourcePage {
    #[serde(rename = "public_url")]
    _public_url: Option<String>,
    #[serde(rename = "_embedded")]
    embedded: Embedded,
}
#[derive(Deserialize)]
pub(crate) struct Embedded {
    total: u64,
    items: Vec<Resource>,
}
#[derive(Deserialize)]
pub(crate) struct Resource {
    #[serde(rename = "type")]
    kind: String,
    path: String,
    name: String,
    size: Option<u64>,
    modified: Option<String>,
    md5: Option<String>,
    resource_id: Option<String>,
    file: Option<String>,
}
#[derive(Deserialize)]
pub(crate) struct DownloadResponse {
    href: String,
}

pub async fn list_root_pdfs(
    public_url: &str,
) -> Result<Vec<RemotePdf>, Box<dyn std::error::Error + Send + Sync>> {
    if super::validate_source_url(public_url) != Ok("yandex_disk") {
        return Err("unsupported Yandex Disk URL".into());
    }
    let client = super::safe_client("yandex", std::time::Duration::from_secs(20))?;
    let mut offset = 0u64;
    let mut found = Vec::new();
    loop {
        let page: ResourcePage = client
            .get(API)
            .query(&[
                ("public_key", public_url),
                ("path", "/"),
                ("limit", "100"),
                ("offset", &offset.to_string()),
            ])
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        let count = page.embedded.items.len() as u64;
        for item in page
            .embedded
            .items
            .into_iter()
            .filter(|i| i.kind == "file" && i.name.to_ascii_lowercase().ends_with(".pdf"))
        {
            let file_url = if let Some(url) = item.file.clone() {
                url
            } else {
                let response: DownloadResponse = client
                    .get(format!("{API}/download"))
                    .query(&[("public_key", public_url), ("path", &item.path)])
                    .send()
                    .await?
                    .error_for_status()?
                    .json()
                    .await?;
                response.href
            };
            let mut public = Url::parse(public_url)?;
            let public_path = item.path.strip_prefix("disk:").unwrap_or(&item.path);
            public.query_pairs_mut().append_pair("path", public_path);
            found.push(RemotePdf {
                provider_id: item
                    .resource_id
                    .clone()
                    .unwrap_or_else(|| item.path.clone()),
                path: item.path,
                filename: item.name,
                size: item.size.unwrap_or(0),
                modified: item.modified,
                fingerprint: item
                    .md5
                    .unwrap_or_else(|| item.resource_id.unwrap_or_default()),
                download_url: file_url,
                public_url: public.to_string(),
            });
        }
        offset += count;
        if count == 0 || offset >= page.embedded.total {
            break;
        }
    }
    Ok(found)
}

pub async fn download(
    pdf: &RemotePdf,
    maximum_bytes: u64,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    if pdf.size > maximum_bytes {
        return Err("file exceeds configured size limit".into());
    }
    super::validate_download_url("yandex_disk", &pdf.download_url)?;
    let client = super::safe_client("yandex", std::time::Duration::from_secs(60))?;
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
            public_url: "https://disk.yandex.ru/d/token".into(),
        };

        let error = download(&pdf, 1024).await.unwrap_err();
        assert_eq!(
            error.to_string(),
            "provider returned a download URL outside its HTTPS host allowlist"
        );
    }

    #[test]
    fn resource_page_accepts_optional_metadata_and_ignores_future_fields() {
        let page: ResourcePage = serde_json::from_str(
            r#"{"public_url":"https://disk.yandex.ru/d/token","_embedded":{"total":1,"items":[{"type":"file","path":"disk:/article.pdf","name":"article.pdf","size":12,"modified":"2026-11-01T00:00:00Z","md5":"abc","resource_id":"id-1","file":"https://storage.yandex.net/file"}]},"new_field":true}"#,
        ).unwrap();
        assert_eq!(
            page._public_url.as_deref(),
            Some("https://disk.yandex.ru/d/token")
        );
        assert_eq!(page.embedded.total, 1);
        let item = &page.embedded.items[0];
        assert_eq!(item.kind, "file");
        assert_eq!(item.name, "article.pdf");
        assert_eq!(item.size, Some(12));
        assert_eq!(item.resource_id.as_deref(), Some("id-1"));
    }

    #[test]
    fn resource_page_rejects_missing_required_and_malformed_data() {
        assert!(
            serde_json::from_str::<ResourcePage>(
                r#"{"_embedded":{"total":1,"items":[{"type":"file","path":"x"}]}}"#
            )
            .is_err()
        );
        assert!(
            serde_json::from_str::<ResourcePage>(r#"{"_embedded":{"total":"many","items":[]}}"#)
                .is_err()
        );
        assert!(serde_json::from_str::<ResourcePage>("{broken").is_err());
        assert!(
            serde_json::from_str::<DownloadResponse>(r#"{"url":"https://example.com/file"}"#)
                .is_err()
        );
    }

    #[test]
    fn download_response_requires_an_href_string() {
        let response: DownloadResponse =
            serde_json::from_str(r#"{"href":"https://storage.yandex.net/file","trace":7}"#)
                .unwrap();
        assert_eq!(response.href, "https://storage.yandex.net/file");
        assert!(serde_json::from_str::<DownloadResponse>(r#"{"href":null}"#).is_err());
    }
}
