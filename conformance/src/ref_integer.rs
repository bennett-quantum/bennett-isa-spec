//! Mathematical integers for constructor scalar expressions (K19/K23).
//! Host sizes are checked only where a count or wire index is consumed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Integer {
    negative: bool,
    words: Vec<u32>,
}
impl From<i64> for Integer {
    fn from(v: i64) -> Self {
        let n = v.unsigned_abs();
        Self::normal(v < 0, vec![n as u32, (n >> 32) as u32])
    }
}
impl PartialOrd for Integer {
    fn partial_cmp(&self, b: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(b))
    }
}
impl Ord for Integer {
    fn cmp(&self, b: &Self) -> std::cmp::Ordering {
        if self.negative != b.negative {
            return b.negative.cmp(&self.negative);
        }
        let order = self.magnitude_cmp(b);
        if self.negative {
            order.reverse()
        } else {
            order
        }
    }
}
impl Integer {
    fn normal(negative: bool, mut words: Vec<u32>) -> Self {
        while words.last() == Some(&0) {
            words.pop();
        }
        Self {
            negative: negative && !words.is_empty(),
            words,
        }
    }
    pub fn parse(s: &str) -> Result<Self, String> {
        let (negative, digits) = s.strip_prefix('-').map_or((false, s), |s| (true, s));
        if digits.is_empty()
            || !digits.bytes().all(|b| b.is_ascii_digit())
            || (digits.len() > 1 && digits.starts_with('0'))
        {
            return Err("invalid integer literal".into());
        }
        let mut n = Self::from(0);
        for digit in digits.bytes() {
            n = n
                .mul(&Self::from(10))
                .add(&Self::from((digit - b'0') as i64));
        }
        n.negative = negative && !n.words.is_empty();
        Ok(n)
    }
    fn magnitude_cmp(&self, b: &Self) -> std::cmp::Ordering {
        self.words
            .len()
            .cmp(&b.words.len())
            .then_with(|| self.words.iter().rev().cmp(b.words.iter().rev()))
    }
    fn magnitude_add(&self, b: &Self) -> Vec<u32> {
        let mut out = Vec::new();
        let mut carry = 0u64;
        for i in 0..self.words.len().max(b.words.len()) {
            let n = self.words.get(i).copied().unwrap_or(0) as u64
                + b.words.get(i).copied().unwrap_or(0) as u64
                + carry;
            out.push(n as u32);
            carry = n >> 32;
        }
        if carry != 0 {
            out.push(carry as u32);
        }
        out
    }
    fn magnitude_sub(&self, b: &Self) -> Vec<u32> {
        let mut out = Vec::new();
        let mut borrow = 0u64;
        for i in 0..self.words.len() {
            let value = self.words[i] as u64;
            let sub = b.words.get(i).copied().unwrap_or(0) as u64 + borrow;
            out.push(value.wrapping_sub(sub) as u32);
            borrow = u64::from(value < sub);
        }
        Self::normal(false, out).words
    }
    pub fn add(&self, b: &Self) -> Self {
        if self.negative == b.negative {
            Self::normal(self.negative, self.magnitude_add(b))
        } else if self.magnitude_cmp(b).is_ge() {
            Self::normal(self.negative, self.magnitude_sub(b))
        } else {
            Self::normal(b.negative, b.magnitude_sub(self))
        }
    }
    pub fn sub(&self, b: &Self) -> Self {
        let mut b = b.clone();
        if !b.words.is_empty() {
            b.negative = !b.negative;
        }
        self.add(&b)
    }
    pub fn mul(&self, b: &Self) -> Self {
        let mut out = vec![0u32; self.words.len() + b.words.len()];
        for (i, a) in self.words.iter().enumerate() {
            let mut carry = 0u64;
            for (j, b) in b.words.iter().enumerate() {
                let n = out[i + j] as u64 + (*a as u64) * (*b as u64) + carry;
                out[i + j] = n as u32;
                carry = n >> 32;
            }
            if !b.words.is_empty() {
                out[i + b.words.len()] = carry as u32;
            }
        }
        Self::normal(self.negative != b.negative, out)
    }
    fn bits(&self) -> usize {
        self.words.last().map_or(0, |w| {
            (self.words.len() - 1) * 32 + (32 - w.leading_zeros()) as usize
        })
    }
    fn bit(&self, i: usize) -> bool {
        self.words
            .get(i / 32)
            .is_some_and(|w| w >> (i % 32) & 1 != 0)
    }
    fn shift_one(&mut self, bit: bool) {
        let mut carry = u32::from(bit);
        for w in &mut self.words {
            let next = *w >> 31;
            *w = (*w << 1) | carry;
            carry = next;
        }
        if carry != 0 {
            self.words.push(carry);
        }
    }
    pub fn div_mod(&self, b: &Self) -> Result<(Self, Self), String> {
        if b.negative || b.words.is_empty() {
            return Err("division needs a positive divisor".into());
        }
        let mut q = vec![0u32; self.words.len()];
        let mut r = Self::from(0);
        for i in (0..self.bits()).rev() {
            r.shift_one(self.bit(i));
            if r.magnitude_cmp(b).is_ge() {
                r.words = r.magnitude_sub(b);
                q[i / 32] |= 1 << (i % 32);
            }
        }
        let q = Self::normal(false, q);
        if self.negative {
            if r.words.is_empty() {
                let mut q = q;
                q.negative = !q.words.is_empty();
                Ok((q, r))
            } else {
                let mut q = q.add(&Self::from(1));
                q.negative = true;
                Ok((q, b.sub(&r)))
            }
        } else {
            Ok((q, r))
        }
    }
    pub fn bit_at(&self, i: &Self) -> Result<Self, String> {
        if self.negative || i.negative {
            return Err("bit needs nonnegative operands".into());
        }
        let Some(index) = i.to_i64().and_then(|i| usize::try_from(i).ok()) else {
            return Ok(Self::from(0));
        };
        Ok(Self::from(i64::from(self.bit(index))))
    }
    pub fn to_i64(&self) -> Option<i64> {
        if self.words.len() > 2 {
            return None;
        }
        let n = self.words.first().copied().unwrap_or(0) as u64
            | ((self.words.get(1).copied().unwrap_or(0) as u64) << 32);
        if self.negative {
            if n == 1 << 63 {
                Some(i64::MIN)
            } else {
                i64::try_from(n).ok().map(|n| -n)
            }
        } else {
            i64::try_from(n).ok()
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_euclidean_arithmetic() {
        for a in -97i64..98 {
            for b in -31i64..32 {
                let x = Integer::from(a);
                let y = Integer::from(b);
                assert_eq!(x.add(&y).to_i64(), Some(a + b));
                assert_eq!(x.sub(&y).to_i64(), Some(a - b));
                assert_eq!(x.mul(&y).to_i64(), Some(a * b));
                if b > 0 {
                    let (q, r) = x.div_mod(&y).unwrap();
                    assert_eq!(q.to_i64(), Some(a.div_euclid(b)));
                    assert_eq!(r.to_i64(), Some(a.rem_euclid(b)));
                }
            }
        }
    }
}
