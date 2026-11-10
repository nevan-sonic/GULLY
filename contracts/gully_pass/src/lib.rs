use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near, AccountId, PanicOnDefault, Promise};

pub type TokenId = String;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TokenMetadata {
    pub title: Option<String>,
    pub description: Option<String>,
    pub media: Option<String>,
    pub neighborhood: String,
    pub issued_at: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Token {
    pub token_id: TokenId,
    pub owner_id: AccountId,
    pub metadata: TokenMetadata,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Default)]
#[serde(crate = "near_sdk::serde")]
pub struct VaultAccount {
    pub balance: u128,
    pub pending_promises: u32,
    pub shop_badges: Vec<String>,
    pub total_earned: u128,
    pub total_paid_out: u128,
    pub visits_count: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TransferRecord {
    pub from: AccountId,
    pub to: AccountId,
    pub timestamp: u64,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct HistoryEntry {
    pub entry_type: String, // "EARNED", "PAID_OUT", "BADGE_AWARDED"
    pub amount: u128,
    pub description: String,
    pub timestamp: u64,
}

#[near(contract_state)]
#[derive(PanicOnDefault)]
pub struct GullyPassContract {
    pub owner_id: AccountId,
    pub tokens_by_id: LookupMap<TokenId, Token>,
    pub vaults: LookupMap<TokenId, VaultAccount>,
    pub transfer_locks: LookupSet<TokenId>,
    pub transfer_history: LookupMap<TokenId, Vec<TransferRecord>>,
    pub vault_history: LookupMap<TokenId, Vec<HistoryEntry>>,
}

#[near]
impl GullyPassContract {
    #[init]
    pub fn new(owner_id: AccountId) -> Self {
        Self {
            owner_id,
            tokens_by_id: LookupMap::new(b"t"),
            vaults: LookupMap::new(b"v"),
            transfer_locks: LookupSet::new(b"l"),
            transfer_history: LookupMap::new(b"h"),
            vault_history: LookupMap::new(b"y"),
        }
    }

    /// Mint a new GULLY Pass NFT to a customer
    pub fn nft_mint(
        &mut self,
        token_id: TokenId,
        receiver_id: AccountId,
        neighborhood: String,
    ) -> Token {
        assert!(
            env::predecessor_account_id() == self.owner_id
                || env::predecessor_account_id() == receiver_id,
            "GULLY: Unauthorized mint"
        );
        assert!(
            self.tokens_by_id.get(&token_id).is_none(),
            "GULLY: Token already exists"
        );

        let metadata = TokenMetadata {
            title: Some(format!("GULLY Pass #{}", token_id)),
            description: Some("Neighborhood Loyalty Membership Pass".to_string()),
            media: None,
            neighborhood,
            issued_at: env::block_timestamp(),
        };

        let token = Token {
            token_id: token_id.clone(),
            owner_id: receiver_id.clone(),
            metadata,
        };

        self.tokens_by_id.insert(&token_id, &token);

        let vault = VaultAccount::default();
        self.vaults.insert(&token_id, &vault);

        let initial_history = vec![HistoryEntry {
            entry_type: "MINT".to_string(),
            amount: 0,
            description: format!("GULLY Pass minted to {}", receiver_id),
            timestamp: env::block_timestamp(),
        }];
        self.vault_history.insert(&token_id, &initial_history);

        let transfer_hist = vec![TransferRecord {
            from: env::current_account_id(),
            to: receiver_id,
            timestamp: env::block_timestamp(),
        }];
        self.transfer_history.insert(&token_id, &transfer_hist);

        token
    }

    /// NEP-171 core transfer with Threat Model guarantees:
    /// 1. Token must not be locked.
    /// 2. Vault must have 0 pending promises (no in-flight scheduled drains).
    /// 3. Updates ownership atomically and updates transfer provenance history.
    pub fn nft_transfer(
        &mut self,
        receiver_id: AccountId,
        token_id: TokenId,
        memo: Option<String>,
    ) {
        let sender_id = env::predecessor_account_id();
        let mut token = self.tokens_by_id.get(&token_id).expect("Token not found");

        assert_eq!(token.owner_id, sender_id, "GULLY: Caller is not token owner");
        assert_ne!(token.owner_id, receiver_id, "GULLY: Cannot transfer to self");
        assert!(
            !self.transfer_locks.contains(&token_id),
            "GULLY: Token is locked in transfer"
        );

        let vault = self.vaults.get(&token_id).expect("Vault not found");
        assert_eq!(
            vault.pending_promises, 0,
            "GULLY: Threat Model Violation: Pending async promises in-flight; transfer blocked"
        );

        // Lock during state mutation
        self.transfer_locks.insert(&token_id);

        let old_owner = token.owner_id.clone();
        token.owner_id = receiver_id.clone();
        self.tokens_by_id.insert(&token_id, &token);

        // Record provenance
        let mut history = self.transfer_history.get(&token_id).unwrap_or_default();
        history.push(TransferRecord {
            from: old_owner,
            to: receiver_id.clone(),
            timestamp: env::block_timestamp(),
        });
        self.transfer_history.insert(&token_id, &history);

        // Unlock
        self.transfer_locks.remove(&token_id);

        env::log_str(&format!(
            "EVENT_JSON:{{\"standard\":\"nep171\",\"version\":\"1.0.0\",\"event\":\"nft_transfer\",\"data\":[{{\"old_owner_id\":\"{}\",\"new_owner_id\":\"{}\",\"token_ids\":[\"{}\"],\"memo\":\"{}\"}}]}}",
            sender_id,
            receiver_id,
            token_id,
            memo.unwrap_or_default()
        ));
    }

    /// Deposit funds/earnings into the Pass Vault (Treasury cut, shopping bonus)
    pub fn vault_deposit(&mut self, token_id: TokenId, amount: U128, description: String) {
        assert!(
            self.tokens_by_id.get(&token_id).is_some(),
            "Token not found"
        );
        let mut vault = self.vaults.get(&token_id).expect("Vault not found");
        let deposit_val: u128 = amount.into();

        vault.balance += deposit_val;
        vault.total_earned += deposit_val;
        self.vaults.insert(&token_id, &vault);

        let mut history = self.vault_history.get(&token_id).unwrap_or_default();
        history.push(HistoryEntry {
            entry_type: "EARNED".to_string(),
            amount: deposit_val,
            description,
            timestamp: env::block_timestamp(),
        });
        self.vault_history.insert(&token_id, &history);
    }

    /// Spend/withdraw from the Pass Vault.
    /// STRICT SECURITY: Only the current registered owner of the NFT can spend.
    pub fn vault_spend(&mut self, token_id: TokenId, amount: U128, purpose: String) {
        let caller = env::predecessor_account_id();
        let token = self.tokens_by_id.get(&token_id).expect("Token not found");

        assert_eq!(
            token.owner_id, caller,
            "GULLY Threat Model: Unauthorized. Caller is not the current NFT owner"
        );
        assert!(
            !self.transfer_locks.contains(&token_id),
            "Token is locked in transfer"
        );

        let mut vault = self.vaults.get(&token_id).expect("Vault not found");
        let spend_val: u128 = amount.into();

        assert!(
            vault.balance >= spend_val,
            "GULLY: Insufficient vault balance"
        );
        vault.balance -= spend_val;
        vault.total_paid_out += spend_val;
        self.vaults.insert(&token_id, &vault);

        let mut history = self.vault_history.get(&token_id).unwrap_or_default();
        history.push(HistoryEntry {
            entry_type: "PAID_OUT".to_string(),
            amount: spend_val,
            description: purpose,
            timestamp: env::block_timestamp(),
        });
        self.vault_history.insert(&token_id, &history);
    }

    /// Award a shop badge upon verified purchase visit
    pub fn award_badge(&mut self, token_id: TokenId, badge_category: String) {
        let mut vault = self.vaults.get(&token_id).expect("Vault not found");
        if !vault.shop_badges.contains(&badge_category) {
            vault.shop_badges.push(badge_category.clone());
        }
        vault.visits_count += 1;
        self.vaults.insert(&token_id, &vault);

        let mut history = self.vault_history.get(&token_id).unwrap_or_default();
        history.push(HistoryEntry {
            entry_type: "BADGE_AWARDED".to_string(),
            amount: 0,
            description: format!("Awarded badge: {}", badge_category),
            timestamp: env::block_timestamp(),
        });
        self.vault_history.insert(&token_id, &history);
    }

    /// View method: NEP-171 token detail
    pub fn nft_token(&self, token_id: TokenId) -> Option<Token> {
        self.tokens_by_id.get(&token_id)
    }

    /// View method: Vault account details
    pub fn get_vault(&self, token_id: TokenId) -> Option<VaultAccount> {
        self.vaults.get(&token_id)
    }

    /// View method: Full transfer history (provenance)
    pub fn get_transfer_history(&self, token_id: TokenId) -> Vec<TransferRecord> {
        self.transfer_history.get(&token_id).unwrap_or_default()
    }

    /// View method: Full earnings and payouts history
    pub fn get_vault_history(&self, token_id: TokenId) -> Vec<HistoryEntry> {
        self.vault_history.get(&token_id).unwrap_or_default()
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
        builder.current_account_id("gully_pass.near".parse().unwrap());
        builder.block_timestamp(1_700_000_000);
        builder
    }

    #[test]
    fn test_mint_and_ownership() {
        let admin: AccountId = "admin.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let context = get_context(admin.clone());
        testing_env!(context.build());

        let mut contract = GullyPassContract::new(admin.clone());
        let token = contract.nft_mint("1".to_string(), alice.clone(), "Indiranagar".to_string());

        assert_eq!(token.owner_id, alice);
        let vault = contract.get_vault("1".to_string()).unwrap();
        assert_eq!(vault.balance, 0);
        assert_eq!(vault.visits_count, 0);
    }

    #[test]
    fn test_vault_deposit_and_spend_by_owner() {
        let admin: AccountId = "admin.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let mut context = get_context(admin.clone());
        testing_env!(context.build());

        let mut contract = GullyPassContract::new(admin.clone());
        contract.nft_mint("1".to_string(), alice.clone(), "Koramangala".to_string());

        // Deposit 100 GULLY coins
        contract.vault_deposit("1".to_string(), U128(100), "Bakery purchase bonus".to_string());
        assert_eq!(contract.get_vault("1".to_string()).unwrap().balance, 100);

        // Alice spends 30 coins for salon discount
        testing_env!(context.predecessor_account_id(alice.clone()).build());
        contract.vault_spend("1".to_string(), U128(30), "Saloon haircut discount".to_string());
        assert_eq!(contract.get_vault("1".to_string()).unwrap().balance, 70);
        assert_eq!(contract.get_vault("1".to_string()).unwrap().total_paid_out, 30);
    }

    #[test]
    #[should_panic(expected = "Caller is not the current NFT owner")]
    fn test_unauthorized_vault_spend_fails() {
        let admin: AccountId = "admin.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let bob: AccountId = "bob.near".parse().unwrap();
        let mut context = get_context(admin.clone());
        testing_env!(context.build());

        let mut contract = GullyPassContract::new(admin.clone());
        contract.nft_mint("1".to_string(), alice.clone(), "Indiranagar".to_string());
        contract.vault_deposit("1".to_string(), U128(100), "Reward".to_string());

        // Bob tries to spend Alice's vault funds
        testing_env!(context.predecessor_account_id(bob.clone()).build());
        contract.vault_spend("1".to_string(), U128(50), "Malicious drain attempt".to_string());
    }

    #[test]
    fn test_transfer_path_and_revocation_of_old_owner() {
        let admin: AccountId = "admin.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let bob: AccountId = "bob.near".parse().unwrap();
        let mut context = get_context(admin.clone());
        testing_env!(context.build());

        let mut contract = GullyPassContract::new(admin.clone());
        contract.nft_mint("1".to_string(), alice.clone(), "Indiranagar".to_string());
        contract.vault_deposit("1".to_string(), U128(200), "Treasury dividend".to_string());

        // Alice transfers pass to Bob
        testing_env!(context.predecessor_account_id(alice.clone()).build());
        contract.nft_transfer(bob.clone(), "1".to_string(), Some("Sold pass to Bob".to_string()));

        // Verification: Bob is now the owner
        assert_eq!(contract.nft_token("1".to_string()).unwrap().owner_id, bob);

        // Bob can spend from the vault
        testing_env!(context.predecessor_account_id(bob.clone()).build());
        contract.vault_spend("1".to_string(), U128(50), "Bob haircut discount".to_string());
        assert_eq!(contract.get_vault("1".to_string()).unwrap().balance, 150);

        // Transfer history check
        let history = contract.get_transfer_history("1".to_string());
        assert_eq!(history.len(), 2);
        assert_eq!(history.last().unwrap().from, alice);
        assert_eq!(history.last().unwrap().to, bob);
    }

    #[test]
    #[should_panic(expected = "Caller is not the current NFT owner")]
    fn test_old_owner_cannot_spend_after_transfer() {
        let admin: AccountId = "admin.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let bob: AccountId = "bob.near".parse().unwrap();
        let mut context = get_context(admin.clone());
        testing_env!(context.build());

        let mut contract = GullyPassContract::new(admin.clone());
        contract.nft_mint("1".to_string(), alice.clone(), "Indiranagar".to_string());
        contract.vault_deposit("1".to_string(), U128(200), "Balance".to_string());

        // Alice transfers to Bob
        testing_env!(context.predecessor_account_id(alice.clone()).build());
        contract.nft_transfer(bob.clone(), "1".to_string(), None);

        // Threat Model Test: Alice tries to spend funds after having transferred the pass!
        // MUST PANIC with unauthorized error
        testing_env!(context.predecessor_account_id(alice.clone()).build());
        contract.vault_spend("1".to_string(), U128(50), "Backdoor exploit attempt".to_string());
    }

    #[test]
    fn test_sale_in_middle_of_payout_race_condition() {
        let admin: AccountId = "admin.near".parse().unwrap();
        let alice: AccountId = "alice.near".parse().unwrap();
        let bob: AccountId = "bob.near".parse().unwrap();
        let mut context = get_context(admin.clone());
        testing_env!(context.build());

        let mut contract = GullyPassContract::new(admin.clone());
        contract.nft_mint("1".to_string(), alice.clone(), "Indiranagar".to_string());

        // Payout occurs right before transfer
        contract.vault_deposit("1".to_string(), U128(100), "Mid-flight payout".to_string());

        // Alice transfers to Bob
        testing_env!(context.predecessor_account_id(alice.clone()).build());
        contract.nft_transfer(bob.clone(), "1".to_string(), None);

        // Another payout lands post-transfer
        testing_env!(context.predecessor_account_id(admin.clone()).build());
        contract.vault_deposit("1".to_string(), U128(50), "Post-transfer dividend".to_string());

        // Total 150 must belong to Bob
        assert_eq!(contract.get_vault("1".to_string()).unwrap().balance, 150);
        testing_env!(context.predecessor_account_id(bob.clone()).build());
        contract.vault_spend("1".to_string(), U128(150), "Bob spends total".to_string());
        assert_eq!(contract.get_vault("1".to_string()).unwrap().balance, 0);
    }
}
