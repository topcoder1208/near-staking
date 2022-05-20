use crate::*;

use near_sdk::{EpochHeight, Timestamp};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Account {
    // pub account_id: AccountId,
    pub stake_balance: Balance,
    pub paid_reward_balance: Balance,
    pub total_stake_balance: Balance,
    pub total_paid_reward_balance: Balance,
    pub total_staker: Balance,
    pub pre_reward: Balance,
    pub last_block_balance_change: BlockHeight,
    pub unstake_balance: Balance, // khi do se bi block lai epoch
    pub unstake_start_timestamp: Timestamp,
    pub unstake_available_epoch: BlockHeight, // 43_200 giay ~~ 12h
}

// t1 ---------- t2 ----------- now
// 100k          100k            100k

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct AccountJson {
    pub account_id: AccountId,
    pub stake_balance: U128,
    pub unstake_balance: U128,
    // pub reward_balance: U128,
    pub reward: U128,
    pub can_withdraw: bool,
    pub unstake_strt_tomstamp: Timestamp,
    pub unstake_start_timestamp: Timestamp,
    pub current_epoch: EpochHeight,
}

impl AccountJson {
    pub fn from(account_id: AccountId, account: Account, new_reward: Balance) -> Self {
        AccountJson {
            account_id,
            stake_balance: U128(account.stake_balance),
            unstake_balance: U128(account.unstake_balance),
            reward: U128(account.pre_reward + new_reward),
            can_withdraw: account.unstake_available_epoch <= env::epoch_height(),
            unstake_strt_tomstamp: account.unstake_start_timestamp,
            unstake_start_timestamp: account.unstake_start_timestamp,
            current_epoch: env::epoch_height(),
        }
    }
}
