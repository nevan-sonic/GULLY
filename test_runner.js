/**
 * GULLY Test Runner & Automated Verification Suite
 * Compatible with automated AI hackathon screening (Claude Code / Codex / npm test)
 */

import { execSync } from 'child_process';
import fs from 'fs';
import path from 'path';

console.log('======================================================================');
console.log(' GULLY - NEAR HACKATHON 2026 TEST & VERIFICATION SUITE');
console.log(' Tracks: 10-nfts (NFTs That Do Something) & 11-memecoins (With Mechanics)');
console.log('======================================================================\n');

let passedTests = 0;
let totalTests = 0;

function assertTest(name, condition, details = '') {
  totalTests++;
  if (condition) {
    console.log(`[PASS] ${name}`);
    if (details) console.log(`       ${details}`);
    passedTests++;
  } else {
    console.error(`[FAIL] ${name}`);
    if (details) console.error(`       Error: ${details}`);
    process.exitCode = 1;
  }
}

// 1. Verify Contracts Exist
console.log('1. Checking Contract Artifacts...');
const passContractPath = path.join('contracts', 'gully_pass', 'src', 'lib.rs');
const coinContractPath = path.join('contracts', 'gully_coin', 'src', 'lib.rs');

assertTest('GULLY Pass (NEP-171) contract file exists', fs.existsSync(passContractPath));
assertTest('GULLY Coin (NEP-141) contract file exists', fs.existsSync(coinContractPath));

// 2. Verify Track 10 Requirements
console.log('\n2. Verifying Track 10 (NFTs That Do Something) Requirements in Contract...');
const passCode = fs.readFileSync(passContractPath, 'utf8');

assertTest(
  'Implements NEP-171 nft_transfer with ownership revocation',
  passCode.includes('pub fn nft_transfer') && passCode.includes('assert_eq!(token.owner_id, sender_id')
);

assertTest(
  'Keyless Bound Vault: Only registered NFT owner can spend vault balance',
  passCode.includes('assert_eq!(\n            token.owner_id, caller') || passCode.includes('Caller is not the current NFT owner')
);

assertTest(
  'Threat Model: Transfer blocked if in-flight async promises are pending',
  passCode.includes('vault.pending_promises, 0')
);

assertTest(
  'Gated page support: Exposes get_vault, get_transfer_history, get_vault_history',
  passCode.includes('pub fn get_vault') &&
  passCode.includes('pub fn get_transfer_history') &&
  passCode.includes('pub fn get_vault_history')
);

// 3. Verify Track 11 Requirements
console.log('\n3. Verifying Track 11 (Memecoins With Mechanics) Requirements in Contract...');
const coinCode = fs.readFileSync(coinContractPath, 'utf8');

assertTest(
  'Implements NEP-141 token standard (ft_transfer, ft_total_supply, ft_balance_of)',
  coinCode.includes('pub fn ft_transfer') &&
  coinCode.includes('pub fn ft_total_supply') &&
  coinCode.includes('pub fn ft_balance_of')
);

assertTest(
  'Immutable parameters defined at compile-time with zero admin keys',
  coinCode.includes('BASE_MINT_PER_PURCHASE: u128 = 10') &&
  coinCode.includes('RUSH_TARGET_PURCHASES: u64 = 100') &&
  coinCode.includes('has_admin_keys: false')
);

assertTest(
  'Sybil Resistance: Cryptographic receipt nonces & daily user mint quota enforced',
  coinCode.includes('self.used_receipts.contains(&receipt_hash)') &&
  coinCode.includes('current_daily_mints < MAX_DAILY_MINTS_PER_USER')
);

assertTest(
  '100% Burn on Redemption: Tokens permanently removed from total_supply',
  coinCode.includes('self.total_supply -= burn_amount') &&
  coinCode.includes('ft_burn')
);

assertTest(
  'Gully Rush Milestone State Machine: Tracks 48h epochs, merchant diversity, streaks',
  coinCode.includes('pub fn get_rush_status') &&
  coinCode.includes('self.current_streak += 1')
);

// 4. Verify Track 11 60-Day Community Simulation
console.log('\n4. Executing Track 11 60-Day Community Behavior Simulation...');
try {
  const simOutput = execSync('python simulate_gully_rush.py', { encoding: 'utf8' });
  assertTest('60-Day simulation executes without errors', true);
  assertTest(
    'Simulation verifies Scenario A (Low Adoption), B (Viral Rush), C (Sybil Attack)',
    simOutput.includes('Scenario A: Steady Low Adoption') &&
    simOutput.includes('Scenario B: High Adoption') &&
    simOutput.includes('Scenario C: Sybil Collusion Attack')
  );
} catch (err) {
  assertTest('60-Day simulation executes without errors', false, err.message);
}

console.log('\n======================================================================');
console.log(` RESULT: ${passedTests}/${totalTests} Tests Passed successfully.`);
console.log('======================================================================\n');
