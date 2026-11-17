use sqlx::SqlitePool;
use std::fmt;

#[derive(Clone)]
pub struct ProxyUrl(url::Url);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProxyUrlError(&'static str);

#[derive(Clone, Debug)]
pub enum EffectiveProxy {
    Direct,
    Proxy(ProxyUrl),
}

impl ProxyUrl {
    pub fn parse(raw: &str) -> Result<Self, ProxyUrlError> {
        let url = url::Url::parse(raw).map_err(|_| ProxyUrlError("proxy URL is malformed"))?;
        if !matches!(url.scheme(), "http" | "socks5") {
            return Err(ProxyUrlError("proxy URL scheme must be http or socks5"));
        }
        if url.host_str().is_none() || url.port().is_none() {
            return Err(ProxyUrlError(
                "proxy URL must include a host and explicit port",
            ));
        }
        if url.query().is_some() || url.fragment().is_some() {
            return Err(ProxyUrlError(
                "proxy URL must not contain a query or fragment",
            ));
        }
        Ok(Self(url))
    }

    pub fn as_url(&self) -> &url::Url {
        &self.0
    }

    pub fn sanitized(&self) -> String {
        let mut url = self.0.clone();
        if !url.username().is_empty() || url.password().is_some() {
            let _ = url.set_username("redacted");
            let _ = url.set_password(Some("redacted"));
        }
        url.to_string()
    }

    pub fn redact_text(&self, message: &str) -> String {
        let mut redacted = message.replace(self.0.as_str(), &self.sanitized());
        for secret in [Some(self.0.username()), self.0.password()]
            .into_iter()
            .flatten()
        {
            if secret.is_empty() {
                continue;
            }
            redacted = redacted.replace(secret, "[redacted]");
            let decoded = percent_encoding::percent_decode_str(secret)
                .decode_utf8_lossy()
                .into_owned();
            if decoded != secret {
                redacted = redacted.replace(&decoded, "[redacted]");
            }
        }
        redacted
    }
}

pub fn validate_environment() -> Result<(), ProxyUrlError> {
    let value = match std::env::var("KPAPERS_PROXY_URL") {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(()),
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(ProxyUrlError("proxy URL must be valid UTF-8"));
        }
    };
    if value.trim().is_empty() {
        return Ok(());
    }
    ProxyUrl::parse(&value).map(|_| ())
}

impl fmt::Debug for ProxyUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ProxyUrl").field(&self.sanitized()).finish()
    }
}

impl fmt::Display for ProxyUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for ProxyUrlError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProxyResolveError(&'static str);

impl fmt::Display for ProxyResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.0)
    }
}

impl std::error::Error for ProxyResolveError {}

pub fn resolve(
    mode: &str,
    feed_url: Option<&str>,
    global_url: Option<&str>,
    environment_url: Option<&str>,
) -> Result<EffectiveProxy, ProxyResolveError> {
    match mode {
        "direct" => Ok(EffectiveProxy::Direct),
        "custom" => ProxyUrl::parse(
            feed_url.ok_or(ProxyResolveError("custom feed proxy is not configured"))?,
        )
        .map(EffectiveProxy::Proxy)
        .map_err(|_| ProxyResolveError("stored feed proxy configuration is invalid")),
        "inherit" => {
            for raw in [global_url, environment_url].into_iter().flatten() {
                if raw.trim().is_empty() {
                    continue;
                }
                return ProxyUrl::parse(raw)
                    .map(EffectiveProxy::Proxy)
                    .map_err(|_| ProxyResolveError("stored proxy configuration is invalid"));
            }
            Ok(EffectiveProxy::Direct)
        }
        _ => Err(ProxyResolveError("stored feed proxy mode is invalid")),
    }
}

pub async fn resolve_for_feed(
    pool: &SqlitePool,
    feed_id: &str,
) -> Result<EffectiveProxy, ProxyResolveError> {
    let row = sqlx::query_as::<_, (String, Option<String>)>(
        "SELECT proxy_mode, proxy_url FROM feeds WHERE id=?",
    )
    .bind(feed_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| ProxyResolveError("could not load feed proxy configuration"))?
    .ok_or(ProxyResolveError("feed proxy configuration is unavailable"))?;
    let global_url = sqlx::query_scalar::<_, String>(
        "SELECT value FROM application_settings WHERE key='proxy_url'",
    )
    .fetch_optional(pool)
    .await
    .map_err(|_| ProxyResolveError("could not load global proxy configuration"))?;
    let environment = match std::env::var("KPAPERS_PROXY_URL") {
        Ok(value) if !value.trim().is_empty() => Some(value),
        Ok(_) | Err(std::env::VarError::NotPresent) => None,
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(ProxyResolveError(
                "environment proxy configuration is invalid",
            ));
        }
    };
    resolve(
        &row.0,
        row.1.as_deref(),
        global_url.as_deref(),
        environment.as_deref(),
    )
}

impl EffectiveProxy {
    pub fn redact_text(&self, message: &str) -> String {
        match self {
            Self::Direct => message.to_owned(),
            Self::Proxy(proxy) => proxy.redact_text(message),
        }
    }

    pub fn configure(
        &self,
        builder: reqwest::ClientBuilder,
    ) -> Result<reqwest::ClientBuilder, reqwest::Error> {
        let builder = builder.no_proxy();
        match self {
            Self::Direct => Ok(builder),
            Self::Proxy(proxy) => {
                let explicit = reqwest::Proxy::all(proxy.as_url().as_str())?;
                Ok(builder.proxy(explicit))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_http_and_socks5_urls_with_optional_credentials() {
        for raw in [
            "http://proxy.example:8080",
            "http://user:secret@proxy.example:8080",
            "socks5://proxy.example:1080",
            "socks5://user:secret@proxy.example:1080",
        ] {
            assert!(ProxyUrl::parse(raw).is_ok(), "{raw}");
        }
    }

    #[test]
    fn rejects_unsupported_or_incomplete_proxy_urls_without_echoing_input() {
        for raw in [
            "https://proxy.example:443",
            "ftp://proxy.example:21",
            "http://proxy.example",
            "http://[not-ip]:8080",
            "not a URL containing secret",
            "socks5://proxy.example:70000",
            "http://proxy.example:8080/path?token=secret",
        ] {
            let error = ProxyUrl::parse(raw).unwrap_err();
            assert!(!error.to_string().contains("secret"));
            assert!(!error.to_string().contains(raw));
        }
    }

    #[test]
    fn hides_proxy_credentials_in_sanitized_and_debug_forms() {
        let proxy = ProxyUrl::parse("http://alice:s3cr3t@proxy.example:8080").unwrap();
        assert_eq!(
            proxy.sanitized(),
            "http://redacted:redacted@proxy.example:8080/"
        );
        assert!(!format!("{proxy:?}").contains("s3cr3t"));
    }

    #[test]
    fn removes_encoded_and_decoded_proxy_credentials_from_error_text() {
        let proxy = ProxyUrl::parse("http://alice:p%40ss@proxy.example:8080").unwrap();
        let message = "request failed for http://alice:p%40ss@proxy.example:8080: p@ss";
        let redacted = proxy.redact_text(message);
        assert!(!redacted.contains("alice"));
        assert!(!redacted.contains("p%40ss"));
        assert!(!redacted.contains("p@ss"));
        assert!(redacted.contains("redacted"));
    }

    #[test]
    fn environment_validation_accepts_empty_or_valid_and_rejects_malformed_values() {
        assert!(parse_environment_value(None).unwrap().is_none());
        assert!(parse_environment_value(Some(" ")).unwrap().is_none());
        assert!(
            parse_environment_value(Some("socks5://proxy.example:1080"))
                .unwrap()
                .is_some()
        );
        let error = parse_environment_value(Some("http://bad.example"))
            .unwrap_err()
            .to_string();
        assert_eq!(error, "proxy URL must include a host and explicit port");
        assert!(!error.contains("bad.example"));
    }

    #[test]
    fn resolves_feed_global_environment_and_direct_precedence() {
        let custom = resolve(
            "custom",
            Some("http://feed.example:8080"),
            Some("http://global.example:8080"),
            Some("socks5://environment.example:1080"),
        )
        .unwrap();
        assert!(
            matches!(custom, EffectiveProxy::Proxy(ref proxy) if proxy.as_url().host_str() == Some("feed.example"))
        );

        assert!(matches!(
            resolve(
                "direct",
                None,
                Some("http://global.example:8080"),
                Some("http://env.example:8080")
            )
            .unwrap(),
            EffectiveProxy::Direct
        ));
        assert!(matches!(
            resolve("inherit", None, Some("http://global.example:8080"), Some("http://env.example:8080")).unwrap(),
            EffectiveProxy::Proxy(ref proxy) if proxy.as_url().host_str() == Some("global.example")
        ));
        assert!(matches!(
            resolve("inherit", None, None, Some("socks5://env.example:1080")).unwrap(),
            EffectiveProxy::Proxy(ref proxy) if proxy.as_url().host_str() == Some("env.example")
        ));
        assert!(matches!(
            resolve("inherit", None, None, None).unwrap(),
            EffectiveProxy::Direct
        ));
    }

    #[test]
    fn invalid_selected_proxy_never_falls_through_to_a_lower_precedence_value() {
        let error = resolve(
            "custom",
            Some("http://invalid.example"),
            Some("http://global.example:8080"),
            None,
        )
        .unwrap_err();
        assert_eq!(
            error.to_string(),
            "stored feed proxy configuration is invalid"
        );
    }

    fn parse_environment_value(value: Option<&str>) -> Result<Option<ProxyUrl>, ProxyUrlError> {
        value
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(ProxyUrl::parse)
            .transpose()
    }
}
