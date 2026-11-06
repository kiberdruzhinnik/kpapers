use super::RemotePdf;
use sha2::{Digest, Sha256};
use std::path::PathBuf;

fn root() -> Result<PathBuf, String> {
    if std::env::var("E2E_FIXTURE_MODE").as_deref() != Ok("true") {
        return Err("local PDF fixtures are disabled".into());
    }
    std::env::var_os("E2E_FIXTURE_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| "E2E_FIXTURE_DIR is not set".into())
}

fn folder(source_url: &str) -> Result<(PathBuf, String), String> {
    let url = url::Url::parse(source_url).map_err(|e| e.to_string())?;
    if url.scheme() != "fixture"
        || url.path() != "/"
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("invalid fixture folder URL".into());
    }
    let name = url.host_str().ok_or("missing fixture folder")?;
    if name.is_empty()
        || !name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
    {
        return Err("invalid fixture folder name".into());
    }
    Ok((root()?.join(name), name.to_owned()))
}

pub async fn list_root_pdfs(source_url: &str) -> Result<Vec<RemotePdf>, String> {
    let (directory, folder_name) = folder(source_url)?;
    let mut entries = tokio::fs::read_dir(&directory)
        .await
        .map_err(|e| e.to_string())?;
    let mut files = Vec::new();
    while let Some(entry) = entries.next_entry().await.map_err(|e| e.to_string())? {
        let path = entry.path();
        if !path.is_file()
            || path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_none_or(|ext| !ext.eq_ignore_ascii_case("pdf"))
        {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("invalid fixture filename")?
            .to_owned();
        if filename.contains('/') || filename.contains('\\') {
            continue;
        }
        let metadata = entry.metadata().await.map_err(|e| e.to_string())?;
        let bytes = tokio::fs::read(&path).await.map_err(|e| e.to_string())?;
        let fingerprint = format!("{:x}", Sha256::digest(&bytes));
        files.push(RemotePdf {
            provider_id: filename.clone(),
            path: filename.clone(),
            filename,
            size: metadata.len(),
            modified: None,
            fingerprint,
            download_url: format!(
                "fixture://{folder_name}/{}",
                entry.file_name().to_string_lossy()
            ),
            public_url: format!(
                "https://fixture.invalid/{folder_name}/{}",
                entry.file_name().to_string_lossy()
            ),
        });
    }
    files.sort_by(|a, b| a.filename.cmp(&b.filename));
    Ok(files)
}

pub async fn download(file: &RemotePdf, maximum: u64) -> Result<Vec<u8>, String> {
    if file.filename.contains('/') || file.filename.contains('\\') {
        return Err("invalid fixture filename".into());
    }
    let folder_url = format!(
        "{}/",
        file.download_url
            .rsplit_once('/')
            .ok_or("invalid fixture download URL")?
            .0
    );
    let (directory, folder_name) = folder(&folder_url)?;
    let path = directory.join(&file.filename);
    let metadata = tokio::fs::metadata(&path)
        .await
        .map_err(|e| e.to_string())?;
    if metadata.len() > maximum {
        return Err("fixture exceeds maximum PDF size".into());
    }
    let bytes = tokio::fs::read(path).await.map_err(|e| e.to_string())?;
    let fingerprint = format!("{:x}", Sha256::digest(&bytes));
    if fingerprint != file.fingerprint {
        return Err(format!("fixture changed after discovery in {folder_name}"));
    }
    Ok(bytes)
}
