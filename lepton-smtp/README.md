# lepton-smtp

Provider-agnostic email delivery: build a service once, send an envelope, inspect a receipt.

Rustdoc is the teaching source of truth (`cargo doc -p lepton-smtp --open`). This README mirrors the guide vs API-reference split.

## What it provides

- Builder-first construction (`EmailServiceBuilder`, `SmtpConfig::builder`)
- Drivers: `smtp`, `direct_mx`, `noop`, and optional `twilio` (SendGrid Mail Send)
- Stock verification + password-reset envelope helpers
- Structured `tracing` (no recipient addresses, bodies, or passwords/API keys in fields)

## Getting started

### First success (Noop)

```bash
cargo run -p lepton-smtp --example noop_send
```

```rust,ignore
use lepton_smtp::{
    verification_email_envelope, EmailDeliveryService, EmailServiceBuilder,
    VerificationEmailFlow,
};

# async fn run() -> Result<(), lepton_smtp::EmailDeliveryError> {
let email = EmailServiceBuilder::new().noop().build()?;
let message = verification_email_envelope(
    "reader@example.test",
    "123456",
    VerificationEmailFlow::Signup,
);
let receipt = email.send(&message).await?;
assert_eq!(receipt.provider, "noop");
# Ok(())
# }
```

### Choose a delivery backend

| Backend | When to use | Guide (rustdoc) | API reference |
|---------|-------------|-----------------|---------------|
| **Noop** | Local / CI | Crate root “Noop” | `EmailServiceBuilder::noop`, `NoopEmailAdapter` |
| **SMTP relay** | Mailpit or SMTP host | Crate root “SMTP (Mailpit or relay)” | `EmailServiceBuilder::smtp`, `SmtpConfig`, `SmtpAdapter` |
| **Direct MX** | Recipient-domain MX (often port 25) | Crate root “Direct MX” | `EmailServiceBuilder::direct_mx`, `DirectMxConfig`, `DirectMxAdapter` |
| **Twilio SendGrid** | Live SendGrid (`twilio` feature) | Crate root “Twilio SendGrid” | `EmailServiceBuilder::twilio`, `TwilioEmailConfig`, `TwilioEmailAdapter` |

### SMTP (Mailpit or relay)

Start Mailpit (`infra/mailpit`), then build, send, and inspect the receipt:

```rust,ignore
use lepton_smtp::{
    verification_email_envelope, EmailDeliveryService, EmailServiceBuilder, SmtpConfig,
    VerificationEmailFlow,
};

# async fn run() -> Result<(), lepton_smtp::EmailDeliveryError> {
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

let message = verification_email_envelope(
    "reader@example.test",
    "123456",
    VerificationEmailFlow::Signup,
);
let receipt = email.send(&message).await?;
assert_eq!(receipt.provider, "smtp");
# Ok(())
# }
```

Local Mailpit: see [`infra/mailpit/README.md`](../infra/mailpit/README.md).

```bash
./infra/mailpit/smtp_smoke.sh
# or: UF_MAILPIT=1 cargo test -p lepton-smtp --test smtp_mailpit
```

### Direct MX

Needs outbound connectivity to MX hosts (often port 25). See rustdoc “Direct MX”.

```rust,ignore
use lepton_smtp::{
    verification_email_envelope, DirectMxConfig, EmailDeliveryService, EmailServiceBuilder,
    VerificationEmailFlow,
};

# async fn run() -> Result<(), lepton_smtp::EmailDeliveryError> {
let email = EmailServiceBuilder::new()
    .direct_mx(
        DirectMxConfig::builder()
            .from_email("noreply@example.test")
            .build()?,
    )
    .build()?;
let message = verification_email_envelope(
    "reader@example.test",
    "123456",
    VerificationEmailFlow::Signup,
);
let receipt = email.send(&message).await?;
assert!(receipt.provider.starts_with("direct_mx:"));
# Ok(())
# }
```

### Twilio SendGrid (`twilio` feature)

```rust,ignore
use lepton_smtp::{
    verification_email_envelope, EmailDeliveryService, EmailServiceBuilder, TwilioEmailConfig,
    VerificationEmailFlow,
};

# async fn run() -> Result<(), Box<dyn std::error::Error>> {
let email = EmailServiceBuilder::new()
    .twilio(
        TwilioEmailConfig::builder()
            .api_key(std::env::var("UF_TWILIO_EMAIL_API_KEY")?)
            .from_email("noreply@example.test")
            .from_name("App")
            .build()?,
    )
    .build()?;
let message = verification_email_envelope(
    "reader@example.test",
    "123456",
    VerificationEmailFlow::Signup,
);
let receipt = email.send(&message).await?;
assert_eq!(receipt.provider, "twilio");
# Ok(())
# }
```

Enable with `lepton-smtp = { version = "…", features = ["twilio"] }`.

## Feature flags

| Feature | Effect |
|---------|--------|
| *(default)* | SMTP / DirectMX / Noop |
| `twilio` | Live `TwilioEmailAdapter` (Twilio SendGrid Mail Send v3) |
| `spectra` | Emit `lepton_email_send{driver,outcome}` via [`lepton-spectra-telemetry`](../lepton-spectra-telemetry/) |

Twilio email uses a **SendGrid API key** (`UF_TWILIO_EMAIL_API_KEY`), not SMS Account SID / Auth Token.

With `spectra`, boot Spectra in the host first. Counters are best-effort and never fail the send. Labels are `driver` + `outcome` only.

## Driver selection (env helper)

- If `UF_EMAIL_DRIVER` is set, it wins (`smtp` / `direct_mx` / `noop` / `twilio` with feature).
- Else if `UF_SMTP_HOST` is empty/unset → **noop** (local/CI default).
- Else → **smtp**.

Prefer builders at boot. Env helpers (`from_env`, `build_email_service_from_env`) load credentials once; do not call them on every send.

## Env helper example (host boot only)

```bash
export UF_EMAIL_DRIVER="smtp"
export UF_EMAIL_FROM="noreply@example.com"
export UF_EMAIL_FROM_NAME="App"
export UF_SMTP_HOST="smtp.your-provider.com"
export UF_SMTP_PORT="587"
export UF_SMTP_USE_TLS="true"
```

Twilio SendGrid (with `twilio` feature):

```bash
export UF_EMAIL_DRIVER="twilio"
export UF_TWILIO_EMAIL_API_KEY="SG...."
export UF_EMAIL_FROM="noreply@example.com"
export UF_EMAIL_FROM_NAME="App"
```

## Integration checklist

1. Build email at host boot (`EmailServiceBuilder::build`).
2. Inject `Arc<dyn EmailDeliveryService>` — never rebuild from env on the send path.
3. Secrets are plain host strings; this crate does not load a secrets manager.
4. Prefer Mailpit for real SMTP; CI unit tests stay Docker-free via Noop.

## Optional integrations

- Auth hosts that inject this service: `lepton-auth` (email feature).
- SMS: [`lepton-sms`](../lepton-sms/).
- Durable delivery attempts and retry: `lepton-auth` `delivery` module (`boson-delivery` feature).
- Spectra counters: feature `spectra` + [`lepton-spectra-telemetry`](../lepton-spectra-telemetry/).
