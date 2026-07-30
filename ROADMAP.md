# Roadmap — Soroban Smart Block Explorer

> SCF Build Award — 3-tranche milestone plan  
> Total timeline: ~14 weeks | Target: Stellar Mainnet

---

## Tranche 1 — MVP (Weeks 1–4)

**Goal:** Working contract registry + decoder on testnet, indexer polling live events, basic frontend.

### Deliverables

| # | Deliverable | Description |
|---|-------------|-------------|
| 1.1 | Soroban contract deployed to testnet | `ExplorerContract` with `register_contract`, `submit_event`, `get_events` fully functional |
| 1.2 | Indexer live on testnet | Polls `SorobanRpc.getEvents()` every 5 s, decodes SEP-41 `transfer`/`mint`/`burn` events into human-readable strings |
| 1.3 | REST API operational | All 5 endpoints (`/api/events`, `/api/events/:seq`, `/api/contracts/:id`, `POST /api/contracts`, `/api/wallet/:address`) returning real data |
| 1.4 | React frontend — Home + Event detail | Paginated event feed with function filter; `/event/:seq` detail page |
| 1.5 | Public GitHub repo | MIT-licensed, README, `.env.example`, Makefile, all code open-source |

**Proof of completion:** Publicly accessible testnet deployment URL + GitHub repo link with passing `cargo test`.

---

## Tranche 2 — Testnet (Weeks 5–10)

**Goal:** Full feature set, ABI registry open to third-party contracts, wallet history, SEP-41 + classic asset support.

### Deliverables

| # | Deliverable | Description |
|---|-------------|-------------|
| 2.1 | ABI registry open to community | Any developer can call `register_contract` to add their contract's function signatures; UI to submit metadata |
| 2.2 | Wallet history page | `/wallet/:address` shows all Soroban interactions for any Stellar address |
| 2.3 | Contract detail page | `/contract/:id` shows ABI metadata, function list, and full event history |
| 2.4 | Classic asset support via Horizon | Decoder resolves classic Stellar asset codes/issuers alongside SEP-41 tokens |
| 2.5 | StellarSwap + Blend integration | Register ABI metadata for at least 2 live testnet DEX/lending contracts; demonstrate decoded swap/lend events |
| 2.6 | Decoder coverage for top 5 function types | `swap`, `transfer`, `mint`, `burn`, `stake` all produce human-readable output |
| 2.7 | Performance: index lag < 10 s | Indexer keeps within 2 ledgers of chain tip under normal load |

**Proof of completion:** Live testnet URL demonstrating decoded events from ≥2 third-party contracts; video walkthrough.

---

## Tranche 3 — Mainnet Launch (Weeks 11–14)

**Goal:** Production deployment on Stellar mainnet, public launch, documentation complete.

### Deliverables

| # | Deliverable | Description |
|---|-------------|-------------|
| 3.1 | Mainnet contract deployed | `ExplorerContract` deployed to Stellar mainnet; contract ID published in `stellar.toml` |
| 3.2 | Indexer on mainnet | Indexing live mainnet Soroban events; PostgreSQL hosted with daily automated backups via `scripts/backup.sh` and cron |
| 3.3 | Public frontend at stable URL | Hosted at a permanent domain; mobile-responsive |
| 3.4 | Developer documentation | `docs/` covering: how to register a contract, ABI schema reference, REST API reference |
| 3.5 | ≥5 mainnet contracts registered | Outreach to Stellar DeFi projects (StellarSwap, Blend, Phoenix DEX, etc.) to register ABI metadata at launch |
| 3.6 | Open-source audit readiness | All contract code open-source; ready for Audit Bank review |

**Proof of completion:** Live mainnet URL with real decoded events; ≥5 registered contracts visible in the UI; all docs published.

---

## Timeline Summary

| Week | Milestone |
|------|-----------|
| 1–2  | Contract development + unit tests |
| 3–4  | Indexer + API + basic frontend → **Tranche 1 complete** |
| 5–6  | ABI registry UI + wallet history |
| 7–8  | Classic asset support + DEX integrations |
| 9–10 | Performance tuning + testnet hardening → **Tranche 2 complete** |
| 11–12 | Mainnet deployment + partner outreach |
| 13–14 | Docs, polish, public launch → **Tranche 3 complete** |
