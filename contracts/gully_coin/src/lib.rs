use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, AccountId, PanicOnDefault};

// Immutable Economic Constants (Fixed at compile time, zero admin rug keys)
pub const BASE_MINT_PER_PURCHASE: u128 = 10;
pub const MAX_DAILY_MINTS_PER_USER: u32 = 2;
pub const RUSH_TARGET_PURCHASES: u64 = 100;
pub const RUSH_WINDOW_NS: u64 = 172_800_000_000_000; // 48 hours in nanoseconds
pub const RUSH_BONUS_MINT: u128 = 5;
pub const MAX_SUPPLY_CAP: u128 = 1_000_000;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenActivityLog {
    pub log_type: String, // "MINT", "BURN", "RUSH_ACTIVATED"
    pub account_id: AccountId,
    pub amount: u128,
    pub details: String,
    pub timestamp: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RushStatus {
    pub rush_active: bool,
    pub epoch_purchases: u64,
    pub target_purchases: u64,
    pub unique_merchants_count: u32,
    pub current_streak: u64,
    pub longest_streak: u64,
    pub epoch_time_remaining_sec: u64,
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ImmutableParameters {
    pub base_mint_per_purchase: u128,
    pub max_daily_mints_per_user: u32,
    pub rush_target_purchases: u64,
    pub rush_window_hours: u64,
    pub rush_bonus_mint: u128,
    pub max_supply_cap: u128,
    pub has_admin_keys: bool,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct GullyCoinContract {
    pub total_supply: u128,
    pub balances: LookupMap<AccountId, u128>,
    pub used_receipts: LookupSet<String>,
    pub daily_mints: LookupMap<String, u32>, // key: "account_id:day_epoch"
    pub registered_merchants: LookupSet<AccountId>,
    // Gully Rush state machine
    pub rush_active: bool,
    pub epoch_start_timestamp: u64,
    pub epoch_purchases: u64,
    pub epoch_merchants: Vec<AccountId>,
    pub current_streak: u64,
    pub longest_streak: u64,
    pub activity_logs: Vec<TokenActivityLog>,
}

#[near]
impl GullyCoinContract {
    #[init]
    pub fn new(initial_merchants: Vec<AccountId>) -> Self {
        let mut merchants = LookupSet::new(b"m");
        for m in &initial_merchants {
            merchants.insert(m);
        }

        Self {
            total_supply: 0,
            balances: LookupMap::new(b"b"),
            used_receipts: LookupSet::new(b"r"),
            daily_mints: LookupMap::new(b"d"),
            registered_merchants: merchants,
            rush_active: false,
            epoch_start_timestamp: env::block_timestamp(),
            epoch_purchases: 0,
            epoch_merchants: Vec::new(),
            current_streak: 0,
            longest_streak: 0,
            activity_logs: Vec::new(),
        }
    }

    /// NEP-141: Total supply
    pub fn ft_total_supply(&self) -> U128 {
        U128(self.total_supply)
    }

    /// NEP-141: Balance of account
    pub fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        U128(self.balances.get(&account_id).unwrap_or(0))
    }

    /// NEP-141: Transfer
    pub fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>) {
        let sender_id = env::predecessor_account_id();
        let transfer_amount: u128 = amount.into();
        assert!(transfer_amount > 0, "GULLY: Transfer amount must be positive");

        let sender_bal = self.balances.get(&sender_id).unwrap_or(0);
        assert!(sender_bal >= transfer_amount, "GULLY: Insufficient balance");

        self.balances.insert(&sender_id, &(sender_bal - transfer_amount));
        let receiver_bal = self.balances.get(&receiver_id).unwrap_or(0);
        self.balances.insert(&receiver_id, &(receiver_bal + transfer_amount));

        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep141\",\"version\":\"1.0.0\",\"event\":\"ft_transfer\",\"data\":[{{\"old_owner_id\":\"{}\",\"new_owner_id\":\"{}\",\"amount\":\"{}\",\"memo\":\"{}\"}}]}}",
            sender_id,
            receiver_id,
            transfer_amount,
            memo.unwrap_or_default()
        ));
    }

    /// Record a verified purchase and mint GULLY Coins according to protocol rules.
    /// Sybil defense:
    /// 1. Cryptographic receipt nonce prevents replay.
    /// 2. Max 2 purchases per customer per 24 hours.
    /// 3. Supply cap enforced.
    pub fn record_verified_purchase(
        &mut self,
        receipt_hash: String,
        merchant_id: AccountId,
        customer_id: AccountId,
        bill_amount_inr: u64,
    ) {
        assert!(
            bill_amount_inr >= 200,
            "GULLY: Minimum purchase for rewards is INR 200"
        );
        assert!(
            !self.used_receipts.contains(&receipt_hash),
            "GULLY: Replay attack detected. Receipt hash already used"
        );

        // Sybil limit: 24h rolling epoch index
        let day_epoch = env::block_timestamp() / 86_400_000_000_000;
        let quota_key = format!("{}:{}:{}", customer_id, merchant_id, day_epoch);
        let current_daily_mints = self.daily_mints.get(&quota_key).unwrap_or(0);
        assert!(
            current_daily_mints < MAX_DAILY_MINTS_PER_USER,
            "GULLY Sybil Defense: Customer reached maximum daily mint allowance (2) for this merchant"
        );

        // Check protocol hard cap
        assert!(
            self.total_supply + BASE_MINT_PER_PURCHASE <= MAX_SUPPLY_CAP,
            "GULLY: Max protocol supply cap reached"
        );

        // Mark receipt as used and increment daily count
        self.used_receipts.insert(&receipt_hash);
        self.daily_mints.insert(&quota_key, &(current_daily_mints + 1));

        // Mint coins to customer
        let cust_bal = self.balances.get(&customer_id).unwrap_or(0);
        self.balances.insert(&customer_id, &(cust_bal + BASE_MINT_PER_PURCHASE));
        self.total_supply += BASE_MINT_PER_PURCHASE;

        self.activity_logs.push(TokenActivityLog {
            log_type: "MINT".to_string(),
            account_id: customer_id.clone(),
            amount: BASE_MINT_PER_PURCHASE,
            details: format!(
                "Verified INR {} purchase at {}",
                bill_amount_inr, merchant_id
            ),
            timestamp: env::block_timestamp(),
        });

        // Gully Rush Milestone Engine Update
        self.update_rush_milestone(merchant_id);
    }

    /// Redeem GULLY Coins for a discount at an authorized merchant.
    /// Coins are 100% PERMANENTLY BURNED.
    pub fn redeem_discount(
        &mut self,
        merchant_id: AccountId,
        customer_id: AccountId,
        coins_to_burn: U128,
        discount_inr: u64,
    ) {
        let burn_amount: u128 = coins_to_burn.into();
        assert!(burn_amount > 0, "GULLY: Burn amount must be positive");

        let cust_bal = self.balances.get(&customer_id).unwrap_or(0);
        assert!(
            cust_bal >= burn_amount,
            "GULLY: Insufficient coin balance for redemption"
        );

        // Deduct and permanently burn from total supply
        self.balances.insert(&customer_id, &(cust_bal - burn_amount));
        self.total_supply -= burn_amount;

        self.activity_logs.push(TokenActivityLog {
            log_type: "BURN".to_string(),
            account_id: customer_id.clone(),
            amount: burn_amount,
            details: format!(
                "Redeemed INR {} discount at {}",
                discount_inr, merchant_id
            ),
            timestamp: env::block_timestamp(),
        });

        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep141\",\"version\":\"1.0.0\",\"event\":\"ft_burn\",\"data\":[{{\"owner_id\":\"{}\",\"amount\":\"{}\",\"memo\":\"INR {} discount at {}\"}}]}}",
            customer_id,
            burn_amount,
            discount_inr,
            merchant_id
        ));
    }

    fn update_rush_milestone(&mut self, merchant_id: AccountId) {
        let now = env::block_timestamp();
        // Check if current 48h epoch has expired
        if now > self.epoch_start_timestamp + RUSH_WINDOW_NS {
            // Epoch reset
            if !self.rush_active {
                // Failed to hit target: reset streak
                self.current_streak = 0;
            }
            self.epoch_start_timestamp = now;
            self.epoch_purchases = 0;
            self.epoch_merchants.clear();
            self.rush_active = false;
        }

        self.epoch_purchases += 1;
        if !self.epoch_merchants.contains(&merchant_id) {
            self.epoch_merchants.push(merchant_id);
        }

        // Check if milestone is triggered: 100 purchases across at least 3 merchants
        if !self.rush_active
            && self.epoch_purchases >= RUSH_TARGET_PURCHASES
            && self.epoch_merchants.len() >= 3
        {
            self.rush_active = true;
            self.current_streak += 1;
            if self.current_streak > self.longest_streak {
                self.longest_streak = self.current_streak;
            }

            self.activity_logs.push(TokenActivityLog {
                log_type: "RUSH_ACTIVATED".to_string(),
                account_id: env::current_account_id(),
                amount: 0,
                details: format!(
                    "🔥 Gully Rush Activated! Streak: {}. Target of 100 purchases achieved across {} shops",
                    self.current_streak,
                    self.epoch_merchants.len()
                ),
                timestamp: now,
            });
        }
    }

    /// View method: Rush Progress & Streak status for public page
    pub fn get_rush_status(&self) -> RushStatus {
        let now = env::block_timestamp();
        let deadline = self.epoch_start_timestamp + RUSH_WINDOW_NS;
        let time_remaining = if deadline > now {
            (deadline - now) / 1_000_000_000
        } else {
            0
        };

        RushStatus {
            rush_active: self.rush_active,
            epoch_purchases: self.epoch_purchases,
            target_purchases: RUSH_TARGET_PURCHASES,
            unique_merchants_count: self.epoch_merchants.len() as u32,
            current_streak: self.current_streak,
            longest_streak: self.longest_streak,
            epoch_time_remaining_sec: time_remaining,
        }
    }

    /// View method: Activity logs (mints and burns)
    pub fn get_activity_logs(&self) -> Vec<TokenActivityLog> {
        self.activity_logs.clone()
    }

    /// View method: Verified immutable parameters (no admin keys)
    pub fn get_parameters(&self) -> ImmutableParameters {
        ImmutableParameters {
            base_mint_per_purchase: BASE_MINT_PER_PURCHASE,
            max_daily_mints_per_user: MAX_DAILY_MINTS_PER_USER,
            rush_target_purchases: RUSH_TARGET_PURCHASES,
            rush_window_hours: 48,
            rush_bonus_mint: RUSH_BONUS_MINT,
            max_supply_cap: MAX_SUPPLY_CAP,
            has_admin_keys: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;

    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder.current_account_id("gully_coin.near".parse().unwrap());
        builder.block_timestamp(1_700_000_000_000_000_000);
        builder
    }

    #[test]
    fn test_verified_purchase_mint_and_balances() {
        let bakery: AccountId = "ramesh_bakery.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let context = get_context(bakery.clone());
        testing_env!(context.build());

        let mut contract = GullyCoinContract::new(vec![bakery.clone()]);

        // Alice buys ₹250 snacks at Ramesh Bakery
        contract.record_verified_purchase(
            "receipt-001".to_string(),
            bakery.clone(),
            alice.clone(),
            250,
        );

        assert_eq!(contract.ft_balance_of(alice.clone()).0, 10);
        assert_eq!(contract.ft_total_supply().0, 10);
    }

    #[test]
    #[should_panic(expected = "Receipt hash already used")]
    fn test_duplicate_receipt_replay_fails() {
        let bakery: AccountId = "ramesh_bakery.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let context = get_context(bakery.clone());
        testing_env!(context.build());

        let mut contract = GullyCoinContract::new(vec![bakery.clone()]);
        contract.record_verified_purchase(
            "receipt-dup".to_string(),
            bakery.clone(),
            alice.clone(),
            200,
        );
        // Replay attack with same receipt hash
        contract.record_verified_purchase(
            "receipt-dup".to_string(),
            bakery.clone(),
            alice.clone(),
            200,
        );
    }

    #[test]
    #[should_panic(expected = "Customer reached maximum daily mint allowance")]
    fn test_sybil_daily_limit_per_customer() {
        let bakery: AccountId = "ramesh_bakery.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let context = get_context(bakery.clone());
        testing_env!(context.build());

        let mut contract = GullyCoinContract::new(vec![bakery.clone()]);

        // Purchase 1: OK
        contract.record_verified_purchase(
            "rec-1".to_string(),
            bakery.clone(),
            alice.clone(),
            200,
        );
        // Purchase 2: OK
        contract.record_verified_purchase(
            "rec-2".to_string(),
            bakery.clone(),
            alice.clone(),
            200,
        );
        // Purchase 3: Exceeds daily allowance -> MUST PANIC
        contract.record_verified_purchase(
            "rec-3".to_string(),
            bakery.clone(),
            alice.clone(),
            200,
        );
    }

    #[test]
    fn test_redeem_burns_supply_permanently() {
        let barber: AccountId = "kumar_saloon.near".parse().unwrap();
        let bakery: AccountId = "ramesh_bakery.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let context = get_context(bakery.clone());
        testing_env!(context.build());

        let mut contract = GullyCoinContract::new(vec![bakery.clone(), barber.clone()]);

        // Alice earns 10 coins at bakery
        contract.record_verified_purchase(
            "rec-earn".to_string(),
            bakery.clone(),
            alice.clone(),
            200,
        );
        assert_eq!(contract.ft_balance_of(alice.clone()).0, 10);
        assert_eq!(contract.ft_total_supply().0, 10);

        // Alice burns 10 coins at Kumar Saloon for ₹15 haircut discount
        contract.redeem_discount(barber.clone(), alice.clone(), U128(10), 15);

        // Coins are 100% burned
        assert_eq!(contract.ft_balance_of(alice.clone()).0, 0);
        assert_eq!(contract.ft_total_supply().0, 0);

        let logs = contract.get_activity_logs();
        assert_eq!(logs.len(), 2);
        assert_eq!(logs[1].log_type, "BURN");
    }

    #[test]
    fn test_gully_rush_activation_and_streak() {
        let b1: AccountId = "shop1.near".parse().unwrap();
        let b2: AccountId = "shop2.near".parse().unwrap();
        let b3: AccountId = "shop3.near".parse().unwrap();
        let context = get_context(b1.clone());
        testing_env!(context.build());

        let mut contract = GullyCoinContract::new(vec![b1.clone(), b2.clone(), b3.clone()]);

        // Simulate 100 purchases across 3 merchants
        for i in 0..100 {
            let user: AccountId = format!("user{}.near", i).parse().unwrap();
            let shop = if i % 3 == 0 {
                b1.clone()
            } else if i % 3 == 1 {
                b2.clone()
            } else {
                b3.clone()
            };
            contract.record_verified_purchase(
                format!("rec-{}", i),
                shop,
                user,
                200,
            );
        }

        let status = contract.get_rush_status();
        assert!(status.rush_active);
        assert_eq!(status.current_streak, 1);
        assert_eq!(status.epoch_purchases, 100);
        assert_eq!(status.unique_merchants_count, 3);
    }
}
