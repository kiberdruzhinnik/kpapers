use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{
    Extension, Json,
    http::{HeaderMap, HeaderValue, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use hmac::{Hmac, Mac};
use rand::{RngCore, rngs::OsRng};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::{
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

type HmacSha256 = Hmac<Sha256>;
const COOKIE: &str = "kpapers_session";
const SESSION_SECONDS: u64 = 43_200;

#[derive(Clone)]
pub struct AuthConfig {
    username: Arc<str>,
    password_hash: Arc<str>,
    signing_key: Arc<[u8]>,
    secure_cookie: bool,
}

impl AuthConfig {
    pub fn from_env() -> Result<Self, String> {
        let username = std::env::var("OPERATOR_USERNAME").unwrap_or_else(|_| "operator".into());
        let password_hash = std::env::var("OPERATOR_PASSWORD_HASH")
            .map_err(|_| "OPERATOR_PASSWORD_HASH must be an Argon2 PHC hash".to_owned())?;
        let key = std::env::var("SESSION_SIGNING_KEY")
            .map_err(|_| "SESSION_SIGNING_KEY must be configured".to_owned())?;
        if key.len() < 32 {
            return Err("SESSION_SIGNING_KEY must be at least 32 bytes".into());
        }
        let secure_cookie = std::env::var("COOKIE_SECURE").map_or(true, |v| v != "false");
        Ok(Self {
            username: username.into(),
            password_hash: password_hash.into(),
            signing_key: key.into_bytes().into(),
            secure_cookie,
        })
    }
    fn sign(&self, bytes: &[u8]) -> Vec<u8> {
        let mut mac = HmacSha256::new_from_slice(&self.signing_key).expect("HMAC accepts any key");
        mac.update(bytes);
        mac.finalize().into_bytes().to_vec()
    }
    fn valid_session(&self, headers: &HeaderMap) -> bool {
        let Some(cookie) = headers.get(header::COOKIE).and_then(|v| v.to_str().ok()) else {
            return false;
        };
        valid_session_cookie(cookie, &self.signing_key, now())
    }
}

pub(crate) fn valid_session_cookie(cookie: &str, signing_key: &[u8], now: u64) -> bool {
    let prefix = format!("{COOKIE}=");
    let Some(value) = cookie
        .split(';')
        .map(str::trim)
        .find_map(|c| c.strip_prefix(&prefix))
    else {
        return false;
    };
    let Ok(decoded) = URL_SAFE_NO_PAD.decode(value) else {
        return false;
    };
    if decoded.len() < 33 {
        return false;
    }
    let (payload, signature) = decoded.split_at(decoded.len() - 32);
    let Ok(mut mac) = HmacSha256::new_from_slice(signing_key) else {
        return false;
    };
    mac.update(payload);
    if mac.verify_slice(signature).is_err() {
        return false;
    }
    std::str::from_utf8(payload)
        .ok()
        .and_then(|p| p.split(':').next())
        .and_then(|e| e.parse::<u64>().ok())
        .is_some_and(|e| e > now)
}

#[doc(hidden)]
pub fn test_auth_config(username: &str, password: &str) -> AuthConfig {
    use argon2::password_hash::{PasswordHasher, SaltString};
    let salt = SaltString::from_b64("c2FsdHNhbHQ").unwrap();
    let password_hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();
    AuthConfig {
        username: username.into(),
        password_hash: password_hash.into(),
        signing_key: vec![b't'; 32].into(),
        secure_cookie: false,
    }
}

#[derive(Deserialize)]
pub struct LoginRequest {
    username: String,
    password: String,
}
#[derive(Serialize)]
pub struct LoginResponse {
    authenticated: bool,
}

pub async fn login(
    Extension(auth): Extension<AuthConfig>,
    Json(input): Json<LoginRequest>,
) -> Response {
    let valid_password = PasswordHash::new(&auth.password_hash)
        .ok()
        .is_some_and(|hash| {
            Argon2::default()
                .verify_password(input.password.as_bytes(), &hash)
                .is_ok()
        });
    if input.username != *auth.username || !valid_password {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut nonce = [0u8; 16];
    OsRng.fill_bytes(&mut nonce);
    let payload = format!(
        "{}:{}",
        now() + SESSION_SECONDS,
        URL_SAFE_NO_PAD.encode(nonce)
    );
    let token = URL_SAFE_NO_PAD
        .encode([payload.as_bytes(), auth.sign(payload.as_bytes()).as_slice()].concat());
    let secure = if auth.secure_cookie { "; Secure" } else { "" };
    let cookie = format!(
        "{COOKIE}={token}; Path=/api/management; HttpOnly; SameSite=Strict; Max-Age={SESSION_SECONDS}{secure}"
    );
    let mut response = Json(LoginResponse {
        authenticated: true,
    })
    .into_response();
    if let Ok(value) = HeaderValue::from_str(&cookie) {
        response.headers_mut().insert(header::SET_COOKIE, value);
    }
    response
}

pub async fn logout() -> Response {
    let mut response = StatusCode::NO_CONTENT.into_response();
    response.headers_mut().insert(
        header::SET_COOKIE,
        HeaderValue::from_static(
            "kpapers_session=; Path=/api/management; HttpOnly; SameSite=Strict; Max-Age=0; Secure",
        ),
    );
    response
}

pub async fn require_session(
    Extension(auth): Extension<AuthConfig>,
    request: axum::http::Request<axum::body::Body>,
    next: Next,
) -> Response {
    if !auth.valid_session(request.headers()) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    next.run(request).await
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> AuthConfig {
        AuthConfig {
            username: "operator".into(),
            password_hash: "unused-in-session-tests".into(),
            signing_key: vec![b'k'; 32].into(),
            secure_cookie: true,
        }
    }

    fn session_cookie(config: &AuthConfig, expiry: u64) -> String {
        let payload = format!("{expiry}:nonce");
        let signature = config.sign(payload.as_bytes());
        let token = URL_SAFE_NO_PAD.encode([payload.as_bytes(), signature.as_slice()].concat());
        format!("{COOKIE}={token}")
    }

    #[test]
    fn accepts_only_current_sessions_with_valid_signatures() {
        let config = config();
        let mut headers = HeaderMap::new();
        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&session_cookie(&config, now() + 60)).unwrap(),
        );
        assert!(config.valid_session(&headers));

        headers.insert(
            header::COOKIE,
            HeaderValue::from_str(&session_cookie(&config, now() - 1)).unwrap(),
        );
        assert!(!config.valid_session(&headers));

        headers.insert(
            header::COOKIE,
            HeaderValue::from_static("kpapers_session=not-base64!"),
        );
        assert!(!config.valid_session(&headers));

        headers.insert(header::COOKIE, HeaderValue::from_static("other=value"));
        assert!(!config.valid_session(&headers));
    }

    #[test]
    fn rejects_tampered_session_signatures() {
        let config = config();
        let cookie = session_cookie(&config, now() + 60);
        let (name, token) = cookie.split_once('=').unwrap();
        let mut decoded = URL_SAFE_NO_PAD.decode(token).unwrap();
        *decoded.last_mut().unwrap() ^= 1;
        let tampered = format!("{name}={}", URL_SAFE_NO_PAD.encode(decoded));
        let mut headers = HeaderMap::new();
        headers.insert(header::COOKIE, HeaderValue::from_str(&tampered).unwrap());
        assert!(!config.valid_session(&headers));
    }
}
