# lepton-sms

Provider-agnostic SMS delivery for Lepton Auth hosts.

## What it provides

- Builder-first SMS construction (`SmsServiceBuilder`)
- Adapters: `noop`, `test` (in-memory for asserts), `http_capture` (lab sink on `:8099`)
- Optional live Twilio Messages REST (`TwilioSmsAdapter`, Cargo feature `twilio`)
- Optional Twilio Verify custom-code OTP delivery (`TwilioVerifySmsAdapter`)
- `TwilioSmsConfig` / `TwilioVerifyConfig` (+ builder / `from_env`) for host wiring
- `SmsEnvelope::otp_code` for Verify `CustomCode` (Valence still verifies the OTP)
- Structured `tracing` (no E.164, body, OTP, or auth tokens in fields)

## Feature flags

| Feature | Effect |
|---------|--------|
| *(default)* | Noop / Test / HTTP capture / custom adapter |
| `twilio` | Live Messages + Verify adapters + builder methods |
| `spectra` | Emit `lepton_sms_send{driver,outcome}` via [`lepton-spectra-telemetry`](../lepton-spectra-telemetry/) |

With `spectra`, boot Spectra in the host first. Counters are best-effort and never fail
the send. Labels are `driver` + `outcome` only — no E.164 or body.

## Concern → API

| Concern | API |
|---------|-----|
| Construct SMS service | [`SmsServiceBuilder`] |
| Send SMS | [`SmsDeliveryService`], [`SmsEnvelope`] |
| Noop / Test | [`NoopSmsAdapter`], [`TestSmsAdapter`] |
| HTTP capture (lab) | [`HttpCaptureSmsAdapter`], [`HttpCaptureSmsConfig`] |
| Twilio Messages config | [`TwilioSmsConfig`] |
| Twilio Verify config | [`TwilioVerifyConfig`] |
| Live Messages send | [`TwilioSmsAdapter`] (`twilio` feature) |
| Live Verify custom-code | [`TwilioVerifySmsAdapter`] (`twilio` feature) |
| Errors | [`SmsDeliveryError`] |

## Builder example (Noop)

```rust,ignore
use std::sync::Arc;
use lepton_sms::{NoopSmsAdapter, SmsServiceBuilder};

let sms = SmsServiceBuilder::new()
    .adapter(Arc::new(NoopSmsAdapter))
    .build()?;
```

## Builder example (HTTP capture sink)

```rust,ignore
use lepton_sms::{HttpCaptureSmsConfig, SmsServiceBuilder};

// cargo run -p lepton-e2e --bin lepton-sms-sink
let sms = SmsServiceBuilder::new()
    .http_capture(HttpCaptureSmsConfig::new("http://127.0.0.1:8099")?)
    .build()?;
```

## Builder example (Twilio Verify — preferred for OTP)

Enable **Custom Verification Code** on the Verify Service. Pass a 4–10 character
`otp_code` on [`SmsEnvelope`] (Lepton Auth uses a 6-digit code).

```rust,ignore
use lepton_sms::{SmsServiceBuilder, TwilioVerifyConfig};

let sms = SmsServiceBuilder::new()
    .twilio_verify(TwilioVerifyConfig::from_env()?)
    .build()?;
```

## Builder example (Twilio Messages)

Prefer an API Key SID (`SK…`) + secret. Account SID (`AC…`) is still required for the
Messages URL. Auth Token remains as a legacy fallback.

```rust,ignore
use lepton_sms::{SmsServiceBuilder, TwilioSmsConfig};

let sms = SmsServiceBuilder::new()
    .twilio(
        TwilioSmsConfig::builder()
            .account_sid(std::env::var("UF_TWILIO_ACCOUNT_SID")?)
            .api_key(std::env::var("UF_TWILIO_API_KEY")?)
            .api_secret(std::env::var("UF_TWILIO_API_SECRET")?)
            .from(std::env::var("UF_TWILIO_FROM")?)
            .build()?,
    )
    .build()?;
```

Enable with `lepton-sms = { version = "…", features = ["twilio"] }`.

Env helper (`TwilioSmsConfig::from_env`): `UF_TWILIO_ACCOUNT_SID`,
`UF_TWILIO_API_KEY`, `UF_TWILIO_API_SECRET`, `UF_TWILIO_FROM`
(or legacy `UF_TWILIO_AUTH_TOKEN` instead of the API key pair).

## Integration checklist

1. Build SMS at host boot and inject into `lepton-auth` (`phone` feature).
2. Never rebuild credentials on the send path.
3. Secrets are plain host strings; this crate does not load a secrets manager.
4. For production SMS, enable `twilio` and supply Account SID / API key / From.

## Related crates

- [`lepton-smtp`](../lepton-smtp/) — email delivery (optional Twilio SendGrid via `twilio` feature)
- [`lepton-auth`](../lepton-auth/) — verification / OTP orchestration
