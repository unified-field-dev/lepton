# lepton-spectra-telemetry

Spectra ops counters and failure events for Lepton delivery and auth funnels.
Workspace member of [`lepton`](../); depends on `uf-spectra`.

```toml
lepton-spectra-telemetry = { path = "../lepton-spectra-telemetry" }
# Or enable emit from adapters / auth library:
# lepton-smtp = { path = "../lepton-smtp", features = ["spectra"] }
# lepton-sms = { path = "../lepton-sms", features = ["spectra"] }
# lepton-auth = { path = "../lepton-auth", features = ["ssr", "spectra"] }
```

```rust,ignore
use lepton_spectra_telemetry::{
    record_email_send, record_signin, AuthFactor, AuthOutcome, EmailSendOutcome, SigninStage,
};

// Host must boot Spectra first (e.g. spectra_uf_embedded::install_embedded_sqlite).
record_email_send("smtp", EmailSendOutcome::Success);
record_signin(SigninStage::Password, AuthOutcome::NeedsMfa, "none", AuthFactor::None);
```

## Catalog

`store` is `lepton`. Labels are ops-id only — closed enums / `reason_class` tokens.
Emails, phones, user ids, passwords, OTPs, tokens, challenge ids, bodies, and free-form
error text are never recorded. Unknown tokens map to `unknown` / `none`.

| Name | Kind | Labels / fields |
|------|------|-----------------|
| `lepton_email_send` | counter | `driver`, `outcome` |
| `lepton_sms_send` | counter | `driver`, `outcome` |
| `lepton_signup` | counter | `outcome`, `error_class` |
| `lepton_signin` | counter | `stage`, `outcome`, `error_class`, `factor` |
| `lepton_oauth` | counter | `provider`, `intent`, `stage`, `outcome`, `error_class` |
| `lepton_verify` | counter | `channel`, `stage`, `outcome`, `error_class` |
| `lepton_password_reset` | counter | `stage`, `outcome`, `error_class` |
| `lepton_totp` | counter | `operation`, `outcome`, `error_class` |
| `lepton_device` | counter | `device_kind`, `operation`, `outcome`, `error_class` |
| `lepton_contact` | counter | `channel`, `operation`, `outcome`, `error_class` |
| `lepton_account` | counter | `operation`, `outcome`, `error_class` |
| `lepton_identity_delete` | counter | `operation`, `outcome`, `error_class` |
| `lepton_step_up` | counter | `path`, `outcome`, `error_class` |
| `lepton_auth_failure` | event | `flow`, `operation`, `error_class`, `provider`, `channel` |

## Concern → API

| Concern | API |
|---------|-----|
| Delivery emit | [`record_email_send`], [`record_sms_send`] |
| Auth funnel emit | [`record_signup`], [`record_signin`], [`record_oauth`], … |
| Failure events | [`log_auth_failure`] |
| Typed helpers | [`helpers`] |
| Topics | [`topics`] |

There is no process-wide install switch. Boot Spectra in the host, then call the helpers
or enable the `spectra` feature on `lepton-smtp` / `lepton-sms` / `lepton-auth`.

## Examples

```bash
cargo run -p lepton-spectra-telemetry --example email_send_record_smoke
```

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
