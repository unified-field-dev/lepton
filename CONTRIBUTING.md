# Contributing to Lepton Auth

Thank you for improving this project.

## Development setup

1. Clone [unified-field-dev/lepton](https://github.com/unified-field-dev/lepton)
2. Install Rust stable
3. From the repository root:

```bash
export CARGO_BUILD_JOBS=1
cargo fmt --check \
  -p lepton-auth -p lepton-identity -p lepton-smtp -p lepton-host-adapter \
  -p lepton
cargo clippy --workspace --all-targets --features ssr -- -D warnings
cargo test --workspace --features ssr
```

How to verify a change: [`docs/VERIFICATION.md`](docs/VERIFICATION.md).

## Code of conduct

Participation is governed by [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). Security reports: [`SECURITY.md`](SECURITY.md).

## Pull requests

- Prefer small, focused PRs.
- Update [`README.md`](README.md) when public API or host wiring steps change.
