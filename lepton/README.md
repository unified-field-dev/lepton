# lepton

Thin user-domain contract crate — re-exports generated SSR bindings from
`lepton-host-adapter` behind the `ssr` feature.

```toml
lepton = { git = "https://github.com/unified-field-dev/lepton", package = "lepton", default-features = false, features = ["ssr"] }
```

```rust
// SSR-enabled crate:
use lepton::generated::*;
```

Hydrate / default builds stay minimal. Prefer depending on `lepton-identity` or
`lepton-host-adapter` directly when you need a clearer boundary.
