# lepton-spectra-telemetry

Spectra ops counters and failure events for Lepton delivery and auth funnels.
Workspace member of [`lepton`](../); depends on `uf-spectra`.

**Source of truth for teaching:** `cargo doc -p lepton-spectra-telemetry --open`.

```toml
lepton-spectra-telemetry = { path = "../lepton-spectra-telemetry" }
# Or enable emit from adapters / auth library:
# lepton-smtp = { path = "../lepton-smtp", features = ["spectra"] }
# lepton-sms = { path = "../lepton-sms", features = ["spectra"] }
# lepton-auth = { path = "../lepton-auth", features = ["ssr", "spectra"] }
```

## Features

- **Delivery counters** — `record_email_send`, `record_sms_send`
- **Auth funnel counters** — `record_signin`, `record_oauth`, and related helpers
- **Failure events** — `log_auth_failure` with bounded `reason_class` tokens

## Getting started

Boot Spectra in the host, then record:

```rust,ignore
use lepton_spectra_telemetry::{record_email_send, EmailSendOutcome};

record_email_send("noop", EmailSendOutcome::Success);
```

Runnable smoke (boots mem Spectra and queries the counter):

```bash
cargo run -p lepton-spectra-telemetry --example email_send_record_smoke
```

Labels are ops-id only. Emails, phones, user ids, passwords, OTPs, tokens, challenge
ids, bodies, and free-form error text are never recorded.

## Feature flags

This crate has no Cargo feature flags. Adapter emit uses `spectra` on
`lepton-smtp` / `lepton-sms` / `lepton-auth`.

## Verify

```bash
export CARGO_TARGET_DIR=target-lepton CARGO_BUILD_JOBS=1
cargo test -p lepton-spectra-telemetry
cargo test -p lepton-smtp --features spectra
cargo test -p lepton-sms --features spectra
cargo check -p lepton-auth --features ssr,full,spectra
```

## License

MIT
