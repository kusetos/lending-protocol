use anchor_lang::prelude::*;

#[error_code]
pub enum LendingError {
    #[msg("Action is paused")]
    Paused,
    #[msg("Math overflow/underflow")]
    MathOverflow,
    #[msg("Invalid oracle data")]
    InvalidOracle,
    #[msg("Health check failed")]
    Unhealthy,
    #[msg("Obligation not found or mismatch")]
    InvalidObligation,
    #[msg("Reserve mismatch or invalid")]
    InvalidReserve,
    #[msg("Unauthorized")] 
    Unauthorized,
    #[msg("Insufficient liquidity in reserve vault")]
    InsufficientLiquidity,
    #[msg("Amount must be > 0")]
    ZeroAmount,
}