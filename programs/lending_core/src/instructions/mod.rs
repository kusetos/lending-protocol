pub mod admin {
    pub mod init_market; 
    pub mod set_pause_flags;
    pub mod add_reserve; 
    pub mod update_reserve_config;
}

pub mod user {
    pub mod deposit; 
    pub mod redeem; 
    pub mod borrow; 
    pub mod repay;
}

pub mod risk { 
    pub mod accrue; 
    pub mod refresh_obligation; 
    pub mod liquidate; 
}

pub use admin::*; 
pub use user::*; 
pub use risk::*;



use anchor_lang::prelude::*;

use crate::state::LendingMarket;

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
        space = 8 + 200,
        seeds = [b"market"],
        bump,
    )]
    pub market: Account<'info, LendingMarket>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn handler(ctx: Context<InitMarket>, args: InitMarketArgs) -> Result<()>{
    let market = &mut ctx.accounts.market;
    market.admin = args.admin;
    market.risk_authority = args.risk_authority;
    market.pause_authority = args.pause_authority;
    Ok(())
}