//! Checked fixed-point helpers (spec CAP-07).
//!
//! All functions return `Err` instead of wrapping or panicking, use a `u128`
//! intermediate, and take the rounding direction explicitly so callers pick
//! the side that is conservative for the protocol.
#![no_std]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MathError {
    Overflow,
    DivideByZero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Rounding {
    Down,
    Up,
}

/// `a * b / d` with a `u128` intermediate and explicit rounding.
pub fn mul_div(a: u64, b: u64, d: u64, rounding: Rounding) -> Result<u64, MathError> {
    if d == 0 {
        return Err(MathError::DivideByZero);
    }
    let product = u128::from(a)
        .checked_mul(u128::from(b))
        .ok_or(MathError::Overflow)?;
    let d = u128::from(d);
    let quotient = product.checked_div(d).ok_or(MathError::DivideByZero)?;
    let remainder = product.checked_rem(d).ok_or(MathError::DivideByZero)?;
    let rounded = match rounding {
        Rounding::Up if remainder != 0 => quotient.checked_add(1).ok_or(MathError::Overflow)?,
        _ => quotient,
    };
    u64::try_from(rounded).map_err(|_| MathError::Overflow)
}

/// Applies a basis-point fee to `amount`.
pub fn bps_of(amount: u64, bps: u64, rounding: Rounding) -> Result<u64, MathError> {
    mul_div(amount, bps, moka_types::units::BPS_DENOMINATOR, rounding)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mul_div_rounds_in_the_requested_direction() {
        assert_eq!(mul_div(10, 1, 3, Rounding::Down), Ok(3));
        assert_eq!(mul_div(10, 1, 3, Rounding::Up), Ok(4));
        assert_eq!(mul_div(9, 1, 3, Rounding::Up), Ok(3));
    }

    #[test]
    fn mul_div_rejects_bad_inputs() {
        assert_eq!(
            mul_div(1, 1, 0, Rounding::Down),
            Err(MathError::DivideByZero)
        );
        assert_eq!(
            mul_div(u64::MAX, 2, 1, Rounding::Down),
            Err(MathError::Overflow)
        );
        // The u128 intermediate lets large products divide back into range.
        assert_eq!(
            mul_div(u64::MAX, u64::MAX, u64::MAX, Rounding::Down),
            Ok(u64::MAX)
        );
    }

    #[test]
    fn bps_fee() {
        assert_eq!(bps_of(1_000_000, 30, Rounding::Up), Ok(3_000));
        assert_eq!(bps_of(1, 30, Rounding::Down), Ok(0));
        assert_eq!(bps_of(1, 30, Rounding::Up), Ok(1));
    }
}
