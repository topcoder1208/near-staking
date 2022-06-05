use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolJson {
    pub total_stake_balance: U128,
    pub total_staker: U128,
    pub is_paused: bool,
    pub total_paid_reward_balance: U128,
}

#[near_bindgen]
impl StakingContract {
    pub fn get_account_info(&self, account_id: AccountId) -> AccountJson {
        let account = self.accounts.get(&account_id);
        if account.is_some() {
            let account = account.unwrap();
            let account = Account::from(account);
            let new_reward = self.internal_caculate_account_reward(&account);
            AccountJson::from(account_id, account, new_reward)
        } else {
            AccountJson::from(
                account_id,
                Account {
                    stake_balance: 0,
                    paid_reward_balance: 0,
                    total_stake_balance: 0,
                    total_paid_reward_balance: 0,
                    total_staker: 0,
                    last_block_balance_change: env::block_index(),
                    unstake_balance: 0,
                    unstake_start_timestamp: 0,
                    unstake_available_epoch: 0,
                    new_account_data: U128(0),
                },
                0,
            )
        }
    }

    pub fn get_account_staked_balance(&self, account_id: AccountId) -> Balance {
        let account = self.accounts.get(&account_id);
        if account.is_some() {
            let account = account.unwrap();
            let account = Account::from(account);
            let new_reward = self.internal_caculate_account_reward(&account);
            new_reward + account.stake_balance
        } else {
            0
        }
    }

    pub fn get_pool_info(&self) -> PoolJson {
        PoolJson {
            total_stake_balance: U128(self.total_stake_balance),
            total_staker: U128(self.total_staker),
            is_paused: self.paused,
            total_paid_reward_balance: U128(self.total_paid_reward_balance),
        }
    }
}
