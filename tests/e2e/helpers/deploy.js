import { execSync } from "child_process";
import { readFileSync, writeFileSync } from "fs";
import { resolve, dirname } from "path";
import { fileURLToPath } from "url";
import { Keypair, SorobanRpc } from "@stellar/stellar-sdk";

const __dirname = dirname(fileURLToPath(import.meta.url));
const ROOT = resolve(__dirname, "..", "..", "..");

// ── Config ───────────────────────────────────────────────────────────
const RPC_URL = process.env.SOROBAN_RPC_URL || "http://localhost:8000/soroban/rpc";
const FRIENDBOT_URL = process.env.FRIENDBOT_URL || "http://localhost:8000/friendbot";
const NETWORK = "local";
const NETWORK_PASSPHRASE = "Standalone Network ; February 2017";
const WASM_PATH =
  process.env.WASM_PATH ||
  resolve(ROOT, "target/wasm32-unknown-unknown/release/soroban_explorer_contract.wasm");
const FIXTURES_DIR = resolve(__dirname, "..", "fixtures");
const ABI_FIXTURE = resolve(FIXTURES_DIR, "explorer-abi.json");

/* Helper: retry a predicate with exponential backoff */
async function waitFor(predicate, { label = "", timeout = 60_000, interval = 1_000 } = {}) {
  const deadline = Date.now() + timeout;
  while (Date.now() < deadline) {
    if (await predicate()) return;
    await new Promise((r) => setTimeout(r, interval));
  }
  throw new Error(`Timed out waiting for: ${label || "condition"}`);
}

/* Helper: shell out and return trimmed stdout */
function run(cmd, opts = {}) {
  return execSync(cmd, {
    encoding: "utf-8",
    maxBuffer: 10 * 1024 * 1024,
    ...opts,
  }).trim();
}

/* Generate a random Stellar keypair and fund it from the sandbox friendbot */
async function createFundedAccount() {
  const kp = Keypair.random();
  const pub = kp.publicKey();

  console.log(`\n  Creating account: ${pub}`);
  const fbRes = await fetch(`${FRIENDBOT_URL}?addr=${pub}`);
  if (!fbRes.ok) {
    const body = await fbRes.text();
    throw new Error(`Friendbot error (${fbRes.status}): ${body}`);
  }

  const server = new SorobanRpc.Server(RPC_URL, { allowHttp: true });
  await waitFor(
    async () => {
      try {
        await server.getAccount(pub);
        return true;
      } catch {
        return false;
      }
    },
    { label: `account ${pub} to be funded`, timeout: 30_000 }
  );

  return { kp, pub, secret: kp.secret() };
}

/* Build the contract WASM via cargo */
function buildWasm() {
  console.log("\n  Building contract WASM…");
  run("cargo build --release --target wasm32-unknown-unknown -p soroban-explorer-contract", {
    cwd: ROOT,
  });
  console.log(`  WASM built: ${WASM_PATH}`);
}

/* Install and activate the soroban CLI via cargo */
function ensureCli() {
  try {
    run("soroban --version");
  } catch {
    console.log("\n  Installing soroban CLI (this may take a while)…");
    run("cargo install soroban-cli --locked", { cwd: ROOT, timeout: 300_000 });
  }
}

/* Deploy the contract to the sandbox using the soroban CLI.
 * Returns the deployed contract ID (strkey C…). */
function deployContract(secretKey) {
  console.log("\n  Deploying contract to sandbox…");
  const cid = run(
    `soroban contract deploy \
      --wasm "${WASM_PATH}" \
      --source ${secretKey} \
      --rpc-url "${RPC_URL}" \
      --network-passphrase "${NETWORK_PASSPHRASE}"`
  );
  console.log(`  Contract deployed: ${cid}`);
  return cid;
}

/* Initialize the contract with an admin address */
function initContract(cid, adminSecret) {
  console.log("\n  Initializing contract…");
  run(
    `soroban contract invoke \
      --id ${cid} \
      --source ${adminSecret} \
      --rpc-url "${RPC_URL}" \
      --network-passphrase "${NETWORK_PASSPHRASE}" \
      -- \
      init \
      --admin ${Keypair.fromSecret(adminSecret).publicKey()}`
  );
  console.log("  Contract initialized.");
}

/* Register the ExplorerContract's own ABI metadata via the REST API */
async function registerContractAbi(cid, apiBase) {
  console.log(`\n  Registering ABI for ${cid} via API…`);
  const fixture = JSON.parse(readFileSync(ABI_FIXTURE, "utf-8"));
  fixture.id = cid;
  fixture.registered_by = Keypair.random().publicKey();

  const res = await fetch(`${apiBase}/api/contracts`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(fixture),
  });
  if (!res.ok) {
    const body = await res.text();
    throw new Error(`Register ABI failed (${res.status}): ${body}`);
  }
  console.log("  ABI registered.");
}

/* Submit a decoded event via the contract's submit_event function.
 * This emits a "decoded" event on-chain which the indexer picks up. */
function submitEvent(cid, adminSecret) {
  const description = "Address GA… swapped 100 USDC → 98.7 XLM on TestDEX";
  console.log(`\n  Submitting event via contract: "${description}"`);

  run(
    `soroban contract invoke \
      --id ${cid} \
      --source ${adminSecret} \
      --rpc-url "${RPC_URL}" \
      --network-passphrase "${NETWORK_PASSPHRASE}" \
      -- \
      submit_event \
      --caller ${Keypair.fromSecret(adminSecret).publicKey()} \
      --contract_id 0000000000000000000000000000000000000000000000000000000000000001 \
      --function swap \
      --ledger 100 \
      --description "${description}" \
      --raw_topics '["swap","USDC", "XLM"]' \
      --raw_data 00`
  );

  console.log("  Event submitted.");
  return description;
}

/* Wait for the indexer to have processed at least N events */
async function waitForEvents(apiBase, minEvents = 1, timeout = 90_000) {
  console.log(`\n  Waiting for indexer to process ≥${minEvents} event(s)…`);
  let events = [];
  await waitFor(
    async () => {
      try {
        const res = await fetch(`${apiBase}/api/events?limit=5`);
        if (!res.ok) return false;
        const body = await res.json();
        events = body.events ?? [];
        return events.length >= minEvents;
      } catch {
        return false;
      }
    },
    { label: "indexer to pick up events", timeout }
  );
  console.log(`  Indexer picked up ${events.length} event(s).`);
  return events;
}

// ── Main ─────────────────────────────────────────────────────────────
export async function setup({ apiBase = "http://localhost:3001" } = {}) {
  console.log("\n══════════════════════════════════════════════");
  console.log("  E2E Test Setup: Deploy & Seed");
  console.log("══════════════════════════════════════════════\n");

  /* 1. Build contract WASM */
  buildWasm();

  /* 2. Ensure soroban CLI is available */
  ensureCli();

  /* 3. Create a funded admin account */
  const { kp, secret } = await createFundedAccount();

  /* 4. Deploy the ExplorerContract */
  const cid = deployContract(secret);

  /* 5. Initialize the contract */
  initContract(cid, secret);

  /* 6. Wait for frontend/API to be ready */
  console.log("\n  Waiting for API at", apiBase);
  await waitFor(
    async () => {
      try {
        const res = await fetch(`${apiBase}/health`);
        return res.ok;
      } catch {
        return false;
      }
    },
    { label: "API to be ready", timeout: 60_000 }
  );

  /* 7. Register ABI metadata */
  await registerContractAbi(cid, apiBase);

  /* 8. Submit a decoded event (emits on-chain "decoded" event) */
  const description = submitEvent(cid, secret);

  /* 9. Wait for the indexer to pick up the event */
  const events = await waitForEvents(apiBase, 1);

  /* 10. Verify the event is the one we submitted */
  const matched = events.find(
    (e) => e.description && e.description.includes(description.slice(0, 30))
  );
  if (matched) {
    console.log("\n  ✓ Event confirmed in API:", matched.description);
  } else {
    console.warn("\n  ⚠  Submitted event not found via API. Check indexer logs.");
    console.log("  Events found:", events.map((e) => e.description));
  }

  console.log("\n══════════════════════════════════════════════");
  console.log("  Setup complete.");
  console.log("══════════════════════════════════════════════\n");

  return { contractId: cid, description, events };
}

/* Allow running directly: node helpers/deploy.js */
if (process.argv[1] === fileURLToPath(import.meta.url)) {
  setup().catch((err) => {
    console.error("Setup failed:", err);
    process.exit(1);
  });
}
