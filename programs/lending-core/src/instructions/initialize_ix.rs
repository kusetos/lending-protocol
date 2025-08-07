use anchor_lang::prelude::*;

use crate::states::LendingPool;

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(
        init, 
        payer = authority, 
        space = 8 + 8 + 8 + 32
    )]
    pub lending_pool: Account<'info, LendingPool>,
    #[account(mut)]
    pub authority: Signer<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle(ctx: Context<Initialize>) -> Result<()> {
    let lending_pool = &mut ctx.accounts.lending_pool;
    lending_pool.total_sol_deposited = 0;
    lending_pool.total_stablecoin_borrowed = 0;
    lending_pool.authority = *ctx.accounts.authority.key;
    Ok(())
}