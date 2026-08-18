# lepton

SSR identity and token model bindings for Unified Field hosts. Enable `ssr` to use
`lepton::generated` (`User`, tokens, and related Valence shapes).

```toml
lepton = { git = "https://github.com/unified-field-dev/lepton", package = "lepton", default-features = false, features = ["ssr"] }
```

```rust
use lepton::generated::User;
```

Default / hydrate builds stay empty. Depend on `lepton-identity` for ownership helpers
without the host adapter graph, or `lepton-host-adapter` when you wire session and token
schemas yourself.
