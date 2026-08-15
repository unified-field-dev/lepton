//! Loopback HTTP catcher for live Google OAuth redirects.

use std::collections::HashMap;
use std::time::Duration;

use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tracing::info;

use crate::error::LiveVerifyError;
use crate::oauth_flow::{OAuthCodeSource, OAuthPhase};

const CALLBACK_HTML: &str =
    "Lepton live OAuth: you can close this tab and return to the terminal.\n";
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(300);

/// One-shot localhost callback source for [`crate::oauth_flow::run_oauth_signup_login_flow`].
///
/// Binds `127.0.0.1` only. Call [`Self::listen_addr`] after [`Self::bind`] for operator docs.
pub struct LocalhostOAuthCodeSource {
    listener: TcpListener,
    redirect_path: String,
    timeout: Duration,
}

impl LocalhostOAuthCodeSource {
    /// Bind a loopback listener on `port` (0 = ephemeral).
    ///
    /// # Errors
    ///
    /// [`LiveVerifyError::Config`] when bind fails.
    pub async fn bind(
        port: u16,
        redirect_path: impl Into<String>,
    ) -> Result<Self, LiveVerifyError> {
        let listener = TcpListener::bind(("127.0.0.1", port))
            .await
            .map_err(|_| LiveVerifyError::config("oauth_callback_bind"))?;
        Ok(Self {
            listener,
            redirect_path: normalize_path(redirect_path.into()),
            timeout: DEFAULT_TIMEOUT,
        })
    }

    /// `host:port` currently bound (always loopback).
    ///
    /// # Errors
    ///
    /// Config when local address is unavailable.
    pub fn listen_addr(&self) -> Result<String, LiveVerifyError> {
        let addr = self
            .listener
            .local_addr()
            .map_err(|_| LiveVerifyError::config("oauth_callback_addr"))?;
        Ok(format!("127.0.0.1:{}", addr.port()))
    }

    /// Absolute callback URL for Google Cloud Console / `OAuthClientConfig`.
    ///
    /// # Errors
    ///
    /// Config when local address is unavailable.
    pub fn callback_url(&self) -> Result<String, LiveVerifyError> {
        Ok(format!(
            "http://{}{}",
            self.listen_addr()?,
            self.redirect_path
        ))
    }
}

#[async_trait]
impl OAuthCodeSource for LocalhostOAuthCodeSource {
    async fn authorization_code(
        &self,
        phase: OAuthPhase,
        authorize_url: &str,
        expected_state: &str,
    ) -> Result<String, LiveVerifyError> {
        info!(phase = "callback_listen", "live_oauth");
        let prompt = match phase {
            OAuthPhase::Signup => {
                "Open this URL in your browser, sign in with Google, and allow access:"
            }
            OAuthPhase::Login => {
                "Open this URL in your browser and sign in with the same Google account:"
            }
        };
        println!();
        println!("{prompt}");
        println!();
        println!("{authorize_url}");
        println!();
        println!("Waiting for redirect …");

        let deadline = tokio::time::Instant::now() + self.timeout;
        loop {
            let remaining = deadline.saturating_duration_since(tokio::time::Instant::now());
            if remaining.is_zero() {
                return Err(LiveVerifyError::config("oauth_callback_timeout"));
            }
            let accept = self.listener.accept();
            let (mut socket, _) = tokio::time::timeout(remaining, accept)
                .await
                .map_err(|_| LiveVerifyError::config("oauth_callback_timeout"))?
                .map_err(|_| LiveVerifyError::CodeSource)?;

            let mut buf = vec![0u8; 16_384];
            let n = match socket.read(&mut buf).await {
                Ok(0) | Err(_) => continue,
                Ok(n) => n,
            };
            let req = String::from_utf8_lossy(&buf[..n]);
            let Some((path_q, _)) = parse_request_target(&req) else {
                let _ = write_http_response(&mut socket, 400, "bad request\n").await;
                continue;
            };
            let (path, query) = split_path_query(path_q);
            if normalize_path(path.to_string()) != self.redirect_path {
                // Ignore favicon / noise; keep waiting for the real callback.
                let _ = write_http_response(&mut socket, 404, "not found\n").await;
                continue;
            }
            let params = parse_query(query);
            if params.contains_key("error") {
                let _ = write_http_response(&mut socket, 400, "oauth error\n").await;
                return Err(LiveVerifyError::oauth("oauth_provider"));
            }
            let state = params.get("state").map(String::as_str).unwrap_or("");
            if state != expected_state {
                let _ = write_http_response(&mut socket, 400, "state mismatch\n").await;
                return Err(LiveVerifyError::oauth("oauth_state"));
            }
            let Some(code) = params
                .get("code")
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
            else {
                let _ = write_http_response(&mut socket, 400, "missing code\n").await;
                return Err(LiveVerifyError::oauth("oauth_provider"));
            };

            let _ = write_http_response(&mut socket, 200, CALLBACK_HTML).await;
            // Do not log or print `code`.
            return Ok(code);
        }
    }
}

fn normalize_path(path: String) -> String {
    if path.starts_with('/') {
        path
    } else {
        format!("/{path}")
    }
}

fn parse_request_target(req: &str) -> Option<(&str, &str)> {
    let line = req.lines().next()?;
    let mut parts = line.split_whitespace();
    let method = parts.next()?;
    let target = parts.next()?;
    let version = parts.next()?;
    if method != "GET" || !version.starts_with("HTTP/") {
        return None;
    }
    Some((target, version))
}

fn split_path_query(target: &str) -> (&str, &str) {
    match target.split_once('?') {
        Some((p, q)) => (p, q),
        None => (target, ""),
    }
}

fn parse_query(query: &str) -> HashMap<String, String> {
    let mut out = HashMap::new();
    if query.is_empty() {
        return out;
    }
    for (k, v) in url::form_urlencoded::parse(query.as_bytes()) {
        out.insert(k.into_owned(), v.into_owned());
    }
    out
}

async fn write_http_response(
    socket: &mut tokio::net::TcpStream,
    status: u16,
    body: &str,
) -> Result<(), LiveVerifyError> {
    let reason = match status {
        200 => "OK",
        400 => "Bad Request",
        404 => "Not Found",
        _ => "Error",
    };
    let resp = format!(
        "HTTP/1.1 {status} {reason}\r\nContent-Type: text/plain; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    socket
        .write_all(resp.as_bytes())
        .await
        .map_err(|_| LiveVerifyError::CodeSource)?;
    let _ = socket.shutdown().await;
    Ok(())
}
