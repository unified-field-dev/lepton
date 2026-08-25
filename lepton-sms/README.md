# lepton-sms

Provider-agnostic SMS delivery: build a service once, send an envelope, inspect a receipt.

Rustdoc is the teaching source of truth (`cargo doc -p lepton-sms --open`). This README mirrors the guide vs API-reference split.

## What it provides

- Builder-first SMS construction (`SmsServiceBuilder`)
- Adapters: `noop`, `test` (in-memory for asserts), `http_capture` (lab sink on `:8099`)
- Optional live Twilio Messages REST (`TwilioSmsAdapter`, Cargo feature `twilio`)
- Optional Twilio Verify custom-code OTP delivery (`TwilioVerifySmsAdapter`)
- `TwilioSmsConfig` / `TwilioVerifyConfig` (+ builder / `from_env`) for host wiring
- `SmsEnvelope::otp_code` for Verify `CustomCode`
- Structured `tracing` (no E.164, body, OTP, or auth tokens in fields)

## Getting started

### First success (Noop)

```bash
cargo run -p lepton-sms --example noop_send
```

```rust,ignore
use lepton_sms::{SmsDeliveryService, SmsEnvelope, SmsServiceBuilder};

# async fn run() -> Result<(), lepton_sms::SmsDeliveryError> {
let sms = SmsServiceBuilder::new().noop().build()?;
let receipt = sms
    .send(&SmsEnvelope {
        to_e164: "+15551234567".into(),
        body: "Your code is 123456".into(),
        otp_code: Some("123456".into()),
    })
    .await?;
assert_eq!(receipt.provider, "noop");
# Ok(())
# }
```

### Choose a delivery backend

| Backend | When to use | Guide (rustdoc) | API reference |
|---------|-------------|-----------------|---------------|
| **Noop** | Local / CI | Crate root “Noop” | `SmsServiceBuilder::noop`, `NoopSmsAdapter` |
| **Test** | Unit tests; recorded envelopes | Crate root “Test” | `SmsServiceBuilder::test`, `TestSmsAdapter` |
| **HTTP capture** | Lab sink (`:8099`) | Crate root “HTTP capture” | `SmsServiceBuilder::http_capture`, `HttpCaptureSmsConfig`, `HttpCaptureSmsAdapter` |
| **Twilio Messages** | Live Messages REST (`twilio`) | Crate root “Twilio Messages” | `SmsServiceBuilder::twilio`, `TwilioSmsConfig`, `TwilioSmsAdapter` |
| **Twilio Verify** | Live OTP CustomCode (`twilio`) | Crate root “Twilio Verify” | `SmsServiceBuilder::twilio_verify`, `TwilioVerifyConfig`, `TwilioVerifySmsAdapter` |

### HTTP capture (lab)

```bash
# cargo run -p lepton-e2e --bin lepton-sms-sink
```

```rust,ignore
use lepton_sms::{HttpCaptureSmsConfig, SmsDeliveryService, SmsEnvelope, SmsServiceBuilder};

# async fn run() -> Result<(), lepton_sms::SmsDeliveryError> {
let sms = SmsServiceBuilder::new()
    .http_capture(HttpCaptureSmsConfig::new("http://127.0.0.1:8099")?)
    .build()?;
let receipt = sms
    .send(&SmsEnvelope {
        to_e164: "+15551234567".into(),
        body: "lab capture".into(),
        otp_code: Some("123456".into()),
    })
    .await?;
assert_eq!(receipt.provider, "http_capture");
# Ok(())
# }
```

### Twilio Verify (preferred for OTP)

Enable **Custom Verification Code** on the Verify Service. Pass a 4–10 character
`otp_code` on `SmsEnvelope`. See rustdoc “Twilio Verify”.

Enable with `lepton-sms = { version = "…", features = ["twilio"] }`.

## Feature flags

| Feature | Effect |
|---------|--------|
| *(default)* | Noop / Test / HTTP capture / custom adapter |
| `twilio` | Live Messages + Verify adapters + builder methods |
| `spectra` | Emit `lepton_sms_send{driver,outcome}` via [`lepton-spectra-telemetry`](../lepton-spectra-telemetry/) |

With `spectra`, boot Spectra in the host first. Counters are best-effort and never fail
the send. Labels are `driver` + `outcome` only — no E.164 or body.

## Integration checklist

1. Build SMS at host boot (`SmsServiceBuilder::build`).
2. Inject `Arc<dyn SmsDeliveryService>` — never rebuild credentials on the send path.
3. Secrets are plain host strings; this crate does not load a secrets manager.
4. For production SMS, enable `twilio` and supply Account SID / API key / From (or Verify config).

## Related crates

- [`lepton-smtp`](../lepton-smtp/) — email delivery (optional Twilio SendGrid via `twilio` feature)
- [`lepton-auth`](../lepton-auth/) — verification / OTP orchestration
