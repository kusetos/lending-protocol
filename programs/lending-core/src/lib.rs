use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token};
use anchor_spl::token_interface::{TokenAccount, Mint};
declare_id!("oxJ2fe6yjYADjuCbyTQ3L9UnTiCYpXAAn7VhThvrbrQ");

#[program]
pub mod lending_core {
    use std::cmp::Reverse;

    use super::*;

    pub fn initialize_market(ctx: Context<InitializeMarket>) -> Result<()> {

        let market = &mut ctx.accounts.lending_market;

        market.admin = ctx.accounts.admin.key();
        market.is_initialized = true;

        emit!(MarketInitialized{
            market: market.key(),
            admin: market.admin,
        });


        Ok(())
    }

    pub fn initialize_reserve(ctx: Context<InitializeReserve>, liquidity_mint: Pubkey) -> Result<()>{
        let reserve = &mut ctx.accounts.reserve;
        reserve.lending_market = ctx.accounts.lending_market.key();
        reserve.liquidity_mint = liquidity_mint;
        reserve.liquidity_vault = ctx.accounts.liquidity_vault.key();
        reserve.deposit_token_mint = ctx.accounts.deposit_token_mint.key();
        reserve.total_liquidity = 0;
        reserve.total_borrowed = 0;
        reserve.interest_rate = 500;
        reserve.last_update = Clock::get()?.unix_timestamp;

        emit!(ReserveInitialized{
            reserve: reserve.key(),
            liquidity_mint,
        });

        Ok(())
    } 

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()>{

        let reserve = &mut ctx.accounts.reserve;
        let user_token_account = &mut ctx.accounts.user_token_account;
        let liquidity_vault = &mut ctx.accounts.liquidity_vault;
        let deposit_token_account = &mut ctx.accounts.deposit_token_account;

        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: user_token_account.to_account_info(),
                    to: liquidity_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            amount,
        )?;

        anchor_spl::token::mint_to(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::MintTo {
                    mint: ctx.accounts.deposit_token_mint.to_account_info(),
                    to: deposit_token_account.to_account_info(),
                    authority: reserve.to_account_info(),
                },
            ),
            amount,
        )?;

        reserve.total_liquidity = reserve.total_liquidity.checked_add(amount).ok_or(error!(ErrorCode::Overflow))?;
        reserve.last_update = Clock::get()?.unix_timestamp;

        emit!(DepositEvent {
            user: ctx.accounts.user.key(),
            reserve: reserve.key(),
            amount,
        });

        Ok(())
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()>{

        let reserve = &mut ctx.accounts.reserve;
        let obligation = &mut ctx.accounts.obligation;

        let price = 100_000_000;
        let collateral_value = obligation.collateral_amount.checked_mul(price).ok_or(error!(ErrorCode::Overflow))?; // Replace with CPI oracle price
        let required_collateral = amount.checked_mul(150).ok_or(error!(ErrorCode::Overflow))?.checked_div(100).ok_or(error!(ErrorCode::Overflow))?;

        if collateral_value < required_collateral {
            return Err(error!(ErrorCode::InsufficientCollateral));
        }

        anchor_spl::token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::Transfer {
                    from: ctx.accounts.liquidity_vault.to_account_info(),
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: reserve.to_account_info(),
                },
            ),
            amount,
        )?;

        reserve.total_borrowed = reserve.total_borrowed.checked_add(amount).ok_or(error!(ErrorCode::Overflow))?;
        obligation.borrowed_amount = obligation.borrowed_amount.checked_add(amount).ok_or(error!(ErrorCode::Overflow))?;
        obligation.last_update = Clock::get()?.unix_timestamp;

        emit!(BorrowEvent {
            user: ctx.accounts.user.key(),
            reserve: reserve.key(),
            amount,
        });

        Ok(())
    }
}



#[derive(Accounts)]
pub struct InitializeMarket<'info>{
    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 1,
    )]
    pub lending_market: Account<'info, LendingMarket>,
    #[account(mut)]
    pub admin: Signer<'info>,
    pub system_program: Program<'info, System>,
}
#[event]
pub struct MarketInitialized{
    pub market: Pubkey,
    pub admin: Pubkey,
}

#[account]
pub struct LendingMarket{
    pub admin: Pubkey,
    pub is_initialized: bool,
}

#[derive(Accounts)]
pub struct InitializeReserve<'info>{
    #[account(
        mut,
        has_one = admin
    )]
    pub lending_market: Account<'info, LendingMarket>,

    #[account(
        init,
        payer = admin,
        space = 8 + 32 + 32 + 32 + 32 + 8 + 8 + 8 + 8,
    )] 
    pub reserve: Account<'info, Reserve>,

    #[account(mut)]
    pub liquidity_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub deposit_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}
#[event]
pub struct ReserveInitialized{
    pub reserve: Pubkey,
    pub liquidity_mint: Pubkey,
}
#[account]
pub struct Reserve{
    pub lending_market: Pubkey,
    pub liquidity_mint: Pubkey,
    pub liquidity_vault: Pubkey,
    pub deposit_token_mint: Pubkey,
    pub total_liquidity: u64,
    pub total_borrowed: u64,
    pub interest_rate: u64,
    pub last_update: i64,
}

#[derive(Accounts)]
pub struct Deposit<'info>{
    #[account(
        mut, 
        has_one = lending_market,
    )]
    pub reserve: Account<'info, Reserve>,
    
    #[account(mut)]
    pub lending_market: Account<'info, LendingMarket>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    #[account(mut)]
    pub liquidity_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub deposit_token_mint: Account<'info, Mint>,

    #[account(mut)]
    pub deposit_token_account: Account<'info, TokenAccount>,

    pub user: Signer<'info>,
    pub token_program: Program<'info, Token>,
}

#[event]
pub struct DepositEvent {
    pub user: Pubkey,
    pub reserve: Pubkey,
    pub amount: u64,
}

#[derive(Accounts)]
pub struct Borrow<'info>{
    #[account(
        mut,
        has_one = lending_market,
    )]
    pub reserve: Account<'info, Reserve>,
    
    pub lending_market: Account<'info, LendingMarket>,

    #[account(
        mut,
        has_one = user,
        has_one = reserve,
    )]
    pub obligation: Account<'info, Obligation>,

    #[account(mut)]
    pub liquidity_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,
    
    pub user: Signer<'info>,
    
    pub token_program: Program<'info, Token>,
}
#[event]
pub struct BorrowEvent {
    pub user: Pubkey,
    pub reserve: Pubkey,
    pub amount: u64,
}

#[account]
pub struct Obligation{
    pub user: Pubkey,
    pub lending_market: Pubkey,
    pub reserve: Pubkey,
    pub collateral_mint: Pubkey,
    pub collateral_amount: u64,
    pub borrowed_mint: Pubkey,
    pub borrowed_amount: u64,
    pub last_update: i64,
}

#[derive(Accounts)]
pub struct UpdateInterestRate<'info>{
    #[account(mut)]
    pub reserve: Account<'info, Reserve>,
}

#[event]
pub struct InterestRateUpdated{
    pub reserve: Pubkey,
    pub interest_rate: u64,
    
}
#[error_code]
pub enum ErrorCode {
    #[msg("Arithmetic overflow")]
    Overflow,
    #[msg("Insufficient collateral")]
    InsufficientCollateral,
}

#[cfg(test)]
mod tests{
    #[test]
    pub fn test_a(){
        let _a= 1;
    }
}