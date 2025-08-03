use anchor_lang::prelude::*;

declare_id!("4kuDEScZ1PwyMATqoHBipKWa72UXzTeuutpaGncxzyE4");

#[program]
pub mod price_oracle {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
