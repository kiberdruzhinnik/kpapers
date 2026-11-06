use super::RemotePdf;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use serde::Deserialize;

const API: &str = "https://cloud.mail.ru/api/v2";

#[derive(Deserialize)]
struct ApiResponse<T> { body: T }
#[derive(Deserialize)]
struct Folder { list: Vec<Entry> }
#[derive(Deserialize)]
struct Entry { #[serde(rename = "type")] kind: String, name: String, weblink: String, size: Option<u64>, mtime: Option<i64>, hash: Option<String> }
#[derive(Deserialize)]
struct Dispatcher { weblink_get: Vec<DispatcherUrl> }
#[derive(Deserialize)]
struct DispatcherUrl { url: String }

pub async fn list_root_pdfs(public_url: &str) -> Result<Vec<RemotePdf>, Box<dyn std::error::Error + Send + Sync>> {
    let parsed = url::Url::parse(public_url)?;
    if parsed.scheme() != "https" || parsed.host_str() != Some("cloud.mail.ru") || parsed.port_or_known_default() != Some(443) || !parsed.username().is_empty() || parsed.password().is_some() || parsed.query().is_some() || parsed.fragment().is_some() { return Err("unsupported Mail Cloud URL".into()); }
    let token = parsed.path().trim_start_matches("/public/");
    if token.split('/').count() != 2 || token.split('/').any(|part| part.is_empty() || part == "." || part == "..") { return Err("invalid Mail Cloud public folder path".into()); }
    let client = super::safe_client("mail", std::time::Duration::from_secs(20))?;
    let response: ApiResponse<Folder> = client.get(format!("{API}/folder")).query(&[("weblink", token), ("folder", "/")]).send().await?.error_for_status()?.json().await?;
    let dispatcher: ApiResponse<Dispatcher> = client.get(format!("{API}/dispatcher")).send().await?.error_for_status()?.json().await?;
    let base = dispatcher.body.weblink_get.first().ok_or("Mail Cloud returned no download dispatcher")?.url.trim_end_matches('/').to_owned();
    Ok(response.body.list.into_iter().filter(|entry| entry.kind == "file" && entry.name.to_ascii_lowercase().ends_with(".pdf"))
        .map(|entry| {
            let download_url = format!("{}/{}", base, utf8_percent_encode(&entry.weblink, NON_ALPHANUMERIC));
            let fingerprint = entry.hash.clone().unwrap_or_else(|| format!("{}:{}", entry.name, entry.size.unwrap_or_default()));
            RemotePdf { provider_id: entry.weblink.clone(), path: entry.weblink.clone(), filename: entry.name, size: entry.size.unwrap_or_default(), modified: entry.mtime.map(|t| t.to_string()), fingerprint, download_url, public_url: entry.weblink }
        }).collect())
}

pub async fn download(pdf: &RemotePdf, maximum_bytes: u64) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    if pdf.size > maximum_bytes { return Err("file exceeds configured size limit".into()); }
    let client = super::safe_client("mail", std::time::Duration::from_secs(60))?;
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
