use anchor_lang::prelude::*;


#[account]
pub struct LendingPool {
    pub total_sol_deposited: u64,
    pub total_stablecoin_borrowed: u64,
    pub authority: Pubkey,
}

#[account]
pub struct UserAccount {
    pub owner: Pubkey,
    pub sol_deposited: u64,
    pub stablecoin_debt: u64,
}