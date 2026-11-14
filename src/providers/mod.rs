pub mod local_fixtures;
pub mod mail_cloud;
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
    let url =
        url::Url::parse(raw).map_err(|_| "provider returned an invalid download URL".to_owned())?;
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

pub fn safe_client(
    provider: &'static str,
    timeout: std::time::Duration,
) -> Result<reqwest::Client, reqwest::Error> {
    reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::custom(move |attempt| {
            let url = attempt.url();
            let host = url.host_str().unwrap_or_default();
            let allowed_host = match provider {
                "yandex" => {
                    host == "cloud-api.yandex.net"
                        || host == "disk.yandex.ru"
                        || host.ends_with(".disk.yandex.ru")
                        || host == "storage.yandex.net"
                        || host.ends_with(".storage.yandex.net")
                }
                "mail" => {
                    host == "cloud.mail.ru"
                        || host.ends_with(".cloud.mail.ru")
                        || host == "datacloudmail.ru"
                        || host.ends_with(".datacloudmail.ru")
                }
                _ => false,
            };
            if attempt.previous().len() < 5
                && url.scheme() == "https"
                && url.port_or_known_default() == Some(443)
                && allowed_host
            {
                attempt.follow()
            } else {
                attempt.stop()
            }
        }))
        .build()
}

pub fn validate_source_url(raw: &str) -> Result<&'static str, &'static str> {
    let url = url::Url::parse(raw).map_err(|_| "invalid URL")?;
    if url.scheme() == "fixture"
        && std::env::var("E2E_FIXTURE_MODE").as_deref() == Ok("true")
        && url.host_str().is_some_and(|host| {
            !host.is_empty()
                && host
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
        })
        && url.path() == "/"
        && url.username().is_empty()
        && url.password().is_none()
        && url.port().is_none()
        && url.query().is_none()
        && url.fragment().is_none()
    {
        return Ok("yandex_disk");
    }
    if url.scheme() != "https"
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err("public links must use HTTPS without credentials, ports, query, or fragment");
    }
    let segments = url
        .path_segments()
        .ok_or("invalid public folder path")?
        .collect::<Vec<_>>();
    match (url.host_str(), segments.as_slice()) {
        (Some("disk.yandex.ru"), ["d", token]) if !token.is_empty() => Ok("yandex_disk"),
        (Some("cloud.mail.ru"), ["public", owner, token])
            if !owner.is_empty() && !token.is_empty() =>
        {
            Ok("mail_cloud")
        }
        _ => Err("URL is not a supported public folder link"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_url_validation_enforces_provider_hosts_and_link_shapes() {
        assert_eq!(
            validate_source_url("https://disk.yandex.ru/d/share-token"),
            Ok("yandex_disk")
        );
        assert_eq!(
            validate_source_url("https://cloud.mail.ru/public/owner/token"),
            Ok("mail_cloud")
        );

        for input in [
            "http://disk.yandex.ru/d/token",
            "https://user@disk.yandex.ru/d/token",
            "https://disk.yandex.ru:444/d/token",
            "https://evil.example/d/token",
            "https://disk.yandex.ru/d/",
            "https://cloud.mail.ru/public/owner",
            "https://cloud.mail.ru/public/owner/token?download=1",
            "https://cloud.mail.ru/public/owner/token#fragment",
        ] {
            assert!(validate_source_url(input).is_err(), "accepted {input}");
        }
    }

    #[test]
    fn download_url_validation_enforces_provider_specific_https_allowlists() {
        for (provider, input) in [
            ("yandex_disk", "https://storage.yandex.net/file.pdf"),
            ("yandex_disk", "https://cdn.storage.yandex.net/file.pdf"),
            ("mail_cloud", "https://datacloudmail.ru/file"),
            ("mail_cloud", "https://cdn.cloud.mail.ru/file"),
        ] {
            assert!(
                validate_download_url(provider, input).is_ok(),
                "rejected allowed destination {input} for {provider}"
            );
        }

        for (provider, input) in [
            ("yandex_disk", "http://storage.yandex.net/file"),
            ("yandex_disk", "https://storage.yandex.net:444/file"),
            ("yandex_disk", "https://evil.example/file"),
            ("yandex_disk", "https://user@storage.yandex.net/file"),
            ("mail_cloud", "http://datacloudmail.ru/file"),
            ("mail_cloud", "https://datacloudmail.ru:444/file"),
            ("mail_cloud", "https://datacloudmail.ru.evil.example/file"),
            ("mail_cloud", "https://user:password@datacloudmail.ru/file"),
            ("unknown", "https://storage.yandex.net/file"),
        ] {
            assert!(
                validate_download_url(provider, input).is_err(),
                "accepted {input} for {provider}"
            );
        }
    }
}
