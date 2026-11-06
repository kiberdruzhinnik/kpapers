pub mod mail_cloud;
pub mod local_fixtures;
pub mod yandex_disk;

#[derive(Clone, Debug)]
pub struct RemotePdf {
    pub provider_id: String,
    pub path: String,
    pub filename: String,
    pub size: u64,
    pub modified: Option<String>,
    pub fingerprint: String,
    pub download_url: String,
    pub public_url: String,
}

pub fn validate_download_url(provider: &str, raw: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(raw).map_err(|_| "provider returned an invalid download URL".to_owned())?;
    let host = url.host_str().unwrap_or_default();
    let allowed_host = match provider {
        "yandex_disk" => {
            host == "cloud-api.yandex.net"
                || host == "disk.yandex.ru"
                || host.ends_with(".disk.yandex.ru")
                || host == "storage.yandex.net"
                || host.ends_with(".storage.yandex.net")
        }
        "mail_cloud" => {
            host == "cloud.mail.ru"
                || host.ends_with(".cloud.mail.ru")
                || host == "datacloudmail.ru"
                || host.ends_with(".datacloudmail.ru")
        }
        _ => false,
    };
    if url.scheme() != "https"
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || !allowed_host
    {
        return Err("provider returned a download URL outside its HTTPS host allowlist".to_owned());
    }
    Ok(url)
}

pub fn safe_client(provider: &'static str, timeout: std::time::Duration) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder().timeout(timeout).redirect(reqwest::redirect::Policy::custom(move |attempt| {
        let url = attempt.url();
        let host = url.host_str().unwrap_or_default();
        let allowed_host = match provider {
            "yandex" => host == "cloud-api.yandex.net" || host == "disk.yandex.ru" || host.ends_with(".disk.yandex.ru") || host == "storage.yandex.net" || host.ends_with(".storage.yandex.net"),
            "mail" => host == "cloud.mail.ru" || host.ends_with(".cloud.mail.ru") || host == "datacloudmail.ru" || host.ends_with(".datacloudmail.ru"),
            _ => false,
        };
        if attempt.previous().len() < 5 && url.scheme() == "https" && url.port_or_known_default() == Some(443) && allowed_host { attempt.follow() } else { attempt.stop() }
    })).build()
}

pub fn validate_source_url(raw: &str) -> Result<&'static str, &'static str> {
    let url = url::Url::parse(raw).map_err(|_| "invalid URL")?;
    if url.scheme() == "fixture" && std::env::var("E2E_FIXTURE_MODE").as_deref() == Ok("true")
        && url.host_str().is_some_and(|host| !host.is_empty() && host.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_'))
        && url.path() == "/" && url.username().is_empty() && url.password().is_none()
        && url.port().is_none() && url.query().is_none() && url.fragment().is_none() {
        return Ok("yandex_disk");
    }
    if url.scheme() != "https" || url.port_or_known_default() != Some(443) || !url.username().is_empty() || url.password().is_some() || url.query().is_some() || url.fragment().is_some() { return Err("public links must use HTTPS without credentials, ports, query, or fragment"); }
    let segments = url.path_segments().ok_or("invalid public folder path")?.collect::<Vec<_>>();
    match (url.host_str(), segments.as_slice()) {
        (Some("disk.yandex.ru"), ["d", token]) if !token.is_empty() => Ok("yandex_disk"),
        (Some("cloud.mail.ru"), ["public", owner, token]) if !owner.is_empty() && !token.is_empty() => Ok("mail_cloud"),
        _ => Err("URL is not a supported public folder link"),
    }
}
