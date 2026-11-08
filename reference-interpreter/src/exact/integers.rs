use super::BigInt;

impl BigInt {
    pub(crate) fn abs(&self) -> Self {
        Self {
            negative: false,
            limbs: self.limbs.clone(),
        }
    }

    pub(crate) fn bit_len(&self) -> usize {
        match self.limbs.last() {
            None => 0,
            Some(last) => (self.limbs.len() - 1) * 32 + (32 - last.leading_zeros()) as usize,
        }
    }

    /// Nonnegative Euclidean division. Callers establish d > 0.
    pub(crate) fn div_rem(&self, d: &Self) -> (Self, Self) {
        assert!(!self.negative && !d.negative && !d.is_zero());
        if self < d {
            return (Self::zero(), self.clone());
        }
        let mut quotient = Self::zero();
        let mut remainder = self.clone();
        for bit in (0..=self.bit_len() - d.bit_len()).rev() {
            let shifted = d.shl(bit as u64);
            if remainder >= shifted {
                remainder = remainder.add(&shifted.neg());
                quotient = quotient.add(&Self::from_i128(1).shl(bit as u64));
            }
        }
        (quotient, remainder)
    }

    pub(crate) fn decimal_integer(text: &str) -> Option<Self> {
        if !crate::object::integer(text) {
            return None;
        }
        let mut value = Self::zero();
        for digit in text.trim_start_matches('-').bytes() {
            value = value
                .mul_small(10)
                .add(&Self::from_i128((digit - b'0') as i128));
        }
        Some(if text.starts_with('-') {
            value.neg()
        } else {
            value
        })
    }
}
