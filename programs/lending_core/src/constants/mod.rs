pub const VERSION: u8 = 1;
pub const MAX_DEPOSIT_LEGS: usize = 8;
pub const MAX_BORROW_LEGS: usize = 8;

pub const RAY: u128 = 1_000_000_000_000_000_000_000_000_000u128;

pub const SLOTS_PER_YEAR: u64 = 63_072_000;

pub mod pause {
    pub const DEPOSIT: u64 = 1 << 0;
    pub const REDEEM: u64 = 1 << 1;
    pub const BORROW: u64 = 1 << 2;
    pub const REPAY: u64 = 1 << 3;
    pub const LIQUIDATE: u64 = 1 << 4;
}