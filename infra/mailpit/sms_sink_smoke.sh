#!/usr/bin/env bash
# Start lepton-sms-sink (if needed) and run the gated UF_SMS_SINK integration test.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SINK_URL="${UF_SMS_SINK_URL:-http://127.0.0.1:8099}"
STARTED=0

cleanup() {
  if [[ "$STARTED" -eq 1 ]] && [[ -n "${SINK_PID:-}" ]]; then
    kill "$SINK_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

if ! curl -sf "${SINK_URL}/v1/messages" >/dev/null 2>&1; then
  echo "[sms-sink] starting lepton-sms-sink…"
  (
    cd "$ROOT"
    CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}" cargo run -p lepton-e2e --bin lepton-sms-sink
  ) &
  SINK_PID=$!
  STARTED=1
  for _ in $(seq 1 60); do
    if curl -sf "${SINK_URL}/v1/messages" >/dev/null 2>&1; then
      break
    fi
    sleep 0.25
  done
  curl -sf "${SINK_URL}/v1/messages" >/dev/null
fi

export UF_SMS_SINK=1
export UF_SMS_SINK_URL="$SINK_URL"
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
cd "$ROOT"
echo "[sms-sink] running validating sms_http_sink tests…"
cargo test -p lepton-sms --test sms_http_sink -- --nocapture
echo "[sms-sink] OK"
