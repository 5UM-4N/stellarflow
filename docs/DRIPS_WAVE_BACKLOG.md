# 🌊 StellarFlow — Drips Wave Contributor Sprint Backlog (15 Scoped Issues)

This document defines **15 self-contained, contributor-workable GitHub Issues** across the three Drips Wave complexity tiers (`Trivial 100 pts`, `Medium 150 pts`, `High 200 pts`).

To publish all 15 issues directly to GitHub with their `Stellar Wave` and complexity labels, run:

```bash
GH_TOKEN="ghp_..." node scripts/seed-wave-issues.mjs
```

---

## Tier 1: Trivial Issues (`100 Points` · `good first issue` · `Stellar Wave`)

1. **`[Wave 100] Add unit tests for BNB 18-decimal to Stellar 7-decimal boundary values in stxPegBridge.ts`**
   - **Target Files:** `frontend/src/services/stxPegBridge.ts`, `frontend/src/services/stxPegBridge.test.ts`
   - **Acceptance Criteria:** Test 1-stroop minimum (`100_000_000_000n`), sub-stroop dust truncation, and `i128` max boundary values with 100% branch coverage.
2. **`[Wave 100] Add CLI subcommand 'stellarflow peg-quote' for BNB-to-Stellar STX 1:1 conversion`**
   - **Target Files:** `bin/stellarflow.js`, `scripts/cli.test.mjs`
   - **Acceptance Criteria:** Support `stellarflow peg-quote --bnb18 <amount> --recipient <G...|C...>`, output 7-decimal Stellar STX, dust remainder, and 32-byte recipient payload, plus automated test in `scripts/cli.test.mjs`.
3. **`[Wave 100] Display SAC token decimals and locked-issuer badge in ContractWindow`**
   - **Target Files:** `frontend/src/components/os/ContractWindow.tsx`
   - **Acceptance Criteria:** Render the 1:1 BNB-Pegged `STX` SAC derivation and `masterWeight: 0` locked-issuer invariant status in `ContractWindow.tsx` with component test.
4. **`[Wave 100] Add Zod schema validation for 0x-prefixed 20-byte BNB Chain addresses in backend API`**
   - **Target Files:** `backend/src/validation/schemas.ts`, `backend/src/validation/schemas.test.ts`
   - **Acceptance Criteria:** Add and test `bnbAddressSchema` rejecting non-hex or wrong-length addresses with descriptive validation errors.
5. **`[Wave 100] Add SEP-0001 stellar.toml validator check to preflight script`**
   - **Target Files:** `frontend/public/.well-known/stellar.toml`, `scripts/check-frontend-ci.mjs`
   - **Acceptance Criteria:** Verify that `stellar.toml` contains valid `[[CONTRACTS]]` and `[[CURRENCIES]]` (`USDC`, `XLM`, `STX`) sections during CI preflight.

---

## Tier 2: Medium Issues (`150 Points` · `Medium` · `Stellar Wave`)

6. **`[Wave 150] Enforce exact env.auths() authorization tree assertions on PegBridgeContract::mint_pegged_from_bnb`**
   - **Target Files:** `contracts/src/peg_bridge.rs`, `contracts/src/peg_bridge_test.rs`
   - **Acceptance Criteria:** Assert the exact `(relayer, InvokeContractArgs)` authorization tuple via `env.auths()` in `peg_bridge_test.rs` and verify unauthorized callers return `ContractError::Unauthorized`.
7. **`[Wave 150] Add ChangeTrust pre-flight check before initiating STX stream to G... recipient`**
   - **Target Files:** `frontend/src/services/stxPegBridge.ts`, `frontend/src/components/CreateStreamForm.tsx`
   - **Acceptance Criteria:** Check whether a classic `G...` recipient has an active `STX` trustline and surface a 1-click `buildStxTrustlineTxXdr` prompt if missing.
8. **`[Wave 150] Index PegMintedEvent and PegBurnedForBnbEvent in backend Soroban RPC indexer`**
   - **Target Files:** `backend/src/services/indexer.ts`, `backend/src/services/eventHistory.ts`
   - **Acceptance Criteria:** Decode `("Peg", "Minted")` and `("Peg", "Burned")` contract events in `indexer.ts` and expose `/api/peg/events` with unit tests.
9. **`[Wave 150] Add proactive TTL extension threshold tests using env.storage().get_ttl()`**
   - **Target Files:** `contracts/src/lib.rs`, `contracts/src/peg_bridge.rs`, `contracts/src/test.rs`
   - **Acceptance Criteria:** Advance ledger sequence in `testutils` and assert `extend_ttl` bumps both `instance()` and `persistent()` stream and nullifier entries to `>= 518_400` ledgers.
10. **`[Wave 150] Add batch claim simulation endpoint for multi-stream recipients`**
    - **Target Files:** `contracts/src/router.rs`, `frontend/src/services/soroban.ts`
    - **Acceptance Criteria:** Wire `get_claimable_batch` across up to 20 stream IDs with RPC simulation and unit tests.

---

## Tier 3: High Issues (`200 Points` · `High` · `Stellar Wave`)

11. **`[Wave 200] Upgrade smart contracts from legacy env.events().publish to #[contractevent] structs (CAP-0067)`**
    - **Target Files:** `contracts/src/lib.rs`, `contracts/src/router.rs`, `contracts/src/peg_bridge.rs`
    - **Acceptance Criteria:** Migrate event emission to typed `#[contractevent]` structs while preserving indexer ABI compatibility and updating all event tests.
12. **`[Wave 200] Migrate unguarded initialize() entrypoints to atomic __constructor (CAP-0058)`**
    - **Target Files:** `contracts/src/lib.rs`, `contracts/src/router.rs`, `contracts/src/peg_bridge.rs`
    - **Acceptance Criteria:** Implement `__constructor(env: Env, ...)` to eliminate front-running windows at deploy time and update test fixtures.
13. **`[Wave 200] Implement cargo-fuzz target for PegBridgeContract 18-to-7 decimal conservation invariant`**
    - **Target Files:** `contracts/fuzz/fuzz_targets/fuzz_stream_lifecycle.rs`, `contracts/src/peg_bridge.rs`
    - **Acceptance Criteria:** Prove via property/fuzz testing that for any `amount_bnb_18 > 0`, `amount_stellar_7 * 10^11 + dust_remainder_18 == amount_bnb_18` and `total_minted_7 - total_burned_7 == active_circulating_peg_7`.
14. **`[Wave 200] Integrate OpenZeppelin Relayer plugin for gasless zero-XLM recipient claims`**
    - **Target Files:** `frontend/src/services/feeBump.ts`, `backend/src/index.ts`
    - **Acceptance Criteria:** Replace local fee-bump simulation with a working sponsored submission endpoint and integration tests.
15. **`[Wave 200] Implement per-epoch cross-chain outflow rate limiter (set_flow_limit) on PegBridgeContract`**
    - **Target Files:** `contracts/src/peg_bridge.rs`, `contracts/src/peg_bridge_test.rs`
    - **Acceptance Criteria:** Add `set_flow_limit`, `flow_out_amount`, and `flow_in_amount` per 24h ledger epoch (following the Axelar ITS flow-limit pattern) so transfers exceeding the epoch cap fail closed.
