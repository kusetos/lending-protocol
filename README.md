# Solana Lending Protocol

## Overview

This is a Solana-based lending protocol built using the Anchor framework. The protocol allows users to deposit SOL as collateral, borrow stablecoins against it, repay loans with interest, and withdraw their collateral. The program enforces a fixed 5% interest rate and a 75% Loan-to-Value (LTV) ratio to ensure safe lending practices.

## Features

- **Initialize**: Sets up the lending pool with initial state.
- **Deposit**: Users can deposit SOL into the lending pool as collateral.
- **Borrow**: Users can borrow stablecoins against their deposited SOL, up to 75% of the collateral's value.
- **Repay**: Users can repay their stablecoin debt, including 5% interest.
- **Withdraw**: Users can withdraw their SOL collateral, provided it doesn't violate the LTV ratio.

## Program Structure

The smart contract is implemented in Rust using Anchor. Key components include:

### Accounts

- **LendingPool**: Stores the total SOL deposited, total stablecoins borrowed, and the authority's public key.
- **UserAccount**: Tracks each user's deposited SOL, stablecoin debt, and owner public key.
- **SolVault**: A Program Derived Address (PDA) that holds the deposited SOL.

### Instructions

- `initialize`: Initializes the lending pool.
- `deposit`: Transfers SOL from the user to the vault and updates user and pool state.
- `borrow`: Allows borrowing stablecoins based on collateral value and LTV ratio.
- `repay`: Handles repayment of stablecoin debt with interest.
- `withdraw`: Transfers SOL back to the user, ensuring the remaining collateral supports the debt.

### Constants

- **INTEREST_RATE**: Fixed at 5%.
- **LTV_RATIO**: Fixed at 75%.

### Error Codes

- `LtvExceeded`: Triggered when borrowing exceeds the allowed LTV ratio.
- `InsufficientCollateral`: Triggered when collateral is insufficient for withdrawal or borrowing.
- `Undercollateralized`: Triggered when withdrawal would leave the account undercollateralized.
- `InsufficientRepayment`: Triggered when repayment is less than the total amount due.

## Usage

1. **Initialize the Lending Pool**: Call the `initialize` instruction to set up the lending pool.

2. **Deposit SOL**: Use the `deposit` instruction to deposit SOL into the vault.

3. **Borrow Stablecoins**: Use the `borrow` instruction to borrow stablecoins against your SOL collateral.

4. **Repay Debt**: Use the `repay` instruction to repay the borrowed stablecoins plus interest.

5. **Withdraw SOL**: Use the `withdraw` instruction to retrieve your SOL, ensuring the LTV ratio is maintained.

## License

This project is licensed under the MIT License.