//! Exact Q(sqrt(2), i) amplitudes for full-basis Clifford+T witnesses.
//! Coefficients are checked i128 dyadics; overflow fails loudly.
use crate::ref_language::{Leaf, Wire};
use std::collections::BTreeMap;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Real {
    pub a: i128,
    pub b: i128,
    pub exponent: u32,
}
impl Real {
    pub fn new(a: i128, b: i128, exponent: u32) -> Self {
        let mut r = Self { a, b, exponent };
        while r.exponent > 0 && r.a % 2 == 0 && r.b % 2 == 0 {
            r.a /= 2;
            r.b /= 2;
            r.exponent -= 1;
        }
        r
    }
    pub fn integer(a: i128) -> Self {
        Self::new(a, 0, 0)
    }
    fn plus(self, r: Self) -> Self {
        let e = self.exponent.max(r.exponent);
        let x = 1i128
            .checked_shl(e - self.exponent)
            .expect("exact amplitude overflow");
        let y = 1i128
            .checked_shl(e - r.exponent)
            .expect("exact amplitude overflow");
        Self::new(
            self.a
                .checked_mul(x)
                .unwrap()
                .checked_add(r.a.checked_mul(y).unwrap())
                .unwrap(),
            self.b
                .checked_mul(x)
                .unwrap()
                .checked_add(r.b.checked_mul(y).unwrap())
                .unwrap(),
            e,
        )
    }
    fn neg(self) -> Self {
        Self::new(
            self.a.checked_neg().unwrap(),
            self.b.checked_neg().unwrap(),
            self.exponent,
        )
    }
    fn times(self, r: Self) -> Self {
        Self::new(
            self.a
                .checked_mul(r.a)
                .unwrap()
                .checked_add(self.b.checked_mul(r.b).unwrap().checked_mul(2).unwrap())
                .unwrap(),
            self.a
                .checked_mul(r.b)
                .unwrap()
                .checked_add(self.b.checked_mul(r.a).unwrap())
                .unwrap(),
            self.exponent.checked_add(r.exponent).unwrap(),
        )
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct Complex {
    pub re: Real,
    pub im: Real,
}
impl Complex {
    pub fn one() -> Self {
        Self {
            re: Real::integer(1),
            im: Real::integer(0),
        }
    }
    pub fn plus(self, r: Self) -> Self {
        Self {
            re: self.re.plus(r.re),
            im: self.im.plus(r.im),
        }
    }
    pub fn negated(self) -> Self {
        Self {
            re: self.re.neg(),
            im: self.im.neg(),
        }
    }
    pub fn times(self, r: Self) -> Self {
        Self {
            re: self.re.times(r.re).plus(self.im.times(r.im).neg()),
            im: self.re.times(r.im).plus(self.im.times(r.re)),
        }
    }
    pub fn conjugate(self) -> Self {
        Self {
            re: self.re,
            im: self.im.neg(),
        }
    }
}
/// Apply emitted leaves with interface wires in caller order. Allocation bits
/// are represented explicitly, checked clean on both allocation and release.
pub fn column(leaves: &[Leaf], inputs: &[Wire], basis: usize) -> Result<Vec<Complex>, String> {
    let mut ids: BTreeMap<Wire, usize> = inputs
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, w)| (w, i))
        .collect();
    let mut count = inputs.len();
    for l in leaves {
        if l.gate == "alloc" {
            let (name, w, _, state) = l.allocation.as_ref().ok_or("missing allocation")?;
            if state != "clean" {
                return Err("quantum witness needs explicit borrowed interface inputs".into());
            }
            for i in 0..*w {
                ids.entry((name.clone(), i)).or_insert_with(|| {
                    let v = count;
                    count += 1;
                    v
                });
            }
        }
    }
    if count > 20 {
        return Err("dense witness exceeds configured 20 live-coordinate capacity".into());
    }
    let mut v = vec![Complex::default(); 1 << count];
    if basis >= 1 << inputs.len() {
        return Err("basis out of range".into());
    }
    v[basis] = Complex::one();
    for l in leaves {
        if matches!(l.gate.as_str(), "alloc" | "release") {
            let (name, w, _, _) = l.allocation.as_ref().ok_or("missing allocation")?;
            let mut mask = 0;
            for i in 0..*w {
                mask |= 1 << ids.get(&(name.clone(), i)).ok_or("unknown allocation")?;
            }
            if v.iter()
                .enumerate()
                .any(|(i, a)| i & mask != 0 && *a != Complex::default())
            {
                return Err("dirty scratch boundary".into());
            }
            continue;
        }
        if l.event.is_some() {
            return Err("classical leaf has no unitary".into());
        }
        let ws = l
            .wires
            .iter()
            .map(|w| ids.get(w).copied().ok_or("unknown wire"))
            .collect::<Result<Vec<_>, _>>()?;
        let controls = l
            .controls
            .iter()
            .map(|w| ids.get(w).copied().ok_or("unknown control"))
            .collect::<Result<Vec<_>, _>>()?;
        let enabled = |i: usize| controls.iter().all(|b| i >> b & 1 == 1);
        let t = *ws.last().ok_or("empty gate")?;
        let bit = 1 << t;
        match l.gate.as_str() {
            "x" | "cx" | "ccx" | "mcx" => {
                for i in 0..v.len() {
                    if i & bit == 0
                        && enabled(i)
                        && ws[..ws.len() - 1].iter().all(|b| i >> b & 1 == 1)
                    {
                        v.swap(i, i | bit)
                    }
                }
            }
            "swap" => {
                let x = 1 << ws[0];
                for i in 0..v.len() {
                    if i & x == 0 && i & bit != 0 && enabled(i) {
                        v.swap(i, i ^ x ^ bit)
                    }
                }
            }
            "h" => {
                let q = Complex {
                    re: Real::new(0, 1, 1),
                    im: Real::integer(0),
                };
                for i in 0..v.len() {
                    if i & bit == 0 && enabled(i) {
                        let a = v[i];
                        let b = v[i | bit];
                        v[i] = a.plus(b).times(q);
                        v[i | bit] = a.plus(b.negated()).times(q);
                    }
                }
            }
            "s" | "sdg" | "t" | "tdg" | "cz" => {
                let q = Real::new(0, 1, 1);
                let phase = match l.gate.as_str() {
                    "s" => Complex {
                        re: Real::integer(0),
                        im: Real::integer(1),
                    },
                    "sdg" => Complex {
                        re: Real::integer(0),
                        im: Real::integer(-1),
                    },
                    "t" => Complex { re: q, im: q },
                    "tdg" => Complex { re: q, im: q.neg() },
                    _ => Complex::one().negated(),
                };
                for (i, a) in v.iter_mut().enumerate() {
                    if i & bit != 0 && enabled(i) && (l.gate != "cz" || i >> ws[0] & 1 == 1) {
                        *a = a.times(phase)
                    }
                }
            }
            _ => return Err(format!("not an exact algebraic gate {}", l.gate)),
        }
    }
    if v.iter()
        .enumerate()
        .any(|(i, a)| i >= 1 << inputs.len() && *a != Complex::default())
    {
        return Err("unclean final workspace".into());
    }
    v.truncate(1 << inputs.len());
    Ok(v)
}
/// Binary64 matrix entries for arbitrary-angle numerical evidence. The input
/// bits are never round-tripped through decimal text; inverse toggles sign only.
pub fn rotation(g: &str, bits: u64) -> Result<[[(f64, f64); 2]; 2], String> {
    let theta = f64::from_bits(bits);
    if !theta.is_finite() {
        return Err("nonfinite angle".into());
    }
    let (s, c) = (theta * 0.5).sin_cos();
    Ok(match g {
        "rx" => [[(c, 0.0), (0.0, -s)], [(0.0, -s), (c, 0.0)]],
        "ry" => [[(c, 0.0), (-s, 0.0)], [(s, 0.0), (c, 0.0)]],
        "rz" => [[(c, -s), (0.0, 0.0)], [(0.0, 0.0), (c, s)]],
        _ => return Err("not a rotation".into()),
    })
}
