use anchor_lang::prelude::*;

declare_id!("CdaNq881svYxjhV9pSDCrHqNetism19UkbJo7StRHvw8");

#[program]
pub mod flash_loan {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
