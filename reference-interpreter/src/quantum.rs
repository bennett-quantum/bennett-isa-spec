//! Small state-vector semantics and a sparse basis-permutation fast path.
use crate::{err, Argument, Error, Result};
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
#[derive(Clone, Debug, PartialEq)]
enum State {
    Basis { bits: Vec<bool>, phase: Complex },
    Vector(Vec<Complex>),
}
#[derive(Clone, Debug, PartialEq)]
pub struct QuantumState {
    ids: Vec<Wire>,
    next: Wire,
    state: State,
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
    pub address_width: u16,
    pub cell_width: u16,
    pub cells: Vec<Vec<Wire>>,
}

impl QuantumState {
    pub fn new() -> Self {
        Self {
            ids: Vec::new(),
            next: 0,
            state: State::Basis {
                bits: Vec::new(),
                phase: Complex::ONE,
            },
        }
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
        if let State::Vector(v) = &mut self.state {
            let bits = self
                .ids
                .len()
                .checked_add(n)
                .ok_or_else(|| Error("§14.2: vector size overflow".into()))?;
            if bits > 24 {
                return err("§14.2", "state-vector simulation limit is 24 live wires");
            }
            v.resize(1usize << bits, Complex::ZERO);
        } else if let State::Basis { bits, .. } = &mut self.state {
            bits.try_reserve(n)
                .map_err(|e| Error(format!("§4.3: wire allocation failed: {e}")))?;
            bits.resize(bits.len() + n, false);
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
    pub fn amplitudes(&self) -> Result<Vec<Complex>> {
        match &self.state {
            State::Vector(v) => Ok(v.clone()),
            State::Basis { bits, phase } => {
                if bits.len() > 24 {
                    return err(
                        "§14.2",
                        "basis tuple exceeds vector materialisation limit of 24 wires",
                    );
                }
                let mut out = vec![Complex::ZERO; 1usize << bits.len()];
                out[bits
                    .iter()
                    .enumerate()
                    .fold(0, |n, (i, b)| n | ((*b as usize) << i))] = *phase;
                Ok(out)
            }
        }
    }
    pub fn read_basis(&self, wires: &[Wire]) -> Result<u128> {
        if wires.len() > 128 {
            return err("§3.3", "integer bundle read exceeds 128 bits");
        }
        let positions = self.positions(wires)?;
        match &self.state {
            State::Basis { bits, .. } => Ok(positions
                .iter()
                .enumerate()
                .fold(0, |n, (i, p)| n | ((bits[*p] as u128) << i))),
            State::Vector(_) => err("§14.2", "basis read requested on a state vector"),
        }
    }
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
            State::Vector(v) => {
                if v.iter()
                    .enumerate()
                    .any(|(i, a)| positions.iter().any(|p| i & (1 << p) != 0) && a.norm2() > 1e-24)
                {
                    return err("§4.3", "E_QREG_NOT_CLEAN: released factor is not zero");
                }
                let mut next = vec![Complex::ZERO; 1usize << (self.ids.len() - positions.len())];
                for (i, a) in v.iter().enumerate() {
                    if positions.iter().all(|p| i & (1 << p) == 0) {
                        next[compress(i, &remove)] = *a;
                    }
                }
                *v = next;
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
            "mcx" if wires.len() >= 1 => wires.len(),
            _ => return err("§4.4", format!("unknown gate {op}")),
        };
        if wires.len() != expected {
            return err("§4.8", "wrong gate arity");
        }
        if matches!(op, "rx" | "ry" | "rz") && angle.is_none_or(|a| !a.is_finite()) {
            return err("§4.4", "rotation needs a finite angle");
        }
        let mut controls: Vec<_> = positions[..own].iter().map(|p| (*p, true)).collect();
        let targets = &positions[own..];
        for (w, value) in predicate {
            let p = self.positions(&[*w])?[0];
            if targets.contains(&p) {
                return err("§3.3", "effective control aliases target");
            }
            if !controls.contains(&(p, *value)) {
                controls.push((p, *value));
            }
        }
        let active = |bits: &[bool]| controls.iter().all(|(p, v)| bits[*p] == *v);
        let permutation = matches!(op, "x" | "cx" | "ccx" | "mcx" | "swap");
        if let State::Basis { bits, phase } = &mut self.state {
            if !active(bits) {
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
            if !matches!(op, "h" | "rx" | "ry") {
                let theta = match op {
                    "s" => std::f64::consts::FRAC_PI_2,
                    "sdg" => -std::f64::consts::FRAC_PI_2,
                    "t" => std::f64::consts::FRAC_PI_4,
                    "tdg" => -std::f64::consts::FRAC_PI_4,
                    "cz" => {
                        if bits[targets[0]] && bits[targets[1]] {
                            std::f64::consts::PI
                        } else {
                            0.0
                        }
                    }
                    "rz" => angle.expect("angle") * if bits[targets[0]] { 0.5 } else { -0.5 },
                    _ => 0.0,
                };
                if op == "rz" || op == "cz" || bits[targets[0]] {
                    *phase = *phase * Complex::phase(theta);
                }
                return Ok(());
            }
        }
        if matches!(self.state, State::Basis { .. }) {
            self.state = State::Vector(self.amplitudes()?);
        }
        let State::Vector(vector) = &mut self.state else {
            unreachable!("materialised vector")
        };
        let enabled = |index: usize| {
            controls
                .iter()
                .all(|(p, v)| (index & (1usize << p) != 0) == *v)
        };
        if op == "swap" {
            let (a, b) = (1usize << targets[0], 1usize << targets[1]);
            for i in 0..vector.len() {
                if enabled(i) && i & a == 0 && i & b != 0 {
                    vector.swap(i, i ^ a ^ b);
                }
            }
            return Ok(());
        }
        if op == "cz" {
            for (i, a) in vector.iter_mut().enumerate() {
                if enabled(i) && targets.iter().all(|p| i & (1usize << p) != 0) {
                    *a = a.scale(-1.0);
                }
            }
            return Ok(());
        }
        let zero = Complex::ZERO;
        let one = Complex::ONE;
        let h = std::f64::consts::FRAC_1_SQRT_2;
        let matrix = match op {
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
            "rx" => {
                let a = angle.expect("angle") / 2.0;
                [
                    one.scale(a.cos()),
                    Complex::new(0.0, -a.sin()),
                    Complex::new(0.0, -a.sin()),
                    one.scale(a.cos()),
                ]
            }
            "ry" => {
                let a = angle.expect("angle") / 2.0;
                [
                    one.scale(a.cos()),
                    one.scale(-a.sin()),
                    one.scale(a.sin()),
                    one.scale(a.cos()),
                ]
            }
            "rz" => {
                let a = angle.expect("angle") / 2.0;
                [Complex::phase(-a), zero, zero, Complex::phase(a)]
            }
            _ => unreachable!("gate validated"),
        };
        let bit = 1usize << targets[0];
        for i in 0..vector.len() {
            if i & bit == 0 && enabled(i) {
                let a = vector[i];
                let b = vector[i | bit];
                vector[i] = matrix[0] * a + matrix[1] * b;
                vector[i | bit] = matrix[2] * a + matrix[3] * b;
            }
        }
        Ok(())
    }
    pub fn measurement_branches(&self, wires: &[Wire]) -> Result<Vec<MeasurementBranch>> {
        let positions = self.positions(wires)?;
        let mut remove = vec![false; self.ids.len()];
        for p in &positions {
            remove[*p] = true;
        }
        if let State::Basis { bits, phase } = &self.state {
            let mut state = self.clone();
            let result = positions.iter().map(|p| bits[*p]).collect();
            let kept: Vec<_> = bits
                .iter()
                .enumerate()
                .filter_map(|(p, b)| (!remove[p]).then_some(*b))
                .collect();
            state.ids = self
                .ids
                .iter()
                .enumerate()
                .filter_map(|(p, w)| (!remove[p]).then_some(*w))
                .collect();
            state.state = State::Basis {
                bits: kept,
                phase: *phase,
            };
            return Ok(vec![MeasurementBranch {
                probability: 1.0,
                bits: result,
                state,
            }]);
        }
        if wires.len() > 24 {
            return err("§14.2", "measurement outcome enumeration exceeds 24 wires");
        }
        let vector = self.amplitudes()?;
        let mut probabilities = vec![0.0; 1usize << positions.len()];
        let outcome = |i: usize| {
            positions
                .iter()
                .enumerate()
                .fold(0, |n, (j, p)| n | (((i >> p) & 1) << j))
        };
        for (i, a) in vector.iter().enumerate() {
            probabilities[outcome(i)] += a.norm2();
        }
        let mut branches = Vec::new();
        for (k, probability) in probabilities.into_iter().enumerate() {
            if probability == 0.0 {
                continue;
            }
            let mut amplitudes = vec![Complex::ZERO; 1usize << (self.ids.len() - positions.len())];
            for (i, a) in vector.iter().enumerate() {
                if outcome(i) == k {
                    amplitudes[compress(i, &remove)] = a.scale(1.0 / probability.sqrt());
                }
            }
            let mut state = self.clone();
            state.ids = self
                .ids
                .iter()
                .enumerate()
                .filter_map(|(p, w)| (!remove[p]).then_some(*w))
                .collect();
            state.state = State::Vector(amplitudes);
            branches.push(MeasurementBranch {
                probability,
                bits: (0..positions.len())
                    .map(|i| k & (1usize << i) != 0)
                    .collect(),
                state,
            });
        }
        Ok(branches)
    }
    pub(crate) fn permute(
        &mut self,
        mut permutation: impl FnMut(&[bool]) -> Result<Vec<bool>>,
    ) -> Result<()> {
        match &mut self.state {
            State::Basis { bits, .. } => {
                let next = permutation(bits)?;
                if next.len() != bits.len() {
                    return err("§14.2", "permutation changes wire count");
                }
                *bits = next;
            }
            State::Vector(v) => {
                let mut out = vec![Complex::ZERO; v.len()];
                let mut seen = vec![false; v.len()];
                for (i, a) in v.iter().enumerate() {
                    let bits: Vec<_> = (0..self.ids.len())
                        .map(|p| i & (1usize << p) != 0)
                        .collect();
                    let mapped = permutation(&bits)?;
                    if mapped.len() != self.ids.len() {
                        return err("§14.2", "permutation changes wire count");
                    }
                    let j = mapped
                        .iter()
                        .enumerate()
                        .fold(0, |n, (p, b)| n | ((*b as usize) << p));
                    if seen[j] {
                        return err("§5.3", "action mapping is not a permutation");
                    }
                    seen[j] = true;
                    out[j] = *a;
                }
                *v = out;
            }
        }
        Ok(())
    }
    pub(crate) fn position(&self, w: Wire) -> Result<usize> {
        Ok(self.positions(&[w])?[0])
    }
}
fn compress(index: usize, remove: &[bool]) -> usize {
    let mut out = 0;
    let mut next = 0;
    for (p, removed) in remove.iter().enumerate() {
        if !removed {
            out |= ((index >> p) & 1) << next;
            next += 1;
        }
    }
    out
}
