//! Mock OIDC sidecar (lab; default bind `127.0.0.1:5556`).
//!
//! Minimal discovery + authorize + token + userinfo for Lepton OAuth lab flows.
//!
//! | Concern | Start here |
//! |---------|------------|
//! | Router (tests) | [`router`] |
//! | Serve | [`serve`], [`serve_for_test`] |
//! | Identity contract | [`identity_from_code`] |

mod identity;
mod redirect;
mod routes;
mod store;

pub use identity::identity_from_code;
pub use redirect::redirect_uri_allowed;
pub use routes::router;
pub use store::{CodeStore, Identity, MAX_STORE_ENTRIES};

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Default loopback bind for the operator bin.
pub const DEFAULT_BIND: &str = "127.0.0.1:5556";

/// Default issuer URL (matches [`DEFAULT_BIND`]).
pub const DEFAULT_ISSUER: &str = "http://127.0.0.1:5556";

/// Serve the IdP on `listener` until the process exits.
///
/// # Errors
///
/// Returns I/O errors from accept/serve.
pub async fn serve(
    listener: TcpListener,
    store: Arc<CodeStore>,
    issuer: impl Into<String>,
) -> std::io::Result<()> {
    let app = router(store, issuer);
    axum::serve(listener, app).await
}

/// Bind `addr`, return `(local_addr, join handle)` for tests.
///
/// Issuer is derived as `http://{local_addr}`.
///
/// # Errors
///
/// Returns bind failures.
pub async fn serve_for_test(
    addr: SocketAddr,
    store: Arc<CodeStore>,
) -> std::io::Result<(SocketAddr, tokio::task::JoinHandle<std::io::Result<()>>)> {
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    let issuer = format!("http://{local}");
    let handle = tokio::spawn(async move { serve(listener, store, issuer).await });
    Ok((local, handle))
}

/// Resolve the default bind address (`127.0.0.1:5556`).
#[must_use]
pub fn default_bind_addr() -> SocketAddr {
    DEFAULT_BIND
        .parse()
        .expect("DEFAULT_BIND is a valid socket addr")
}
