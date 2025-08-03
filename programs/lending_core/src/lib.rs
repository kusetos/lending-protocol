use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint};

declare_id!("oxJ2fe6yjYADjuCbyTQ3L9UnTiCYpXAAn7VhThvrbrQ");

pub mod state;
pub mod instructions;
pub mod constants;
pub mod events;
pub mod math;
pub mod errors;

#[program]
pub mod lending_core {

    use super::*;
    use crate::instruction::{self, InitMarket};

    pub fn init_market(ctx: Context<InitMarket>, args: InitMarketArgs) -> Result<()>{
        init_market::handler(ctx, args);
        Ok(())
    }
}

#[cfg(test)]
mod tests{
    #[test]
    pub fn test_a(){
        let _a= 1;
    }
}