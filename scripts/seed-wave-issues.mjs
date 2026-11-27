#!/usr/bin/env node
/**
 * Seeds the 15 curated Drips Wave contributor issues into GitHub with
 * `Stellar Wave` and complexity point labels (`100`, `150`, `200`).
 *
 * Usage:
 *   GH_TOKEN="ghp_..." node scripts/seed-wave-issues.mjs [--dry-run]
 */

const REPO = process.env.GH_REPO || 'Steller-escaprt/stellarflow';
const TOKEN = process.env.GH_TOKEN || process.env.GITHUB_TOKEN || '';
const DRY_RUN = process.argv.includes('--dry-run') || !TOKEN;

export const WAVE_ISSUES = [
  {
    title: '[Wave 100] Add unit tests for BNB 18-decimal to Stellar 7-decimal boundary values in stxPegBridge.ts',
    labels: ['Stellar Wave', 'good first issue', 'Trivial', 'points: 100', 'testing'],
    body: `### Drips Wave Complexity Tier\n**Trivial (100 points)**\n\n### Context & Target Files\n- \`frontend/src/services/stxPegBridge.ts\`\n- \`frontend/src/services/stxPegBridge.test.ts\`\n\n### Acceptance Criteria\n- [ ] Test 1-stroop minimum (\`100_000_000_000n\`), sub-stroop dust truncation, and \`i128\` max boundary values\n- [ ] Verify \`stellarRecipientToBytes32\` rejects invalid checksums and muxed \`M...\` addresses\n- [ ] All Vitest suites pass (\`cd frontend && npm test -- --run\`)`,
  },
  {
    title: "[Wave 100] Add CLI subcommand 'stellarflow peg-quote' for BNB-to-Stellar STX 1:1 conversion",
    labels: ['Stellar Wave', 'good first issue', 'Trivial', 'points: 100', 'cli'],
    body: `### Drips Wave Complexity Tier\n**Trivial (100 points)**\n\n### Context & Target Files\n- \`bin/stellarflow.js\`\n- \`scripts/cli.test.mjs\`\n\n### Acceptance Criteria\n- [ ] Support \`stellarflow peg-quote --bnb18 <amount> --recipient <G...|C...>\`\n- [ ] Output 7-decimal Stellar STX amount, 18-decimal dust remainder, and 32-byte hex recipient payload\n- [ ] Add automated test in \`scripts/cli.test.mjs\``,
  },
  {
    title: '[Wave 100] Display SAC token decimals and locked-issuer badge in ContractWindow',
    labels: ['Stellar Wave', 'good first issue', 'Trivial', 'points: 100', 'frontend'],
    body: `### Drips Wave Complexity Tier\n**Trivial (100 points)**\n\n### Context & Target Files\n- \`frontend/src/components/os/ContractWindow.tsx\`\n\n### Acceptance Criteria\n- [ ] Render the 1:1 BNB-Pegged \`STX\` SAC address and \`masterWeight: 0\` locked-issuer invariant status\n- [ ] Add component unit test verifying rendered SAC metadata`,
  },
  {
    title: '[Wave 100] Add Zod schema validation for 0x-prefixed 20-byte BNB Chain addresses in backend API',
    labels: ['Stellar Wave', 'good first issue', 'Trivial', 'points: 100', 'backend'],
    body: `### Drips Wave Complexity Tier\n**Trivial (100 points)**\n\n### Context & Target Files\n- \`backend/src/validation/schemas.ts\`\n- \`backend/src/validation/schemas.test.ts\`\n\n### Acceptance Criteria\n- [ ] Export \`bnbAddressSchema\` validating \`/^0x[0-9a-fA-F]{40}$/\`\n- [ ] Add unit tests covering valid addresses, malformed hex, and wrong-length strings`,
  },
  {
    title: '[Wave 100] Add SEP-0001 stellar.toml validator check to frontend CI preflight script',
    labels: ['Stellar Wave', 'good first issue', 'Trivial', 'points: 100', 'devops'],
    body: `### Drips Wave Complexity Tier\n**Trivial (100 points)**\n\n### Context & Target Files\n- \`frontend/public/.well-known/stellar.toml\`\n- \`scripts/check-frontend-ci.mjs\`\n\n### Acceptance Criteria\n- [ ] Verify \`stellar.toml\` contains \`[[CONTRACTS]]\` and \`[[CURRENCIES]]\` entries for \`USDC\`, \`XLM\`, and \`STX\`\n- [ ] Add unit test in \`scripts/check-frontend-ci.test.mjs\``,
  },
  {
    title: '[Wave 150] Enforce exact env.auths() authorization tree assertions on PegBridgeContract::mint_pegged_from_bnb',
    labels: ['Stellar Wave', 'Medium', 'points: 150', 'smart-contract', 'security'],
    body: `### Drips Wave Complexity Tier\n**Medium (150 points)**\n\n### Context & Target Files\n- \`contracts/src/peg_bridge.rs\`\n- \`contracts/src/peg_bridge_test.rs\`\n\n### Acceptance Criteria\n- [ ] Assert the exact \`env.auths()\` authorization tree for \`mint_pegged_from_bnb\` and \`burn_for_bnb_redemption\`\n- [ ] Verify unauthorized relayer addresses return \`ContractError::Unauthorized\``,
  },
  {
    title: '[Wave 150] Add ChangeTrust pre-flight check before initiating STX stream to G... recipient',
    labels: ['Stellar Wave', 'Medium', 'points: 150', 'frontend'],
    body: `### Drips Wave Complexity Tier\n**Medium (150 points)**\n\n### Context & Target Files\n- \`frontend/src/services/stxPegBridge.ts\`\n- \`frontend/src/components/CreateStreamForm.tsx\`\n\n### Acceptance Criteria\n- [ ] Query account trustlines when \`STX\` is selected as the stream asset\n- [ ] Prompt the user with a 1-click \`buildStxTrustlineTxXdr\` action if the recipient lacks the \`STX\` trustline`,
  },
  {
    title: '[Wave 150] Index PegMintedEvent and PegBurnedForBnbEvent in backend Soroban RPC indexer',
    labels: ['Stellar Wave', 'Medium', 'points: 150', 'backend'],
    body: `### Drips Wave Complexity Tier\n**Medium (150 points)**\n\n### Context & Target Files\n- \`backend/src/services/indexer.ts\`\n- \`backend/src/services/eventHistory.ts\`\n\n### Acceptance Criteria\n- [ ] Decode \`("Peg", "Minted")\` and \`("Peg", "Burned")\` Soroban contract events\n- [ ] Persist peg events and add integration tests`,
  },
  {
    title: '[Wave 150] Add proactive TTL extension threshold tests using env.storage()',
    labels: ['Stellar Wave', 'Medium', 'points: 150', 'smart-contract', 'testing'],
    body: `### Drips Wave Complexity Tier\n**Medium (150 points)**\n\n### Context & Target Files\n- \`contracts/src/lib.rs\`\n- \`contracts/src/peg_bridge.rs\`\n- \`contracts/src/test.rs\`\n\n### Acceptance Criteria\n- [ ] Advance ledger sequence in \`testutils\` and assert \`extend_ttl\` bumps \`instance()\` and \`persistent()\` entries to \`>= 518_400\` ledgers`,
  },
  {
    title: '[Wave 150] Add batch claim simulation helper and UI summary for multi-stream recipients',
    labels: ['Stellar Wave', 'Medium', 'points: 150', 'frontend'],
    body: `### Drips Wave Complexity Tier\n**Medium (150 points)**\n\n### Context & Target Files\n- \`frontend/src/services/soroban.ts\`\n- \`frontend/src/components/RecipientDashboard.tsx\`\n\n### Acceptance Criteria\n- [ ] Batch query up to 20 stream IDs via \`getClaimableBatch\` and display aggregate claimable balance with unit tests`,
  },
  {
    title: '[Wave 200] Upgrade smart contracts from legacy env.events().publish to #[contractevent] structs (CAP-0067)',
    labels: ['Stellar Wave', 'High', 'points: 200', 'smart-contract'],
    body: `### Drips Wave Complexity Tier\n**High (200 points)**\n\n### Context & Target Files\n- \`contracts/src/lib.rs\`\n- \`contracts/src/router.rs\`\n- \`contracts/src/peg_bridge.rs\`\n\n### Acceptance Criteria\n- [ ] Migrate contract events to typed \`#[contractevent]\` structs while maintaining indexer compatibility and passing all contract tests`,
  },
  {
    title: '[Wave 200] Migrate unguarded initialize() entrypoints to atomic __constructor (CAP-0058)',
    labels: ['Stellar Wave', 'High', 'points: 200', 'smart-contract', 'security'],
    body: `### Drips Wave Complexity Tier\n**High (200 points)**\n\n### Context & Target Files\n- \`contracts/src/lib.rs\`\n- \`contracts/src/router.rs\`\n- \`contracts/src/peg_bridge.rs\`\n\n### Acceptance Criteria\n- [ ] Implement atomic \`__constructor\` deployment initialization and update all contract test fixtures`,
  },
  {
    title: '[Wave 200] Implement cargo-fuzz target for PegBridgeContract 18-to-7 decimal conservation invariant',
    labels: ['Stellar Wave', 'High', 'points: 200', 'smart-contract', 'security'],
    body: `### Drips Wave Complexity Tier\n**High (200 points)**\n\n### Context & Target Files\n- \`contracts/fuzz/fuzz_targets/fuzz_stream_lifecycle.rs\`\n- \`contracts/src/peg_bridge.rs\`\n\n### Acceptance Criteria\n- [ ] Prove via property/fuzz testing that \`amount_stellar_7 * 10^11 + dust_remainder_18 == amount_bnb_18\` and \`total_minted_7 - total_burned_7 == active_circulating_peg_7\``,
  },
  {
    title: '[Wave 200] Integrate OpenZeppelin Relayer plugin for gasless zero-XLM recipient claims',
    labels: ['Stellar Wave', 'High', 'points: 200', 'backend', 'frontend'],
    body: `### Drips Wave Complexity Tier\n**High (200 points)**\n\n### Context & Target Files\n- \`frontend/src/services/feeBump.ts\`\n- \`backend/src/index.ts\`\n\n### Acceptance Criteria\n- [ ] Wire a sponsored fee-bump submission endpoint using OpenZeppelin Relayer / Stellar Channels and add integration tests`,
  },
  {
    title: '[Wave 200] Implement per-epoch cross-chain outflow rate limiter (set_flow_limit) on PegBridgeContract',
    labels: ['Stellar Wave', 'High', 'points: 200', 'smart-contract', 'security'],
    body: `### Drips Wave Complexity Tier\n**High (200 points)**\n\n### Context & Target Files\n- \`contracts/src/peg_bridge.rs\`\n- \`contracts/src/peg_bridge_test.rs\`\n\n### Acceptance Criteria\n- [ ] Add \`set_flow_limit\`, \`flow_out_amount\`, and \`flow_in_amount\` per 24h ledger epoch so transfers exceeding the epoch cap fail closed`,
  },
];

async function run() {
  console.log(`=== StellarFlow Drips Wave Issue Seeder (${WAVE_ISSUES.length} issues) ===`);
  if (DRY_RUN) {
    for (const [idx, issue] of WAVE_ISSUES.entries()) {
      console.log(`#${idx + 1}: ${issue.title} [${issue.labels.join(', ')}]`);
    }
    console.log('\nDry run complete. Set GH_TOKEN="ghp_..." to create these 15 issues on GitHub.');
    return;
  }

  for (const issue of WAVE_ISSUES) {
    const res = await fetch(`https://api.github.com/repos/${REPO}/issues`, {
      method: 'POST',
      headers: {
        Authorization: `Bearer ${TOKEN}`,
        Accept: 'application/vnd.github+json',
        'User-Agent': 'StellarFlow-Wave-Seeder',
      },
      body: JSON.stringify(issue),
    });
    if (!res.ok) {
      const text = await res.text();
      console.error(`Failed to create "${issue.title}": HTTP ${res.status} ${text}`);
    } else {
      const created = await res.json();
      console.log(`Created Issue #${created.number}: ${created.title}`);
    }
  }
}

if (import.meta.url === `file://${process.argv[1]}` || process.argv[1]?.endsWith('seed-wave-issues.mjs')) {
  run();
}
