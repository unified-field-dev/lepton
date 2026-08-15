#!/usr/bin/env bash
# Start lepton-mock-oidc (if needed) and probe discovery + authorize.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
ISSUER="${UF_MOCK_OIDC_URL:-http://127.0.0.1:5556}"
STARTED=0

cleanup() {
  if [[ "$STARTED" -eq 1 ]] && [[ -n "${OIDC_PID:-}" ]]; then
    kill "$OIDC_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

if ! curl -sf "${ISSUER}/.well-known/openid-configuration" >/dev/null 2>&1; then
  echo "[mock-oidc] starting lepton-mock-oidc…"
  (
    cd "$ROOT"
    CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}" cargo run -p lepton-e2e --bin lepton-mock-oidc
  ) &
  OIDC_PID=$!
  STARTED=1
  for _ in $(seq 1 60); do
    if curl -sf "${ISSUER}/.well-known/openid-configuration" >/dev/null 2>&1; then
      break
    fi
    sleep 0.25
  done
fi

curl -sf "${ISSUER}/.well-known/openid-configuration" | grep -q authorization_endpoint
# Expect redirect (3xx) with code=
code=$(curl -sI "${ISSUER}/authorize?state=smoke&redirect_uri=http://127.0.0.1:3000/cb&provider=google" \
  | tr -d '\r' | awk -F': ' 'tolower($1)=="location"{print $2}')
echo "$code" | grep -q 'code='
echo "[mock-oidc] OK — discovery + authorize redirect"
