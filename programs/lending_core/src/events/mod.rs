use anchor_lang::prelude::*;

#[event]
pub struct EDeposit {
    pub user: Pubkey,
    pub reserve: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ERedeem {
    pub user: Pubkey,
    pub reserve: Pubkey,
    pub amount: u64,
}

#[event]
pub struct EBorrow {
    pub user: Pubkey,
    pub reserve: Pubkey,
    pub amount: u64,
}

#[event]
pub struct ERepay {
    pub user: Pubkey,
    pub reserve: Pubkey,
    pub amount: u64,
}

#[event]
pub struct EAccrue {
    pub reserve: Pubkey,
    pub borrow_index: u128,
    pub supply_index: u128,
    pub last_update_slot: u64,
}

#[event]
pub struct ELiquidate {
    pub liquidator: Pubkey,
    pub repay_reserve: Pubkey,
    pub seize_reserve: Pubkey,
    pub repaid: u64,
    pub seized_liquidity: u64,
}