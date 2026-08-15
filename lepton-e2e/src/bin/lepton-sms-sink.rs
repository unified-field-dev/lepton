//! SMS HTTP capture sink on `127.0.0.1:8099` (lab only).
//!
//! ```bash
//! cargo run -p lepton-e2e --bin lepton-sms-sink
//! ```

use std::sync::Arc;

use lepton_e2e::sms_sink::{default_bind_addr, serve, MessageStore};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let addr = default_bind_addr();
    let listener = TcpListener::bind(addr).await?;
    eprintln!("lepton-sms-sink listening on http://{addr}");
    serve(listener, Arc::new(MessageStore::new())).await
}
