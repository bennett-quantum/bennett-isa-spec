//! Exact amplitude arithmetic for the §14.2 state-vector simulator.
//!
//! Every amplitude the gates of §4.4 can produce from |0...0> lies in the ring
//! K[z, 1/z] of Laurent polynomials over K = Q(sqrt 2, i) with dyadic rational
//! coordinates, where z = e^{i 2^-1075}: h contributes sqrt(2)/2, s and t
//! contribute i and (1+i) sqrt(2)/2, and a rotation by a finite binary64
//! angle theta contributes e^{+-i theta/2} = z^{+-m} with the exact integer
//! m = theta * 2^1074 (theta/2 is an integer multiple of 2^-1075 because every
//! finite binary64 value is an integer multiple of 2^-1074).
//!
//! z is transcendental over the algebraic numbers (Lindemann-Weierstrass:
//! e^alpha is transcendental for every nonzero algebraic alpha, here
//! alpha = i 2^-1075), so a Laurent polynomial in z is zero iff all of its
//! coefficients are zero. Cleanliness (§3.3 "clean on every basis
//! configuration", §4.3 qfree) is therefore decided exactly: no tolerance.
use std::cmp::Ordering;
use std::collections::BTreeMap;

/// Signed arbitrary-precision integer, magnitude in little-endian u32 limbs.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct BigInt {
    negative: bool,
    limbs: Vec<u32>,
}
impl BigInt {
    pub fn zero() -> Self {
        Self::default()
    }
    pub fn from_i128(v: i128) -> Self {
        let negative = v < 0;
        let mut m = v.unsigned_abs();
        let mut limbs = Vec::new();
        while m != 0 {
            limbs.push(m as u32);
            m >>= 32;
        }
        Self { negative, limbs }
    }
    pub fn is_zero(&self) -> bool {
        self.limbs.is_empty()
    }
    fn normalize(mut self) -> Self {
        while self.limbs.last() == Some(&0) {
            self.limbs.pop();
        }
        if self.limbs.is_empty() {
            self.negative = false;
        }
        self
    }
    fn cmp_magnitude(a: &[u32], b: &[u32]) -> Ordering {
        a.len()
            .cmp(&b.len())
            .then_with(|| a.iter().rev().cmp(b.iter().rev()))
    }
    fn add_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len().max(b.len()) + 1);
        let mut carry = 0u64;
        for i in 0..a.len().max(b.len()) {
            let s = *a.get(i).unwrap_or(&0) as u64 + *b.get(i).unwrap_or(&0) as u64 + carry;
            out.push(s as u32);
            carry = s >> 32;
        }
        if carry != 0 {
            out.push(carry as u32);
        }
        out
    }
    /// a - b for |a| >= |b|.
    fn sub_magnitude(a: &[u32], b: &[u32]) -> Vec<u32> {
        let mut out = Vec::with_capacity(a.len());
        let mut borrow = 0i64;
        for (i, x) in a.iter().enumerate() {
            let mut d = *x as i64 - *b.get(i).unwrap_or(&0) as i64 - borrow;
            borrow = 0;
            if d < 0 {
                d += 1 << 32;
                borrow = 1;
            }
            out.push(d as u32);
        }
        out
    }
    pub fn add(&self, other: &Self) -> Self {
        if self.negative == other.negative {
            return Self {
                negative: self.negative,
                limbs: Self::add_magnitude(&self.limbs, &other.limbs),
            }
            .normalize();
        }
        match Self::cmp_magnitude(&self.limbs, &other.limbs) {
            Ordering::Equal => Self::zero(),
            Ordering::Greater => Self {
                negative: self.negative,
                limbs: Self::sub_magnitude(&self.limbs, &other.limbs),
            }
            .normalize(),
            Ordering::Less => Self {
                negative: other.negative,
                limbs: Self::sub_magnitude(&other.limbs, &self.limbs),
            }
            .normalize(),
        }
    }
    pub fn neg(&self) -> Self {
        Self {
            negative: !self.negative && !self.is_zero(),
            limbs: self.limbs.clone(),
        }
    }
    pub fn mul(&self, other: &Self) -> Self {
        if self.is_zero() || other.is_zero() {
            return Self::zero();
        }
        let mut out = vec![0u32; self.limbs.len() + other.limbs.len()];
        for (i, a) in self.limbs.iter().enumerate() {
            let mut carry = 0u64;
            for (j, b) in other.limbs.iter().enumerate() {
                let t = out[i + j] as u64 + (*a as u64) * (*b as u64) + carry;
                out[i + j] = t as u32;
                carry = t >> 32;
            }
            let mut k = i + other.limbs.len();
            while carry != 0 {
                let t = out[k] as u64 + carry;
                out[k] = t as u32;
                carry = t >> 32;
                k += 1;
            }
        }
        Self {
            negative: self.negative != other.negative,
            limbs: out,
        }
        .normalize()
    }
    pub fn shl(&self, bits: u64) -> Self {
        if self.is_zero() {
            return Self::zero();
        }
        let words = (bits / 32) as usize;
        let shift = (bits % 32) as u32;
        let mut limbs = vec![0u32; words];
        let mut carry = 0u32;
        for l in &self.limbs {
            limbs.push((l << shift) | carry);
            carry = if shift == 0 { 0 } else { l >> (32 - shift) };
        }
        if carry != 0 {
            limbs.push(carry);
        }
        Self {
            negative: self.negative,
            limbs,
        }
        .normalize()
    }
    pub fn trailing_zeros(&self) -> u64 {
        let mut n = 0u64;
        for l in &self.limbs {
            if *l == 0 {
                n += 32;
            } else {
                return n + l.trailing_zeros() as u64;
            }
        }
        n
    }
    /// Exact right shift of a value divisible by 2^bits.
    fn shr_exact(&self, bits: u64) -> Self {
        let words = (bits / 32) as usize;
        let shift = (bits % 32) as u32;
        let src = &self.limbs[words.min(self.limbs.len())..];
        let mut limbs = Vec::with_capacity(src.len());
        for (i, l) in src.iter().enumerate() {
            let high = if shift == 0 {
                0
            } else {
                src.get(i + 1).map_or(0, |h| h << (32 - shift))
            };
            limbs.push((l >> shift) | high);
        }
        Self {
            negative: self.negative,
            limbs,
        }
        .normalize()
    }
}
impl PartialOrd for BigInt {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for BigInt {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.negative, other.negative) {
            (false, true) => Ordering::Greater,
            (true, false) => Ordering::Less,
            (false, false) => Self::cmp_magnitude(&self.limbs, &other.limbs),
            (true, true) => Self::cmp_magnitude(&other.limbs, &self.limbs),
        }
    }
}

/// A dyadic rational n * 2^-k, normalized (n odd or k = 0).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Dyadic {
    n: BigInt,
    k: u64,
}
impl Dyadic {
    pub fn int(v: i128) -> Self {
        Self {
            n: BigInt::from_i128(v),
            k: 0,
        }
    }
    /// v * 2^-k.
    pub fn scaled(v: i128, k: u64) -> Self {
        Self {
            n: BigInt::from_i128(v),
            k,
        }
        .normalize()
    }
    fn normalize(mut self) -> Self {
        if self.n.is_zero() {
            self.k = 0;
            return self;
        }
        let t = self.n.trailing_zeros().min(self.k);
        if t > 0 {
            self.n = self.n.shr_exact(t);
            self.k -= t;
        }
        self
    }
    pub fn is_zero(&self) -> bool {
        self.n.is_zero()
    }
    pub fn add(&self, o: &Self) -> Self {
        let k = self.k.max(o.k);
        Self {
            n: self.n.shl(k - self.k).add(&o.n.shl(k - o.k)),
            k,
        }
        .normalize()
    }
    pub fn neg(&self) -> Self {
        Self {
            n: self.n.neg(),
            k: self.k,
        }
    }
    pub fn mul(&self, o: &Self) -> Self {
        Self {
            n: self.n.mul(&o.n),
            k: self.k + o.k,
        }
        .normalize()
    }
    pub fn to_f64(&self) -> f64 {
        let mut v = 0f64;
        for (i, l) in self.n.limbs.iter().enumerate() {
            v += *l as f64 * 2f64.powi(32 * i as i32);
        }
        if self.n.negative {
            v = -v;
        }
        v * 2f64.powi(-(self.k.min(i32::MAX as u64) as i32))
    }
}
/// a + b sqrt(2), a and b dyadic.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Root2 {
    a: Dyadic,
    b: Dyadic,
}
impl Root2 {
    fn is_zero(&self) -> bool {
        self.a.is_zero() && self.b.is_zero()
    }
    fn add(&self, o: &Self) -> Self {
        Self {
            a: self.a.add(&o.a),
            b: self.b.add(&o.b),
        }
    }
    fn neg(&self) -> Self {
        Self {
            a: self.a.neg(),
            b: self.b.neg(),
        }
    }
    fn mul(&self, o: &Self) -> Self {
        // (a + b r)(c + d r) = (ac + 2bd) + (ad + bc) r.
        Self {
            a: self.a.mul(&o.a).add(&self.b.mul(&o.b).mul(&Dyadic::int(2))),
            b: self.a.mul(&o.b).add(&self.b.mul(&o.a)),
        }
    }
    fn to_f64(&self) -> f64 {
        self.a.to_f64() + self.b.to_f64() * std::f64::consts::SQRT_2
    }
}
/// x + y i over Q(sqrt 2).
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Coefficient {
    re: Root2,
    im: Root2,
}
impl Coefficient {
    pub fn is_zero(&self) -> bool {
        self.re.is_zero() && self.im.is_zero()
    }
    fn add(&self, o: &Self) -> Self {
        Self {
            re: self.re.add(&o.re),
            im: self.im.add(&o.im),
        }
    }
    fn mul(&self, o: &Self) -> Self {
        Self {
            re: self.re.mul(&o.re).add(&self.im.mul(&o.im).neg()),
            im: self.re.mul(&o.im).add(&self.im.mul(&o.re)),
        }
    }
    fn new(re_a: Dyadic, re_b: Dyadic, im_a: Dyadic, im_b: Dyadic) -> Self {
        Self {
            re: Root2 { a: re_a, b: re_b },
            im: Root2 { a: im_a, b: im_b },
        }
    }
}
/// An exact amplitude: a Laurent polynomial in z with K coefficients.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Amplitude {
    terms: BTreeMap<BigInt, Coefficient>,
}
impl Amplitude {
    pub fn zero() -> Self {
        Self::default()
    }
    pub fn one() -> Self {
        Self::constant(Coefficient::new(
            Dyadic::int(1),
            Dyadic::default(),
            Dyadic::default(),
            Dyadic::default(),
        ))
    }
    fn constant(c: Coefficient) -> Self {
        Self::monomial(BigInt::zero(), c)
    }
    fn monomial(e: BigInt, c: Coefficient) -> Self {
        let mut terms = BTreeMap::new();
        if !c.is_zero() {
            terms.insert(e, c);
        }
        Self { terms }
    }
    pub fn is_zero(&self) -> bool {
        self.terms.is_empty()
    }
    pub fn add(&self, o: &Self) -> Self {
        let mut terms = self.terms.clone();
        for (e, c) in &o.terms {
            let sum = terms.get(e).map_or_else(|| c.clone(), |p| p.add(c));
            if sum.is_zero() {
                terms.remove(e);
            } else {
                terms.insert(e.clone(), sum);
            }
        }
        Self { terms }
    }
    pub fn mul(&self, o: &Self) -> Self {
        let mut out = Self::zero();
        for (e, c) in &self.terms {
            for (f, d) in &o.terms {
                out = out.add(&Self::monomial(e.add(f), c.mul(d)));
            }
        }
        out
    }
    /// A floating-point value of the amplitude, for diagnostics only.
    pub fn approximate(&self) -> (f64, f64) {
        let mut re = 0f64;
        let mut im = 0f64;
        for (e, c) in &self.terms {
            let angle = Dyadic {
                n: e.clone(),
                k: 1075,
            }
            .normalize()
            .to_f64();
            let (s, co) = angle.sin_cos();
            let (x, y) = (c.re.to_f64(), c.im.to_f64());
            re += x * co - y * s;
            im += x * s + y * co;
        }
        (re, im)
    }
}

/// A finite binary64 as the dyadic rational it denotes exactly.
fn dyadic_from_f64(v: f64) -> Option<Dyadic> {
    if !v.is_finite() {
        return None;
    }
    if v == 0.0 {
        return Some(Dyadic::default());
    }
    let bits = v.to_bits();
    let negative = bits >> 63 == 1;
    let exponent = ((bits >> 52) & 0x7ff) as i64;
    let fraction = (bits & ((1u64 << 52) - 1)) as i128;
    // v = mantissa * 2^e with e >= -1074 (subnormals have no implicit bit).
    let (mantissa, e) = if exponent == 0 {
        (fraction, -1074i64)
    } else {
        (fraction | (1i128 << 52), exponent - 1075)
    };
    let n = BigInt::from_i128(if negative { -mantissa } else { mantissa });
    Some(if e >= 0 {
        Dyadic {
            n: n.shl(e as u64),
            k: 0,
        }
        .normalize()
    } else {
        Dyadic { n, k: (-e) as u64 }.normalize()
    })
}
/// The exact amplitude of a pair of finite binary64 values (each denotes a
/// dyadic rational, held exactly; the pair is zero iff both are zero).
pub fn amplitude_from_f64(re: f64, im: f64) -> Option<Amplitude> {
    Some(Amplitude::constant(Coefficient::new(
        dyadic_from_f64(re)?,
        Dyadic::default(),
        dyadic_from_f64(im)?,
        Dyadic::default(),
    )))
}

/// Exact entries of the §4.4 gates. `m` is theta * 2^1074 (theta/2 in units
/// of 2^-1075) for the rotations.
pub fn rotation_exponent(theta: f64) -> Option<BigInt> {
    if !theta.is_finite() {
        return None;
    }
    if theta == 0.0 {
        return Some(BigInt::zero());
    }
    let bits = theta.to_bits();
    let negative = bits >> 63 == 1;
    let exponent = ((bits >> 52) & 0x7ff) as i64;
    let fraction = (bits & ((1u64 << 52) - 1)) as i128;
    // theta = mantissa * 2^(e) with e >= -1074.
    let (mantissa, e) = if exponent == 0 {
        (fraction, -1074i64)
    } else {
        (fraction | (1i128 << 52), exponent - 1075)
    };
    let m = BigInt::from_i128(if negative { -mantissa } else { mantissa });
    // m * 2^(e + 1074) is an integer because e >= -1074.
    Some(m.shl((e + 1074) as u64))
}
pub fn gate_matrix(op: &str, angle: Option<f64>) -> Option<[Amplitude; 4]> {
    let zero = Dyadic::default;
    let half_root = || Dyadic::scaled(1, 1);
    let c = |re: (i128, u64, i128, u64), im: (i128, u64, i128, u64)| {
        Amplitude::constant(Coefficient::new(
            Dyadic::scaled(re.0, re.1),
            Dyadic::scaled(re.2, re.3),
            Dyadic::scaled(im.0, im.1),
            Dyadic::scaled(im.2, im.3),
        ))
    };
    let one = Amplitude::one;
    let z = Amplitude::zero;
    let i = || c((0, 0, 0, 0), (1, 0, 0, 0));
    let minus_i = || c((0, 0, 0, 0), (-1, 0, 0, 0));
    let h = || Amplitude::constant(Coefficient::new(zero(), half_root(), zero(), zero()));
    let minus_h = || {
        Amplitude::constant(Coefficient::new(
            zero(),
            Dyadic::scaled(-1, 1),
            zero(),
            zero(),
        ))
    };
    let phase = |e: BigInt| {
        Amplitude::monomial(e, Coefficient::new(Dyadic::int(1), zero(), zero(), zero()))
    };
    let half = |v: i128, imag: bool, e: BigInt| {
        let d = Dyadic::scaled(v, 1);
        Amplitude::monomial(
            e,
            if imag {
                Coefficient::new(zero(), zero(), d, zero())
            } else {
                Coefficient::new(d, zero(), zero(), zero())
            },
        )
    };
    Some(match op {
        "x" | "cx" | "ccx" | "mcx" => [z(), one(), one(), z()],
        "h" => [h(), h(), h(), minus_h()],
        "s" => [one(), z(), z(), i()],
        "sdg" => [one(), z(), z(), minus_i()],
        "t" => [one(), z(), z(), c((0, 0, 1, 1), (0, 0, 1, 1))],
        "tdg" => [one(), z(), z(), c((0, 0, 1, 1), (0, 0, -1, 1))],
        "rz" => {
            let m = rotation_exponent(angle?)?;
            [phase(m.neg()), z(), z(), phase(m)]
        }
        "rx" | "ry" => {
            let m = rotation_exponent(angle?)?;
            // cos = (z^m + z^-m)/2; sin = (z^m - z^-m)/(2i) = -i(z^m - z^-m)/2.
            let cos = half(1, false, m.clone()).add(&half(1, false, m.neg()));
            let minus_i_sin = half(-1, false, m.clone()).add(&half(1, false, m.neg()));
            let sin = half(-1, true, m.clone()).add(&half(1, true, m.neg()));
            if op == "rx" {
                [cos.clone(), minus_i_sin.clone(), minus_i_sin, cos]
            } else {
                let minus_sin = half(1, true, m.clone()).add(&half(-1, true, m.neg()));
                [cos.clone(), minus_sin, sin, cos]
            }
        }
        _ => return None,
    })
}
/// The exact amplitude -1 (cz, and controlled phases).
pub fn minus_one() -> Amplitude {
    Amplitude::constant(Coefficient::new(
        Dyadic::int(-1),
        Dyadic::default(),
        Dyadic::default(),
        Dyadic::default(),
    ))
}

impl BigInt {
    fn mul_small(&self, m: u32) -> Self {
        self.mul(&Self::from_i128(m as i128))
    }
    /// (quotient, remainder) of a nonnegative value by a small divisor.
    fn divmod_small(&self, d: u32) -> (Self, u32) {
        let mut out = vec![0u32; self.limbs.len()];
        let mut rem = 0u64;
        for (i, l) in self.limbs.iter().enumerate().rev() {
            let cur = (rem << 32) | *l as u64;
            out[i] = (cur / d as u64) as u32;
            rem = cur % d as u64;
        }
        (
            Self {
                negative: self.negative,
                limbs: out,
            }
            .normalize(),
            rem as u32,
        )
    }
    pub fn to_decimal(&self) -> String {
        if self.is_zero() {
            return "0".into();
        }
        let mut digits = Vec::new();
        let mut v = Self {
            negative: false,
            limbs: self.limbs.clone(),
        };
        while !v.is_zero() {
            let (q, r) = v.divmod_small(10);
            digits.push(b'0' + r as u8);
            v = q;
        }
        if self.negative {
            digits.push(b'-');
        }
        digits.reverse();
        String::from_utf8(digits).unwrap_or_default()
    }
}
/// An exact decimal m * 10^e, for §12.3 error composition (no rounding).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Decimal {
    m: BigInt,
    e: i64,
}
impl Decimal {
    pub fn zero() -> Self {
        Self {
            m: BigInt::zero(),
            e: 0,
        }
    }
    /// Parses a nonnegative JSON decimal (digits, optional fraction, optional exponent).
    pub fn parse(s: &str) -> Option<Self> {
        let (mantissa, exponent) = match s.split_once(['e', 'E']) {
            Some((m, e)) => (m, e.trim_start_matches('+').parse::<i64>().ok()?),
            None => (s, 0),
        };
        let (int, frac) = mantissa.split_once('.').unwrap_or((mantissa, ""));
        if int.is_empty()
            || !int.bytes().all(|b| b.is_ascii_digit())
            || !frac.bytes().all(|b| b.is_ascii_digit())
        {
            return None;
        }
        let mut m = BigInt::zero();
        for d in int.bytes().chain(frac.bytes()) {
            m = m.mul_small(10).add(&BigInt::from_i128((d - b'0') as i128));
        }
        Some(Self {
            m,
            e: exponent - frac.len() as i64,
        })
    }
    fn align(&self, e: i64) -> BigInt {
        let mut m = self.m.clone();
        for _ in e..self.e {
            m = m.mul_small(10);
        }
        m
    }
    pub fn add(&self, o: &Self) -> Self {
        let e = self.e.min(o.e);
        Self {
            m: self.align(e).add(&o.align(e)),
            e,
        }
    }
    pub fn mul_int(&self, n: u128) -> Self {
        Self {
            m: self.m.mul(&BigInt::from_i128(n as i128)),
            e: self.e,
        }
    }
    /// Plain decimal text without an exponent, trailing zeros removed.
    pub fn to_text(&self) -> String {
        let mut m = self.m.clone();
        let mut e = self.e;
        while e < 0 && !m.is_zero() {
            let (q, r) = m.divmod_small(10);
            if r != 0 {
                break;
            }
            m = q;
            e += 1;
        }
        let digits = m.to_decimal();
        if m.is_zero() {
            return "0".into();
        }
        if e >= 0 {
            return format!("{digits}{}", "0".repeat(e as usize));
        }
        let point = digits.len() as i64 + e;
        if point > 0 {
            format!(
                "{}.{}",
                &digits[..point as usize],
                &digits[point as usize..]
            )
        } else {
            format!("0.{}{digits}", "0".repeat((-point) as usize))
        }
    }
}
