use anchor_lang::prelude::*;

use crate::states::{LendingPool, UserAccount};
use crate::error_code::ErrorCode;


#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut)]
    pub lending_pool: Account<'info, LendingPool>,
    #[account(mut, has_one = owner)]
    pub user_account: Account<'info, UserAccount>,
    pub owner: Signer<'info>,
}

pub fn handle(ctx: Context<Repay>, amount: u64) -> Result<()>  {
    let user_account = &mut ctx.accounts.user_account;
    let lending_pool = &mut ctx.accounts.lending_pool;

    let interest_due = user_account.stablecoin_debt.checked_mul(INTEREST_RATE).unwrap().checked_div(100).unwrap();
    let total_due = user_account.stablecoin_debt.checked_add(interest_due).unwrap();

    require!(amount >= total_due, ErrorCode::InsufficientRepayment);

    msg!("Repaying {} stablecoins, which includes {} in interest.", total_due, interest_due);

    lending_pool.total_stablecoin_borrowed = lending_pool.total_stablecoin_borrowed.checked_sub(user_account.stablecoin_debt).unwrap();
    user_account.stablecoin_debt = 0;

    Ok(())
}
