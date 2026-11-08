use super::Rational;
use crate::{err, exact::BigInt, Result};

/// Closed real enclosures with directed rounding to `precision_bits`
/// significant binary digits after each operation (§12.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Interval {
    lo: Rational,
    hi: Rational,
    precision_bits: usize,
}

impl Interval {
    pub fn point(value: Rational, precision_bits: usize) -> Result<Self> {
        Self::bounds(value.clone(), value, precision_bits)
    }
    pub fn bounds(lo: Rational, hi: Rational, precision_bits: usize) -> Result<Self> {
        if precision_bits == 0 {
            return err("§12.1", "precision_bits must be positive");
        }
        if lo > hi {
            return err("§12.1", "reversed interval endpoints");
        }
        Ok(Self {
            lo: round(&lo, precision_bits, false)?,
            hi: round(&hi, precision_bits, true)?,
            precision_bits,
        })
    }
    pub fn lower(&self) -> &Rational {
        &self.lo
    }
    pub fn upper(&self) -> &Rational {
        &self.hi
    }
    pub fn precision_bits(&self) -> usize {
        self.precision_bits
    }
    pub fn neg(&self) -> Self {
        Self {
            lo: self.hi.neg(),
            hi: self.lo.neg(),
            precision_bits: self.precision_bits,
        }
    }
    fn same_precision(&self, other: &Self) -> Result<()> {
        if self.precision_bits != other.precision_bits {
            return err("§12.1", "interval precision mismatch");
        }
        Ok(())
    }
    pub fn add(&self, other: &Self) -> Result<Self> {
        self.same_precision(other)?;
        Self::bounds(
            self.lo.add(&other.lo),
            self.hi.add(&other.hi),
            self.precision_bits,
        )
    }
    pub fn sub(&self, other: &Self) -> Result<Self> {
        self.add(&other.neg())
    }
    pub fn mul(&self, other: &Self) -> Result<Self> {
        self.same_precision(other)?;
        let products = [
            self.lo.mul(&other.lo),
            self.lo.mul(&other.hi),
            self.hi.mul(&other.lo),
            self.hi.mul(&other.hi),
        ];
        let lo = products.iter().min().expect("four endpoints").clone();
        let hi = products.iter().max().expect("four endpoints").clone();
        Self::bounds(lo, hi, self.precision_bits)
    }
    pub fn div(&self, other: &Self) -> Result<Self> {
        self.same_precision(other)?;
        let zero = Rational::integer(0);
        if other.lo <= zero && other.hi >= zero {
            return err("§12.1", "interval divisor contains zero");
        }
        let one = Rational::integer(1);
        let reciprocal = Self::bounds(
            one.div(&other.hi)?,
            one.div(&other.lo)?,
            self.precision_bits,
        )?;
        self.mul(&reciprocal)
    }
    pub fn square(&self) -> Result<Self> {
        let a = self.lo.mul(&self.lo);
        let b = self.hi.mul(&self.hi);
        let zero = Rational::integer(0);
        let lo = if self.lo <= zero && self.hi >= zero {
            zero
        } else {
            a.clone().min(b.clone())
        };
        Self::bounds(lo, a.max(b), self.precision_bits)
    }
    pub fn sqrt(&self) -> Result<Self> {
        if self.lo < Rational::integer(0) {
            return err("§12.1", "negative square-root interval");
        }
        Ok(Self {
            lo: sqrt_round(&self.lo, self.precision_bits, false)?,
            hi: sqrt_round(&self.hi, self.precision_bits, true)?,
            precision_bits: self.precision_bits,
        })
    }
}

fn quantum(value: &Rational, precision_bits: usize) -> Result<Rational> {
    let precision = i64::try_from(precision_bits)
        .map_err(|_| crate::resource_error("§12.1", "precision exceeds host range"))?;
    let exponent = value
        .binary_exponent()?
        .checked_sub(precision - 1)
        .ok_or_else(|| crate::resource_error("§12.1", "rounding exponent exceeds host range"))?;
    Ok(Rational::power_two(exponent))
}

fn round(value: &Rational, precision_bits: usize, up: bool) -> Result<Rational> {
    if *value == Rational::integer(0) {
        return Ok(value.clone());
    }
    let step = quantum(&value.abs(), precision_bits)?;
    let scaled = value.div(&step)?;
    Ok(if up { scaled.ceil() } else { scaled.floor() }.mul(&step))
}

fn sqrt_round(value: &Rational, precision_bits: usize, up: bool) -> Result<Rational> {
    if *value == Rational::integer(0) {
        return Ok(value.clone());
    }
    let exponent = value.binary_exponent()?.div_euclid(2);
    let precision = i64::try_from(precision_bits)
        .map_err(|_| crate::resource_error("§12.1", "precision exceeds host range"))?;
    let exponent = exponent
        .checked_sub(precision - 1)
        .ok_or_else(|| crate::resource_error("§12.1", "square-root exponent exceeds host range"))?;
    let step = Rational::power_two(exponent);
    let scaled = value.div(&step.mul(&step))?;
    let floor = scaled.n.div_rem(&scaled.d).0.sqrt_floor();
    let exact = floor.mul(&floor).mul(&scaled.d) == scaled.n;
    let numerator = if up && !exact {
        floor.add(&BigInt::from_i128(1))
    } else {
        floor
    };
    Ok(Rational::reduced(numerator, BigInt::from_i128(1)).mul(&step))
}
