use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    pub reward_numberator: u32,
    pub reward_denominator: u64,
}

impl Default for Config {
    fn default() -> Self {
        // APR 15 -> 18%
        // reward per block = (18% * total_supply) / (365 * 24 * 60 * 60)
        Config {
            reward_numberator: 715,
            reward_denominator: 1_000_000_000,
        }
    }
}

// APR = (1 + (reward_numberator / reward_denominator))
// APR 15% = (1 + (15 / 100))
