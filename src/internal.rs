use crate::*;

#[near_bindgen]
impl StakingContract {
    pub(crate) fn internal_unstake(&mut self, account_id: AccountId, amount: Balance) {
        let account = self.accounts.get(&account_id);
        assert!(account.is_some(), "Account not found");
        let account = account.unwrap();
        let mut account = Account::from(account);

        assert!(amount <= account.stake_balance, "Amount is not enough");

        let new_reward = self.internal_caculate_account_reward(&account);

        // update account data
        account.stake_balance -= amount + new_reward;
        account.paid_reward_balance += new_reward;
        account.last_block_balance_change = env::block_index();
        account.unstake_balance += amount;

        account.unstake_start_timestamp = env::block_timestamp();
        account.unstake_available_epoch = env::block_index();

        if account.stake_balance == 0 {
            self.total_staker -= 1;
        }

        self.accounts
            .insert(&account_id, &UpgradableAccount::from(account));

        self.last_block_balance_change = env::block_index();
        self.total_stake_balance -= amount;
    }

    pub(crate) fn internal_withdraw(&mut self, account_id: AccountId, amount: Balance) -> Account {
        let account = self.accounts.get(&account_id);   
        assert!(account.is_some(), "Account not found");
        let account = account.unwrap();
        let account = Account::from(account);

        assert!(amount <= account.unstake_balance, "Amount is not enough");
        assert!(account.unstake_available_epoch <= env::epoch_height(), "Unstake is not available");

        // create new account data because of using old account data for reject if transfer failed
        let new_account = Account {
            stake_balance: account.stake_balance,

            last_block_balance_change: account.last_block_balance_change,
            unstake_balance: 0,
            unstake_start_timestamp: 0,
            unstake_available_epoch: 0,
            new_account_data: account.new_account_data,

            paid_reward_balance: account.paid_reward_balance,
            total_stake_balance: account.total_stake_balance + amount,
            total_paid_reward_balance: account.total_paid_reward_balance,
            total_staker: account.total_staker,
        };

        self.accounts.insert(&account_id, &UpgradableAccount::from(new_account));

        account
    }
 
    pub(crate) fn internal_deposit_and_stake(
        &mut self,
        account_id: AccountId,
        amount: u128,
    ) {
        // validate account
        let account = self.accounts.get(&account_id);
        assert!(account.is_some(), "Account not found");
        assert_eq!(self.paused, false, "Contract is paused");
        assert_eq!(
            self.ft_contract_id,
            env::predecessor_account_id(),
            "FT contract id is not the same as the sender"
        );

        let account = account.unwrap();
        let mut account = Account::from(account);

        // user is first time staking
        if account.stake_balance == 0 {
            self.total_staker += 1;
        }

        let new_reward = self.internal_caculate_account_reward(&account);

        // update account data
        account.stake_balance += amount + new_reward;
        account.paid_reward_balance += new_reward;
        account.last_block_balance_change = env::block_index();

        self.accounts
            .insert(&account_id, &UpgradableAccount::from(account));

        // update pool data
        self.total_stake_balance += amount;
        self.last_block_balance_change = env::block_index();
    }

    pub(crate) fn internal_create_account(&mut self, account_id: AccountId) {
        let account = Account {
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
        };

        let account = UpgradableAccount::from(account);

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

    pub(crate) fn internal_restake(&mut self) {
        assert_eq!(self.paused, false, "Contract is paused");
        let accounts = self.accounts.to_vec();

        for (account_id, _account) in accounts.iter() {
            // validate account
            let account = self.accounts.get(&account_id);

            let account = account.unwrap();
            let mut account = Account::from(account);

            let new_reward = self.internal_caculate_account_reward(&account);

            // update account data
            account.stake_balance += new_reward;
            account.paid_reward_balance += new_reward;
            account.last_block_balance_change = env::block_index();

            self.accounts
                .insert(&account_id, &UpgradableAccount::from(account));

            // update pool data
            self.total_stake_balance += new_reward;
            self.last_block_balance_change = env::block_index();
        }
    }
}
