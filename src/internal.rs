use crate::*;

#[near_bindgen]
impl StakingContract {
    pub(crate) fn internal_create_account(&mut self, account_id: AccountId) {
        let account = Account {
            stake_balance: 0,
            paid_reward_balance: 0,
            total_stake_balance: 0,
            total_paid_reward_balance: 0,
            total_staker: 0,
            pre_reward: 0,
            last_block_balance_change: env::block_index(),
            unstake_balance: 0,
            unstake_start_timestamp: 0,
            unstake_available_epoch: 0,
        };

        self.accounts.insert(&account_id, &account);
    }

    pub(crate) fn internal_caculate_account_reward(&self, account: &Account) -> Balance {
        let last_block = if self.paused {
            self.pause_in_block
        } else {
            env::block_index()
        };

        let diff_block = (last_block - account.last_block_balance_change) as u128;
        let reward = (account.stake_balance * self.config.reward_numberator as u128 * diff_block)
            / self.config.reward_denominator as u128;

        reward
    }

    pub(crate) fn internal_caculate_total_reward(&self) -> Balance {
        let last_block = if self.paused {
            self.pause_in_block
        } else {
            env::block_index()
        };

        let diff_block = (last_block - self.last_block_balance_change) as u128;
        let reward =
            (self.total_stake_balance * self.config.reward_numberator as u128 * diff_block)
                / self.config.reward_denominator as u128;

        reward
    }
}
