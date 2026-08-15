# lepton-smtp

Provider-agnostic email delivery for Lepton Auth hosts.

SMS lives in [`lepton-sms`](../lepton-sms/).

## What it provides

- Builder-first email construction (`EmailServiceBuilder`, `SmtpConfig::builder`)
- Optional host env helpers (`from_env`, `build_email_service_from_env`) — boot only
- Drivers: `smtp`, `direct_mx`, `noop`, and optional `twilio` (SendGrid Mail Send)
- Verification + password-reset envelope builders
- Structured `tracing` (no recipient addresses, bodies, or passwords/API keys in fields)

## Feature flags

| Feature | Effect |
|---------|--------|
| *(default)* | SMTP / DirectMX / Noop |
| `twilio` | Live `TwilioEmailAdapter` (Twilio SendGrid Mail Send v3) |
| `spectra` | Emit `lepton_email_send{driver,outcome}` via [`lepton-spectra-telemetry`](../lepton-spectra-telemetry/) |

Twilio email uses a **SendGrid API key** (`UF_TWILIO_EMAIL_API_KEY`), not SMS Account SID / Auth Token.

With `spectra`, boot Spectra in the host first (for example
`spectra_uf_embedded::install_embedded_sqlite`). Counters are best-effort and never
fail the send. Labels are `driver` + `outcome` only — no recipient or body.

## Concern → API

| Concern | API |
|---------|-----|
| Construct email service | [`EmailServiceBuilder`], [`SmtpConfig::builder`] |
| Host env helper | [`EmailServiceBuilder::from_env`], [`build_email_service_from_env`] |
| Send email | [`EmailDeliveryService`] |
| Twilio / SendGrid | [`TwilioEmailConfig`], [`TwilioEmailAdapter`] (`twilio` feature) |
| Errors | [`EmailDeliveryError`] |

## Driver selection

- If `UF_EMAIL_DRIVER` is set, it wins (`smtp` / `direct_mx` / `noop` / `twilio` with feature).
- Else if `UF_SMTP_HOST` is empty/unset → **noop** (local/CI default).
- Else → **smtp**.

## Builder example (preferred)

```rust,ignore
use lepton_smtp::{EmailServiceBuilder, SmtpConfig};

let email = EmailServiceBuilder::new()
    .smtp(
        SmtpConfig::builder()
            .host("127.0.0.1")
            .port(1025)
            .use_tls(false)
            .from_email("noreply@example.test")
            .build()?,
    )
    .build()?;
```

## Builder example (Twilio SendGrid)

```rust,ignore
use lepton_smtp::{EmailServiceBuilder, TwilioEmailConfig};

let email = EmailServiceBuilder::new()
    .twilio(
        TwilioEmailConfig::builder()
            .api_key(std::env::var("UF_TWILIO_EMAIL_API_KEY")?)
            .from_email("noreply@example.com")
            .from_name("App")
            .build()?,
    )
    .build()?;
```

Enable with `lepton-smtp = { version = "…", features = ["twilio"] }`.

## Local Mailpit harness

See [`infra/mailpit/README.md`](../infra/mailpit/README.md):

```bash
./infra/mailpit/smtp_smoke.sh
```

Mailpit needs plain SMTP (`use_tls=false`, no credentials).

## Env helper example (host boot only)

```bash
export UF_EMAIL_DRIVER="smtp"
export UF_EMAIL_FROM="noreply@example.com"
export UF_EMAIL_FROM_NAME="Lepton Auth"
export UF_SMTP_HOST="smtp.your-provider.com"
export UF_SMTP_PORT="587"
export UF_SMTP_USE_TLS="true"
```

Twilio SendGrid (with `twilio` feature):

```bash
export UF_EMAIL_DRIVER="twilio"
export UF_TWILIO_EMAIL_API_KEY="SG...."
export UF_EMAIL_FROM="noreply@example.com"
export UF_EMAIL_FROM_NAME="Lepton Auth"
```

Do not call `from_env` on every send — inject `Arc<dyn EmailDeliveryService>` once (see `lepton-auth::services`).

## Direct-to-MX

Commonly needs outbound port `25`. Prefer `smtp` relay for local development.

## Integration checklist

1. Build email at host boot.
2. Inject into `lepton-auth` — never rebuild from env on the send path.
3. Secrets are plain host strings; this crate does not load a secrets manager.
4. Prefer the Mailpit harness for real SMTP; CI unit tests stay Docker-free.

Durable delivery attempts and retry: `lepton-auth` `delivery` module (`boson-delivery` feature).
Spectra counters: feature `spectra` + [`lepton-spectra-telemetry`](../lepton-spectra-telemetry/).
