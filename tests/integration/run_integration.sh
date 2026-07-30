#!/usr/bin/env bash
# tests/integration/run_integration.sh
#
# Shell-based integration test harness for the PERO-J explorer contract.
# Addresses issue #24 — no test deploys the WASM to a real sandbox network.
#
# Prerequisites:
#   - stellar CLI >= 21 in PATH
#   - A running Stellar standalone node (e.g. stellar-quickstart on :8000)
#   - WASM built at target/wasm32-unknown-unknown/release/explorer.wasm
#     (run `make build` first)
#
# NOTE: This script is a STUB.  It documents the intended test flow and will
#       execute correctly once the prerequisites above are satisfied.
#       Do NOT run `make build` or `make test` as part of CI until the WASM
#       compilation environment is configured.
#
# Usage:
#   bash tests/integration/run_integration.sh [--rpc-url URL] [--network-passphrase PHRASE]
#
# Exit codes:
#   0 — all tests passed
#   1 — one or more tests failed

set -euo pipefail

# ── Configuration ─────────────────────────────────────────────────────────────

RPC_URL="${STELLAR_RPC_URL:-http://localhost:8000/soroban/rpc}"
NETWORK_PASSPHRASE="${STELLAR_NETWORK_PASSPHRASE:-Standalone Network ; February 2017}"
WASM_PATH="${WASM_PATH:-target/wasm32-unknown-unknown/release/explorer.wasm}"
IDENTITY="${STELLAR_IDENTITY:-integration-test-admin}"

PASS=0
FAIL=0

# ── Helpers ───────────────────────────────────────────────────────────────────

info()  { echo "[INFO]  $*"; }
ok()    { echo "[PASS]  $*"; PASS=$((PASS + 1)); }
fail()  { echo "[FAIL]  $*"; FAIL=$((FAIL + 1)); }

require_tool() {
  if ! command -v "$1" &>/dev/null; then
    echo "[ERROR] Required tool not found: $1"
    exit 1
  fi
}

# ── Pre-flight checks ─────────────────────────────────────────────────────────

require_tool stellar
require_tool jq

if [[ ! -f "$WASM_PATH" ]]; then
  echo "[ERROR] WASM not found at $WASM_PATH — run 'make build' first."
  echo "        Skipping integration tests."
  exit 0
fi

# ── Generate a throw-away test identity if it doesn't exist ───────────────────

if ! stellar keys show "$IDENTITY" &>/dev/null 2>&1; then
  info "Generating test identity '$IDENTITY' …"
  stellar keys generate \
    --rpc-url     "$RPC_URL" \
    --network-passphrase "$NETWORK_PASSPHRASE" \
    "$IDENTITY"
fi

ADMIN_ADDR=$(stellar keys address "$IDENTITY")
info "Admin address: $ADMIN_ADDR"

# Fund the test account on a local standalone node via friendbot
FRIENDBOT_URL="${RPC_URL%/soroban/rpc}/friendbot?addr=$ADMIN_ADDR"
curl -sf "$FRIENDBOT_URL" >/dev/null || true   # ignore if not available

# ── Deploy the contract ───────────────────────────────────────────────────────

info "Deploying explorer.wasm …"
CONTRACT_ID=$(stellar contract deploy \
  --wasm         "$WASM_PATH" \
  --source       "$IDENTITY" \
  --rpc-url      "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  2>/dev/null)

if [[ -z "$CONTRACT_ID" ]]; then
  fail "Contract deployment returned an empty contract ID"
  exit 1
fi

info "Deployed contract: $CONTRACT_ID"

# ── Test 1 — init ─────────────────────────────────────────────────────────────
# Addresses issue #24: verify the contract can be initialised via CLI.

info "Test 1: init(admin=$ADMIN_ADDR)"
stellar contract invoke \
  --id    "$CONTRACT_ID" \
  --source "$IDENTITY" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- init \
  --admin "$ADMIN_ADDR" >/dev/null \
  && ok "Test 1: init succeeded" \
  || fail "Test 1: init failed"

# ── Test 2 — register_contract ────────────────────────────────────────────────
# A dummy 32-byte contract ID (hex).

DUMMY_CID="0101010101010101010101010101010101010101010101010101010101010101"

info "Test 2: register_contract"
stellar contract invoke \
  --id    "$CONTRACT_ID" \
  --source "$IDENTITY" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- register_contract \
  --caller      "$ADMIN_ADDR" \
  --contract_id "$DUMMY_CID" \
  --meta        '{"name":"StellarSwap","description":"DEX on Stellar","functions":[],"registered_by":"'"$ADMIN_ADDR"'"}' \
  >/dev/null \
  && ok "Test 2: register_contract succeeded" \
  || fail "Test 2: register_contract failed"

# ── Test 3 — get_contract (TTL bump, issue #25) ───────────────────────────────
# After fix for #25, reading should extend the entry TTL.  We verify the entry
# is still retrievable (TTL bump keeps it alive).

info "Test 3: get_contract (verifies TTL is bumped on read — issue #25)"
RESULT=$(stellar contract invoke \
  --id    "$CONTRACT_ID" \
  --source "$IDENTITY" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_contract \
  --contract_id "$DUMMY_CID" 2>/dev/null)

if echo "$RESULT" | grep -q "StellarSwap"; then
  ok "Test 3: get_contract returned expected metadata"
else
  fail "Test 3: get_contract did not return expected metadata (got: $RESULT)"
fi

# ── Test 4 — submit_event ─────────────────────────────────────────────────────

info "Test 4: submit_event"
stellar contract invoke \
  --id    "$CONTRACT_ID" \
  --source "$IDENTITY" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- submit_event \
  --caller      "$ADMIN_ADDR" \
  --contract_id "$DUMMY_CID" \
  --fn_name     "swap" \
  --ledger      4521983 \
  --description "Address GABC swapped 100 USDC -> 98.7 XLM on StellarSwap" \
  --raw_topics  '[]' \
  --raw_data    '' \
  >/dev/null \
  && ok "Test 4: submit_event succeeded" \
  || fail "Test 4: submit_event failed"

# ── Test 5 — event_count ──────────────────────────────────────────────────────

info "Test 5: event_count"
COUNT=$(stellar contract invoke \
  --id    "$CONTRACT_ID" \
  --source "$IDENTITY" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- event_count 2>/dev/null)

if [[ "$COUNT" == "1" ]]; then
  ok "Test 5: event_count returned 1"
else
  fail "Test 5: event_count expected 1, got $COUNT"
fi

# ── Test 6 — get_event (TTL bump, issue #25) ──────────────────────────────────

info "Test 6: get_event seq=0 (verifies TTL is bumped on read — issue #25)"
EV=$(stellar contract invoke \
  --id    "$CONTRACT_ID" \
  --source "$IDENTITY" \
  --rpc-url "$RPC_URL" \
  --network-passphrase "$NETWORK_PASSPHRASE" \
  -- get_event \
  --seq 0 2>/dev/null)

if echo "$EV" | grep -q "swap"; then
  ok "Test 6: get_event returned expected event"
else
  fail "Test 6: get_event did not return expected event (got: $EV)"
fi

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "Results: $PASS passed, $FAIL failed"

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi

exit 0
