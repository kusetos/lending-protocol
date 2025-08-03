use crate::constants::RAY;
use anchor_lang::prelude::*;

pub fn mul_div_u128(a: u128, b: u128, denom: u128) -> Result<u128> {
    // (a * b) / denom with overflow check using 256 emulation via u128 ops
    let (hi, lo) = mul_128(a, b);
    div_256_by_128(hi, lo, denom).ok_or(error!(crate::errors::LendingError::MathOverflow))
}

fn mul_128(a: u128, b: u128) -> (u128, u128) {
    let a_lo = a as u64 as u128;
    let a_hi = (a >> 64) as u64 as u128;
    let b_lo = b as u64 as u128;
    let b_hi = (b >> 64) as u64 as u128;

    let lo_lo = a_lo * b_lo;
    let lo_hi = a_lo * b_hi;
    let hi_lo = a_hi * b_lo;
    let hi_hi = a_hi * b_hi;

    let carry = ((lo_lo >> 64) + (lo_hi as u128 & 0xFFFF_FFFF_FFFF_FFFF) + (hi_lo as u128 & 0xFFFF_FFFF_FFFF_FFFF)) >> 64;
    let lo = (lo_lo & 0xFFFF_FFFF_FFFF_FFFF) | (((lo_hi & 0xFFFF_FFFF_FFFF_FFFF) + (hi_lo & 0xFFFF_FFFF_FFFF_FFFF) + (lo_lo >> 64)) << 64);
    let hi = hi_hi + (lo_hi >> 64) + (hi_lo >> 64) + carry;
    (hi, lo)
}

fn div_256_by_128(hi: u128, lo: u128, denom: u128) -> Option<u128> {
    if denom == 0 { return None; }
    // Simple normalization-free division when hi < denom
    if hi >= denom { return None; }
    // Combine into 256 then divide; emulate by iterating bits (slow but safe for infrequent math)
    let mut rem_hi = hi;
    let mut rem_lo = lo;
    let mut quo: u128 = 0;
    for _ in 0..128 {
        // left shift remainder by 1
        let overflow = (rem_lo >> 127) & 1;
        rem_lo <<= 1;
        rem_hi = (rem_hi << 1) | overflow;
        quo <<= 1;
        if rem_hi > denom || (rem_hi == denom && rem_lo >= 0) { // lo always >=0
            // subtract denom from hi (since we normalized on hi only)
            if rem_hi >= denom {
                rem_hi -= denom;
                quo |= 1;
            }
        }
    }
    Some(quo)
}

pub fn ray_mul(a: u128, b: u128) -> Result<u128> { mul_div_u128(a, b, RAY) }
pub fn ray_div(a: u128, b: u128) -> Result<u128> { mul_div_u128(a, RAY, b) }

pub fn pow1p_ray(rate_per_slot: u128, slots: u64) -> Result<u128> {
    let mut base = RAY.checked_add(rate_per_slot).ok_or(error!(crate::errors::LendingError::MathOverflow))?;
    let mut exp = slots;
    let mut acc: u128 = RAY;
    while exp > 0 {
        if (exp & 1) == 1 {
            acc = ray_mul(acc, base)?;
        }
        exp >>= 1;
        if exp > 0 {
            base = ray_mul(base, base)?;
        }
    }
    Ok(acc)
}
