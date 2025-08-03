use anchor_lang::prelude::*;

#[account]
pub struct LendingMarket{
    pub version: u8,
    pub admin: Pubkey,
    pub risk_authority: Pubkey,
    pub pause_authority: Pubkey,
    pub pause_flags: u64,
    pub bump: u8,
}
impl LendingMarket {
    pub const LEN: usize = 8 + 1 + 32*3 + 8 + 1 + 32;
}
#[account]
pub struct Reserve{
    pub market: Pubkey,
    pub liquidity_mint: Pubkey,
    pub vault: Pubkey,
    pub fee_vault: Pubkey,
    pub supplay_index: u128,
    pub borrow_index: u128,
    pub last_update_slot: u64,
    pub config: ReserveConfig,
    pub totals: ReserveTotals,
    pub bumps: ReserveBumps,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ReserveTotals {
    pub total_liquidity: u64,
    pub total_borrows_scaled: u128,
    pub available_liquidity: u64,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default)]
pub struct ReserveBumps { pub reserve: u8, pub vault: u8, pub fee_vault: u8 }

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Default, Copy)]
pub struct ReserveConfig {
    pub ltv_bps: u16,                // e.g. 7500 = 75%
    pub liq_threshold_bps: u16,      // e.g. 8000 = 80%
    pub close_factor_bps: u16,       // e.g. 5000 = 50%
    pub liq_bonus_bps: u16,          // e.g. 800 = 8%
    pub reserve_factor_bps: u16,     // protocol share of interest
    pub base_rate_bps: u16,          // APR base
    pub slope1_bps: u16,             // APR slope below kink
    pub slope2_bps: u16,             // APR slope above kink
    pub kink_bps: u16,               // utilization kink 0..10000
}
#[account]
pub struct Obligation{
    pub owner: Pubkey,
    pub deposits: Vec<CollateralPosition>,
    pub borrows: Vec<BorrowPosition>,
}
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CollateralPosition {
    pub reserve: Pubkey,
    pub amount: u64,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct BorrowPosition {
    pub reserve: Pubkey,
    pub scaled_principal: u128,
}