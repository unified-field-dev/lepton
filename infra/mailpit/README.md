# Mailpit harness

Local sidecars for exercising Lepton delivery with **real SMTP**, not noop.

Library code takes plain builder config. This directory is **host wiring** only
(env files for scripts). Secrets tools are out of scope for lepton crates.

## Services

| Service | Ports | Role |
|---------|-------|------|
| Mailpit | SMTP `1025`, UI/API `8025` | Capture outbound mail |
| Mock OIDC | HTTP `5556` | Lab IdP (`lepton-mock-oidc`) |
| SMS HTTP sink | HTTP `8099` | Capture outbound SMS (`lepton-sms-sink`) |

All ports bind to `127.0.0.1` only. This is a local development harness — do not
expose Mailpit on a shared or public interface, and never point it at real inboxes.

Primary coverage for the OIDC and SMS sidecars is **default CI**
(`cargo test -p lepton-e2e --test mock_oidc_http --test sms_sink_http`).
Operator smoke scripts below are liveness only.

```bash
cargo run -p lepton-e2e --bin lepton-mock-oidc
./infra/mailpit/mock_oidc_smoke.sh

cargo run -p lepton-e2e --bin lepton-sms-sink
./infra/mailpit/sms_sink_smoke.sh   # sets UF_SMS_SINK=1
```

Optional live Twilio / SendGrid: see commented vars in [`mailpit.env.example`](mailpit.env.example).
Enable crate features `lepton-smtp/twilio` and `lepton-sms/twilio` on the host; default CI stays offline.

Interactive end-to-end (stdin for email token + SMS OTP):
[`lepton-e2e/README.md`](../../lepton-e2e/README.md) — `lepton-live-verify` behind `UF_LEPTON_LIVE_TWILIO=1`.

## Quick start

```bash
docker compose -f infra/mailpit/docker-compose.yml up -d
./infra/mailpit/smtp_smoke.sh
```

Or manually:

```bash
set -a; source infra/mailpit/mailpit.env.example; set +a
CARGO_BUILD_JOBS=1 cargo test -p lepton-smtp --test smtp_mailpit -- --nocapture
```

Success: tests assert Mailpit inbox messages (subject/recipient/token fragment).
UI: [http://127.0.0.1:8025](http://127.0.0.1:8025).

Teaching example (builder → SMTP receipt):

```bash
CARGO_BUILD_JOBS=1 cargo run -p lepton-auth --example auth_flows_smtp_mailpit --features ssr
```

## Pitfalls

- Without `UF_SMTP_HOST`, `EmailDriver` defaults to **noop** — delivery “succeeds” with no mail.
- Mailpit needs plain SMTP: `UF_SMTP_USE_TLS=false` and no username/password.
- Auth UI may hide delivery failures (anti-enumeration). Trust the Mailpit inbox + `[lepton-smtp]` / tracing, not the browser alone.
- Default `cargo test` skips Mailpit unless `UF_MAILPIT=1`.
- SMS sink gated integ skips unless `UF_SMS_SINK=1` (CI-always sink tests use an ephemeral port).
- Mock OIDC binds `127.0.0.1` only; Playwright host auto-starts it on `:5556`.

## See also

- [`lepton-smtp` README](../../lepton-smtp/README.md) — builder-first API
- [`lepton-sms` README](../../lepton-sms/README.md) — HTTP capture + Twilio adapters
- [`lepton-e2e` README](../../lepton-e2e/README.md) — mock OIDC / SMS sink bins
