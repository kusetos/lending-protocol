use anchor_lang::prelude::*;

use crate::states::{LendingPool, UserAccount};
use crate::error_code::ErrorCode;

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub lending_pool: Account<'info, LendingPool>,
    #[account(mut, has_one = owner)]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        mut, 
        seeds = [b"sol_vault", lending_pool.key().as_ref()],
        bump)]
    pub sol_vault: AccountInfo<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle(ctx: Context<Withdraw>, amount: u64) -> Result<()>  {
    let user_account = &mut ctx.accounts.user_account;
    let lending_pool = &mut ctx.accounts.lending_pool;

    let sol_price_usd = 100;
    let remaining_sol = user_account.sol_deposited.checked_sub(amount).ok_or(ErrorCode::InsufficientCollateral)?;
    let collateral_value = remaining_sol.checked_mul(sol_price_usd).unwrap();
    let max_borrow_for_remaining = collateral_value.checked_mul(LTV_RATIO).unwrap().checked_div(100).unwrap();

    require!(user_account.stablecoin_debt <= max_borrow_for_remaining, ErrorCode::Undercollateralized);
    require!(user_account.sol_deposited >= amount, ErrorCode::InsufficientCollateral);
    
    let lending_pool_key = lending_pool.key();
    let vault_seed = &[b"sol_vault", lending_pool_key.as_ref()];
    let (_, vault_bump) = Pubkey::find_program_address(vault_seed, ctx.program_id);
    let vault_seeds = &[b"sol_vault", lending_pool_key.as_ref(), &[vault_bump]];
    let vault_signer = &[&vault_seeds[..]];

    let ix = system_instruction::transfer(
        &ctx.accounts.sol_vault.key(),
        &ctx.accounts.owner.key(),
        amount,
    );
    
    invoke_signed(
        &ix,
        &[
            ctx.accounts.sol_vault.to_account_info(),
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
        vault_signer,
    )?;

    user_account.sol_deposited -= amount;
    lending_pool.total_sol_deposited -= amount;

    Ok(())
}