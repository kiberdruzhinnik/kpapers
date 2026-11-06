use super::RemotePdf;
use serde::Deserialize;
use url::Url;

const API: &str = "https://cloud-api.yandex.net/v1/disk/public/resources";

#[derive(Deserialize)]
struct ResourcePage { #[serde(rename = "public_url")] public_url: Option<String>, #[serde(rename = "_embedded")] embedded: Embedded }
#[derive(Deserialize)]
struct Embedded { total: u64, items: Vec<Resource> }
#[derive(Deserialize)]
struct Resource { #[serde(rename = "type")] kind: String, path: String, name: String, size: Option<u64>, modified: Option<String>, md5: Option<String>, resource_id: Option<String>, file: Option<String> }
#[derive(Deserialize)]
struct DownloadResponse { href: String }

pub async fn list_root_pdfs(public_url: &str) -> Result<Vec<RemotePdf>, Box<dyn std::error::Error + Send + Sync>> {
    if super::validate_source_url(public_url) != Ok("yandex_disk") { return Err("unsupported Yandex Disk URL".into()); }
    let client = super::safe_client("yandex", std::time::Duration::from_secs(20))?;
    let mut offset = 0u64;
    let mut found = Vec::new();
    loop {
        let page: ResourcePage = client.get(API).query(&[("public_key", public_url), ("path", "/"), ("limit", "100"), ("offset", &offset.to_string())]).send().await?.error_for_status()?.json().await?;
        let count = page.embedded.items.len() as u64;
        for item in page.embedded.items.into_iter().filter(|i| i.kind == "file" && i.name.to_ascii_lowercase().ends_with(".pdf")) {
            let file_url = if let Some(url) = item.file.clone() { url } else {
                let response: DownloadResponse = client.get(format!("{API}/download")).query(&[("public_key", public_url), ("path", &item.path)]).send().await?.error_for_status()?.json().await?;
                response.href
            };
            let mut public = Url::parse(public_url)?;
            let public_path = item.path.strip_prefix("disk:").unwrap_or(&item.path);
            public.query_pairs_mut().append_pair("path", public_path);
            found.push(RemotePdf { provider_id: item.resource_id.clone().unwrap_or_else(|| item.path.clone()), path: item.path, filename: item.name, size: item.size.unwrap_or(0), modified: item.modified, fingerprint: item.md5.unwrap_or_else(|| item.resource_id.unwrap_or_default()), download_url: file_url, public_url: public.to_string() });
        }
        offset += count;
        if count == 0 || offset >= page.embedded.total { break; }
    }
    Ok(found)
}

pub async fn download(pdf: &RemotePdf, maximum_bytes: u64) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    if pdf.size > maximum_bytes { return Err("file exceeds configured size limit".into()); }
    let client = super::safe_client("yandex", std::time::Duration::from_secs(60))?;
    let response = client.get(&pdf.download_url).send().await?.error_for_status()?;
    if response.status().is_redirection() || response.content_length().is_some_and(|size| size > maximum_bytes) { return Err("download destination rejected or exceeds size limit".into()); }
    let mut response = response;
    let mut bytes = Vec::new();
    while let Some(chunk) = response.chunk().await? {
        if bytes.len() as u64 + chunk.len() as u64 > maximum_bytes { return Err("download exceeds configured size limit".into()); }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes)
}
