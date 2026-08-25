# lepton

SSR identity and token model bindings for Unified Field hosts. Enable `ssr` to use
`lepton::generated` (`User`, tokens, and related Valence shapes).

**Source of truth for teaching:** `cargo doc -p lepton --features ssr --open`.

```toml
lepton = { git = "https://github.com/unified-field-dev/lepton", package = "lepton", default-features = false, features = ["ssr"] }
```

## Features

- **SSR model bindings** — `generated` identity and token types from the host adapter

## Getting started

```rust
// Cargo.toml: lepton = { features = ["ssr"] }
use lepton::generated::{User, UserStatus};

fn is_active(user: &User) -> bool {
    matches!(user.status(), Some(UserStatus::Active))
}
```

Prefer `lepton-identity` for password hashing and ownership, or `lepton-host-adapter`
when wiring axum-login yourself.

## Feature flags

| Feature | Effect |
|---------|--------|
| `ssr` | Exposes `generated` from `lepton-host-adapter` |
| *(default)* | Empty surface until `ssr` is enabled |

## License

MIT. See the workspace [LICENSE](../LICENSE).
