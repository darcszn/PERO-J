# Changelog

All notable changes to PERO-J are documented here.
Format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Bug Fixes

- Harden init, add indexer allowlist, cap paging, emit update event ([`8bbe4cc`](../../commit/8bbe4cc83c34cbd85d4b04bb86e8547fcf38b3e5))

Closes [#1](../../issues/1), [#2](../../issues/2), [#3](../../issues/3), [#4](../../issues/4).

  [#1](../../issues/1) init() could be replayed if the instance entry expired. The Admin guard
  now lives in persistent storage, and every mutating entry point calls
  bump_ttl() so an active contract never lets its state lapse.

  [#2](../../issues/2) submit_event() only accepted the single admin address, forcing the
  indexer hot wallet to hold the cold admin key. Adds a persistent
  IndexerAllowlist (Vec<Address>, capped at MAX_INDEXERS = 20) with
  add_indexer/remove_indexer admin functions plus get_indexers/is_indexer
  readers; submit_event now accepts the admin or any allowlisted address.

  [#3](../../issues/3) get_events() accepted a raw u32 limit and would iterate until the host
  ran out of CPU instructions. Rejects limit > MAX_PAGE (200) with
  Error::LimitExceeded, and the end-offset computation is now saturating.

  [#4](../../issues/4) update_contract() was silent, so the indexer could not invalidate its
  ABI cache without polling. It now publishes ("update", contract_id) ->
  meta.name, mirroring register_contract.

  Two pre-existing build breakages had to be fixed for any of this to
  compile or run in CI:
  - Error was declared #[contracttype] rather than #[contracterror], so
    every panic_with_error! failed to typecheck and the crate did not build.
  - Cargo.lock resolved soroban-env-host 21.2.1 against ed25519-dalek 3.0.0,
    which it does not support; pinned back to 2.2.0.

  13 unit tests pass, including a regression test for [#1](../../issues/1) that fails against
  the old instance-storage implementation


- Add database backup strategy ([#106](../../issues/106)) ([`ecd0c8f`](../../commit/ecd0c8f9b30750147e55230e550aca6dd412daf0))

- Add scripts/backup.sh using pg_dump with configurable env vars
  - Document cron job: 0 2 * * * backup.sh >> /var/log/backup.log 2>&1
  - Document restore procedure in README.md
  - Document cloud deployment backups (RDS, Cloud SQL, Supabase, Neon)
  - Update ROADMAP Tranche 3 deliverable 3.2 to reflect implementation
  - Add PR_DESCRIPTION.md with detailed description closing [#106](../../issues/106)

  closes [#106](../../issues/106)


- Log malformed SAC_ASSETS entries and startup asset count ([`c88a102`](../../commit/c88a10289345f382fee9a7c81a5438698fcbffb5))

- Resolve frontend CI failures ([`3f89ce9`](../../commit/3f89ce9d7c66e2ab70f5ffcf8c3e8ec63053c05e))

- Add skipLibCheck and vite/client types to tsconfig.json
  - Replace process.env with import.meta.env.DEV in ErrorBoundary
  - Add distinctFunctions endpoint to API client and server
  - Add getDistinctFunctions query to database module


- Remove unused xdr and StrKey imports ([#30](../../issues/30)) ([`562ca48`](../../commit/562ca484aa0af97ece6b1570db9a0821a9b08808))

Only scValToNative is used in decoder.js. The xdr and StrKey named
  imports were present in an earlier version but are no longer referenced.
  Removing dead imports reduces the module's dependency surface and makes
  it clear what the module actually relies on.

  Closes [#30](../../issues/30)


- Handle BigInt in JSON.stringify for raw_data ([`10708cd`](../../commit/10708cd79455d3b74f75b7f9daef07840821e69a))

scValToNative returns BigInt for i64/u64/i128/u128 values. Passing the
  decoded data directly to JSON.stringify() threw:
    TypeError: Do not know how to serialize a BigInt


- Redact sensitive args and truncate to 64 chars in genericDescription ([#29](../../issues/29)) ([`852cb12`](../../commit/852cb12ef7522bfc20e79275e5ef304014717d86))

- Add isSensitive() helper that flags:
    * 56-char G-prefixed strings that are not valid strkeys
    * Raw hex blobs of 64+ chars (32+ byte nonces/keys)
    * Base64 blobs of 44+ chars (32-byte secrets)
  - Add sanitiseArg() that redacts sensitive values with [REDACTED]
    and truncates any value > 64 chars to 'first…last' form
  - genericDescription now maps args through sanitiseArg instead of
    String(), preventing private data from leaking into PostgreSQL
    and the public API

  Closes [#29](../../issues/29)


- Reload SAC map on SIGHUP signal ([#34](../../issues/34)) ([`2640006`](../../commit/264000636ba52fdffae2ebffd1b344a2169bf229))

- Clamp scvI128/scvI256 bit-shifting to signed range ([#33](../../issues/33)) ([`8fe5111`](../../commit/8fe51110357b58c743084a5f97f78550e7651cbf))

- Resolve assigned issue fixes ([`34a1377`](../../commit/34a1377ba354aa5d72de57a3f095f9822e8462e1))

- Return readable strings for opaque ScVal variants ([`2b2dd69`](../../commit/2b2dd692d4b1d37f64ba6f4c5a18a139bf8512c1))

scvLedgerKeyContractInstance, scvLedgerKeyNonce, and scvContractInstance
  previously returned plain objects that serialised to [object Object] when
  coerced to string (e.g. in genericDescription).

  - scvLedgerKeyContractInstance → "<contract-instance>"
  - scvContractInstance          → "<contract-instance>"
  - scvLedgerKeyNonce            → "<nonce:{n}>" (preserves nonce value)

  Adds indexer/test/scval.test.js with 4 test cases covering all three
  variants and the join-into-description scenario


- Bound caller-supplied payloads, add get_admin, harden event counter ([`280bd83`](../../commit/280bd83d5e678d50c71cf91e4dbd15d51d4c5ffc))

- submit_event rejects raw_data larger than MAX_RAW_DATA_BYTES (4096) before
    touching storage, so a single call cannot bloat on-chain state ([#7](../../issues/7))
  - add get_admin() view so off-chain tooling can read the admin without
    guessing the DataKey encoding ([#10](../../issues/10))
  - event_count()/get_events() now read the sequence counter through a helper
    that panics with NotInitialized instead of collapsing a missing counter
    into a misleading 0 ([#11](../../issues/11))
  - register_contract()/update_contract() reject ABI metadata above
    MAX_FUNCTIONS (64) or MAX_PARAMS (32) per function ([#12](../../issues/12))

  Also fixes two pre-existing build blockers that made the crate impossible to
  compile or test: Error carried #[contracttype] instead of #[contracterror]
  (so panic_with_error! never type-checked), and the lockfile resolved
  ed25519-dalek to 3.0.0, which soroban-env-host (">=2.0.0") cannot build against


- Improve rpc and database resilience ([`76f1d37`](../../commit/76f1d37ba0c5947589f722cc5eabcfee315b66bc))

- Resolve assigned event API and DB issues ([`5d6f68d`](../../commit/5d6f68d79689f2efd244dc8e76531ff9c7cb9bf3))

- Resolve issues [#70](../../issues/70), [#71](../../issues/71), [#72](../../issues/72), and [#73](../../issues/73) in indexer service ([`0a97e3f`](../../commit/0a97e3f9c87f346c2f7087c1caf641a39a47245a))

Detailed Summary of Changes:

  1. Fix Express Async Route Error Handling ([#73](../../issues/73)):
  - Introduced an asyncHandler wrapper function in indexer/src/api.js to catch promise rejections in async route handlers and forward them to Express's next(err).
  - Added a centralized Express error-handling middleware to safely return 500 error responses and prevent process crashes.
  - Refactored all API route callbacks to use asyncHandler.

  2. Graceful Database Connection Pool Termination on Unhandled Rejections ([#72](../../issues/72)):
  - Registered process.on('unhandledRejection') in indexer/src/db.js to log errors and await pool.end() before process exit, preventing PostgreSQL connection leaks.
  - Added db.close() method for graceful shutdown.

  3. Persist and Query SAC Asset Codes in Events Table ([#71](../../issues/71)):
  - Added sac_asset TEXT column to events table in db.init() with migration support.
  - Updated db.upsertEvent() to persist sac_asset when Stellar Asset Contract (SAC) events are processed.

  4. Add Container Health and Readiness Probes ([#70](../../issues/70)):
  - Implemented db.ping() in indexer/src/db.js to verify database connectivity.
  - Enhanced GET /health endpoint to check database status, include latestLedger, and return HTTP 503 on database disconnection or indexer lag.
  - Added GET /ready endpoint for Kubernetes and Docker Compose readiness probes.

  Closes [#70](../../issues/70)
  Closes [#71](../../issues/71)
  Closes [#72](../../issues/72)
  Closes [#73](../../issues/73)


- Resolve issues [#69](../../issues/69), [#68](../../issues/68), [#67](../../issues/67), and [#66](../../issues/66) simultaneously ([`fbff2bc`](../../commit/fbff2bcd27f3fbe11b8d379348684b4691835407))

1. Issue [#69](../../issues/69) - Add pagination to getWalletEvents & WalletPage:
  - Updated db.getWalletEvents to accept page and limit parameters, query total event count, and fetch paginated results using LIMIT and OFFSET in SQL.
  - Updated GET /api/wallet/:address endpoint to pass query parameters (page and limit) to db.getWalletEvents and return a wrapper object containing { events, total, page, limit }.
  - Updated frontend api.ts and WalletPage.tsx to pass page parameter to api.wallet and render Prev/Next pagination UI controls.

  2. Issue [#68](../../issues/68) - Handle sourceAccountNotFound in sep41Metadata simulation:
  - Updated simulateCall in sep41Metadata.js to catch sourceAccountNotFound simulation errors when sequence is "0" and automatically retry simulation with sequence "1".
  - Added JSDoc documentation and environment variable fallback (process.env.OPERATIONAL_ACCOUNT) for the simulation dummy source account.

  3. Issue [#67](../../issues/67) - Implement express-rate-limit middleware on Express API:
  - Added express-rate-limit package dependency to indexer/package.json.
  - Configured and registered rateLimit middleware in indexer/src/api.js (windowMs: 60,000 ms, max: 100 requests) to protect all API endpoints against DoS attacks.

  4. Issue [#66](../../issues/66) - Validate seq parameter in GET /api/events/:seq:
  - Added validation for req.params.seq in GET /api/events/:seq to ensure it is a non-negative integer using parseInt and regex pattern matching.
  - Returned HTTP 400 with { error: "seq must be a non-negative integer" } when given invalid inputs like non-numeric strings or negative numbers.

  Closes [#69](../../issues/69)
  Closes [#68](../../issues/68)
  Closes [#67](../../issues/67)
  Closes [#66](../../issues/66)


- Resolve frontend issues [#90](../../issues/90) [#91](../../issues/91) [#92](../../issues/92) [#93](../../issues/93) ([`5da7959`](../../commit/5da79592839c0627076ad979a3c751add9bbf36c))

[#90](../../issues/90) - Add Skeleton component with shimmer animation; replace all plain
       'Loading…' text in Home, ContractPage, WalletPage, and EventPage
       with shaped placeholder rows that match the EventTable layout,
       eliminating layout shift on data load.

  [#91](../../issues/91) - Read API base URL from VITE_API_URL env variable with '/api'
       fallback so the frontend works on separate-origin deployments
       (e.g. CDN frontend + api.pero-j.io API) without CORS errors.
       Document VITE_API_URL in .env.example.

  [#92](../../issues/92) - Configure QueryClient with staleTime: 30_000 (30 s) to prevent
       stale event-feed data. Add registerContract to api.ts and wire
       useMutation + queryClient.invalidateQueries({ queryKey: ['contract', id] })
       in ContractPage so metadata updates are reflected immediately
       after a successful POST /api/contracts.

  [#93](../../issues/93) - Add created_at?: string to DecodedEvent interface. Display it
       in EventPage as a human-readable UTC timestamp using
       new Date(ev.created_at).toUTCString(), giving users a readable
       time alongside the raw ledger number


- Resolve issues [#118](../../issues/118)-[#121](../../issues/121) — health endpoint, transfer_admin, ABI fixtures, load test ([`2a1664e`](../../commit/2a1664e8dcbfdd90dfd52bebdb20dce3ccd8802b))

Issue [#118](../../issues/118) — Contract admin key management
  - Add transfer_admin(current_admin, new_admin) to ExplorerContract; both
    parties must authorize to prevent accidental lock-out
  - Add three unit tests: happy path, unauthorized caller
  - Add SECURITY.md documenting key-management best practices and
    emergency recovery procedure

  Issue [#119](../../issues/119) — Indexer lag monitoring (GET /health)
  - Export shared health state (lastIndexedAt, lastLedger, startedAt)
    from index.js and update it after each ledger batch
  - Add GET /health endpoint to api.js: returns lag_seconds, uptime_seconds,
    last_ledger, last_indexed_at; HTTP 200 healthy / 503 degraded
  - Alert threshold configurable via LAG_ALERT_THRESHOLD_S env var (default 30)
  - Document uptime monitor setup and example responses in README

  Issue [#120](../../issues/120) — ABI fixtures for Tranche 2 deliverable 2.5
  - Add indexer/fixtures/stellarswap-abi.json (swap, add/remove liquidity, get_price)
  - Add indexer/fixtures/blend-abi.json (supply, withdraw, borrow, repay, liquidate)
  - Add make seed-testnet target to register ABIs via POST /api/contracts

  Issue [#121](../../issues/121) — Load testing
  - Add tests/load/api_load_test.js (k6): 100 VUs × 60 s on GET /api/events
  - Separate health probe scenario samples lag_seconds every 10 s
  - Thresholds: p95 < 500 ms, p99 < 1 s, error rate < 1 %, lag < 30 s
  - Add make load-test target



### Documentation

- Auto-update CHANGELOG.md [skip ci] ([`f30e7e1`](../../commit/f30e7e125cfcd9a1fcd32a1ff7a30ff28327e27a))

- Auto-update CHANGELOG.md [skip ci] ([`75074c9`](../../commit/75074c9a7160fcddaa5f8adc1d25ae11946c5ef1))

- Auto-update CHANGELOG.md [skip ci] ([`ada6b96`](../../commit/ada6b962732d5808ce0350f76055574f64a4d03e))

- Auto-update CHANGELOG.md [skip ci] ([`2b5bd65`](../../commit/2b5bd65f45eb55ef5534d7bf951e345dc508960e))

- Auto-update CHANGELOG.md [skip ci] ([`ce0cc39`](../../commit/ce0cc3991e752b191d69dc4091f8319076736068))

- Auto-update CHANGELOG.md [skip ci] ([`aeb2d39`](../../commit/aeb2d3966824bee99844d95bc2502022cb190f4e))

- Document GET /api/tokens/:id/volume and add decimals param ([`b512953`](../../commit/b5129536d0e0108978c33c1d93f53dae4ecbce8c))

- Auto-update CHANGELOG.md [skip ci] ([`0ff82b0`](../../commit/0ff82b0afa73997fface376a6441959c44ac7560))

- Auto-update CHANGELOG.md [skip ci] ([`8a5ef83`](../../commit/8a5ef8389fca5e8ec39a134e68bc9f3077fd2ddc))

- Auto-update CHANGELOG.md [skip ci] ([`3846d59`](../../commit/3846d5983235a147141d0b7341965b12303bb9ff))

- Auto-update CHANGELOG.md [skip ci] ([`e65789e`](../../commit/e65789ed540c0e4e8a424058d88eef23d566590f))

- Auto-update CHANGELOG.md [skip ci] ([`f7201b3`](../../commit/f7201b383ce7af4f9288e5087d3bd72113a40868))

- Auto-update CHANGELOG.md [skip ci] ([`2dc0400`](../../commit/2dc0400a37bd7dd94418273a2ae1abd86dd27cf5))

- Auto-update CHANGELOG.md [skip ci] ([`e3eabd2`](../../commit/e3eabd2e1d7b827fce42b24299ccbf3c47f325a7))

- Auto-update CHANGELOG.md [skip ci] ([`5d273dd`](../../commit/5d273dd1fe7383d2cfedccf43cccf53864bfddd1))

- Auto-update CHANGELOG.md [skip ci] ([`0c9ac09`](../../commit/0c9ac099ee45bfbd619c0455d6b292da263fcf4b))

- Auto-update CHANGELOG.md [skip ci] ([`4c6fcf4`](../../commit/4c6fcf4402649059fa993750584eac43dbcd7187))

- Auto-update CHANGELOG.md [skip ci] ([`b031ae7`](../../commit/b031ae7d10100b2a0e2bb5d4ae558d28d2965481))

- Auto-update CHANGELOG.md [skip ci] ([`3d538ec`](../../commit/3d538ec8e415785667740a702d09681f3b5aa3f3))

- Auto-update CHANGELOG.md [skip ci] ([`a951154`](../../commit/a95115440341111dd1c03dec58e94b8a7f810566))

- Auto-update CHANGELOG.md [skip ci] ([`fe6b590`](../../commit/fe6b590e8af9352bdec58a86ed65e447b403abbb))

- Auto-update CHANGELOG.md [skip ci] ([`cbaa3b9`](../../commit/cbaa3b99d8819537d2b1eec0794c4853e1726690))

- Auto-update CHANGELOG.md [skip ci] ([`07fedca`](../../commit/07fedca8254be28b38f5dd745581b05a02e4a2e4))

- Auto-update CHANGELOG.md [skip ci] ([`174d064`](../../commit/174d0645d2f1d14d56d3bf38fe6bde255e41520c))

- Auto-update CHANGELOG.md [skip ci] ([`c76b2f6`](../../commit/c76b2f6b16e1e36518bc31bc7b882e579afd63e7))

- Auto-update CHANGELOG.md [skip ci] ([`c72538b`](../../commit/c72538b09a9fcef2c86c50026af99e3d6705245d))

- Auto-update CHANGELOG.md [skip ci] ([`6cd5c41`](../../commit/6cd5c41f21f1399094420f1a03251b7924386e62))

- Auto-update CHANGELOG.md [skip ci] ([`90304fb`](../../commit/90304fb2a734e9e4f64660d636d42f6a79576cdb))

- Auto-update CHANGELOG.md [skip ci] ([`bcd14c7`](../../commit/bcd14c7dcd79720c93033f0414469926f468a40f))

- Auto-update CHANGELOG.md [skip ci] ([`b99cbce`](../../commit/b99cbce6f39353cbb35e53db58021846fb2f3ad9))

- Auto-update CHANGELOG.md [skip ci] ([`b0178f9`](../../commit/b0178f92e59be41908343c246fa84d333f4f7987))

- Auto-update CHANGELOG.md [skip ci] ([`2f01e74`](../../commit/2f01e7440fb80286f823cee453cfe22aeb4e6553))

- Auto-update CHANGELOG.md [skip ci] ([`64c2488`](../../commit/64c2488417fa0d6f93df6d6dc8b500cab5b767db))

- Auto-update CHANGELOG.md [skip ci] ([`193eeee`](../../commit/193eeee984e6e2cf5193124884bae892d6d428d9))

- Add issue and PR templates ([`a63913f`](../../commit/a63913ff65e8116ff4fdde38bbed0774001f6d9e))

- Fill TEAM.md with Sunday Abel's real information ([`93c8347`](../../commit/93c8347905599d235cf94ec8b8d7ca488ab370e4))

- Fill TEAM.md with real team information ([`d45fdb4`](../../commit/d45fdb41136db37d887b1fdf8159721ecb01b6b4))


### Features

- Add end-to-end tests verifying full pipeline (contract → indexer → API → frontend) ([`b774907`](../../commit/b774907e44bc12dfc98166ccb479dac83c91db7a))

Adds a tests/e2e/ directory with Playwright-based E2E tests that:

  - Start a full local stack via Docker Compose (postgres + indexer + frontend)
    with a Stellar Soroban sandbox (stellar/quickstart)
  - Build and deploy the ExplorerContract WASM to the sandbox
  - Initialize the contract and register ABI metadata
  - Submit a test event via the contract's submit_event function
  - Wait for the indexer to poll and decode the event
  - Verify the event appears in the REST API
  - Assert the frontend renders the decoded description
  - Test navigation to event detail and contract pages
  - Verify pagination controls are functional

  The test infrastructure includes:
  - docker-compose.e2e.yml       — Extended stack with Soroban sandbox
  - helpers/deploy.js             — Automated contract deployment + seeding
  - fixtures/explorer-abi.json    — ABI fixture for the ExplorerContract
  - e2e.test.js                  — 7 Playwright test cases
  - playwright.config.ts          — Playwright configuration
  - package.json                 — Dependencies (@playwright/test, @stellar/stellar-sdk)
  - Makefile targets             — e2e-setup, e2e-build, e2e-up, e2e-down,
                                    e2e-deploy, e2e-test, e2e, e2e-ci
  - CI job in ci.yml             — Full E2E on PR/push to main

  closes [#109](../../issues/109)


- Add database backup script and documentation ([`aeb57d5`](../../commit/aeb57d5ed4f50e45ad3fb210f3fd7052d7108286))

Add automated PostgreSQL backup via scripts/backup.sh using pg_dump
  with configurable retention. Document daily cron job, restore
  procedure, and cloud deployment backup options (RDS, Cloud SQL).

  Closes [#106](../../issues/106)


- Add CI pipeline, Docker Compose infrastructure, and frontend containerization ([`99a233a`](../../commit/99a233a5e5a70cf17f53f05e8a956393bf6b048c))

- Add CHANGELOG, JSDoc types, Node version enforcement, and linting ([`b47f6dd`](../../commit/b47f6dd58d2b4e6997f1e52781c3ac4ebad33ccc))

Add automated changelog generation:
    - cliff.toml: git-cliff config following Keep a Changelog format
    - CHANGELOG.md: seeded from git history, auto-updated on push via GitHub Actions
    - .github/workflows/changelog.yml: auto-commit CHANGELOG.md when conventional commits are pushed
    - make changelog: local regeneration target

  Add shared type definitions via JSDoc:
    - indexer/src/types.js: DecodedEvent, ContractMeta, HealthState, VolumeResult typedefs
    - decoder.js, db.js, index.js: annotated with @typedef imports and full @param/@returns
    - indexer/jsconfig.json: enable checkJs and strictNullChecks for editor type checking

  Enforce Node 20+:
    - indexer/package.json: add engines field
    - indexer/.npmrc: engine-strict=true to fail npm install on old Node

  Add code quality tooling:
    - indexer/eslint.config.js: ESLint 9 flat config (eslint:recommended + strict rules)
    - indexer/.prettierrc: consistent formatting (2 spaces, double quotes, trailing commas)
    - indexer/package.json: add lint, format, format:check scripts
    - Formatted all indexer/src/**/*.js for style consistency

  Update documentation:
    - README.md: add CHANGELOG.md link to SCF documents table
    - Makefile: add changelog target


- SAC detection, SEP-41 metadata fetcher, compliance validator, 24h volume endpoint ([`057cf07`](../../commit/057cf0757b4973bea27ef31f6d314aec26850023))

- sac.js: detect SAC bridge contracts, append classic asset code in descriptions
  - sep41Metadata.js: fetch name/symbol/decimals via simulateTransaction (read-only)
  - validateSep41.js: simulate all 10 mandatory SEP-41 functions, return compliance bool
  - db.js: get24hVolume() aggregates transfer events with NUMERIC precision
  - api.js: GET /api/tokens/:id/volume returns 24h rolling volume, zero float rounding


- Implement ScVal→JS converter and ContractAuth decoder ([`ff28c8a`](../../commit/ff28c8affce32c1d63ed0d61573fa62343e6d69c))

Closes [#3](../../issues/3) — Parse ScVal Types to Native JavaScript Types
  Closes [#4](../../issues/4) — Extract and Decode ContractAuth Arrays

  ---

  ## Issue [#3](../../issues/3) — ScVal to Native JS Type Converter (indexer/src/scval.js)

  ### Problem
  The existing decoder.js called scValToNative() from @stellar/stellar-sdk directly,
  which works for simple cases but loses precision on large integers (i64/u64/i128/u128/
  i256/u256) because JavaScript's Number type only has 53 bits of safe integer precision.
  There was also no centralised, well-typed utility that the rest of the codebase could
  import for consistent ScVal handling.

  ### Solution
  Created indexer/src/scval.js exporting a single function scValToJs(val).

  How it works:
  - Switches on val.switch().name to handle every ScVal variant explicitly.
  - Primitive types (bool, void, u32, i32, string, symbol, bytes) map directly to their
    JS equivalents.
  - Large integer types (u64, i64, timepoint, duration, u128, i128, u256, i256) are
    returned as native BigInt values, reconstructed from their hi/lo word pairs using
    bitwise shift operations, preventing any precision loss.
  - scvVec recursively maps each element through scValToJs, producing a plain JS array.
  - scvMap iterates the key/value pairs and builds a plain JS object, with keys coerced
    to strings.
  - scvAddress decodes both scAddressTypeAccount (Ed25519 public key → G... address via
    StrKey.encodeEd25519PublicKey) and scAddressTypeContract (contract hash → C... address
    via StrKey.encodeContract).
  - Ledger key and contract instance variants return descriptive sentinel objects rather
    than throwing.
  - Unknown/unhandled variants fall back to String(val) so the function never throws a
    runtime error, satisfying the acceptance criterion.

  ---

  ## Issue [#4](../../issues/4) — ContractAuth Array Extractor/Decoder (indexer/src/auth.js)

  ### Problem
  When a Soroban transaction is submitted, the InvokeHostFunctionOp XDR contains an
  auth[] vector of SorobanAuthorizationEntry objects. These entries record exactly which
  addresses authorised the invocation, the replay-prevention nonce each signer used, and
  the full tree of contract function calls being authorised. None of this was surfaced by
  the indexer, making it impossible to display authorisation information in the explorer.

  ### Solution
  Created indexer/src/auth.js exporting extractContractAuth(input).

  How it works:
  - Input flexibility: accepts either a base64 XDR string (TransactionEnvelope or bare
    Operation) or an already-parsed InvokeHostFunctionOp object. The function tries to
    parse as a full envelope first, then falls back to a bare operation, so callers do
    not need to pre-parse.
  - Auth entry decoding (decodeAuthEntry): inspects the credentials discriminant.
    - sorobanCredentialsAddress: extracts the signer address (account → G... string,
      contract → C... string) and the nonce as a BigInt.
    - sorobanCredentialsSourceAccount: signer and nonce remain null (source account
      authorisation carries no explicit address/nonce fields).
  - Invocation tree decoding (decodeInvocation, recursive): decodes the rootInvocation
    and all nested subInvocations into plain objects containing:
    - type: 'contractFn' | 'createContract' | raw discriminant name
    - contractId: C... encoded contract address
    - functionName: string name of the authorised function
    - args: array of native JS values produced by scValToJs (reuses issue [#3](../../issues/3) utility)
    - subInvocations: recursively decoded child invocations
  - Return shape per entry: { signer, nonce, rootInvocation } — directly satisfying the
    acceptance criteria of exposing the signer address, the nonce, and the root function
    call authorised


- Add XDR ContractEvent decoder utility ([`27f0a64`](../../commit/27f0a64fc7a9dfaced7942c6d5847cf4f643f7a8))

- Add indexer/src/xdr_decoder.js: decodeContractEvent(base64Xdr)
    decodes a raw ContractEvent XDR string into { contractId, type,
    topics, value } using @stellar/stellar-sdk xdr + scValToNative.
    Handles SYSTEM, CONTRACT, and DIAGNOSTIC event types. BigInt values
    are serialised as strings for JSON safety.

  - Add indexer/test/xdr_decoder.test.js: 5 unit tests covering all
    three event types, required-field presence, and BigInt serialisation.

  - Add "test" script to indexer/package.json (node --test)


- Add full Soroban Smart Block Explorer ([`90eb8a3`](../../commit/90eb8a37521e0e8efb3a6bc1a6da83c755298a8f))

- Soroban smart contract (ContractRegistry + EventDecoder)
  - Node.js indexer: Soroban RPC polling, XDR decoder, PostgreSQL, REST API
  - React frontend: Home, ContractPage, WalletPage, EventPage
  - SCF submission docs: ROADMAP.md, BUDGET.md, TEAM.md, MANIFEST.md
  - stellar.toml, Makefile, .env.example, LICENSE, .gitignore



### Miscellaneous

- Add governance, security, and development guidelines ([`5a1beab`](../../commit/5a1beab905033480fd579388b275e34fef45d31f))

- Add CONTRIBUTING.md with dev environment setup, branch naming convention,
    conventional commit format, PR checklist, code style standards, and bug
    report template
  - Add SECURITY.md with vulnerability disclosure process, supported versions,
    response timelines (24h acknowledgment, 72h triage), and responsible disclosure
    guidelines
  - Complete SEP-1 stellar.toml with mandatory fields: SIGNING_KEY, DOCUMENTATION
    (ORG_NAME, ORG_GITHUB, ORG_DESCRIPTION, ORG_URL, ORG_SUPPORT_EMAIL), and
    PRINCIPALS metadata. Include signing instructions for file verification.
  - Set NODE_ENV=production in Makefile indexer target to enable production
    optimizations in pino, express, and other Node.js libraries



### Testing

- Add WASM sandbox integration test stubs ([`34a2dba`](../../commit/34a2dba4cf9dec0223758b8b5891a33584a9b96a))

Add a tests/integration/ directory with stub tests that document the
  intended integration test flow for the explorer contract deployed to a
  real Stellar CLI sandbox.  No WASM is compiled and no sandbox is
  required to run the stubs.

  Files added:
  - tests/integration/README.md         — setup instructions
  - tests/integration/run_integration.sh — shell CLI test harness
  - tests/integration/rust/Cargo.toml   — standalone test crate
  - tests/integration/rust/src/lib.rs   — Rust stub tests

  Closes [#24](../../issues/24)
  Closes [#25](../../issues/25)
  Closes [#27](../../issues/27)
  Closes [#28](../../issues/28)




