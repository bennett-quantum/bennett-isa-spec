//! Concrete little-endian bit vectors, with no host integer width restriction.
use crate::{err, Error, Result};
use std::cmp::Ordering;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BitVector {
    width: usize,
    words: Vec<u64>,
}
impl BitVector {
    pub fn zero(width: usize) -> Result<Self> {
        if width == 0 {
            return err("§3.3", "bit-vector width must be positive");
        }
        let count = width.checked_add(63).ok_or_else(|| {
            Error("§3.3: host limit: bit-vector width plus word padding exceeds usize".into())
        })? / 64;
        let mut words = Vec::new();
        words.try_reserve_exact(count).map_err(|e| {
            Error(format!(
                "§3.3: host limit: bit-vector storage allocation failed: {e}"
            ))
        })?;
        words.resize(count, 0);
        Ok(Self { width, words })
    }
    /// `value` as a bit-vector of `width` bits; an unrepresentable value is an
    /// error, never a truncation.
    pub fn from_u128_exact(width: usize, value: u128) -> Result<Self> {
        if width < 128 && value >> width != 0 {
            return err("§3.3", format!("value {value} does not fit {width} bits"));
        }
        Self::from_u128(width, value)
    }
    /// XOR `src` into the low `src.width()` bits of `self`.
    pub fn xor_low(&mut self, src: &Self) -> Result<()> {
        if src.width > self.width {
            return err("§3.6", "XOR source is wider than its destination");
        }
        for (d, s) in self.words.iter_mut().zip(&src.words) {
            *d ^= s;
        }
        Ok(())
    }
    /// True iff every bit at index >= n is zero.
    pub fn high_clear(&self, n: usize) -> bool {
        (n..self.width).all(|i| !self.get(i))
    }
    /// Exchange the low `n` bits of two vectors (§3.7: a location of width N
    /// exchanged with a wider cell exchanges the cell's low N bits).
    pub fn swap_low(&mut self, other: &mut Self, n: usize) -> Result<()> {
        if n > self.width || n > other.width {
            return err("§3.6", "exchange wider than an operand");
        }
        for i in 0..n {
            let (a, b) = (self.get(i), other.get(i));
            self.put(i, b);
            other.put(i, a);
        }
        Ok(())
    }
    pub fn width(&self) -> usize {
        self.width
    }
    pub fn is_zero(&self) -> bool {
        self.words.iter().all(|w| *w == 0)
    }
    pub fn from_u128(width: usize, value: u128) -> Result<Self> {
        let mut out = Self::zero(width)?;
        out.words[0] = value as u64;
        if out.words.len() > 1 {
            out.words[1] = (value >> 64) as u64;
        }
        out.trim();
        Ok(out)
    }
    pub fn from_le_bytes(width: usize, bytes: &[u8]) -> Result<Self> {
        let mut out = Self::zero(width)?;
        if bytes.len() != width.div_ceil(8) {
            return err("§3.3", "byte count differs from bit-vector width");
        }
        if !width.is_multiple_of(8) && bytes.last().is_some_and(|b| *b >> (width % 8) != 0) {
            return err("§3.3", "nonzero high padding bits");
        }
        for (i, b) in bytes.iter().enumerate() {
            out.words[i / 8] |= (*b as u64) << ((i % 8) * 8);
        }
        Ok(out)
    }
    pub fn to_le_bytes(&self) -> Vec<u8> {
        (0..self.width.div_ceil(8))
            .map(|i| (self.words[i / 8] >> ((i % 8) * 8)) as u8)
            .collect()
    }
    pub fn parse_decimal(width: usize, s: &str) -> Result<Self> {
        if !crate::object::integer(s) {
            return err("§16.1", "invalid canonical integer literal");
        }
        let mut out = Self::zero(width)?;
        for digit in s.trim_start_matches('-').bytes() {
            let mut carry = (digit - b'0') as u128;
            for word in &mut out.words {
                let n = (*word as u128) * 10 + carry;
                *word = n as u64;
                carry = n >> 64;
            }
            out.trim();
        }
        if s.starts_with('-') {
            out = out.negate();
        }
        Ok(out)
    }
    pub fn parse_unsigned_exact(width: usize, s: &str) -> Result<Self> {
        if !crate::object::integer(s) || s.starts_with('-') {
            return err(
                "§16.1",
                format!("unsigned canonical INT required, found {s}"),
            );
        }
        let generous = s
            .len()
            .checked_mul(4)
            .ok_or_else(|| Error("§3.3: host limit: literal spelling size exceeds usize".into()))?
            .max(width);
        let value = Self::parse_decimal(generous, s)?;
        if (width..generous).any(|i| value.get(i)) {
            return err("§3.3", "integer literal exceeds declared width");
        }
        value.resize(width, false)
    }
    pub fn to_decimal(&self) -> String {
        if self.is_zero() {
            return "0".into();
        }
        let mut words = self.words.clone();
        let mut digits = Vec::new();
        while words.iter().any(|w| *w != 0) {
            let mut remainder = 0u128;
            for word in words.iter_mut().rev() {
                let n = (remainder << 64) | (*word as u128);
                *word = (n / 10) as u64;
                remainder = n % 10;
            }
            digits.push(b'0' + remainder as u8);
        }
        digits.reverse();
        String::from_utf8(digits).expect("decimal ASCII")
    }
    pub fn to_u128(&self) -> Result<u128> {
        if self.words.iter().skip(2).any(|w| *w != 0) {
            return err("§3.3", "bit-vector value exceeds u128");
        }
        Ok(self.words[0] as u128 | ((self.words.get(1).copied().unwrap_or(0) as u128) << 64))
    }
    pub fn bit(&self, index: usize) -> Result<bool> {
        if index >= self.width {
            return err("§3.3", "bit index out of range");
        }
        Ok(self.get(index))
    }
    pub fn set(&mut self, index: usize, value: bool) -> Result<()> {
        if index >= self.width {
            return err("§3.3", "bit index out of range");
        }
        self.put(index, value);
        Ok(())
    }
    fn get(&self, index: usize) -> bool {
        self.words[index / 64] & (1u64 << (index % 64)) != 0
    }
    fn put(&mut self, index: usize, value: bool) {
        let bit = 1u64 << (index % 64);
        if value {
            self.words[index / 64] |= bit;
        } else {
            self.words[index / 64] &= !bit;
        }
    }
    fn trim(&mut self) {
        if !self.width.is_multiple_of(64) {
            *self.words.last_mut().expect("positive width") &= (1u64 << (self.width % 64)) - 1;
        }
    }
    fn same(&self, other: &Self) -> Result<()> {
        if self.width != other.width {
            err("§3.3", "bit-vector operand width mismatch")
        } else {
            Ok(())
        }
    }
    pub fn resize(&self, width: usize, sign_extend: bool) -> Result<Self> {
        let mut out = Self::zero(width)?;
        for i in 0..width.min(self.width) {
            out.put(i, self.get(i));
        }
        if sign_extend && width > self.width && self.get(self.width - 1) {
            for i in self.width..width {
                out.put(i, true);
            }
        }
        Ok(out)
    }
    pub fn unsigned_cmp(&self, other: &Self) -> Result<Ordering> {
        self.same(other)?;
        Ok(self.words.iter().rev().cmp(other.words.iter().rev()))
    }
    pub fn signed_cmp(&self, other: &Self) -> Result<Ordering> {
        self.same(other)?;
        let a = self.get(self.width - 1);
        let b = other.get(self.width - 1);
        if a != b {
            Ok(if a { Ordering::Less } else { Ordering::Greater })
        } else {
            self.unsigned_cmp(other)
        }
    }
    fn negate(&self) -> Self {
        let mut out = self.clone();
        let mut carry = true;
        for word in &mut out.words {
            let (n, c) = (!*word).overflowing_add(u64::from(carry));
            *word = n;
            carry = c;
        }
        out.trim();
        out
    }
    pub fn binary(&self, op: &str, other: &Self) -> Result<Self> {
        self.same(other)?;
        let mut out = Self::zero(self.width)?;
        match op {
            "and" | "or" | "xor" => {
                for ((out, a), b) in out.words.iter_mut().zip(&self.words).zip(&other.words) {
                    *out = match op {
                        "and" => a & b,
                        "or" => a | b,
                        _ => a ^ b,
                    };
                }
            }
            "add" => {
                let mut carry = 0u128;
                for ((out, a), b) in out.words.iter_mut().zip(&self.words).zip(&other.words) {
                    let n = *a as u128 + *b as u128 + carry;
                    *out = n as u64;
                    carry = n >> 64;
                }
            }
            "sub" => return self.binary("add", &other.negate()),
            "mul" => {
                for (i, a) in self.words.iter().enumerate() {
                    let mut carry = 0u128;
                    for (j, b) in other.words.iter().enumerate().take(self.words.len() - i) {
                        let n = (*a as u128) * (*b as u128) + out.words[i + j] as u128 + carry;
                        out.words[i + j] = n as u64;
                        carry = n >> 64;
                    }
                }
            }
            "shl" | "lshr" | "ashr" => {
                let shift = other
                    .to_u128()
                    .ok()
                    .and_then(|n| usize::try_from(n).ok())
                    .unwrap_or(usize::MAX);
                for i in 0..self.width {
                    let bit = if op == "shl" {
                        i.checked_sub(shift).is_some_and(|j| self.get(j))
                    } else if shift >= self.width {
                        // §4.1: "shift >= N -> 0", for ashr as for shl and lshr.
                        false
                    } else if let Some(j) = i.checked_add(shift).filter(|j| *j < self.width) {
                        self.get(j)
                    } else {
                        op == "ashr" && self.get(self.width - 1)
                    };
                    out.put(i, bit);
                }
            }
            "udiv" | "urem" | "sdiv" | "srem" => {
                let (q, r) = self.div_rem(other, op.starts_with('s'))?;
                return Ok(if op.ends_with("div") { q } else { r });
            }
            _ => return err("§4.8", format!("unknown bit-vector operation {op}")),
        }
        out.trim();
        Ok(out)
    }
    /// The §4.2 lift.div flag: Some(5) for a zero divisor, Some(6) for signed
    /// MIN / -1, else None.
    pub fn division_fault(&self, other: &Self, signed: bool) -> Result<Option<u128>> {
        self.same(other)?;
        if other.is_zero() {
            return Ok(Some(5));
        }
        let min = {
            let mut m = Self::zero(self.width)?;
            m.put(self.width - 1, true);
            m
        };
        let minus_one = Self::zero(self.width)?.negate_one();
        Ok((signed && *self == min && *other == minus_one).then_some(6))
    }
    fn negate_one(mut self) -> Self {
        self.words.fill(u64::MAX);
        self.trim();
        self
    }
    /// icmp.{eq,ne,ult,ule,slt,sle}.
    pub fn compare(&self, pred: &str, other: &Self) -> Result<bool> {
        let order = if pred.starts_with('s') {
            self.signed_cmp(other)?
        } else {
            self.unsigned_cmp(other)?
        };
        Ok(match pred {
            "eq" => order.is_eq(),
            "ne" => !order.is_eq(),
            "ult" | "slt" => order.is_lt(),
            "ule" | "sle" => !order.is_gt(),
            _ => return err("§4.8", format!("unknown comparison predicate {pred}")),
        })
    }
    pub fn div_rem(&self, other: &Self, signed: bool) -> Result<(Self, Self)> {
        self.same(other)?;
        if other.is_zero() {
            return err("§5.4", "FAULT_UB_DIV0 (5)");
        }
        let negative_a = signed && self.get(self.width - 1);
        let negative_b = signed && other.get(self.width - 1);
        if negative_a
            && self.words.iter().enumerate().all(|(i, w)| {
                *w == if i == (self.width - 1) / 64 {
                    1u64 << ((self.width - 1) % 64)
                } else {
                    0
                }
            })
            && other.words.iter().enumerate().all(|(i, w)| {
                *w == if i + 1 == other.words.len() && !self.width.is_multiple_of(64) {
                    (1u64 << (self.width % 64)) - 1
                } else {
                    u64::MAX
                }
            })
        {
            return err("§5.4", "FAULT_UB_SDIV_OVERFLOW (6)");
        }
        let a = if negative_a {
            self.negate()
        } else {
            self.clone()
        };
        let b = if negative_b {
            other.negate()
        } else {
            other.clone()
        };
        let mut q = Self::zero(self.width)?;
        let mut r = Self::zero(self.width)?;
        for i in (0..self.width).rev() {
            // r<b before the shift. Preserve the one extra comparison bit.
            let overflow = r.get(self.width - 1);
            let mut carry = u64::from(a.get(i));
            for word in &mut r.words {
                let next = *word >> 63;
                *word = (*word << 1) | carry;
                carry = next;
            }
            r.trim();
            if overflow || r.unsigned_cmp(&b)? != Ordering::Less {
                r = r.binary("sub", &b)?;
                q.put(i, true);
            }
        }
        if negative_a != negative_b {
            q = q.negate();
        }
        if negative_a {
            r = r.negate();
        }
        Ok((q, r))
    }
}
