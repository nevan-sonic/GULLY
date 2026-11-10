# GULLY — Bringing Web3 to Everyday Indian Shops

> **NEAR HACKATHON 2026 Submission**  
> **Dual Track Entry**:  
> - **Track 10 (id: `nfts`)**: *NFTs That Do Something* — Membership NFTs that carry their own wallet/sub-account with dynamic transfer security.  
> - **Track 11 (id: `memecoins`)**: *Memecoins With Mechanics* — Community token powered by real local commerce with onchain "Gully Rush" milestones, immutable parameters, and 100% burn-on-redemption.

[![NEAR Protocol](https://img.shields.io/badge/NEAR-Protocol-black?style=flat&logo=near)](https://near.org)
[![CI Status](https://github.com/nevan-sonic/GULLY/actions/workflows/ci.yml/badge.svg)](https://github.com/nevan-sonic/GULLY/actions)
[![Live Demo](https://img.shields.io/badge/Live_Demo-GitHub_Pages-brightgreen)](https://nevan-sonic.github.io/GULLY/)
[![Track: NFTs](https://img.shields.io/badge/Track-NFTs_That_Do_Something-blue)](https://github.com/nearai/near-hackathon-prompt-examples/blob/main/tracks/10-nfts.md)
[![Track: Memecoins](https://img.shields.io/badge/Track-Memecoins_With_Mechanics-yellow)](https://github.com/nearai/near-hackathon-prompt-examples/blob/main/tracks/11-memecoins.md)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

> 🌐 **Live Interactive App**: [https://nevan-sonic.github.io/GULLY/](https://nevan-sonic.github.io/GULLY/)  
> 📹 **Demo Video Script & X Post Kit**: [DEMO_AND_SUBMISSION.md](DEMO_AND_SUBMISSION.md)

---

## 1. Executive Summary

In India's retail ecosystem, billions of dollars flow through millions of local shops (kiranas, bakeries, salons, tea stalls). However, small merchants lack modern customer retention tools, and centralized loyalty programs lock customers into single-merchant silos while extracting high fees.

**GULLY** transforms neighborhood commerce using two interconnected NEAR innovations:
1. **GULLY Pass (NEP-171)**: A dynamic membership NFT that acts as an autonomous account/vault holding the customer's rewards, visit credentials, and access rights. When transferred, all funds, benefits, and authority transition atomically to the new owner, revoking the previous owner's access.
2. **GULLY Coin (NEP-141)**: A community token with onchain mechanics. Coins are minted exclusively through verified local purchases and burned on discount redemption. A collective milestone engine—**The Gully Rush**—triggers neighborhood-wide bonus events when local shopping activity surges.

---

## 2. Track 10: "NFTs That Do Something" — Architecture & Threat Model

### How Control Passes on Transfer and What Stops the Old Owner
Track 10 requires an NEP-171 membership NFT that carries its own wallet/sub-account, ensuring that selling the NFT transfers complete control of the wallet, balances, and access rights without backdoor access.

```
       [ Customer Wallet ]
              │
              ▼ (Calls transfer)
    ┌────────────────────┐
    │  NEP-171 Contract  │ ──► [Locked State during Transfer]
    └─────────┬──────────┘
              │ (Cross-Contract Assertion)
              ▼
    ┌────────────────────┐
    │ GULLY Pass Vault   │ ──► Dynamic Owner Check:
    │ (Keyless Contract) │     assert_eq!(caller, nft.owner_of(token_id))
    └────────────────────┘     [Old keys cannot exist; zero stored user keys]
```

#### Transfer Security Guarantees
1. **Keyless Smart Vault (No Dangling Access Keys)**:
   The sub-account (`pass-xxx.gully.near`) operates as a keyless smart contract. It holds **zero user `FullAccess` or `FunctionCall` keys**. All administrative operations (`withdraw_coins`, `claim_reward`, `redeem_discount`) perform dynamic verification against the NEP-171 contract:
   ```rust
   assert_eq!(
       env::predecessor_account_id(),
       ext_nft::ext(nft_contract_id).nft_token(token_id).owner_id,
       "GULLY: Caller is not the current NFT owner"
   );
   ```
   Because no private keys exist on the vault, an ex-owner has zero cryptographic capability to invoke the vault post-transfer.
2. **Transfer Lock & Atomic In-Flight Protection**:
   When `nft_transfer` or `nft_transfer_call` is triggered:
   - The token state transitions to `LOCKED`.
   - All outgoing operations on the vault panic.
   - Any in-flight promises or payouts verify `current_owner == initiating_owner`. If the owner changed, outgoing funds revert to the new owner.
3. **Pending Promise Gate**:
   The vault maintains a `pending_promises: u32` counter. The NFT contract will reject transfers (`assert_eq!(pending_promises, 0)`) while asynchronous operations are outstanding, preventing scheduled siphon attacks.

---

## 3. Track 11: "Memecoins With Mechanics" — Gully Rush & Token Dynamics

### Onchain Mechanics (Not in a Centralized Backend)
* **Purchase-Driven Minting**: 10 GULLY Coins are minted per verified ₹200+ purchase at participating merchants.
* **100% Burn on Redemption**: When a customer redeems 10 coins for a ₹15 discount at a partner shop, all 10 coins are permanently burned onchain via `ft_burn`.
* **The Gully Rush Milestone**:
  - If a neighborhood records **100 verified purchases** across at least **3 merchants** within a rolling **48-hour epoch**, a 48-hour "Gully Rush" activates.
  - Active shoppers receive a +5 coin bonus.
  - Participating merchants' pre-approved special offers activate.
* **Immutable Parameters (Zero Admin Rug Keys)**:
  - All milestone thresholds, mint ratios, and burn mechanics are compile-time constants or immutable storage initialized in `new()`. There are **no admin setters** to alter issuance or rug the community.

### Sybil Resistance & Limits
1. **Daily User Quota**: Maximum 2 eligible mints per customer per merchant in a rolling 24-hour window, with a 2-hour cooldown.
2. **Cryptographic Receipt Nonce**: Every purchase requires a unique hash `hash(merchant_id, receipt_id, timestamp, amount)`. Duplicate receipts are rejected onchain.
3. **Merchant Minting Ceilings**: Each merchant has a hard daily issuance quota based on their verified tier.
4. **Limits (Transparently Stated)**: While cryptographic nonces prevent replay attacks and daily caps throttle spam, collusion between a rogue merchant and multiple dummy accounts is bounded by the merchant's maximum daily mint allowance.

---

## 4. 60-Day Token Supply Simulation

Track 11 requires simulating 60 days of community behavior before finalizing parameters. Run the simulator via:

```bash
python simulate_gully_rush.py
```

### Simulation Results Summary

| Scenario | 60-Day Minted | 60-Day Burned | Burn Ratio | Final Supply | Rush Events Triggered |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **A: Steady Low Adoption (Pilot)** | 20,440 GULLY | 16,414 GULLY | **80.3%** | 4,026 GULLY | 0 events (thresholds safe) |
| **B: High Adoption (Viral Rush)** | 107,080 GULLY | 98,431 GULLY | **91.9%** | 8,649 GULLY | 47 events (high velocity) |
| **C: Sybil Attack Attempt** | 32,550 GULLY | 27,007 GULLY | **83.0%** | 5,543 GULLY | 13 events (rate-limited) |

*Observation*: The 100% burn-on-redemption mechanism maintains a healthy circulating supply under 10,000 GULLY even under viral adoption, preventing hyperinflation while rewarding active shoppers.

---

## 5. Repository Structure

```
├── contracts/
│   ├── gully_pass/         # NEP-171 Membership NFT + Keyless Vault Logic
│   └── gully_coin/         # NEP-141 Token + Gully Rush Milestone State Machine
├── frontend/               # Next.js / React Mobile Web App
│   ├── src/
│   │   ├── components/     # Pass Gated View, Cashier Terminal, Rush Progress
│   │   └── lib/            # NEAR RPC, FastAuth, Relayer Integration
├── simulation/
│   └── simulate_gully_rush.py # Track 11 60-Day Simulation Script
├── tests/                  # Integration and Transfer Path Security Tests
├── .gitignore
├── Cargo.toml
└── README.md
```

---

## 6. Getting Started & Verification

### Run the Simulation
```bash
python simulate_gully_rush.py
```

### Run Tests
```bash
cargo test
# or
npm test
```

---

## 7. License
MIT License. Built for NEAR Hackathon 2026.
