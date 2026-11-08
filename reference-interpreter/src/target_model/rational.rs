use crate::{err, exact::BigInt, Error, Result};
use std::cmp::Ordering;

/// An arbitrary-precision reduced rational; no host floating arithmetic enters
/// target matching, model bindings or process-checker verdicts (§§8.1,12.1).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rational {
    pub(super) n: BigInt,
    pub(super) d: BigInt,
}

impl Rational {
    pub(super) fn power_two(exponent: i64) -> Self {
        let one = BigInt::from_i128(1);
        if exponent >= 0 {
            Self {
                n: one.shl(exponent as u64),
                d: one,
            }
        } else {
            Self {
                n: one.clone(),
                d: one.shl(exponent.unsigned_abs()),
            }
        }
    }
    pub(super) fn binary_exponent(&self) -> Result<i64> {
        assert!(self.n > BigInt::zero());
        let n_bits = i64::try_from(self.n.bit_len())
            .map_err(|_| crate::resource_error("§12.1", "numerator width exceeds host range"))?;
        let d_bits = i64::try_from(self.d.bit_len())
            .map_err(|_| crate::resource_error("§12.1", "denominator width exceeds host range"))?;
        let exponent = n_bits - d_bits;
        Ok(if *self < Self::power_two(exponent) {
            exponent - 1
        } else {
            exponent
        })
    }
    pub fn integer(value: i128) -> Self {
        Self {
            n: BigInt::from_i128(value),
            d: BigInt::from_i128(1),
        }
    }
    pub(super) fn reduced(n: BigInt, d: BigInt) -> Self {
        assert!(d > BigInt::zero());
        let (mut a, mut b) = (n.abs(), d.clone());
        while !b.is_zero() {
            let r = a.div_rem(&b).1;
            a = b;
            b = r;
        }
        let numerator = n.abs().div_rem(&a).0;
        Self {
            n: if n < BigInt::zero() {
                numerator.neg()
            } else {
                numerator
            },
            d: d.div_rem(&a).0,
        }
    }
    /// §8.3 exact model leaves must already use their reduced spelling.
    pub fn leaf(numerator: &str, denominator: &str) -> Result<Self> {
        let n = BigInt::decimal_integer(numerator)
            .ok_or_else(|| Error::new("§8.3: noncanonical rational numerator".into()))?;
        let d = BigInt::decimal_integer(denominator)
            .filter(|d| *d > BigInt::zero())
            .ok_or_else(|| Error::new("§8.3: rational denominator must be positive".into()))?;
        let value = Self::reduced(n.clone(), d.clone());
        if value.n != n || value.d != d || value.pair() != (numerator.into(), denominator.into()) {
            return err("§8.3", "rational leaf must be canonical and reduced");
        }
        Ok(value)
    }
    pub fn decimal(text: &str) -> Result<Self> {
        let (mantissa, exponent) = text.split_once(['e', 'E']).unwrap_or((text, "0"));
        let exponent: i64 = exponent
            .parse()
            .map_err(|_| crate::resource_error("§8.1", "decimal exponent exceeds host range"))?;
        let negative = mantissa.starts_with('-');
        let mantissa = mantissa.strip_prefix('-').unwrap_or(mantissa);
        let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        if whole.is_empty()
            || !whole
                .bytes()
                .chain(fraction.bytes())
                .all(|b| b.is_ascii_digit())
            || (whole.len() > 1 && whole.starts_with('0'))
            || mantissa.ends_with('.')
        {
            return err("§8.1", format!("invalid rational decimal {text}"));
        }
        let digits = format!("{whole}{fraction}");
        let mut n = BigInt::zero();
        for digit in digits.bytes() {
            n = n
                .mul(&BigInt::from_i128(10))
                .add(&BigInt::from_i128((digit - b'0') as i128));
        }
        if negative {
            n = n.neg();
        }
        let scale =
            exponent
                .checked_sub(i64::try_from(fraction.len()).map_err(|_| {
                    crate::resource_error("§8.1", "decimal length exceeds host range")
                })?)
                .ok_or_else(|| crate::resource_error("§8.1", "decimal scale exceeds host range"))?;
        let power = pow10(scale.unsigned_abs());
        Ok(if scale >= 0 {
            Self::reduced(n.mul(&power), BigInt::from_i128(1))
        } else {
            Self::reduced(n, power)
        })
    }
    /// §3.1/K20 binds finite IEEE-754 bytes, including subnormal angles.
    pub fn binary64(bits: u64) -> Result<Self> {
        let exponent = ((bits >> 52) & 0x7ff) as i64;
        if exponent == 0x7ff {
            return err("§3.1/§8.3", "nonfinite angle binding");
        }
        let mantissa = (bits & ((1 << 52) - 1)) | if exponent == 0 { 0 } else { 1 << 52 };
        let shift = if exponent == 0 {
            -1074
        } else {
            exponent - 1075
        };
        let n = BigInt::from_u128(mantissa as u128);
        let n = if bits >> 63 == 0 { n } else { n.neg() };
        Ok(if shift >= 0 {
            Self::reduced(n.shl(shift as u64), BigInt::from_i128(1))
        } else {
            Self::reduced(n, BigInt::from_i128(1).shl((-shift) as u64))
        })
    }
    pub fn pair(&self) -> (String, String) {
        (self.n.to_decimal(), self.d.to_decimal())
    }
    pub fn add(&self, other: &Self) -> Self {
        Self::reduced(
            self.n.mul(&other.d).add(&other.n.mul(&self.d)),
            self.d.mul(&other.d),
        )
    }
    pub fn neg(&self) -> Self {
        Self {
            n: self.n.neg(),
            d: self.d.clone(),
        }
    }
    pub fn abs(&self) -> Self {
        Self {
            n: self.n.abs(),
            d: self.d.clone(),
        }
    }
    pub fn sub(&self, other: &Self) -> Self {
        self.add(&other.neg())
    }
    pub fn mul(&self, other: &Self) -> Self {
        Self::reduced(self.n.mul(&other.n), self.d.mul(&other.d))
    }
    pub fn div(&self, other: &Self) -> Result<Self> {
        if other.n.is_zero() {
            return err("§8.1/§12.1", "division by zero");
        }
        let n = self.n.mul(&other.d);
        Ok(Self::reduced(
            if other.n < BigInt::zero() { n.neg() } else { n },
            self.d.mul(&other.n.abs()),
        ))
    }
    pub fn ceil(&self) -> Self {
        let (q, r) = self.n.abs().div_rem(&self.d);
        let n = if self.n < BigInt::zero() {
            q.neg()
        } else if r.is_zero() {
            q
        } else {
            q.add(&BigInt::from_i128(1))
        };
        Self {
            n,
            d: BigInt::from_i128(1),
        }
    }
    pub fn floor(&self) -> Self {
        self.neg().ceil().neg()
    }
}

impl Ord for Rational {
    fn cmp(&self, other: &Self) -> Ordering {
        self.n.mul(&other.d).cmp(&other.n.mul(&self.d))
    }
}
impl PartialOrd for Rational {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

fn pow10(mut exponent: u64) -> BigInt {
    let mut value = BigInt::from_i128(1);
    let mut power = BigInt::from_i128(10);
    while exponent != 0 {
        if exponent & 1 != 0 {
            value = value.mul(&power);
        }
        exponent >>= 1;
        if exponent != 0 {
            power = power.mul(&power);
        }
    }
    value
}
