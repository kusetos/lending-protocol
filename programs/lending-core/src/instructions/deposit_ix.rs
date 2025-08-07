use anchor_lang::prelude::*;

use crate::states::{LendingPool, UserAccount};


#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub lending_pool: Account<'info, LendingPool>,
    #[account(
        init_if_needed, 
        payer = user, 
        space = 8 + 8 + 8 + 32, 
        seeds = [b"user_account", user.key().as_ref()], 
        bump)]
    pub user_account: Account<'info, UserAccount>,
    #[account(
        mut, 
        seeds = [b"sol_vault", lending_pool.key().as_ref()], 
        bump)]
    pub sol_vault: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle(ctx: Context<Deposit>, amount: u64) -> Result<()>  {
    let ix = system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.sol_vault.key(),
        amount,
    );
    invoke(
        &ix,
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.sol_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    let user_account = &mut ctx.accounts.user_account;
    user_account.sol_deposited += amount;
    user_account.owner = *ctx.accounts.user.key;

    let lending_pool = &mut ctx.accounts.lending_pool;
    lending_pool.total_sol_deposited += amount;

    Ok(())
}