use anchor_lang::prelude::*;

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