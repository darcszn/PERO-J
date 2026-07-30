# Integration Tests

This directory contains integration tests that validate the compiled WASM contract
against a real Stellar CLI sandbox environment.  Unlike the in-process
`soroban_sdk::Env` mock used by the unit tests in `contracts/explorer/src/lib.rs`,
these tests exercise the full WASM execution path and can catch:

- Host-function mismatches between the mock and the real WASM host
- TTL / ledger-entry expiry behaviour (issue #25)
- CLI round-trip correctness for `register_contract`, `get_contract`,
  `submit_event`, `get_event`, and `event_count`

## Prerequisites

```bash
# Stellar CLI >= 21
stellar --version

# Docker (for stellar-quickstart sandbox)
docker --version
```

## Layout

```
tests/integration/
├── README.md            — this file
├── run_integration.sh   — shell-based CLI test harness (issue #24)
└── rust/
    ├── Cargo.toml       — standalone integration-test crate
    └── src/
        └── lib.rs       — Rust-based integration test stubs (issue #24)
```

## Running the shell tests

```bash
# Start a local sandbox (quickstart) first:
docker run --rm -d \
  -p 8000:8000 \
  --name stellar-quickstart \
  stellar/quickstart:testing \
  --standalone --enable-soroban-rpc

# Then run:
bash tests/integration/run_integration.sh
```

## Running the Rust tests

> **Note:** These are stubs only.  Full execution requires a running sandbox and
> a deployed contract address.

```bash
cd tests/integration/rust
cargo test -- --nocapture
```
