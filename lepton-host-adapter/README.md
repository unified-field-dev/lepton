# lepton-host-adapter

Axum-login backend and token schemas that bridge **lepton-identity** to **higgs-identity** / **higgs-host**.

```toml
lepton-host-adapter = { git = "https://github.com/unified-field-dev/lepton", package = "lepton-host-adapter", default-features = false, features = ["ssr"] }
```

```rust
// Backend + session_snapshot_middleware (+ optional PhotonAuth) for SSR hosts.
// Password-reset and email-verification schemas live alongside the adapter.
```

## About

- Session snapshots without importing concrete user models into higgs-host
- Token / reset / verification Valence schemas
- Prefer the [lepton](https://github.com/unified-field-dev/lepton) workspace when you also need auth UI or SMTP

## Examples

Axum login → `SessionSnapshot` smoke (SQLite `:memory:`):
[`examples/README.md`](examples/README.md).

## Verify

```bash
export CARGO_BUILD_JOBS=1
cargo check -p lepton-host-adapter --features ssr
CARGO_BUILD_JOBS=1 cargo run -p lepton-host-adapter --example axum_session_snapshot --features ssr
```

## License

MIT. See the workspace [LICENSE](../LICENSE).
