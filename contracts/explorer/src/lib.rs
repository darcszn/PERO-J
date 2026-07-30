#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, symbol_short,
    Address, Bytes, BytesN, Env, String, Symbol, Vec,
    panic_with_error,
};

// ── Error codes ──────────────────────────────────────────────────────────────
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    NotFound       = 1,
    Unauthorized   = 2,
    AlreadyExists  = 3,
    LimitExceeded  = 4,
}

// ── Input limits ─────────────────────────────────────────────────────────────
// Bounds on caller-supplied payloads so a single call cannot bloat on-chain
// storage (and the rent every user pays for it) without limit.

/// Largest `raw_data` blob accepted by `submit_event`.
pub const MAX_RAW_DATA_BYTES: u32 = 4096;
/// Largest number of functions accepted in a `ContractMeta`.
pub const MAX_FUNCTIONS: u32 = 64;
/// Largest number of parameters accepted per function.
pub const MAX_PARAMS: u32 = 32;

// ── Storage keys ─────────────────────────────────────────────────────────────
#[contracttype]
pub enum DataKey {
    Admin,
    Contract(BytesN<32>),   // contract_id → ContractMeta
    EventLog(u64),          // seq → DecodedEvent
    EventSeq,
    IndexerAllowlist,       // → Vec<Address> of trusted event submitters
}

// ── Limits & TTL ──────────────────────────────────────────────────────────────

/// Maximum number of events `get_events` will read in a single call.
/// Guards against a caller passing a huge `limit` and exhausting the host's
/// CPU-instruction budget.
pub const MAX_PAGE: u32 = 200;

/// Maximum number of addresses that may sit on the indexer allowlist.
pub const MAX_INDEXERS: u32 = 20;

const DAY_IN_LEDGERS: u32 = 17_280;
/// Extend an entry whenever its remaining TTL drops below 30 days …
const TTL_THRESHOLD: u32 = 30 * DAY_IN_LEDGERS;
/// … and bump it back out to 90 days.
const TTL_EXTEND_TO: u32 = 90 * DAY_IN_LEDGERS;

// ── Data types ────────────────────────────────────────────────────────────────

/// ABI-like metadata for a registered contract.
#[contracttype]
#[derive(Clone)]
pub struct ContractMeta {
    pub name:        String,          // e.g. "StellarSwap"
    pub description: String,
    pub functions:   Vec<FunctionAbi>,
    pub registered_by: Address,
}

/// Describes one callable function so the explorer can decode calls.
#[contracttype]
#[derive(Clone)]
pub struct FunctionAbi {
    pub name:        Symbol,          // e.g. symbol_short!("swap")
    pub description: String,          // "Swap token_in for token_out"
    pub params:      Vec<ParamDef>,
}

/// One parameter definition.
#[contracttype]
#[derive(Clone)]
pub struct ParamDef {
    pub name:     Symbol,
    pub kind:     Symbol,   // "address" | "i128" | "symbol" | "bytes"
}

/// A decoded, human-readable event stored on-chain.
#[contracttype]
#[derive(Clone)]
pub struct DecodedEvent {
    pub seq:          u64,
    pub contract_id:  BytesN<32>,
    pub function:     Symbol,
    pub ledger:       u32,
    pub description:  String,   // "Address GA… swapped 100 USDC → 98.7 XLM"
    pub raw_topics:   Vec<String>,
    pub raw_data:     Bytes,
}

// ── Contract ──────────────────────────────────────────────────────────────────
#[contract]
pub struct ExplorerContract;

// Internal helpers — deliberately outside `#[contractimpl]` so they are not
// exported as contract entry points.
impl ExplorerContract {
    /// Read the admin address from *persistent* storage.
    ///
    /// The admin lives in persistent storage (never instance storage) so that an
    /// expired instance entry can never make the contract look uninitialised.
    fn admin(env: &Env) -> Address {
        env.storage()
            .persistent()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(env, Error::NotFound))
    }

    /// Bump the TTL of the instance entry and of the admin entry.
    ///
    /// Called from every mutating entry point so that an active contract can
    /// never let its state — in particular the `Admin` guard used by `init` —
    /// lapse and become re-writable.
    fn bump_ttl(env: &Env) {
        env.storage().instance().extend_ttl(TTL_THRESHOLD, TTL_EXTEND_TO);
        if env.storage().persistent().has(&DataKey::Admin) {
            env.storage()
                .persistent()
                .extend_ttl(&DataKey::Admin, TTL_THRESHOLD, TTL_EXTEND_TO);
        }
    }

    /// Panic unless `caller` is the admin or sits on the indexer allowlist.
    fn require_submitter(env: &Env, caller: &Address) {
        if *caller == Self::admin(env) {
            return;
        }
        let allowlist: Vec<Address> = env
            .storage()
            .persistent()
            .get(&DataKey::IndexerAllowlist)
            .unwrap_or_else(|| Vec::new(env));
        if !allowlist.contains(caller) {
            panic_with_error!(env, Error::Unauthorized);
        }
    }
}

#[contractimpl]
impl ExplorerContract {

    // ── Admin ─────────────────────────────────────────────────────────────────

    /// Initialise with an admin address (call once).
    ///
    /// The `Admin` marker is written to **persistent** storage: instance storage
    /// has a TTL, and if it lapsed an attacker could call `init` again and take
    /// over the contract.  Persistent entries can be archived but never silently
    /// disappear, so this guard is permanent.
    pub fn init(env: Env, admin: Address) {
        if env.storage().persistent().has(&DataKey::Admin) {
            panic_with_error!(&env, Error::AlreadyExists);
        }
        env.storage().persistent().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::EventSeq, &0u64);
        Self::bump_ttl(&env);
    }

    /// Return the current admin address.
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance()
            .get(&DataKey::Admin)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized))
    }

    /// Transfer admin rights to a new address.
    ///
    /// Only the current admin may call this.  Both `current_admin` and
    /// `new_admin` must authorise the transaction so that a mis-typed address
    /// cannot accidentally lock the contract.
    ///
    /// Key-management recommendation: keep `new_admin` on a hardware wallet
    /// (Ledger/Trezor) or use a multi-sig account.  See SECURITY.md for the
    /// emergency recovery procedure.
    pub fn transfer_admin(env: Env, current_admin: Address, new_admin: Address) {
        current_admin.require_auth();
        new_admin.require_auth();

        let stored = Self::admin(&env);

        if current_admin != stored {
            panic_with_error!(&env, Error::Unauthorized);
        }

        env.storage().persistent().set(&DataKey::Admin, &new_admin);
        Self::bump_ttl(&env);
        env.events().publish(
            (symbol_short!("adm_xfr"),),
            (current_admin, new_admin),
        );
    }

    // ── Indexer allowlist ─────────────────────────────────────────────────────

    /// Whitelist `indexer` as a trusted event submitter.
    ///
    /// In production the indexer runs as a hot wallet that must be able to call
    /// `submit_event` without holding the (cold) admin key.
    pub fn add_indexer(env: Env, admin: Address, indexer: Address) {
        admin.require_auth();
        if admin != Self::admin(&env) {
            panic_with_error!(&env, Error::Unauthorized);
        }

        let mut allowlist: Vec<Address> = env.storage().persistent()
            .get(&DataKey::IndexerAllowlist)
            .unwrap_or_else(|| Vec::new(&env));

        if allowlist.contains(&indexer) {
            panic_with_error!(&env, Error::AlreadyExists);
        }
        if allowlist.len() >= MAX_INDEXERS {
            panic_with_error!(&env, Error::LimitExceeded);
        }

        allowlist.push_back(indexer.clone());
        env.storage().persistent().set(&DataKey::IndexerAllowlist, &allowlist);
        Self::bump_ttl(&env);
        env.storage().persistent().extend_ttl(
            &DataKey::IndexerAllowlist, TTL_THRESHOLD, TTL_EXTEND_TO,
        );

        env.events().publish((symbol_short!("idx_add"),), indexer);
    }

    /// Revoke a previously whitelisted indexer.
    pub fn remove_indexer(env: Env, admin: Address, indexer: Address) {
        admin.require_auth();
        if admin != Self::admin(&env) {
            panic_with_error!(&env, Error::Unauthorized);
        }

        let mut allowlist: Vec<Address> = env.storage().persistent()
            .get(&DataKey::IndexerAllowlist)
            .unwrap_or_else(|| Vec::new(&env));

        let idx = allowlist
            .first_index_of(&indexer)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotFound));

        allowlist.remove(idx);
        env.storage().persistent().set(&DataKey::IndexerAllowlist, &allowlist);
        Self::bump_ttl(&env);

        env.events().publish((symbol_short!("idx_rm"),), indexer);
    }

    /// List every address currently allowed to submit events (excluding admin).
    pub fn get_indexers(env: Env) -> Vec<Address> {
        env.storage().persistent()
            .get(&DataKey::IndexerAllowlist)
            .unwrap_or_else(|| Vec::new(&env))
    }

    /// True if `address` may call `submit_event` (admin or allowlisted indexer).
    pub fn is_indexer(env: Env, address: Address) -> bool {
        let allowlist: Vec<Address> = env.storage().persistent()
            .get(&DataKey::IndexerAllowlist)
            .unwrap_or_else(|| Vec::new(&env));
        allowlist.contains(&address)
            || env.storage().persistent().get::<_, Address>(&DataKey::Admin)
                  .map_or(false, |a| a == address)
    }

    // ── Contract Registry ─────────────────────────────────────────────────────

    /// Register ABI-like metadata for a Soroban contract.
    pub fn register_contract(
        env:         Env,
        caller:      Address,
        contract_id: BytesN<32>,
        meta:        ContractMeta,
    ) {
        caller.require_auth();
        Self::validate_meta(&env, &meta);
        let key = DataKey::Contract(contract_id.clone());
        if env.storage().persistent().has(&key) {
            panic_with_error!(&env, Error::AlreadyExists);
        }
        env.storage().persistent().set(&key, &meta);
        Self::bump_ttl(&env);
        env.events().publish(
            (symbol_short!("register"), contract_id),
            meta.name,
        );
    }

    /// Update metadata (admin or original registrant only).
    pub fn update_contract(
        env:         Env,
        caller:      Address,
        contract_id: BytesN<32>,
        meta:        ContractMeta,
    ) {
        caller.require_auth();
        Self::validate_meta(&env, &meta);
        let key = DataKey::Contract(contract_id.clone());
        let existing: ContractMeta = env.storage().persistent()
            .get(&key).unwrap_or_else(|| panic_with_error!(&env, Error::NotFound));

        let admin = Self::admin(&env);
        if caller != existing.registered_by && caller != admin {
            panic_with_error!(&env, Error::Unauthorized);
        }
        env.storage().persistent().set(&key, &meta);
        Self::bump_ttl(&env);

        // Emitted so the indexer can invalidate its in-memory ABI cache instead
        // of polling storage for metadata revisions.
        env.events().publish(
            (symbol_short!("update"), contract_id),
            meta.name,
        );
    }

    /// Fetch metadata for a contract.
    pub fn get_contract(env: Env, contract_id: BytesN<32>) -> ContractMeta {
        env.storage().persistent()
            .get(&DataKey::Contract(contract_id))
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotFound))
    }

    // ── Event Decoder ─────────────────────────────────────────────────────────

    /// Submit a decoded event (called by the off-chain indexer via a trusted tx).
    /// The indexer decodes raw XDR and calls this to persist a human-readable record.
    pub fn submit_event(
        env:         Env,
        caller:      Address,
        contract_id: BytesN<32>,
        function:    Symbol,
        ledger:      u32,
        description: String,
        raw_topics:  Vec<String>,
        raw_data:    Bytes,
    ) {
        caller.require_auth();
        // Only the admin or an allowlisted indexer may submit events.
        Self::require_submitter(&env, &caller);

        let seq: u64 = Self::event_seq(&env);
        let event = DecodedEvent {
            seq,
            contract_id: contract_id.clone(),
            function: function.clone(),
            ledger,
            description: description.clone(),
            raw_topics,
            raw_data,
        };
        env.storage().persistent().set(&DataKey::EventLog(seq), &event);
        env.storage().instance().set(&DataKey::EventSeq, &(seq + 1));
        Self::bump_ttl(&env);

        env.events().publish(
            (symbol_short!("decoded"), contract_id, function),
            description,
        );
    }

    /// Fetch a single decoded event by sequence number.
    pub fn get_event(env: Env, seq: u64) -> DecodedEvent {
        env.storage().persistent()
            .get(&DataKey::EventLog(seq))
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotFound))
    }

    /// Return the total number of stored events.
    ///
    /// Panics with `NotInitialized` if the counter is absent, rather than
    /// reporting a misleading 0.
    pub fn event_count(env: Env) -> u64 {
        Self::event_seq(&env)
    }

    /// Fetch a page of events `[from, from+limit)`.
    ///
    /// `limit` is capped at [`MAX_PAGE`]; a larger value panics rather than
    /// burning the whole transaction's CPU budget on storage reads.
    pub fn get_events(env: Env, from: u64, limit: u32) -> Vec<DecodedEvent> {
        if limit > MAX_PAGE {
            panic_with_error!(&env, Error::LimitExceeded);
        }
        let total: u64 = env.storage().instance().get(&DataKey::EventSeq).unwrap_or(0);
        let mut out: Vec<DecodedEvent> = Vec::new(&env);
        let end = from.saturating_add(limit as u64).min(total);
        for seq in from..end {
            if let Some(ev) = env.storage().persistent().get(&DataKey::EventLog(seq)) {
                out.push_back(ev);
            }
        }
        out
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{
        testutils::{storage::Instance as _, Address as _, Events as _, Ledger as _},
        Env, IntoVal, TryFromVal,
    };

    fn setup() -> (Env, ExplorerContractClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, ExplorerContract);
        let client = ExplorerContractClient::new(&env, &id);
        (env, client)
    }

    #[test]
    fn test_init_and_register() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let cid: BytesN<32> = BytesN::from_array(&env, &[1u8; 32]);
        let meta = ContractMeta {
            name: String::from_str(&env, "StellarSwap"),
            description: String::from_str(&env, "DEX on Stellar"),
            functions: Vec::new(&env),
            registered_by: admin.clone(),
        };
        client.register_contract(&admin, &cid, &meta);
        let fetched = client.get_contract(&cid);
        assert_eq!(fetched.name, String::from_str(&env, "StellarSwap"));
    }

    #[test]
    fn test_submit_and_get_event() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let cid: BytesN<32> = BytesN::from_array(&env, &[2u8; 32]);
        client.submit_event(
            &admin,
            &cid,
            &symbol_short!("swap"),
            &4521983u32,
            &String::from_str(&env, "Address GABC... swapped 100 USDC → 98.7 XLM on StellarSwap"),
            &Vec::new(&env),
            &Bytes::new(&env),
        );

        assert_eq!(client.event_count(), 1u64);
        let ev = client.get_event(&0u64);
        assert_eq!(ev.ledger, 4521983u32);
    }

    #[test]
    #[should_panic]
    fn test_double_init_panics() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);
        client.init(&admin); // should panic
    }

    #[test]
    fn test_transfer_admin() {
        let (env, client) = setup();
        let admin     = Address::generate(&env);
        let new_admin = Address::generate(&env);
        client.init(&admin);

        // Transfer admin rights to new_admin
        client.transfer_admin(&admin, &new_admin);

        // new_admin can now submit events; old admin cannot
        let cid: BytesN<32> = BytesN::from_array(&env, &[3u8; 32]);
        client.submit_event(
            &new_admin,
            &cid,
            &symbol_short!("test"),
            &1u32,
            &String::from_str(&env, "test event"),
            &Vec::new(&env),
            &Bytes::new(&env),
        );
        assert_eq!(client.event_count(), 1u64);
    }

    #[test]
    fn test_get_admin() {
        let (env, client) = setup();
        let admin     = Address::generate(&env);
        let new_admin = Address::generate(&env);
        client.init(&admin);
        assert_eq!(client.get_admin(), admin);

        client.transfer_admin(&admin, &new_admin);
        assert_eq!(client.get_admin(), new_admin);
    }

    #[test]
    #[should_panic]
    fn test_get_admin_uninitialised_panics() {
        let (_env, client) = setup();
        client.get_admin();
    }

    #[test]
    #[should_panic]
    fn test_event_count_uninitialised_panics() {
        let (_env, client) = setup();
        client.event_count();
    }

    #[test]
    fn test_submit_event_max_raw_data_ok() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let cid: BytesN<32> = BytesN::from_array(&env, &[4u8; 32]);
        let mut raw = Bytes::new(&env);
        for _ in 0..MAX_RAW_DATA_BYTES {
            raw.push_back(0u8);
        }
        client.submit_event(
            &admin, &cid, &symbol_short!("swap"), &1u32,
            &String::from_str(&env, "at the limit"),
            &Vec::new(&env), &raw,
        );
        assert_eq!(client.event_count(), 1u64);
    }

    #[test]
    #[should_panic]
    fn test_submit_event_oversized_raw_data_panics() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let cid: BytesN<32> = BytesN::from_array(&env, &[5u8; 32]);
        let mut raw = Bytes::new(&env);
        for _ in 0..(MAX_RAW_DATA_BYTES + 1) {
            raw.push_back(0u8);
        }
        client.submit_event(
            &admin, &cid, &symbol_short!("swap"), &1u32,
            &String::from_str(&env, "one byte too many"),
            &Vec::new(&env), &raw,
        );
    }

    /// Build a `ContractMeta` with `n` functions, each carrying `params` params.
    fn meta_with(env: &Env, admin: &Address, n: u32, params: u32) -> ContractMeta {
        let mut functions: Vec<FunctionAbi> = Vec::new(env);
        for _ in 0..n {
            let mut p: Vec<ParamDef> = Vec::new(env);
            for _ in 0..params {
                p.push_back(ParamDef {
                    name: symbol_short!("amount"),
                    kind: symbol_short!("i128"),
                });
            }
            functions.push_back(FunctionAbi {
                name:        symbol_short!("swap"),
                description: String::from_str(env, "swap"),
                params:      p,
            });
        }
        ContractMeta {
            name:          String::from_str(env, "StellarSwap"),
            description:   String::from_str(env, "DEX on Stellar"),
            functions,
            registered_by: admin.clone(),
        }
    }

    #[test]
    fn test_register_contract_at_abi_limits_ok() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let cid: BytesN<32> = BytesN::from_array(&env, &[6u8; 32]);
        let meta = meta_with(&env, &admin, MAX_FUNCTIONS, MAX_PARAMS);
        client.register_contract(&admin, &cid, &meta);
        assert_eq!(client.get_contract(&cid).functions.len(), MAX_FUNCTIONS);
    }

    #[test]
    #[should_panic]
    fn test_register_contract_too_many_functions_panics() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let cid: BytesN<32> = BytesN::from_array(&env, &[7u8; 32]);
        let meta = meta_with(&env, &admin, MAX_FUNCTIONS + 1, 0);
        client.register_contract(&admin, &cid, &meta);
    }

    #[test]
    #[should_panic]
    fn test_register_contract_too_many_params_panics() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let cid: BytesN<32> = BytesN::from_array(&env, &[8u8; 32]);
        let meta = meta_with(&env, &admin, 1, MAX_PARAMS + 1);
        client.register_contract(&admin, &cid, &meta);
    }

    #[test]
    #[should_panic]
    fn test_update_contract_too_many_functions_panics() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let cid: BytesN<32> = BytesN::from_array(&env, &[9u8; 32]);
        client.register_contract(&admin, &cid, &meta_with(&env, &admin, 1, 1));
        client.update_contract(&admin, &cid, &meta_with(&env, &admin, MAX_FUNCTIONS + 1, 0));
    }

    #[test]
    #[should_panic]
    fn test_transfer_admin_wrong_caller_panics() {
        let (env, client) = setup();
        let admin    = Address::generate(&env);
        let attacker = Address::generate(&env);
        client.init(&admin);
        // attacker tries to hijack admin — must panic
        client.transfer_admin(&attacker, &attacker);
    }

    // ── #1 — init is permanently irreversible ────────────────────────────────

    /// The admin guard must not live in instance storage.
    ///
    /// Wiping the instance `Admin` slot models the entry lapsing; a second
    /// `init` still has to fail because the real guard is persistent.  Against
    /// the old instance-storage implementation this test fails: `init` would
    /// succeed and hand the contract to `attacker`.
    #[test]
    #[should_panic(expected = "Error(Contract, #3)")]
    fn test_init_is_irreversible_without_instance_entry() {
        let env = Env::default();
        env.mock_all_auths();
        let id = env.register_contract(None, ExplorerContract);
        let client = ExplorerContractClient::new(&env, &id);

        let admin    = Address::generate(&env);
        let attacker = Address::generate(&env);
        client.init(&admin);

        env.as_contract(&id, || {
            env.storage().instance().remove(&DataKey::Admin);
        });

        client.init(&attacker); // must still panic — admin is persistent
    }

    /// Every mutating entry point bumps the instance TTL, so an actively used
    /// contract never lets its state lapse in the first place.
    #[test]
    fn test_mutating_calls_bump_ttl() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let start = env.ledger().sequence();
        env.ledger().with_mut(|l| l.sequence_number = start + 100_000);

        let cid: BytesN<32> = BytesN::from_array(&env, &[4u8; 32]);
        client.submit_event(
            &admin,
            &cid,
            &symbol_short!("swap"),
            &1u32,
            &String::from_str(&env, "keeps the instance alive"),
            &Vec::new(&env),
            &Bytes::new(&env),
        );

        // TTL was pushed well past the 30-day bump threshold.
        let ttl = env.as_contract(&client.address, || env.storage().instance().get_ttl());
        assert!(ttl >= TTL_THRESHOLD, "instance ttl {} not bumped", ttl);
    }

    // ── #2 — indexer allowlist ───────────────────────────────────────────────

    #[test]
    fn test_allowlisted_indexer_can_submit() {
        let (env, client) = setup();
        let admin   = Address::generate(&env);
        let indexer = Address::generate(&env);
        client.init(&admin);
        client.add_indexer(&admin, &indexer);

        assert!(client.is_indexer(&indexer));
        assert_eq!(client.get_indexers().len(), 1);

        let cid: BytesN<32> = BytesN::from_array(&env, &[7u8; 32]);
        client.submit_event(
            &indexer,
            &cid,
            &symbol_short!("swap"),
            &42u32,
            &String::from_str(&env, "indexer submitted"),
            &Vec::new(&env),
            &Bytes::new(&env),
        );
        assert_eq!(client.event_count(), 1u64);
    }

    #[test]
    #[should_panic]
    fn test_removed_indexer_cannot_submit() {
        let (env, client) = setup();
        let admin   = Address::generate(&env);
        let indexer = Address::generate(&env);
        client.init(&admin);
        client.add_indexer(&admin, &indexer);
        client.remove_indexer(&admin, &indexer);
        assert!(!client.is_indexer(&indexer));

        let cid: BytesN<32> = BytesN::from_array(&env, &[8u8; 32]);
        client.submit_event(
            &indexer,
            &cid,
            &symbol_short!("swap"),
            &1u32,
            &String::from_str(&env, "should fail"),
            &Vec::new(&env),
            &Bytes::new(&env),
        );
    }

    #[test]
    #[should_panic]
    fn test_non_admin_cannot_add_indexer() {
        let (env, client) = setup();
        let admin    = Address::generate(&env);
        let attacker = Address::generate(&env);
        client.init(&admin);
        client.add_indexer(&attacker, &attacker);
    }

    // ── #3 — get_events page cap ─────────────────────────────────────────────

    #[test]
    #[should_panic]
    fn test_get_events_rejects_oversized_limit() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);
        client.get_events(&0u64, &u32::MAX);
    }

    #[test]
    fn test_get_events_accepts_max_page() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);
        assert_eq!(client.get_events(&0u64, &MAX_PAGE).len(), 0);
    }

    // ── #4 — update_contract emits an event ──────────────────────────────────

    #[test]
    fn test_update_contract_emits_event() {
        let (env, client) = setup();
        let admin = Address::generate(&env);
        client.init(&admin);

        let cid: BytesN<32> = BytesN::from_array(&env, &[9u8; 32]);
        let meta = ContractMeta {
            name: String::from_str(&env, "StellarSwap"),
            description: String::from_str(&env, "DEX on Stellar"),
            functions: Vec::new(&env),
            registered_by: admin.clone(),
        };
        client.register_contract(&admin, &cid, &meta);

        let updated = ContractMeta {
            name: String::from_str(&env, "StellarSwap v2"),
            ..meta
        };
        client.update_contract(&admin, &cid, &updated);

        let (_, topics, data) = env.events().all().last().unwrap();
        assert_eq!(
            topics,
            (symbol_short!("update"), cid.clone()).into_val(&env),
        );
        assert_eq!(
            String::try_from_val(&env, &data).unwrap(),
            String::from_str(&env, "StellarSwap v2"),
        );
        assert_eq!(client.get_contract(&cid).name, String::from_str(&env, "StellarSwap v2"));
    }
}
