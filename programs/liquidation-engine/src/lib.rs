use anchor_lang::prelude::*;

declare_id!("D5kBiiVuwtDS8PwY9VgSKMT5h4Yw4eAcnpQfvkhaymaE");

#[program]
pub mod liquidation_engine {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
