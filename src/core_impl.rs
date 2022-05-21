use near_sdk::{Gas, PromiseResult};

use crate::*;

pub const DEPOSIT_ONE_YOCTO: Balance = 1;
pub const FT_TRANSFER_GAS: Gas = 10_000_000_000_000;
pub const NO_DEPOSIT: Balance = 0;
pub const FT_HARVEST_CALLBACK_GAS: Gas = 10_000_000_000;
pub trait FungibleTokenReceiver {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128>;
}

#[ext_contract(ext_ft_contract)]
pub trait FunngibleToken {
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: U128, memo: Option<String>);
}

#[ext_contract(ext_self)]
pub trait ExtStakingContract {
    fn ft_transfer_callback(&mut self, amount: U128, account_id: AccountId);

    fn ft_withdraw_callback(&mut self, account_id: AccountId, old_account: Account);
}

#[near_bindgen]
impl FungibleTokenReceiver for StakingContract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        self.internal_deposit_and_stake(sender_id, amount.0, msg);

        // if fail -> internal will reject and transfer will be rejected
        // ft_resold_transfer will transfer

        // success return amount = 0
        PromiseOrValue::Value(U128(0))
    }
}

#[near_bindgen]
impl StakingContract {
    #[private]
    pub fn unstake(&mut self, amount: U128) {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();

        self.internal_instake(account_id, amount.0);
    }

    #[private]
    pub fn withraw(&mut self, amount: U128) -> Promise {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();

        let old_account = self.internal_withdraw(account_id.clone(), amount.0);

        ext_ft_contract::ft_transfer(
            account_id.clone(),
            U128(old_account.unstake_balance),
            None,
            &self.ft_contract_id,
            DEPOSIT_ONE_YOCTO,
            FT_TRANSFER_GAS,
        )
        .then(ext_self::ft_withdraw_callback(
            account_id,
            old_account,
            &env::current_account_id(),
            NO_DEPOSIT,
            FT_HARVEST_CALLBACK_GAS,
        ))
    }

    pub fn harvest(&mut self) -> Promise {
        assert_one_yocto();

        let account_id = env::predecessor_account_id();
        let upgradable_account = self.accounts.get(&account_id);
        assert!(upgradable_account.is_some(), "Account not found");

        let account = Account::from(upgradable_account.unwrap());

        let new_reward = self.internal_caculate_account_reward(&account);
        let current_reward = account.pre_reward + new_reward;

        assert!(current_reward > 0, "No reward to harvest");

        ext_ft_contract::ft_transfer(
            account_id.clone(),
            U128(current_reward),
            None,
            &self.ft_contract_id,
            DEPOSIT_ONE_YOCTO,
            FT_TRANSFER_GAS,
        )
        .then(ext_self::ft_transfer_callback(
            U128(current_reward),
            account_id.clone(),
            &env::current_account_id(),
            NO_DEPOSIT,
            FT_HARVEST_CALLBACK_GAS,
        ))
    }

    #[private]
    pub fn ft_transfer_callback(&mut self, amount: U128, account_id: AccountId) -> U128 {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ft_transfer_callback should only be called once"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),

            PromiseResult::Failed => env::panic(b"ft_transfer_callback failed"),
            PromiseResult::Successful(_value) => {
                let account = self.accounts.get(&account_id);
                assert!(account.is_some(), "Account not found");
                let account = account.unwrap();
                let mut account = Account::from(account);
                account.pre_reward = 0;
                account.last_block_balance_change = env::block_index();
                self.accounts
                    .insert(&account_id, &UpgradableAccount::from(account));

                self.total_paid_reward_balance += amount.0;

                amount
            }
        }
    }

    #[private]
    pub fn ft_withdraw_callback(&mut self, account_id: AccountId, old_account: Account) -> U128 {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ft_withdraw_callback should only be called once"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),

            PromiseResult::Successful(_) => U128(old_account.unstake_balance),

            PromiseResult::Failed => {
                // handle rollback data
                self.accounts
                    .insert(&account_id, &UpgradableAccount::from(old_account));
                U128(0)
            }
        }
    }
}
