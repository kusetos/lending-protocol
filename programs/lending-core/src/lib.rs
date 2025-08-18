use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

declare_id!("GQ3V6NVzk2VyWS5dv8df9ijawcRs4s3enwkqBE4BmoTn");

const INTEREST_RATE: u64 = 5; // 5% fixed interest rate
const LTV_RATIO: u64 = 75; // 75% LoantoValue ratio

#[program]
pub mod lending_core {
    use anchor_lang::solana_program::{program::{invoke, invoke_signed}, system_instruction};

    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let lending_pool = &mut ctx.accounts.lending_pool;
        lending_pool.total_sol_deposited = 0;
        lending_pool.total_stablecoin_borrowed = 0;
        lending_pool.authority = *ctx.accounts.authority.key;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()>  {
        // Transfer SOL from user to the lending pool's vault
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

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()>  {
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

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()>  {
        let user_account = &mut ctx.accounts.user_account;
        let lending_pool = &mut ctx.accounts.lending_pool;

        // Calculate interest due
        let interest_due = user_account.stablecoin_debt.checked_mul(INTEREST_RATE).unwrap().checked_div(100).unwrap();
        let total_due = user_account.stablecoin_debt.checked_add(interest_due).unwrap();

        require!(amount >= total_due, ErrorCode::InsufficientRepayment);

        // In a real implementation, you would transfer the stablecoin from the user to be burned/locked.
        // For this MVP, we just update the state.
        msg!("Repaying {} stablecoins, which includes {} in interest.", total_due, interest_due);

        lending_pool.total_stablecoin_borrowed = lending_pool.total_stablecoin_borrowed.checked_sub(user_account.stablecoin_debt).unwrap();
        user_account.stablecoin_debt = 0;

        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()>  {
        let user_account = &mut ctx.accounts.user_account;
        let lending_pool = &mut ctx.accounts.lending_pool;

        let sol_price_usd = 100; // Fixed price for simplicity
        let remaining_sol = user_account.sol_deposited.checked_sub(amount).ok_or(ErrorCode::InsufficientCollateral)?;
        let collateral_value = remaining_sol.checked_mul(sol_price_usd).unwrap();
        let max_borrow_for_remaining = collateral_value.checked_mul(LTV_RATIO).unwrap().checked_div(100).unwrap();

        require!(user_account.stablecoin_debt <= max_borrow_for_remaining, ErrorCode::Undercollateralized);
        require!(user_account.sol_deposited >= amount, ErrorCode::InsufficientCollateral);
        
        // Use the correct PDA transfer method
        let lending_pool_key = lending_pool.key();
        let vault_seed = &[b"sol_vault", lending_pool_key.as_ref()];
        let (_, vault_bump) = Pubkey::find_program_address(vault_seed, ctx.program_id);
        let vault_seeds = &[b"sol_vault", lending_pool_key.as_ref(), &[vault_bump]];
        let vault_signer = &[&vault_seeds[..]];

        // Transfer SOL from vault to user using PDA signing
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

}


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
    /// CHECK: pda
    pub sol_vault: AccountInfo<'info>,
    #[account(mut)]
    pub user: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Borrow<'info> {
    #[account(mut)]
    pub lending_pool: Account<'info, LendingPool>,
    #[account(mut, has_one = owner)]
    pub user_account: Account<'info, UserAccount>,
    pub owner: Signer<'info>,
}

#[derive(Accounts)]
pub struct Repay<'info> {
    #[account(mut)]
    pub lending_pool: Account<'info, LendingPool>,
    #[account(mut, has_one = owner)]
    pub user_account: Account<'info, UserAccount>,
    pub owner: Signer<'info>,
}

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
    /// CHECK: pda
    pub sol_vault: AccountInfo<'info>,
    #[account(mut)]
    pub owner: Signer<'info>,
    pub system_program: Program<'info, System>,
}

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

#[error_code]
pub enum ErrorCode {
    #[msg("Loan-to-Value ratio exceeded.")]
    LtvExceeded,
    #[msg("Insufficient collateral for this operation.")]
    InsufficientCollateral,
    #[msg("Withdrawal would leave the account undercollateralized.")]
    Undercollateralized,
    #[msg("The repaid amount is less than the total amount due.")]
    InsufficientRepayment,
}