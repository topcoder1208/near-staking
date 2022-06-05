use crate::*;

pub(crate) fn asser_at_least_one_yocto() {
    let attach_deposit = env::attached_deposit();
    assert!(
        env::attached_deposit() >= 1,
        "Required deposit is more than 1 yocto NEAR. Now {}",
        attach_deposit
    );
}

pub(crate) fn assert_one_yocto() {
    assert!(
        env::attached_deposit() == 1,
        "Required deposit is exact 1 yocto NEAR"
    );
}

pub(crate) fn refund_deposit(storage_used: u64) {
    let required_cost = env::storage_byte_cost() * Balance::from(storage_used);
    let attach_deposit = env::attached_deposit();

    assert!(
        attach_deposit >= required_cost,
        "Required deposit {} yocto NEAR is more than {} yocto NEAR attached deposit",
        required_cost,
        attach_deposit
    );

    let refund = attach_deposit - required_cost;
    if refund > 0 {
        Promise::new(env::predecessor_account_id()).transfer(refund);
    }
}
