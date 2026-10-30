//! Google OAuth 2.0 for installed apps: PKCE + loopback redirect.
//! The refresh token lives in the OS credential store (Windows Credential Manager);
//! the short-lived access token is cached in memory only.

use std::time::{Duration, Instant};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    sync::Mutex,
};

use crate::error::{Error, Result};

const AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const REVOKE_URL: &str = "https://oauth2.googleapis.com/revoke";
const SCOPE: &str = "https://www.googleapis.com/auth/calendar.readonly";
const KEYRING_SERVICE: &str = "com.calview.app";
const KEYRING_USER: &str = "google-refresh-token";
const LOGIN_TIMEOUT: Duration = Duration::from_secs(300);

const DONE_PAGE: &str = "<!doctype html><html><body style=\"font-family:system-ui;text-align:center;padding-top:4rem\">\
<h2>Signed in</h2><p>You can close this tab and return to the app.</p></body></html>";

#[derive(Default)]
pub struct AuthState {
    access: Mutex<Option<(String, Instant)>>,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: u64,
    refresh_token: Option<String>,
}

#[derive(Deserialize)]
struct TokenError {
    error: String,
    error_description: Option<String>,
}

fn client_id() -> Result<&'static str> {
    option_env!("GOOGLE_CLIENT_ID").filter(|s| !s.is_empty()).ok_or(Error::Config)
}

fn client_secret() -> Result<&'static str> {
    option_env!("GOOGLE_CLIENT_SECRET").filter(|s| !s.is_empty()).ok_or(Error::Config)
}

fn entry() -> Result<keyring::Entry> {
    Ok(keyring::Entry::new(KEYRING_SERVICE, KEYRING_USER)?)
}

fn random_b64(bytes: usize) -> String {
    let mut buf = vec![0u8; bytes];
    rand::thread_rng().fill_bytes(&mut buf);
    URL_SAFE_NO_PAD.encode(buf)
}

pub fn is_logged_in() -> bool {
    entry().and_then(|e| Ok(e.get_password()?)).is_ok()
}

pub async fn login(app: &AppHandle, state: &AuthState, http: &reqwest::Client) -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let redirect_uri = format!("http://127.0.0.1:{}", listener.local_addr()?.port());
    let verifier = random_b64(32);
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let csrf = random_b64(16);

    let url = reqwest::Url::parse_with_params(
        AUTH_URL,
        &[
            ("client_id", client_id()?),
            ("redirect_uri", redirect_uri.as_str()),
            ("response_type", "code"),
            ("scope", SCOPE),
            ("code_challenge", challenge.as_str()),
            ("code_challenge_method", "S256"),
            ("state", csrf.as_str()),
            ("access_type", "offline"),
            ("prompt", "consent"),
        ],
    )
    .map_err(|e| Error::Other(e.to_string()))?;

    app.opener()
        .open_url(url.as_str(), None::<&str>)
        .map_err(|e| Error::Other(format!("Could not open browser: {e}")))?;

    let code = tokio::time::timeout(LOGIN_TIMEOUT, wait_for_code(&listener, &csrf))
        .await
        .map_err(|_| Error::Other("Sign-in timed out. Please try again.".into()))??;

    let tokens = request_token(
        http,
        &[
            ("grant_type", "authorization_code"),
            ("code", code.as_str()),
            ("redirect_uri", redirect_uri.as_str()),
            ("code_verifier", verifier.as_str()),
        ],
    )
    .await?;

    let refresh = tokens
        .refresh_token
        .ok_or_else(|| Error::Other("Google did not return a refresh token".into()))?;
    entry()?.set_password(&refresh)?;
    *state.access.lock().await = Some((
        tokens.access_token,
        Instant::now() + Duration::from_secs(tokens.expires_in),
    ));
    Ok(())
}

/// Accepts connections on the loopback listener until Google redirects back with `?code=`.
async fn wait_for_code(listener: &TcpListener, expected_state: &str) -> Result<String> {
    loop {
        let (mut stream, _) = listener.accept().await?;
        let mut buf = vec![0u8; 8192];
        let n = stream.read(&mut buf).await?;
        let request = String::from_utf8_lossy(&buf[..n]);
        let path = request
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .unwrap_or("/");

        let url = reqwest::Url::parse(&format!("http://127.0.0.1{path}"))
            .map_err(|e| Error::Other(e.to_string()))?;
        let mut code = None;
        let mut state = None;
        let mut error = None;
        for (k, v) in url.query_pairs() {
            match k.as_ref() {
                "code" => code = Some(v.into_owned()),
                "state" => state = Some(v.into_owned()),
                "error" => error = Some(v.into_owned()),
                _ => {}
            }
        }

        if code.is_none() && error.is_none() {
            // e.g. /favicon.ico — ignore and keep waiting
            let _ = stream
                .write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .await;
            continue;
        }

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            DONE_PAGE.len(),
            DONE_PAGE
        );
        let _ = stream.write_all(response.as_bytes()).await;

        if let Some(err) = error {
            return Err(Error::Other(format!("Sign-in was cancelled ({err})")));
        }
        if state.as_deref() != Some(expected_state) {
            return Err(Error::Other("Sign-in failed: state mismatch".into()));
        }
        return Ok(code.unwrap());
    }
}

async fn request_token(http: &reqwest::Client, params: &[(&str, &str)]) -> Result<TokenResponse> {
    let mut form = vec![("client_id", client_id()?), ("client_secret", client_secret()?)];
    form.extend_from_slice(params);
    let res = http.post(TOKEN_URL).form(&form).send().await?;
    if res.status().is_success() {
        return Ok(res.json().await?);
    }
    let err: TokenError = res
        .json()
        .await
        .unwrap_or(TokenError { error: "unknown".into(), error_description: None });
    if err.error == "invalid_grant" {
        return Err(Error::AuthExpired);
    }
    Err(Error::Other(format!(
        "Token request failed: {}",
        err.error_description.unwrap_or(err.error)
    )))
}

/// Returns a valid access token, refreshing it with the stored refresh token if needed.
pub async fn access_token(state: &AuthState, http: &reqwest::Client) -> Result<String> {
    // Holding the lock across the refresh keeps concurrent callers from refreshing twice.
    let mut cached = state.access.lock().await;
    if let Some((token, expires)) = cached.as_ref() {
        if Instant::now() + Duration::from_secs(60) < *expires {
            return Ok(token.clone());
        }
    }

    let refresh = match entry()?.get_password() {
        Ok(r) => r,
        Err(keyring::Error::NoEntry) => return Err(Error::AuthExpired),
        Err(e) => return Err(e.into()),
    };
    let tokens = match request_token(
        http,
        &[("grant_type", "refresh_token"), ("refresh_token", refresh.as_str())],
    )
    .await
    {
        Err(Error::AuthExpired) => {
            let _ = entry()?.delete_credential();
            return Err(Error::AuthExpired);
        }
        other => other?,
    };

    *cached = Some((
        tokens.access_token.clone(),
        Instant::now() + Duration::from_secs(tokens.expires_in),
    ));
    Ok(tokens.access_token)
}

pub async fn logout(state: &AuthState, http: &reqwest::Client) -> Result<()> {
    *state.access.lock().await = None;
    let entry = entry()?;
    if let Ok(refresh) = entry.get_password() {
        // Best effort: revoke at Google so the grant disappears from the account too.
        let _ = http.post(REVOKE_URL).form(&[("token", refresh.as_str())]).send().await;
    }
    match entry.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(e.into()),
    }
}
