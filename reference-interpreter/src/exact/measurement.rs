//! Outward-rounded dyadic enclosures for Born comparisons. Precision refines
//! without a cutoff; exact Laurent-polynomial zero is decided before refinement.
use super::*;

fn cmp(a: &Dyadic, b: &Dyadic) -> Ordering {
    let k = a.k.max(b.k);
    a.n.shl(k - a.k).cmp(&b.n.shl(k - b.k))
}
fn rounded(x: &Dyadic, p: u64, up: bool) -> Dyadic {
    if x.k <= p {
        return x.clone();
    }
    let shift = x.k - p;
    let lost = !x.is_zero() && x.n.trailing_zeros() < shift;
    let mut n = x.n.shr(shift);
    if lost && (up != x.n.negative) {
        n = n.add(&BigInt::from_i128(if up { 1 } else { -1 }));
    }
    Dyadic { n, k: p }.normalize()
}
#[derive(Clone)]
struct Interval {
    lo: Dyadic,
    hi: Dyadic,
}
impl Interval {
    fn point(x: Dyadic) -> Self {
        Self {
            lo: x.clone(),
            hi: x,
        }
    }
    fn int(x: i128) -> Self {
        Self::point(Dyadic::int(x))
    }
    fn bounds(lo: Dyadic, hi: Dyadic, p: u64) -> Self {
        Self {
            lo: rounded(&lo, p, false),
            hi: rounded(&hi, p, true),
        }
    }
    fn add(&self, b: &Self, p: u64) -> Self {
        Self::bounds(self.lo.add(&b.lo), self.hi.add(&b.hi), p)
    }
    fn neg(&self) -> Self {
        Self {
            lo: self.hi.neg(),
            hi: self.lo.neg(),
        }
    }
    fn mul(&self, b: &Self, p: u64) -> Self {
        let mut v = [
            self.lo.mul(&b.lo),
            self.lo.mul(&b.hi),
            self.hi.mul(&b.lo),
            self.hi.mul(&b.hi),
        ];
        v.sort_by(cmp);
        Self::bounds(v[0].clone(), v[3].clone(), p)
    }
    fn div_small(&self, d: u32, p: u64) -> Self {
        let bound = |x: &Dyadic, up| {
            // Round x/d to the p-bit grid, with a guard against signed
            // truncation. Multiplication by 2^p is exact before division.
            let x = rounded(x, p, up);
            let (mut n, rem) = x.n.shl(p - x.k).divmod_small(d);
            if rem != 0 && (up != x.n.negative) {
                n = n.add(&BigInt::from_i128(if up { 1 } else { -1 }));
            }
            Dyadic { n, k: p }.normalize()
        };
        Self {
            lo: bound(&self.lo, false),
            hi: bound(&self.hi, true),
        }
    }
    fn midpoint(&self) -> Dyadic {
        self.lo.add(&self.hi).mul(&Dyadic::scaled(1, 1))
    }
}
fn sqrt2(p: u64) -> Interval {
    // Binary digit construction: r^2 <= 2, (r + 2^-p)^2 > 2.
    let mut lo = Dyadic::int(1);
    for bit in 1..=p {
        let next = lo.add(&Dyadic::scaled(1, bit));
        if cmp(&next.mul(&next), &Dyadic::int(2)) != Ordering::Greater {
            lo = next;
        }
    }
    Interval {
        hi: lo.add(&Dyadic::scaled(1, p)),
        lo,
    }
}
fn phase(e: &BigInt, p: u64) -> (Interval, Interval) {
    let bits = e.limbs.last().map_or(0, |n| {
        32 * (e.limbs.len() as u64 - 1) + u64::from(32 - n.leading_zeros())
    });
    let squares = bits.saturating_sub(1074);
    let x = Interval::point(
        Dyadic {
            n: e.clone(),
            k: 1075 + squares,
        }
        .normalize(),
    );
    // exp(ix) with |x| <= 1/2. After N terms the tail is at most
    // 2*(1/2)^(N+1)/(N+1)!; the integer bound below is conservative.
    let work = p + 16;
    let (mut re, mut im) = (Interval::int(1), Interval::int(0));
    let (mut tr, mut ti) = (Interval::int(1), Interval::int(0));
    let (mut n, mut bound_bits) = (0u32, 0u64);
    while bound_bits < p + 8 {
        n += 1;
        bound_bits += 1 + u64::from(31 - n.leading_zeros());
        (tr, ti) = (
            ti.mul(&x, work).neg().div_small(n, work),
            tr.mul(&x, work).div_small(n, work),
        );
        re = re.add(&tr, work);
        im = im.add(&ti, work);
    }
    let error = Interval {
        lo: Dyadic::scaled(-1, p + 4),
        hi: Dyadic::scaled(1, p + 4),
    };
    re = re.add(&error, work);
    im = im.add(&error, work);
    for _ in 0..squares {
        (re, im) = (
            re.mul(&re, work).add(&im.mul(&im, work).neg(), work),
            re.mul(&im, work).mul(&Interval::int(2), work),
        );
    }
    (re, im)
}
fn evaluate(a: &Amplitude, p: u64) -> (Interval, Interval) {
    let root = if a
        .terms
        .values()
        .any(|c| !c.re.b.is_zero() || !c.im.b.is_zero())
    {
        sqrt2(p + 16)
    } else {
        Interval::int(0)
    };
    let coeff = |r: &Root2| {
        Interval::point(r.a.clone()).add(&Interval::point(r.b.clone()).mul(&root, p), p)
    };
    let (mut re, mut im) = (Interval::int(0), Interval::int(0));
    for (e, c) in &a.terms {
        let (co, si) = phase(e, p);
        let (x, y) = (coeff(&c.re), coeff(&c.im));
        re = re.add(&x.mul(&co, p).add(&y.mul(&si, p).neg(), p), p);
        im = im.add(&x.mul(&si, p).add(&y.mul(&co, p), p), p);
    }
    (re, im)
}
/// Mantissa and binary exponent, never collapsing a tiny nonzero value.
fn scaled(x: &Dyadic) -> (f64, i64) {
    if x.is_zero() {
        return (0.0, 0);
    }
    let len = x.n.limbs.len();
    let bits = 32 * (len as u64 - 1) + u64::from(32 - x.n.limbs[len - 1].leading_zeros());
    let shift = bits.saturating_sub(53);
    let top = x.n.shr(shift);
    let mut m = 0.0;
    for l in top.limbs.iter().rev() {
        m = m * 4294967296.0 + f64::from(*l);
    }
    if x.n.negative {
        m = -m;
    }
    (
        m * 2f64.powi(-((bits - shift) as i32)),
        bits as i64 - x.k as i64,
    )
}
fn at_scale(x: &Dyadic, exponent: i64) -> f64 {
    let (m, e) = scaled(x);
    if m == 0.0 || e - exponent < -1074 {
        0.0
    } else {
        m * 2f64.powi((e - exponent) as i32)
    }
}
impl Amplitude {
    pub(crate) fn norm_squared(&self) -> Self {
        let conjugate = Self {
            terms: self
                .terms
                .iter()
                .map(|(e, c)| {
                    (
                        e.neg(),
                        Coefficient {
                            re: c.re.clone(),
                            im: c.im.neg(),
                        },
                    )
                })
                .collect(),
        };
        self.mul(&conjugate)
    }
    pub(crate) fn times_f64(&self, x: f64) -> crate::Result<Self> {
        let d = dyadic_from_f64(x)
            .ok_or_else(|| crate::Error("§14.2: nonfinite measurement scale".into()))?;
        Ok(self.mul(&Self::constant(Coefficient::new(
            d,
            Dyadic::default(),
            Dyadic::default(),
            Dyadic::default(),
        ))))
    }
    /// Sign of a real polynomial. Exact zero terminates even at CDF ties;
    /// otherwise converging enclosures eventually exclude zero.
    pub(crate) fn real_sign(&self) -> Ordering {
        if self.is_zero() {
            return Ordering::Equal;
        }
        let mut p = 64;
        loop {
            let (v, _) = evaluate(self, p);
            if cmp(&v.lo, &Dyadic::int(0)) == Ordering::Greater {
                return Ordering::Greater;
            }
            if cmp(&v.hi, &Dyadic::int(0)) == Ordering::Less {
                return Ordering::Less;
            }
            p *= 2;
        }
    }
}
/// Binary64 diagnostics of a projection, normalized with a shared extended
/// exponent. Refine until every supported amplitude has relative accuracy;
/// exact shadows stay unnormalised so future probabilities use norm ratios.
pub(crate) fn normalized(amplitudes: &[Amplitude]) -> Vec<(f64, f64)> {
    let mut p = 64;
    loop {
        let mut values = Vec::new();
        let mut ready = true;
        for a in amplitudes {
            if a.is_zero() {
                // A zero diagnostic neither needs refinement nor determines
                // the common scale of the supported amplitudes.
                values.push((Dyadic::int(0), Dyadic::int(0), i64::MIN));
                continue;
            }
            let (r, i) = evaluate(a, p);
            let (re, im) = (r.midpoint(), i.midpoint());
            let exponent = [scaled(&re), scaled(&im)]
                .into_iter()
                .filter(|(m, _)| *m != 0.0)
                .map(|(_, e)| e)
                .max();
            let Some(e) = exponent else {
                ready = false;
                break;
            };
            let width = r.hi.add(&r.lo.neg()).add(&i.hi.add(&i.lo.neg()));
            let tolerance = Dyadic {
                n: BigInt::from_i128(1),
                k: if e < 50 { (50 - e) as u64 } else { 0 },
            };
            if cmp(&width, &tolerance) == Ordering::Greater {
                ready = false;
                break;
            }
            values.push((re, im, e));
        }
        if ready {
            let exponent = values.iter().map(|(_, _, e)| *e).max().unwrap_or(0);
            let values: Vec<_> = values
                .iter()
                .map(|(r, i, _)| (at_scale(r, exponent), at_scale(i, exponent)))
                .collect();
            let norm = values
                .iter()
                .fold(0.0f64, |s, (r, i)| s.hypot(*r).hypot(*i));
            if norm == 0.0 {
                return vec![(0.0, 0.0); values.len()];
            }
            return values
                .into_iter()
                .map(|(r, i)| (r / norm, i / norm))
                .collect();
        }
        p *= 2;
    }
}
/// Approximate positive weight ratio for display only; sampling uses real_sign.
pub(crate) fn probability(weight: &Amplitude, total: &Amplitude) -> f64 {
    if weight.is_zero() {
        return 0.0;
    }
    let mut p = 64;
    loop {
        let (w, _) = evaluate(weight, p);
        let (t, _) = evaluate(total, p);
        if cmp(&w.lo, &Dyadic::int(0)) == Ordering::Greater
            && cmp(&t.lo, &Dyadic::int(0)) == Ordering::Greater
        {
            let (wm, we) = scaled(&w.midpoint());
            let (tm, te) = scaled(&t.midpoint());
            return if we - te < -1074 {
                0.0
            } else {
                (wm / tm) * 2f64.powi((we - te) as i32)
            };
        }
        p *= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_amplitudes_normalize_without_refining() {
        let zero = Amplitude::zero();
        assert_eq!(normalized(std::slice::from_ref(&zero)), vec![(0.0, 0.0)]);
        let one = zero.add(&Amplitude::one());
        assert_eq!(
            normalized(&[zero.clone(), one, zero]),
            vec![(0.0, 0.0), (1.0, 0.0), (0.0, 0.0)]
        );
    }

    #[test]
    fn zero_probability_weight_does_not_refine() {
        assert_eq!(probability(&Amplitude::zero(), &Amplitude::one()), 0.0);
    }
}
