use anchor_lang::prelude::*;
use crate::state::*;
use crate::errors::LendingError;

#[derive(Accounts)]
pub struct SetPauseFlags<'info>{
    #[account(
        mut,
        seeds = [b"market"],
        bump = market.bump
    )]
    pub market: Account<'info, LendingMarket>,
    pub pause_authority: Signer<'info>,
}

pub fn handler(ctx: Context<SetPauseFlags>, flags: u64) -> Result<()>{
    require_keys_eq!(ctx.accounts.market.pause_authority, ctx.accounts.pause_authority.key(), LendingError::Unauthorized);
    ctx.accounts.market.pause_flags = flags;
    Ok(())
}