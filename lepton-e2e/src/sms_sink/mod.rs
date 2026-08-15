//! SMS HTTP capture sink (lab; default bind `127.0.0.1:8099`).
//!
//! Captures outbound SMS JSON for local asserts — the SMS counterpart to Mailpit.
//!
//! | Concern | Start here |
//! |---------|------------|
//! | Router (tests) | [`router`] |
//! | Serve | [`serve`], [`serve_for_test`] |
//! | Store | [`MessageStore`], [`CapturedSms`] |

mod routes;
mod store;

pub use routes::router;
pub use store::{CapturedSms, MessageStore, MAX_BODY_BYTES, MAX_STORE_MESSAGES};

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;

/// Default loopback bind for the operator bin.
pub const DEFAULT_BIND: &str = "127.0.0.1:8099";

/// Serve the sink on `listener` until the process exits.
///
/// # Errors
///
/// Returns I/O errors from accept/serve.
pub async fn serve(listener: TcpListener, store: Arc<MessageStore>) -> std::io::Result<()> {
    let app = router(store);
    axum::serve(listener, app).await
}

/// Bind `addr`, return `(local_addr, join handle)` for tests.
///
/// # Errors
///
/// Returns bind failures.
pub async fn serve_for_test(
    addr: SocketAddr,
    store: Arc<MessageStore>,
) -> std::io::Result<(SocketAddr, tokio::task::JoinHandle<std::io::Result<()>>)> {
    let listener = TcpListener::bind(addr).await?;
    let local = listener.local_addr()?;
    let handle = tokio::spawn(async move { serve(listener, store).await });
    Ok((local, handle))
}

/// Resolve the default bind address (`127.0.0.1:8099`).
///
/// # Panics
///
/// Never — address is a compile-time constant.
#[must_use]
pub fn default_bind_addr() -> SocketAddr {
    DEFAULT_BIND
        .parse()
        .expect("DEFAULT_BIND is a valid socket addr")
}
