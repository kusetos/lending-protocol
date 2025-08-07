use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, TokenAccount, Token};

use crate::states::{LendingPool, UserAccount};
declare_id!("So1end111111111111111111111111111111111111111");

pub mod states;
pub mod instructions;
pub mod error_code;

const INTEREST_RATE: u64 = 5;
const LTV_RATIO: u64 = 75;

#[program]
pub mod lending_core {
    use anchor_lang::solana_program::{program::invoke_signed, system_instruction};

    use instructions::{Borrow, Deposit, Initialize};

    use crate::instructions::{Repay, Withdraw};

    use super::*;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize_ix::handle(ctx)
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()>  {
        instructions::deposit_ix::handle(ctx, amount)
    }

    pub fn borrow(ctx: Context<Borrow>, amount: u64) -> Result<()>  {
        instructions::borrow_ix::handle(ctx, amount)
    }

    pub fn repay(ctx: Context<Repay>, amount: u64) -> Result<()>  {
        instructions::repay_ix::handle(ctx, amount)
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()>  {
        instructions::withdraw_ix::handle(ctx, amount)
    }

}


