pub mod local_fixtures;
pub mod mail_cloud;
pub mod yandex_disk;

use crate::proxy::EffectiveProxy;

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

const MAX_DOWNLOAD_REDIRECTS: usize = 5;

pub async fn download_from_url(
    raw: &str,
    maximum_bytes: u64,
    proxy: &EffectiveProxy,
) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
    let mut current =
        url::Url::parse(raw).map_err(|_| "provider returned an invalid download URL")?;
    for redirect_count in 0..=MAX_DOWNLOAD_REDIRECTS {
        let (host, port) = validate_download_url_shape(&current)?;
        let addresses = resolve_public_addresses(&host, port).await?;
        let client = download_client(&host, &addresses, std::time::Duration::from_secs(60), proxy)
            .map_err(|error| safe_reqwest_error(error, proxy))?;
        let response = client
            .get(current.clone())
            .send()
            .await
            .map_err(|error| safe_reqwest_error(error, proxy))?;
        if response.status().is_redirection() {
            if redirect_count == MAX_DOWNLOAD_REDIRECTS {
                return Err("download exceeded the redirect limit".into());
            }
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or("download redirect did not include a valid location")?;
            current = validate_redirect_target(&current, location).await?;
            continue;
        }
        if !response.status().is_success() {
            return Err(format!("download server returned HTTP {}", response.status()).into());
        }
        if response_size_exceeds(maximum_bytes, response.content_length()) {
            return Err("download exceeds configured size limit".into());
        }
        let mut response = response;
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|error| safe_reqwest_error(error, proxy))?
        {
            if !download_chunk_fits(maximum_bytes, bytes.len() as u64, chunk.len() as u64) {
                return Err("download exceeds configured size limit".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        return Ok(bytes);
    }
    Err("download exceeded the redirect limit".into())
}

fn response_size_exceeds(maximum_bytes: u64, content_length: Option<u64>) -> bool {
    content_length.is_some_and(|size| size > maximum_bytes)
}

fn download_chunk_fits(maximum_bytes: u64, current_bytes: u64, chunk_bytes: u64) -> bool {
    current_bytes
        .checked_add(chunk_bytes)
        .is_some_and(|total| total <= maximum_bytes)
}

fn download_client(
    host: &str,
    addresses: &[std::net::SocketAddr],
    timeout: std::time::Duration,
    proxy: &EffectiveProxy,
) -> Result<reqwest::Client, reqwest::Error> {
    let builder = reqwest::Client::builder()
        .timeout(timeout)
        .redirect(reqwest::redirect::Policy::none());
    match proxy {
        EffectiveProxy::Direct => builder.no_proxy().resolve_to_addrs(host, addresses).build(),
        EffectiveProxy::Proxy(_) => proxy.configure(builder)?.build(),
    }
}

async fn validate_redirect_target(
    current: &url::Url,
    location: &str,
) -> Result<url::Url, Box<dyn std::error::Error + Send + Sync>> {
    let target = current
        .join(location)
        .map_err(|_| "download redirect included an invalid location")?;
    let (host, port) = validate_download_url_shape(&target)?;
    resolve_public_addresses(&host, port).await?;
    Ok(target)
}

pub async fn validate_public_download_url(
    raw: &str,
) -> Result<url::Url, Box<dyn std::error::Error + Send + Sync>> {
    let url = url::Url::parse(raw).map_err(|_| "provider returned an invalid download URL")?;
    let (host, port) = validate_download_url_shape(&url)?;
    resolve_public_addresses(&host, port).await?;
    Ok(url)
}

pub fn validate_download_url_shape(
    url: &url::Url,
) -> Result<(String, u16), Box<dyn std::error::Error + Send + Sync>> {
    if url.scheme() != "https" {
        return Err("download URL must use HTTPS".into());
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err("download URL must not contain credentials".into());
    }
    let host = url.host_str().ok_or("download URL has no host")?.to_owned();
    let port = url
        .port_or_known_default()
        .ok_or("download URL has no port")?;
    Ok((host, port))
}

async fn resolve_public_addresses(
    host: &str,
    port: u16,
) -> Result<Vec<std::net::SocketAddr>, Box<dyn std::error::Error + Send + Sync>> {
    let addresses = if let Ok(ip) = host.parse::<std::net::IpAddr>() {
        vec![std::net::SocketAddr::new(ip, port)]
    } else {
        tokio::time::timeout(
            std::time::Duration::from_secs(10),
            tokio::net::lookup_host((host, port)),
        )
        .await
        .map_err(|_| "download host resolution timed out")?
        .map_err(|_| "download host could not be resolved")?
        .collect::<Vec<_>>()
    };
    if addresses.is_empty() {
        return Err("download host resolved to no addresses".into());
    }
    if addresses
        .iter()
        .any(|address| !is_public_address(address.ip()))
    {
        return Err("download host resolves to a private or non-public address".into());
    }
    Ok(addresses)
}

fn is_public_address(ip: std::net::IpAddr) -> bool {
    use std::net::IpAddr;
    match ip {
        IpAddr::V4(ip) => !is_in_networks(
            IpAddr::V4(ip),
            &[
                "0.0.0.0/8",
                "10.0.0.0/8",
                "100.64.0.0/10",
                "127.0.0.0/8",
                "169.254.0.0/16",
                "172.16.0.0/12",
                "192.0.0.0/24",
                "192.0.2.0/24",
                "192.88.99.0/24",
                "192.168.0.0/16",
                "198.18.0.0/15",
                "198.51.100.0/24",
                "203.0.113.0/24",
                "224.0.0.0/4",
                "240.0.0.0/4",
            ],
        ),
        IpAddr::V6(ip) => {
            if let Some(mapped) = ip.to_ipv4_mapped() {
                return is_public_address(IpAddr::V4(mapped));
            }
            is_in_networks(
                IpAddr::V6(ip),
                &["2000::/3", "2001::/23", "2001:db8::/32", "2002::/16"],
            ) && !is_in_networks(IpAddr::V6(ip), &["2001::/23", "2001:db8::/32", "2002::/16"])
        }
    }
}

fn is_in_networks(ip: std::net::IpAddr, networks: &[&str]) -> bool {
    networks.iter().any(|network| {
        network
            .parse::<ipnet::IpNet>()
            .is_ok_and(|network| network.contains(&ip))
    })
}

fn safe_reqwest_error(error: reqwest::Error, proxy: &EffectiveProxy) -> String {
    proxy
        .redact_text(&error.without_url().to_string())
        .chars()
        .take(300)
        .collect()
}

pub fn safe_client(
    provider: &'static str,
    timeout: std::time::Duration,
    proxy: &EffectiveProxy,
) -> Result<reqwest::Client, reqwest::Error> {
    let builder = proxy.configure(reqwest::Client::builder().timeout(timeout))?;
    builder
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

    #[tokio::test]
    async fn provider_clients_use_authenticated_http_proxy_for_api_requests() {
        for (provider, target) in [
            (
                "yandex",
                "https://cloud-api.yandex.net/v1/disk/public/resources",
            ),
            ("mail", "https://cloud.mail.ru/api/v2/folder"),
        ] {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            let address = listener.local_addr().unwrap();
            let proxy_server = std::thread::spawn(move || {
                use std::io::{Read, Write};
                let (mut stream, _) = listener.accept().unwrap();
                let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(3)));
                let mut request = Vec::new();
                let mut chunk = [0u8; 1024];
                while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                    let read = stream.read(&mut chunk).unwrap();
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&chunk[..read]);
                    if request.len() > 8192 {
                        break;
                    }
                }
                stream
                    .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n")
                    .unwrap();
                String::from_utf8_lossy(&request).into_owned()
            });
            let proxy =
                crate::proxy::ProxyUrl::parse(&format!("http://alice:secret@{address}")).unwrap();
            let policy = EffectiveProxy::Proxy(proxy);
            let client = safe_client(provider, std::time::Duration::from_secs(3), &policy).unwrap();
            let error = client.get(target).send().await.unwrap_err();
            let message = policy.redact_text(&error.without_url().to_string());
            assert!(!message.contains("alice"));
            assert!(!message.contains("secret"));
            let request = proxy_server.join().unwrap();
            assert!(request.starts_with(&format!(
                "CONNECT {}:443 ",
                url::Url::parse(target).unwrap().host_str().unwrap()
            )));
            let authorization = request
                .lines()
                .find(|line| {
                    line.to_ascii_lowercase()
                        .starts_with("proxy-authorization:")
                })
                .expect("proxy authentication header");
            assert_eq!(
                authorization.split_once(':').unwrap().1.trim(),
                "Basic YWxpY2U6c2VjcmV0"
            );
        }
    }

    #[tokio::test]
    async fn provider_client_authenticates_to_socks5_proxy() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let proxy_server = std::thread::spawn(move || {
            use std::io::{Read, Write};
            let (mut stream, _) = listener.accept().unwrap();
            let mut greeting = [0u8; 2];
            stream.read_exact(&mut greeting).unwrap();
            let mut methods = vec![0; greeting[1] as usize];
            stream.read_exact(&mut methods).unwrap();
            assert_eq!(greeting[0], 5);
            assert!(methods.contains(&2));
            stream.write_all(&[5, 2]).unwrap();

            let mut auth_header = [0u8; 2];
            stream.read_exact(&mut auth_header).unwrap();
            let mut username = vec![0; auth_header[1] as usize];
            stream.read_exact(&mut username).unwrap();
            let mut password_length = [0u8; 1];
            stream.read_exact(&mut password_length).unwrap();
            let mut password = vec![0; password_length[0] as usize];
            stream.read_exact(&mut password).unwrap();
            stream.write_all(&[1, 0]).unwrap();

            let mut connect_header = [0u8; 4];
            stream.read_exact(&mut connect_header).unwrap();
            let host = match connect_header[3] {
                1 => {
                    let mut address = [0; 4];
                    stream.read_exact(&mut address).unwrap();
                    std::net::Ipv4Addr::from(address).to_string()
                }
                3 => {
                    let mut length = [0; 1];
                    stream.read_exact(&mut length).unwrap();
                    let mut address = vec![0; length[0] as usize];
                    stream.read_exact(&mut address).unwrap();
                    String::from_utf8(address).unwrap()
                }
                4 => {
                    let mut address = [0; 16];
                    stream.read_exact(&mut address).unwrap();
                    std::net::Ipv6Addr::from(address).to_string()
                }
                other => panic!("unexpected SOCKS address type {other}"),
            };
            let mut port = [0; 2];
            stream.read_exact(&mut port).unwrap();
            stream.write_all(&[5, 5, 0, 1, 127, 0, 0, 1, 0, 0]).unwrap();
            (
                String::from_utf8(username).unwrap(),
                String::from_utf8(password).unwrap(),
                host,
                u16::from_be_bytes(port),
            )
        });

        let proxy =
            crate::proxy::ProxyUrl::parse(&format!("socks5://alice:secret@{address}")).unwrap();
        let policy = EffectiveProxy::Proxy(proxy);
        let client = safe_client("yandex", std::time::Duration::from_secs(3), &policy).unwrap();
        assert!(
            client
                .get("https://cloud-api.yandex.net/v1/disk/public/resources")
                .send()
                .await
                .is_err()
        );
        let (username, password, host, port) = proxy_server.join().unwrap();
        assert_eq!(username, "alice");
        assert_eq!(password, "secret");
        assert_eq!(port, 443);
        let ip = host.parse::<std::net::IpAddr>().unwrap();
        assert!(is_public_address(ip), "SOCKS5 target was not public: {ip}");
    }

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
    fn public_address_check_blocks_private_and_special_use_networks() {
        for address in [
            "10.0.0.1",
            "172.16.0.1",
            "192.168.1.1",
            "100.64.0.1",
            "127.0.0.1",
            "169.254.1.2",
            "192.0.2.1",
            "198.18.0.1",
            "224.0.0.1",
            "::",
            "::1",
            "fc00::1",
            "fe80::1",
            "ff02::1",
            "2001:db8::1",
            "2002::1",
            "::ffff:192.168.1.1",
        ] {
            let ip = address.parse().unwrap();
            assert!(
                !is_public_address(ip),
                "accepted non-public address {address}"
            );
        }
        for address in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
            let ip = address.parse().unwrap();
            assert!(is_public_address(ip), "rejected public address {address}");
        }
    }

    #[tokio::test]
    async fn download_validation_requires_https_and_rejects_private_ip_literals() {
        for (url, expected) in [
            ("http://8.8.8.8/file", "download URL must use HTTPS"),
            (
                "https://127.0.0.1/file",
                "download host resolves to a private or non-public address",
            ),
            (
                "https://user:secret@8.8.8.8/file",
                "download URL must not contain credentials",
            ),
        ] {
            let error = download_from_url(url, 1024, &crate::proxy::EffectiveProxy::Direct)
                .await
                .unwrap_err();
            assert_eq!(error.to_string(), expected);
        }
    }

    #[tokio::test]
    async fn proxied_download_uses_connect_and_rejects_private_destination_before_connecting() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        let proxy_server = std::thread::spawn(move || {
            use std::io::{Read, Write};
            let (mut stream, _) = listener.accept().unwrap();
            let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(3)));
            let mut request = Vec::new();
            let mut chunk = [0u8; 1024];
            while !request.windows(4).any(|window| window == b"\r\n\r\n") {
                let read = stream.read(&mut chunk).unwrap();
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&chunk[..read]);
                if request.len() > 8192 {
                    break;
                }
            }
            stream
                .write_all(b"HTTP/1.1 502 Bad Gateway\r\nContent-Length: 0\r\n\r\n")
                .unwrap();
            String::from_utf8_lossy(&request).into_owned()
        });
        let proxy =
            crate::proxy::ProxyUrl::parse(&format!("http://alice:secret@{address}")).unwrap();
        let policy = EffectiveProxy::Proxy(proxy);
        let error = download_from_url("https://8.8.8.8/file.pdf", 1024, &policy)
            .await
            .unwrap_err();
        assert!(!error.to_string().contains("secret"));
        let request = proxy_server.join().unwrap();
        assert!(request.starts_with("CONNECT 8.8.8.8:443 "), "{request}");

        let error = download_from_url("https://127.0.0.1/private.pdf", 1024, &policy)
            .await
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "download host resolves to a private or non-public address"
        );
    }

    #[test]
    fn download_size_guards_reject_content_lengths_and_streamed_chunks_over_the_limit() {
        assert!(!response_size_exceeds(10, None));
        assert!(!response_size_exceeds(10, Some(10)));
        assert!(response_size_exceeds(10, Some(11)));
        assert!(download_chunk_fits(10, 5, 5));
        assert!(!download_chunk_fits(10, 5, 6));
        assert!(!download_chunk_fits(u64::MAX, u64::MAX, 1));
    }

    #[test]
    fn public_download_hosts_are_not_restricted_to_provider_domains() {
        let url = url::Url::parse("https://cdn.example.org:8443/a.pdf").unwrap();
        assert_eq!(
            validate_download_url_shape(&url).unwrap(),
            ("cdn.example.org".to_owned(), 8443)
        );
    }

    #[tokio::test]
    async fn redirect_validation_rejects_non_https_and_private_targets() {
        let current = url::Url::parse("https://downloads.example.org/file.pdf").unwrap();
        let error = validate_redirect_target(&current, "http://downloads.example.org/file.pdf")
            .await
            .unwrap_err();
        assert_eq!(error.to_string(), "download URL must use HTTPS");
        let error = validate_redirect_target(&current, "https://127.0.0.1/private")
            .await
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "download host resolves to a private or non-public address"
        );
    }

    #[test]
    fn pinned_client_uses_only_the_supplied_resolution() {
        let validated = ["8.8.8.8:443".parse().unwrap()];
        assert!(
            download_client(
                "controlled-resolution.test",
                &validated,
                std::time::Duration::from_secs(2),
                &EffectiveProxy::Direct,
            )
            .is_ok()
        );
    }
}
