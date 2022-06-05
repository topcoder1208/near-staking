use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, near_bindgen, ext_contract, AccountId, Balance, BlockHeight, BorshStorageKey, PanicOnDefault, Promise,
    PromiseOrValue
};

mod account;
mod config;
mod enumeration;
mod internal;
mod util;
mod core_impl;

use crate::account::*;
use crate::config::*;
use crate::util::*;

#[derive(BorshDeserialize, BorshSerialize, BorshStorageKey)]
pub enum StorageKey {
    AccountKey,
}

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct StakingContract {
    pub owner_id: AccountId,
    pub ft_contract_id: AccountId,
    pub config: Config,
    pub total_stake_balance: Balance,
    pub total_paid_reward_balance: Balance,
    pub total_staker: Balance,
    pub last_block_balance_change: BlockHeight,
    pub accounts: UnorderedMap<AccountId, UpgradableAccount>,
    pub paused: bool,
    pub pause_in_block: BlockHeight,
    pub new_data: U128,
}

#[near_bindgen]
impl StakingContract {

    #[init]
    pub fn new_default_config(owner_id: AccountId, ft_contract_id: AccountId) -> Self {
        Self::new(owner_id, ft_contract_id, Config::default())
    }

    #[init]
    pub fn new(owner_id: AccountId, ft_contract_id: AccountId, config: Config) -> Self {
        StakingContract {
            owner_id,
            ft_contract_id,
            config,
            total_stake_balance: 0,
            total_paid_reward_balance: 0,
            total_staker: 0,
            last_block_balance_change: env::block_index(),
            accounts: UnorderedMap::new(StorageKey::AccountKey),
            paused: false,
            pause_in_block: 0,
            new_data: U128(0),
        }
    }

    #[payable]
    pub fn storage_deposit(&mut self, account_id: Option<AccountId>) {
        asser_at_least_one_yocto();
        let account = account_id.unwrap_or_else(|| env::predecessor_account_id());
        let account_stake = self.accounts.get(&account);
        if account_stake.is_some() {
            // refund deposit if account already exists
            refund_deposit(0);
        } else {
            // create new account and return excess deposit
            let before_storage_usage = env::storage_usage();

            self.internal_create_account(account.clone());

            let after_storage_usage = env::storage_usage();

            let storage_used = after_storage_usage - before_storage_usage;
            refund_deposit(storage_used);
        }
    }

    pub fn storage_balance_of(self, account_id: AccountId) -> U128 {
        let account = self.accounts.get(&account_id);
        if account.is_some() {
            let account = account.unwrap();
            let account = Account::from(account);
            U128(account.stake_balance)
        } else {
            U128(0)
        }
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod test {
    use super::*;
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;
    use near_sdk::{MockedBlockchain, VMContext};

    fn get_context(is_view: bool) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(accounts(0))
            .predecessor_account_id(accounts(0))
            .is_view(is_view);

        builder
    }

    #[test]
    fn test_init_contract() {
        let context = get_context(false);
        testing_env!(context.build());

        let config = Config {
            reward_numberator: 500,
            reward_denominator: 1_000_000_000,
        };

        let contract = StakingContract::new(
            accounts(1).to_string(),
            "ft_contract".to_string(),
            config.clone(),
        );

        assert_eq!(contract.owner_id, accounts(1).to_string());
        assert_eq!(contract.ft_contract_id, "ft_contract".to_string());
        assert_eq!(config.reward_numberator, contract.config.reward_numberator);
        assert_eq!(contract.paused, false);
    }
}
