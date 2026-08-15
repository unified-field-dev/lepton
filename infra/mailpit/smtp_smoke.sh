#!/usr/bin/env bash
# Orchestrate Mailpit + validating lepton-smtp Mailpit integration test.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
COMPOSE_FILE="${ROOT}/infra/mailpit/docker-compose.yml"
ENV_FILE="${ROOT}/infra/mailpit/mailpit.env.example"
MAILPIT_URL="${UF_MAILPIT_URL:-http://127.0.0.1:8025}"
DOWN=0

usage() {
  echo "Usage: $0 [--down]" >&2
  exit 2
}

for arg in "$@"; do
  case "$arg" in
    --down) DOWN=1 ;;
    -h|--help) usage ;;
    *) usage ;;
  esac
done

cleanup() {
  if [[ "$DOWN" -eq 1 ]]; then
    docker compose -f "$COMPOSE_FILE" down >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

echo "[mailpit] starting Mailpit…"
docker compose -f "$COMPOSE_FILE" up -d

echo "[mailpit] waiting for ${MAILPIT_URL}…"
for _ in $(seq 1 40); do
  if curl -sf "${MAILPIT_URL}/api/v1/info" >/dev/null; then
    break
  fi
  sleep 0.25
done
curl -sf "${MAILPIT_URL}/api/v1/info" >/dev/null

set -a
# shellcheck disable=SC1090
source "$ENV_FILE"
set +a

export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
cd "$ROOT"
echo "[mailpit] running validating smtp_mailpit tests…"
cargo test -p lepton-smtp --test smtp_mailpit -- --nocapture

echo "[mailpit] OK — Mailpit accepted SMTP delivery"
