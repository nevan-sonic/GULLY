# GULLY — Hackathon Submission & Demo Kit

**NEAR Hackathon 2026** (Nov 1–14, 2026)  
**Dual Track Entry**:  
- **Track 10 (id: `nfts`)**: *NFTs That Do Something*  
- **Track 11 (id: `memecoins`)**: *Memecoins With Mechanics*  

---

## 1. Submission Form Details

| Field | Submission Value |
| :--- | :--- |
| **Project Name** | **GULLY — Bringing Web3 to Everyday Indian Shops** |
| **Tagline** | One Pass. One Coin. Every Neighborhood. Programmable local commerce powered by NEAR. |
| **Selected Tracks** | Track 10: NFTs That Do Something (`nfts`), Track 11: Memecoins With Mechanics (`memecoins`) |
| **GitHub Repository** | `https://github.com/nevan-sonic/GULLY` |
| **Live Interactive Demo** | `https://nevan-sonic.github.io/GULLY/` |
| **Tech Stack** | NEAR Protocol (Rust Smart Contracts), NEP-171, NEP-141, Python Simulation, Tailwind / JS Web Terminal |
| **Key Standards** | NEP-171 (Dynamic NFT), NEP-141 (Fungible Token), NEP-145 (Storage), NEP-366 (Delegated / Relayed) |

---

## 2. 3-Minute Video Demo Script

### [0:00 - 0:45] The Hook & The Retail Problem
* **Visual**: Show a vibrant street scene of Indian local businesses (Bakery, Saloon, Kirana).
* **Narrator**: 
  > *"Every day, billions of rupees flow through millions of local shops in India. But small merchants have no collaborative retention tools, and paper coupons never leave the counter. What if shopping at your neighborhood bakery unlocked a haircut discount at the barbershop down the street—without either shop surrendering control to a centralized middleman, and without customers ever having to understand crypto?*
  > 
  > *Meet GULLY: a decentralized neighborhood loyalty economy built on NEAR Protocol, combining dynamic NFTs with self-governing community memecoins."*

### [0:45 - 1:45] Track 10: NFTs That Do Something (GULLY Pass + Keyless Vault)
* **Visual**: Open `https://nevan-sonic.github.io/GULLY/` on the **🎫 My GULLY Pass** tab.
* **Narrator**: 
  > *"First, Track 10: NFTs That Do Something. The GULLY Pass isn't a JPEG. It is an NEP-171 membership NFT that acts as an autonomous account vault—`pass-101.gully.near`. It directly holds the customer's coin balances, shop visit badges, and cross-shop discount rights.*
  > 
  > *Notice the gated holder check. Currently, `alice.near` is logged in. She can view her 25 GULLY coins, her shop badges, and her full history of what this pass earned and paid out.*
  > 
  > *Now watch the transfer security: When Alice sells or transfers her pass to `bob.near`, the entire vault, balance, and rights transfer atomically. Because our vault is keyless and verifies caller identity against the NFT owner onchain, Alice is instantly locked out with zero retained keys. Switching to Bob, he now has full control."*

### [1:45 - 2:30] Track 11: Memecoins With Mechanics (Gully Rush & Burn Engine)
* **Visual**: Click on the **⚡ Gully Rush & Token** tab, then the **🏪 Merchant Terminal** tab.
* **Narrator**: 
  > *"Second, Track 11: Memecoins With Mechanics. GULLY Coins are NEP-141 tokens governed entirely by onchain state machines, with zero admin rug keys.*
  > 
  > *Coins cannot be bought from an open DEX; they are minted exclusively through verified local purchases. When customer Rahul spends ₹220 at Ramesh Bakery, the cashier terminal records the purchase onchain. 10 GULLY coins are minted into his pass vault.*
  > 
  > *Next, Rahul visits Kumar Saloon. Kumar offers ₹15 off a haircut in exchange for 10 coins. When redeemed, those 10 coins are 100% permanently burned onchain via `ft_burn`—deflating the circulating supply.*
  > 
  > *When our neighborhood records 100 verified purchases across 3 merchants within 48 hours, the smart contract activates 'The Gully Rush'—triggering participant bonuses and increasing the community streak counter."*

### [2:30 - 3:00] Sybil Defense, Simulation & Vision
* **Visual**: Show `simulate_gully_rush.py` terminal output and GitHub Actions CI badge.
* **Narrator**: 
  > *"Our smart contracts enforce cryptographic receipt nonces and a strict 2-mint daily cap to prevent collusion. We simulated 60 days of community shopping behavior, proving our 100% burn-on-redemption mechanism prevents hyperinflation under all scenarios.*
  > 
  > *GULLY brings Web3 utility to everyday commerce: Savings for shoppers, sales for shopkeepers, and immutable rules enforced by NEAR. Check out our public repo on GitHub!"*

---

## 3. Public Post on X (Twitter) Template

*(Required by NEAR Hackathon submission rules)*

```
🚨 Built for @NEARProtocol Hackathon 2026! 🇮🇳⚡

Introducing GULLY — Bringing Web3 to Everyday Indian Shops.

One Pass. One Coin. Every Neighborhood. 

Entered in 2 tracks:
🏷️ Track 10: NFTs That Do Something (id: nfts)
🪙 Track 11: Memecoins With Mechanics (id: memecoins)

🧵👇 [1/5]

1/ In India, millions of local shops (kiranas, bakeries, salons) struggle with customer retention. 

With GULLY, buying ₹200 snacks at Ramesh Bakery earns GULLY Coins, unlocking a ₹15 discount at Kumar Saloon. 

No crypto confusion. Just UPI + QR code.

2/ Track 10: GULLY Pass (NEP-171)
Not a JPEG. An NFT that is an autonomous keyless account/vault holding tokens & badges. 

When transferred, the wallet and rights move with it. Zero backdoor keys can persist. 

3/ Track 11: GULLY Coin (NEP-141)
A community memecoin with mechanics:
🔥 100% burned on redemption
⚡ "Gully Rush" neighborhood milestone engine & streaks
🔒 100% immutable parameters — ZERO admin rug keys
🛡️ Strict Sybil defense with receipt nonces

4/ We modeled 60 days of community shopping behavior across 3 scenarios (pilot, viral rush, and collusion attacks) in Python. 

Burn ratio exceeds 80–91%, maintaining deflationary circulating supply.

5/ 💻 Public Code: https://github.com/nevan-sonic/GULLY
🌐 Live Interactive App: https://nevan-sonic.github.io/GULLY/
🧪 13/13 Automated Tests Passing!

Built with ❤️ for #NEARHackathon2026 @near_ai
```
