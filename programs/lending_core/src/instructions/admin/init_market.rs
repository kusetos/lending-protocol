use anchor_lang::prelude::*;

use crate::{constants::VERSION, state::LendingMarket};

#[derive(AnchorSerialize, AnchorDeserialize)]
pub struct InitMarketArgs{
    pub admin: Pubkey,
    pub risk_authority: Pubkey,
    pub pause_authority: Pubkey,
}

#[derive(Accounts)]
#[instruction(args: InitMarketArgs)]
pub struct InitMarket<'info>{
    #[account(
        init,
        payer = payer,
        space = 8 + 256,
        seeds = [b"market"],
        bump,
    )]
    pub market: Account<'info, LendingMarket>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitMarket>, args: InitMarketArgs) -> Result<()>{
    let bump = ctx.bumps.market;
    let market = &mut ctx.accounts.market;

    market.version = VERSION;
    market.admin = args.admin;
    market.risk_authority = args.risk_authority;
    market.pause_authority = args.pause_authority;
    market.pause_flags = 0;
    market.bump =bump;

    Ok(())
}