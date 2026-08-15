//! Mock OIDC sidecar on `127.0.0.1:5556` (lab only).
//!
//! ```bash
//! cargo run -p lepton-e2e --bin lepton-mock-oidc
//! ```

use std::sync::Arc;

use lepton_e2e::mock_oidc::{default_bind_addr, serve, CodeStore, DEFAULT_ISSUER};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = default_bind_addr();
    let listener = TcpListener::bind(addr).await?;
    eprintln!("lepton-mock-oidc listening on {DEFAULT_ISSUER}");
    serve(listener, Arc::new(CodeStore::new()), DEFAULT_ISSUER).await
}
