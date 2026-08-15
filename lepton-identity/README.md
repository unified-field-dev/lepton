# lepton-identity

Leptos-free Valence models and helpers for host identity: users, accounts, memberships, and related profile material. Codegen runs from `schemas/` via `build.rs`; password hashing lives in `src/auth.rs`.

```toml
lepton-identity = { git = "https://github.com/unified-field-dev/lepton", package = "lepton-identity" }
```

```rust
use lepton_identity::auth::hash_password;
// Generated User / Account / membership models from schemas/
```

## About

- Generated identity schemas without pulling Leptos
- `hash_password` and auth primitives for headless workers
- Compose with `lepton-host-adapter`, higgs, and higgs-host for sessions

Full auth UI and SMTP live in the [lepton](https://github.com/unified-field-dev/lepton) workspace root.

## Verify

```bash
export CARGO_BUILD_JOBS=1
cargo test -p lepton-identity
```

## License

MIT. See the workspace [LICENSE](../LICENSE).
