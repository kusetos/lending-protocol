use anchor_lang::prelude::*;

declare_id!("oxJ2fe6yjYADjuCbyTQ3L9UnTiCYpXAAn7VhThvrbrQ");

#[program]
pub mod lending_core {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
