use anchor_lang::prelude::*;

use crate::states::{LendingPool, UserAccount};
use crate::error_code::ErrorCode;


#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub lending_pool: Account<'info, LendingPool>,
    #[account(mut, has_one = owner)]
    pub user_account: Account<'info, UserAccount>,
    pub owner: Signer<'info>,
}

pub fn handle(ctx: Context<Borrow>, amount: u64) -> Result<()>  {
    let user_account = &mut ctx.accounts.user_account;
    let lending_pool = &mut ctx.accounts.lending_pool;

    let sol_price_usd = 100;
    let collateral_value = user_account.sol_deposited.checked_mul(sol_price_usd).unwrap();
    let current_debt_value = user_account.stablecoin_debt;

    let max_borrow_value = collateral_value.checked_mul(LTV_RATIO).unwrap().checked_div(100).unwrap();

    require!(current_debt_value.checked_add(amount).unwrap() <= max_borrow_value, ErrorCode::LtvExceeded);

    user_account.stablecoin_debt += amount;
    lending_pool.total_stablecoin_borrowed += amount;

    msg!("Successfully borrowed {} stablecoins.", amount);

    Ok(())
}