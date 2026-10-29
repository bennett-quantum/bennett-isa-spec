//! §14.2 state semantics: a sparse basis-permutation fast path and a state
//! vector, each carrying an exact shadow (src/exact.rs) beside the binary64
//! amplitudes. The binary64 amplitudes serve Born probabilities; every
//! cleanliness and basis-value decision uses the exact amplitudes, so a
//! component is zero iff it is exactly zero (no tolerance, council B15).
//! The vector has no fixed wire limit: it is bounded only by host memory,
//! and an allocation the host cannot provide fails loudly.
use crate::{
    err,
    exact::{self, Amplitude},
    Argument, Error, Result,
};
use std::collections::{BTreeMap, BTreeSet};
pub type Wire = u64;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Complex {
    pub re: f64,
    pub im: f64,
}
impl Complex {
    pub const ZERO: Self = Self { re: 0.0, im: 0.0 };
    pub const ONE: Self = Self { re: 1.0, im: 0.0 };
    pub fn new(re: f64, im: f64) -> Self {
        Self { re, im }
    }
    fn phase(a: f64) -> Self {
        Self::new(a.cos(), a.sin())
    }
    fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    fn scale(self, s: f64) -> Self {
        Self::new(self.re * s, self.im * s)
    }
}
impl std::ops::Add for Complex {
    type Output = Self;
    fn add(self, b: Self) -> Self {
        Self::new(self.re + b.re, self.im + b.im)
    }
}
impl std::ops::Mul for Complex {
    type Output = Self;
    fn mul(self, b: Self) -> Self {
        Self::new(
            self.re * b.re - self.im * b.im,
            self.re * b.im + self.im * b.re,
        )
    }
}
/// A basis index over arbitrarily many wires: little-endian words, no
/// trailing zero word (so allocating |0> wires leaves every key unchanged).
type Key = Vec<u64>;
fn key_bit(k: &Key, p: usize) -> bool {
    k.get(p / 64).is_some_and(|w| w >> (p % 64) & 1 == 1)
}
fn key_with(mut k: Key, p: usize, v: bool) -> Key {
    if k.len() <= p / 64 {
        if !v {
            return k;
        }
        k.resize(p / 64 + 1, 0);
    }
    if v {
        k[p / 64] |= 1 << (p % 64);
    } else {
        k[p / 64] &= !(1 << (p % 64));
    }
    while k.last() == Some(&0) {
        k.pop();
    }
    k
}
fn key_of(bits: &[bool]) -> Key {
    let mut k = Vec::new();
    for (p, b) in bits.iter().enumerate() {
        if *b {
            k = key_with(k, p, true);
        }
    }
    k
}
fn key_bits(k: &Key, n: usize) -> Vec<bool> {
    (0..n).map(|p| key_bit(k, p)).collect()
}
/// Drops the removed positions (release, measurement).
fn compress(k: &Key, remove: &[bool]) -> Key {
    let mut out = Vec::new();
    let mut next = 0;
    for (p, removed) in remove.iter().enumerate() {
        if !removed {
            if key_bit(k, p) {
                out = key_with(out, next, true);
            }
            next += 1;
        }
    }
    out
}
#[derive(Clone, Debug, PartialEq)]
enum State {
    Basis {
        bits: Vec<bool>,
        phase: Complex,
        exact: Amplitude,
    },
    /// Sparse superposition: exactly the components whose exact amplitude is
    /// nonzero, each with its binary64 value for Born probabilities.
    Vector {
        amplitudes: BTreeMap<Key, (Complex, Amplitude)>,
    },
}
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Content(Vec<Wire>, Vec<(Key, Amplitude)>);
#[derive(Clone, Debug)]
pub struct QuantumState {
    ids: Vec<Wire>,
    next: Wire,
    state: State,
}
impl PartialEq for QuantumState {
    /// Exact equality of the states (binary64 values are not compared).
    fn eq(&self, other: &Self) -> bool {
        if self.ids != other.ids || self.next != other.next {
            return false;
        }
        let support = |s: &State| -> BTreeMap<Key, Amplitude> {
            match s {
                State::Basis { bits, exact, .. } => BTreeMap::from([(key_of(bits), exact.clone())]),
                State::Vector { amplitudes } => amplitudes
                    .iter()
                    .map(|(k, (_, e))| (k.clone(), e.clone()))
                    .collect(),
            }
        };
        support(&self.state) == support(&other.state)
    }
}
impl Default for QuantumState {
    fn default() -> Self {
        Self::new()
    }
}
#[derive(Clone, Debug)]
pub struct MeasurementBranch {
    pub probability: f64,
    pub bits: Vec<bool>,
    pub state: QuantumState,
}
#[derive(Clone, Debug, Default)]
pub struct QuantumBindings {
    pub bundles: BTreeMap<String, Vec<Wire>>,
    pub regions: BTreeMap<String, QuantumRegion>,
    pub args: BTreeMap<String, Argument>,
    pub borrowed: Vec<Vec<Wire>>,
}
#[derive(Clone, Debug)]
pub struct QuantumRegion {
    pub address_width: u64,
    pub cell_width: u64,
    pub cells: Vec<Vec<Wire>>,
}
fn insert(map: &mut BTreeMap<Key, (Complex, Amplitude)>, k: Key, f: Complex, e: Amplitude) {
    if e.is_zero() {
        map.remove(&k);
    } else {
        map.insert(k, (f, e));
    }
}

impl QuantumState {
    pub(crate) fn next_wire_id(&self) -> u64 {
        self.next
    }
    pub(crate) fn restore_next_wire_id(&mut self, next: u64) -> Result<()> {
        if self.ids.iter().any(|id| *id >= next) {
            return err("§7.4", "cannot restore wire counter over live wires");
        }
        self.next = next;
        Ok(())
    }
    /// Re-inserts clean released wires at their former positions (§7.4 reversal).
    pub(crate) fn restore_clean(&mut self, positions: &[(usize, Wire)]) -> Result<()> {
        let n = self
            .ids
            .len()
            .checked_add(positions.len())
            .ok_or_else(|| Error("§7.4: wire count overflow".into()))?;
        let mut removed = BTreeMap::new();
        for (pos, wire) in positions {
            if *pos >= n || removed.insert(*pos, *wire).is_some() || self.ids.contains(wire) {
                return err("§7.4", "invalid released-wire history");
            }
        }
        let mut ids = Vec::with_capacity(n);
        let mut source = self.ids.iter();
        for i in 0..n {
            ids.push(match removed.get(&i) {
                Some(w) => *w,
                None => *source
                    .next()
                    .ok_or_else(|| Error("§7.4: restored wire count".into()))?,
            });
        }
        let expand = |k: &Key| {
            let mut out = Vec::new();
            let mut bit = 0;
            for pos in 0..n {
                if !removed.contains_key(&pos) {
                    if key_bit(k, bit) {
                        out = key_with(out, pos, true);
                    }
                    bit += 1;
                }
            }
            out
        };
        match &mut self.state {
            State::Basis { bits, .. } => {
                let mut old = bits.iter();
                let mut next = Vec::with_capacity(n);
                for i in 0..n {
                    next.push(if removed.contains_key(&i) {
                        false
                    } else {
                        *old.next()
                            .ok_or_else(|| Error("§7.4: basis width".into()))?
                    });
                }
                *bits = next;
            }
            State::Vector { amplitudes } => {
                *amplitudes = std::mem::take(amplitudes)
                    .into_iter()
                    .map(|(k, a)| (expand(&k), a))
                    .collect();
            }
        }
        self.ids = ids;
        Ok(())
    }
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            next: 0,
            state: State::Basis {
                bits: Vec::new(),
                phase: Complex::ONE,
                exact: Amplitude::one(),
            },
        }
    }
    /// The exact state without its wire-identity counter (loop periods).
    pub(crate) fn content(&self) -> Content {
        Content(
            self.ids.clone(),
            match &self.state {
                State::Basis { bits, exact, .. } => vec![(key_of(bits), exact.clone())],
                State::Vector { amplitudes } => amplitudes
                    .iter()
                    .map(|(k, (_, e))| (k.clone(), e.clone()))
                    .collect(),
            },
        )
    }
    pub fn wire_count(&self) -> usize {
        self.ids.len()
    }
    pub fn is_basis(&self) -> bool {
        matches!(self.state, State::Basis { .. })
    }
    pub fn allocate(&mut self, n: usize) -> Result<Vec<Wire>> {
        let end = self
            .next
            .checked_add(n as u64)
            .ok_or_else(|| Error("§4.3: wire identity overflow".into()))?;
        let total = self
            .ids
            .len()
            .checked_add(n)
            .ok_or_else(|| Error("§4.3: wire count overflow".into()))?;
        // New wires are |0> at the high positions: sparse keys are unchanged.
        if let State::Basis { bits, .. } = &mut self.state {
            bits.try_reserve(n)
                .map_err(|e| Error(format!("host limit: wire allocation failed ({e})")))?;
            bits.resize(total, false);
        }
        let wires: Vec<_> = (self.next..end).collect();
        self.next = end;
        self.ids.extend(&wires);
        Ok(wires)
    }
    fn positions(&self, wires: &[Wire]) -> Result<Vec<usize>> {
        let mut seen = BTreeSet::new();
        wires
            .iter()
            .map(|w| {
                if !seen.insert(w) {
                    return err("§3.3", "duplicate wire operand");
                }
                self.ids
                    .iter()
                    .position(|n| n == w)
                    .ok_or_else(|| Error(format!("§3.3: dead or unknown wire {w}")))
            })
            .collect()
    }
    /// Dense binary64 amplitudes in allocation order (bundles little-endian).
    /// Materialising 2^n entries is a host-memory operation and fails loudly.
    pub fn amplitudes(&self) -> Result<Vec<Complex>> {
        let n = self.ids.len();
        if n >= usize::BITS as usize - 1 {
            return err(
                "host limit",
                format!("2^{n} dense amplitudes exceed the host address space"),
            );
        }
        let len = 1usize << n;
        let mut out = Vec::new();
        out.try_reserve_exact(len).map_err(|e| {
            Error(format!(
                "host limit: cannot allocate {len} amplitudes ({e})"
            ))
        })?;
        out.resize(len, Complex::ZERO);
        let dense = |k: &Key| {
            k.iter()
                .enumerate()
                .fold(0usize, |n, (i, w)| n | ((*w as usize) << (64 * i)))
        };
        match &self.state {
            State::Basis { bits, phase, .. } => out[dense(&key_of(bits))] = *phase,
            State::Vector { amplitudes } => {
                for (k, (a, _)) in amplitudes {
                    out[dense(k)] = *a;
                }
            }
        }
        Ok(out)
    }
    /// The exact support as (basis bits, approximate amplitude), for diagnostics.
    pub fn exact_support(&self) -> Vec<(Vec<bool>, (f64, f64))> {
        let n = self.ids.len();
        match &self.state {
            State::Basis { bits, exact, .. } => vec![(bits.clone(), exact.approximate())],
            State::Vector { amplitudes } => amplitudes
                .iter()
                .map(|(k, (_, e))| (key_bits(k, n), e.approximate()))
                .collect(),
        }
    }
    /// The value of `wires` when it is the same on every component of the
    /// exact state; otherwise an error.
    pub fn read_basis(&self, wires: &[Wire]) -> Result<u128> {
        if wires.len() > 128 {
            return err(
                "§3.3",
                "integer bundle read exceeds 128 bits; read bit by bit",
            );
        }
        let positions = self.positions(wires)?;
        let read = |bit: &dyn Fn(usize) -> bool| {
            positions
                .iter()
                .enumerate()
                .fold(0u128, |n, (j, p)| n | ((bit(*p) as u128) << j))
        };
        match &self.state {
            State::Basis { bits, .. } => Ok(read(&|p| bits[p])),
            State::Vector { amplitudes } => {
                let values: BTreeSet<u128> = amplitudes
                    .keys()
                    .map(|k| read(&|p| key_bit(k, p)))
                    .collect();
                match values.len() {
                    1 => Ok(*values.iter().next().expect("one value")),
                    _ => err("§14.2", "wires are not in a basis state"),
                }
            }
        }
    }
    /// §4.3 qfree / §3.3 qreg.free: release wires that are |0> on every
    /// component, decided exactly; on failure nothing changes.
    pub fn release(&mut self, wires: &[Wire]) -> Result<()> {
        let positions = self.positions(wires)?;
        let mut remove = vec![false; self.ids.len()];
        for p in &positions {
            remove[*p] = true;
        }
        match &mut self.state {
            State::Basis { bits, .. } => {
                if positions.iter().any(|p| bits[*p]) {
                    return err("§4.3", "E_QREG_NOT_CLEAN: released wire is one");
                }
                let mut index = 0;
                bits.retain(|_| {
                    let keep = !remove[index];
                    index += 1;
                    keep
                });
            }
            State::Vector { amplitudes } => {
                if let Some((k, (_, e))) = amplitudes
                    .iter()
                    .find(|(k, _)| positions.iter().any(|p| key_bit(k, *p)))
                {
                    return err(
                        "§4.3",
                        format!(
                            "E_QREG_NOT_CLEAN: released factor has an exactly nonzero component {:?} (about {:?})",
                            key_bits(k, remove.len()),
                            e.approximate()
                        ),
                    );
                }
                *amplitudes = std::mem::take(amplitudes)
                    .into_iter()
                    .map(|(k, a)| (compress(&k, &remove), a))
                    .collect();
            }
        }
        let mut index = 0;
        self.ids.retain(|_| {
            let keep = !remove[index];
            index += 1;
            keep
        });
        Ok(())
    }
    pub fn gate(
        &mut self,
        op: &str,
        wires: &[Wire],
        angle: Option<f64>,
        predicate: &[Wire],
    ) -> Result<()> {
        self.controlled_gate(
            op,
            wires,
            angle,
            &predicate.iter().map(|w| (*w, true)).collect::<Vec<_>>(),
        )
    }
    pub(crate) fn controlled_gate(
        &mut self,
        op: &str,
        wires: &[Wire],
        angle: Option<f64>,
        predicate: &[(Wire, bool)],
    ) -> Result<()> {
        let positions = self.positions(wires)?;
        let own = match op {
            "cx" => 1,
            "ccx" => 2,
            "mcx" => wires.len().saturating_sub(1),
            _ => 0,
        };
        let expected = match op {
            "x" | "h" | "s" | "sdg" | "t" | "tdg" | "rx" | "ry" | "rz" => 1,
            "cx" | "swap" | "cz" => 2,
            "ccx" => 3,
            "mcx" if !wires.is_empty() => wires.len(),
            _ => return err("§4.4", format!("unknown gate {op}")),
        };
        if wires.len() != expected {
            return err("§4.8", "wrong gate arity");
        }
        if matches!(op, "rx" | "ry" | "rz") && angle.is_none_or(|a| !a.is_finite()) {
            return err("§4.4", "rotation needs a finite angle");
        }
        let mut controls: Vec<_> = positions[..own].iter().map(|p| (*p, true)).collect();
        let targets = positions[own..].to_vec();
        for (w, value) in predicate {
            let p = self.positions(&[*w])?[0];
            if targets.contains(&p) {
                return err("§3.3", "effective control aliases target");
            }
            if !controls.contains(&(p, *value)) {
                controls.push((p, *value));
            }
        }
        let permutation = matches!(op, "x" | "cx" | "ccx" | "mcx" | "swap");
        let diagonal = matches!(op, "s" | "sdg" | "t" | "tdg" | "cz" | "rz");
        let f64_matrix = |op: &str| -> Result<[Complex; 4]> {
            let zero = Complex::ZERO;
            let one = Complex::ONE;
            let h = std::f64::consts::FRAC_1_SQRT_2;
            Ok(match op {
                "x" | "cx" | "ccx" | "mcx" => [zero, one, one, zero],
                "h" => [one.scale(h), one.scale(h), one.scale(h), one.scale(-h)],
                "s" => [one, zero, zero, Complex::new(0.0, 1.0)],
                "sdg" => [one, zero, zero, Complex::new(0.0, -1.0)],
                "t" => [one, zero, zero, Complex::phase(std::f64::consts::FRAC_PI_4)],
                "tdg" => [
                    one,
                    zero,
                    zero,
                    Complex::phase(-std::f64::consts::FRAC_PI_4),
                ],
                "cz" => [one, zero, zero, one.scale(-1.0)],
                _ => {
                    let a = angle.ok_or_else(|| Error("§4.4: rotation angle".into()))? / 2.0;
                    match op {
                        "rx" => [
                            one.scale(a.cos()),
                            Complex::new(0.0, -a.sin()),
                            Complex::new(0.0, -a.sin()),
                            one.scale(a.cos()),
                        ],
                        "ry" => [
                            one.scale(a.cos()),
                            one.scale(-a.sin()),
                            one.scale(a.sin()),
                            one.scale(a.cos()),
                        ],
                        _ => [Complex::phase(-a), zero, zero, Complex::phase(a)],
                    }
                }
            })
        };
        let exact_matrix = || -> Result<[Amplitude; 4]> {
            if op == "cz" {
                return Ok([
                    Amplitude::one(),
                    Amplitude::zero(),
                    Amplitude::zero(),
                    exact::minus_one(),
                ]);
            }
            exact::gate_matrix(op, angle)
                .ok_or_else(|| Error(format!("§4.4: no exact entries for {op}")))
        };
        if let State::Basis { bits, phase, exact } = &mut self.state {
            if !controls.iter().all(|(p, v)| bits[*p] == *v) {
                return Ok(());
            }
            if permutation {
                if op == "swap" {
                    bits.swap(targets[0], targets[1]);
                } else {
                    bits[targets[0]] ^= true;
                }
                return Ok(());
            }
            if diagonal {
                // cz acts as a phase on its first target when the second is 1.
                let on = if op == "cz" { bits[targets[1]] } else { true };
                if on {
                    let bit = bits[targets[0]];
                    let (f, e) = (f64_matrix(op)?, exact_matrix()?);
                    let j = if bit { 3 } else { 0 };
                    *phase = *phase * f[j];
                    *exact = exact.mul(&e[j]);
                }
                return Ok(());
            }
            let key = key_of(bits);
            self.state = State::Vector {
                amplitudes: BTreeMap::from([(key, (*phase, exact.clone()))]),
            };
        }
        let State::Vector { amplitudes } = &mut self.state else {
            return err("§14.2", "state vector materialisation failed");
        };
        let enabled = |k: &Key| controls.iter().all(|(p, v)| key_bit(k, *p) == *v);
        if op == "swap" {
            let (a, b) = (targets[0], targets[1]);
            let moved: Vec<Key> = amplitudes
                .keys()
                .filter(|k| enabled(k) && key_bit(k, a) != key_bit(k, b))
                .cloned()
                .collect();
            let values: Vec<_> = moved
                .iter()
                .map(|k| (k.clone(), amplitudes.remove(k).expect("listed key")))
                .collect();
            for (k, v) in values {
                let flipped = key_with(key_with(k.clone(), a, !key_bit(&k, a)), b, !key_bit(&k, b));
                amplitudes.insert(flipped, v);
            }
            return Ok(());
        }
        if op == "cz" {
            for (k, (f, e)) in amplitudes.iter_mut() {
                if enabled(k) && key_bit(k, targets[0]) && key_bit(k, targets[1]) {
                    *f = f.scale(-1.0);
                    *e = e.mul(&exact::minus_one());
                }
            }
            return Ok(());
        }
        let (fm, em) = (f64_matrix(op)?, exact_matrix()?);
        let t = targets[0];
        let lows: BTreeSet<Key> = amplitudes
            .keys()
            .filter(|k| enabled(k))
            .map(|k| key_with(k.clone(), t, false))
            .collect();
        for low in lows {
            let high = key_with(low.clone(), t, true);
            let (fa, ea) = amplitudes
                .remove(&low)
                .unwrap_or((Complex::ZERO, Amplitude::zero()));
            let (fb, eb) = amplitudes
                .remove(&high)
                .unwrap_or((Complex::ZERO, Amplitude::zero()));
            insert(
                amplitudes,
                low,
                fm[0] * fa + fm[1] * fb,
                em[0].mul(&ea).add(&em[1].mul(&eb)),
            );
            insert(
                amplitudes,
                high,
                fm[2] * fa + fm[3] * fb,
                em[2].mul(&ea).add(&em[3].mul(&eb)),
            );
        }
        Ok(())
    }
    pub fn measurement_branches(&self, wires: &[Wire]) -> Result<Vec<MeasurementBranch>> {
        let positions = self.positions(wires)?;
        let mut remove = vec![false; self.ids.len()];
        for p in &positions {
            remove[*p] = true;
        }
        let kept_ids: Vec<Wire> = self
            .ids
            .iter()
            .enumerate()
            .filter_map(|(p, w)| (!remove[p]).then_some(*w))
            .collect();
        if let State::Basis { bits, phase, exact } = &self.state {
            let mut state = self.clone();
            let result = positions.iter().map(|p| bits[*p]).collect();
            let kept: Vec<_> = bits
                .iter()
                .enumerate()
                .filter_map(|(p, b)| (!remove[p]).then_some(*b))
                .collect();
            state.ids = kept_ids;
            state.state = State::Basis {
                bits: kept,
                phase: *phase,
                exact: exact.clone(),
            };
            return Ok(vec![MeasurementBranch {
                probability: 1.0,
                bits: result,
                state,
            }]);
        }
        let State::Vector { amplitudes } = &self.state else {
            return err("§14.2", "unreachable state form");
        };
        let outcome = |k: &Key| {
            positions
                .iter()
                .map(|p| key_bit(k, *p))
                .collect::<Vec<bool>>()
        };
        // Outcomes with an exactly nonzero component; the exact projection is
        // left unnormalised (cleanliness and basis values are scale-invariant).
        let mut groups: BTreeMap<Vec<bool>, BTreeMap<Key, (Complex, Amplitude)>> = BTreeMap::new();
        for (k, a) in amplitudes {
            groups
                .entry(outcome(k))
                .or_default()
                .insert(compress(k, &remove), a.clone());
        }
        let mut branches = Vec::new();
        for (bits, mut group) in groups {
            let probability: f64 = group.values().map(|(f, _)| f.norm2()).sum();
            for (f, _) in group.values_mut() {
                *f = f.scale(1.0 / probability.sqrt());
            }
            let mut state = self.clone();
            state.ids = kept_ids.clone();
            state.state = State::Vector { amplitudes: group };
            branches.push(MeasurementBranch {
                probability,
                bits,
                state,
            });
        }
        Ok(branches)
    }
    /// Applies a basis permutation (oracle constructions, §5.2). On a sparse
    /// superposition the map must be injective on the support.
    pub(crate) fn permute(
        &mut self,
        mut permutation: impl FnMut(&[bool]) -> Result<Vec<bool>>,
    ) -> Result<()> {
        let n = self.ids.len();
        match &mut self.state {
            State::Basis { bits, .. } => {
                let next = permutation(bits)?;
                if next.len() != bits.len() {
                    return err("§14.2", "permutation changes wire count");
                }
                *bits = next;
            }
            State::Vector { amplitudes } => {
                let mut out = BTreeMap::new();
                for (k, a) in std::mem::take(amplitudes) {
                    let mapped = permutation(&key_bits(&k, n))?;
                    if mapped.len() != n {
                        return err("§14.2", "permutation changes wire count");
                    }
                    if out.insert(key_of(&mapped), a).is_some() {
                        return err("§5.3", "action mapping is not a permutation");
                    }
                }
                *amplitudes = out;
            }
        }
        Ok(())
    }
    pub(crate) fn position(&self, w: Wire) -> Result<usize> {
        Ok(self.positions(&[w])?[0])
    }
}
