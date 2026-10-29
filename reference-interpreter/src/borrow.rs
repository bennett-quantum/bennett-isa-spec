//! §5.6 borrowed systems: the translation lemma, and an exact symbolic decision
//! of the frame factorisation P_F = P' ⊗ I_B.
//!
//! A frame is a permutation, so its action on basis states is a vector of
//! Boolean functions. Each is held in algebraic normal form (ANF: an XOR of
//! AND-monomials over the frame's input wires), which is canonical: two ANFs
//! denote the same function iff they are equal. P_F = P' ⊗ I_B iff every
//! borrowed output equals its input variable and no other output mentions a
//! borrowed variable. This is a total, exact decision over the frame's full
//! domain (T3), with no wire-count limit; loops are composed by repeated
//! squaring, never unrolled bound times. Calls whose black-box outputs would
//! depend on borrowed wires cannot be decided this way: they require the
//! §12 borrow_frames claim the specification names, checked when present
//! (src/claims.rs). SPEC-QUESTIONS 20 records this reading.
use crate::{
    err,
    object::list,
    validation::{gate, gate_operands, normalize_call},
    Error, Function, Instruction, Object, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

pub fn validate_translation_lemma(
    v: &[Instruction],
    w: &[Instruction],
    x: &BTreeSet<String>,
    borrowed: &BTreeSet<String>,
) -> Result<()> {
    if !x.is_disjoint(borrowed) {
        return err("§5.6", "translation lemma requires X disjoint from B");
    }
    let flips = |sequence: &[Instruction]| -> Result<Vec<(Vec<String>, String)>> {
        sequence
            .iter()
            .map(|i| {
                if !["x", "cx", "ccx", "mcx"].contains(&i.op.as_str()) {
                    return err("§5.6", "translation lemma contains a non-X-family gate");
                }
                let (c, t, _, _) = gate_operands(i)?;
                Ok((c, t[0].clone()))
            })
            .collect()
    };
    let v = flips(v)?;
    let w = flips(w)?;
    let controls: BTreeSet<_> = w.iter().flat_map(|(c, _)| c.iter().cloned()).collect();
    for (c, t) in v {
        if !borrowed.contains(&t) || c.iter().any(|c| !x.contains(c)) {
            return err("§5.6", "V must target only B with controls only in X");
        }
    }
    for (c, t) in w {
        if x.contains(&t) || borrowed.contains(&t) || controls.contains(&t) {
            return err("§5.6", "W target overlaps X, B, or W's controls");
        }
        if c.iter().filter(|c| borrowed.contains(*c)).count() > 1 {
            return err("§5.6", "W gate has more than one borrowed control");
        }
    }
    Ok(())
}

/// An ANF monomial: sorted, distinct variable ids (the empty monomial is 1).
type Mono = Vec<u32>;
#[derive(Clone, Debug, PartialEq, Eq, Default)]
struct Poly(BTreeSet<Mono>);
impl Poly {
    fn zero() -> Self {
        Self::default()
    }
    fn one() -> Self {
        Self(BTreeSet::from([Vec::new()]))
    }
    fn var(v: u32) -> Self {
        Self(BTreeSet::from([vec![v]]))
    }
    fn xor(&self, o: &Self) -> Self {
        Self(self.0.symmetric_difference(&o.0).cloned().collect())
    }
    fn and(&self, o: &Self) -> Self {
        let mut out = BTreeSet::new();
        for a in &self.0 {
            for b in &o.0 {
                let mut m: Mono = a.iter().chain(b).copied().collect();
                m.sort_unstable();
                m.dedup();
                if !out.insert(m.clone()) {
                    out.remove(&m);
                }
            }
        }
        Self(out)
    }
    fn mentions(&self, vars: &BTreeSet<u32>) -> bool {
        self.0.iter().flatten().any(|v| vars.contains(v))
    }
    fn substitute(&self, map: &BTreeMap<u32, Poly>) -> Self {
        let mut out = Self::zero();
        for m in &self.0 {
            let mut term = Self::one();
            for v in m {
                term = term.and(map.get(v).unwrap_or(&Self::var(*v)));
            }
            out = out.xor(&term);
        }
        out
    }
}

/// A compiled operation on named wires.
#[derive(Clone, Debug)]
enum Node {
    /// Target ^= AND of controls (with polarity).
    Flip(Vec<(String, bool)>, String),
    /// Controlled exchange of two wires.
    Swap(Vec<(String, bool)>, String, String),
    /// `n` repetitions of a permutation-only body.
    Repeat(u64, Vec<Node>),
    /// A black-box function of `inputs` XORed (or, for qinplace, assigned)
    /// into `outputs`, keyed by the callee: oracle calls and qinplace.
    Opaque {
        key: String,
        controls: Vec<(String, bool)>,
        inputs: Vec<String>,
        outputs: Vec<String>,
        assign: bool,
    },
    /// A non-permutation operation (non-diagonal or diagonal gate, measurement-free
    /// non-permutation call): it ends a frame and may not touch a borrowed wire.
    Other(Vec<String>),
    Alloc(Vec<String>, Vec<bool>),
    Free(Vec<String>),
}
impl Node {
    fn wires(&self, out: &mut BTreeSet<String>) {
        match self {
            Self::Flip(c, t) => {
                out.extend(c.iter().map(|(w, _)| w.clone()));
                out.insert(t.clone());
            }
            Self::Swap(c, a, b) => {
                out.extend(c.iter().map(|(w, _)| w.clone()));
                out.insert(a.clone());
                out.insert(b.clone());
            }
            Self::Repeat(_, body) => body.iter().for_each(|n| n.wires(out)),
            Self::Opaque {
                controls,
                inputs,
                outputs,
                ..
            } => {
                out.extend(controls.iter().map(|(w, _)| w.clone()));
                out.extend(inputs.iter().cloned());
                out.extend(outputs.iter().cloned());
            }
            Self::Other(w) | Self::Alloc(w, _) | Self::Free(w) => out.extend(w.iter().cloned()),
        }
    }
    fn permutation(&self) -> bool {
        !matches!(self, Self::Other(_))
    }
    fn adjoint(&self) -> Self {
        match self {
            Self::Repeat(n, body) => {
                Self::Repeat(*n, body.iter().rev().map(Self::adjoint).collect())
            }
            Self::Opaque {
                assign: true,
                key,
                controls,
                inputs,
                outputs,
            } => Self::Opaque {
                key: format!("inverse({key})"),
                controls: controls.clone(),
                inputs: inputs.clone(),
                outputs: outputs.clone(),
                assign: true,
            },
            Self::Alloc(w, image) => {
                let _ = image;
                Self::Free(w.clone())
            }
            Self::Free(w) => Self::Alloc(w.clone(), vec![false; w.len()]),
            other => other.clone(),
        }
    }
}

struct Compiler<'a> {
    object: &'a Object,
    bundles: BTreeMap<String, Vec<String>>,
    regions: BTreeMap<String, (u64, Vec<Vec<String>>)>,
    borrowed: BTreeSet<String>,
    classical: BTreeMap<String, String>,
    generation: usize,
}
impl Compiler<'_> {
    fn classical(&self, name: &str) -> Result<String> {
        self.classical.get(name).cloned().ok_or_else(|| Error(format!("§3.3: unknown classical binding {name}")))
    }
    fn bundle(&self, name: &str) -> Result<Vec<String>> {
        if let Some((base, index)) = name.split_once('[') {
            let index = crate::object::host_count(index.trim_end_matches(']'), "wire index")?;
            return self
                .bundles
                .get(base)
                .and_then(|w| w.get(index as usize))
                .map(|w| vec![w.clone()])
                .ok_or_else(|| Error(format!("§3.3: unknown indexed operand {name}")));
        }
        self.bundles
            .get(name)
            .cloned()
            .ok_or_else(|| Error(format!("§3.3: unknown operand {name}")))
    }
    fn scalar(&self, name: &str) -> Result<String> {
        let w = self.bundle(name)?;
        if w.len() != 1 {
            return err("§3.3", format!("{name} must be one wire"));
        }
        Ok(w[0].clone())
    }
    fn fresh(&mut self, width: usize) -> Vec<String> {
        let generation = self.generation;
        self.generation += 1;
        (0..width)
            .map(|bit| format!("v{generation}_{bit}"))
            .collect()
    }
    fn produce(&mut self, name: &str, width: usize) -> Vec<String> {
        let wires = self.fresh(width);
        self.bundles.insert(name.into(), wires.clone());
        wires
    }
    /// The address condition `a == i` as literals on the address wires.
    fn address(&self, a: &str, i: usize) -> Result<Vec<(String, bool)>> {
        Ok(self
            .bundle(a)?
            .into_iter()
            .enumerate()
            .map(|(bit, w)| (w, i >> bit & 1 == 1))
            .collect())
    }
    fn declare_regions(&mut self, f: &Function, out: &mut Vec<Node>) -> Result<()> {
        for i in crate::validation::region_declarations(f) {
            let a = &i.operands;
                    let aw = crate::object::host_count(&a[1], "qregion address width")?;
                    let w = crate::object::host_count(&a[2], "qregion cell width")? as usize;
                    let cells = crate::oracle::power_of_two(aw as u128)
                        .and_then(|n| usize::try_from(n).ok())
                        .ok_or_else(|| {
                            Error("host limit: region exceeds host address space".into())
                        })?;
                    let image = match a.get(3) {
                        Some(img) => self
                            .object
                            .data
                            .get(img.trim_start_matches('@'))
                            .cloned()
                            .ok_or_else(|| Error("§4.6: unresolved qregion image".into()))?,
                        None => vec![0; (cells * w).div_ceil(8)],
                    };
                    let mut all = Vec::new();
                    for _ in 0..cells {
                        all.push(self.fresh(w));
                    }
                    let flat: Vec<String> = all.iter().flatten().cloned().collect();
                    let bits = (0..flat.len())
                        .map(|b| image[b / 8] >> (b % 8) & 1 == 1)
                        .collect();
                    out.push(Node::Alloc(flat, bits));
                    self.regions.insert(a[0].clone(), (aw, all));
        }
        Ok(())
    }
    fn compile(
        &mut self,
        seq: &[Instruction],
        context: &[(String, bool)],
        out: &mut Vec<Node>,
    ) -> Result<()> {
        let mut predicate = context.to_vec();
        for i in seq {
            let a = &i.operands;
            let op = i.op.as_str();
            let with = |extra: &[(String, bool)], base: &[(String, bool)]| {
                let mut c = base.to_vec();
                for e in extra {
                    if !c.contains(e) {
                        c.push(e.clone());
                    }
                }
                c
            };
            match op {
                "qalloc" | "qborrow" => {
                    let n = crate::object::host_count(&a[1], "allocation width")? as usize;
                    let wires = self.produce(&i.lhs[0], n);
                    if op == "qborrow" {
                        self.borrowed.extend(wires);
                    } else {
                        out.push(Node::Alloc(wires.clone(), vec![false; wires.len()]));
                    }
                }
                "qfree" => out.push(Node::Free(self.bundle(&a[0])?)),
                "qreturn" => {}
                "qpred" => predicate.push((self.scalar(&a[0])?, true)),
                "qunpred" => {
                    predicate
                        .pop()
                        .ok_or_else(|| Error("§3.5: predicate underflow".into()))?;
                }
                "qcopy" | "quncopy" | "qxor" => {
                    let source = self.bundle(&a[0])?;
                    let target = match op {
                        "qcopy" => {
                            let w = self.produce(&i.lhs[0], source.len());
                            out.push(Node::Alloc(w.clone(), vec![false; w.len()]));
                            w
                        }
                        _ => self.bundle(&a[1])?,
                    };
                    for (s, t) in source.iter().zip(&target) {
                        out.push(Node::Flip(
                            with(&[(s.clone(), true)], &predicate),
                            t.clone(),
                        ));
                    }
                    if op == "quncopy" {
                        out.push(Node::Free(target));
                    }
                }
                "qexch" => {
                    for (s, t) in self.bundle(&a[0])?.into_iter().zip(self.bundle(&a[1])?) {
                        out.push(Node::Swap(predicate.clone(), s, t));
                    }
                }
                "qif" => {
                    let selector = self.scalar(&a[0])?;
                    self.compile(
                        &i.bodies[0],
                        &with(&[(selector.clone(), true)], &predicate),
                        out,
                    )?;
                    self.compile(&i.bodies[1], &with(&[(selector, false)], &predicate), out)?;
                }
                "qloop" => {
                    let n = crate::object::host_count(&a[0], "qloop trip count")?;
                    if a.len() == 3 {
                        // Coherent while: iteration i runs under cond[0..=i] (§4.6),
                        // n = the width of cond, which is materialised anyway.
                        let selectors = self.bundle(&a[2])?;
                        for iteration in 0..n as usize {
                            let c: Vec<_> = selectors[..=iteration]
                                .iter()
                                .map(|w| (w.clone(), true))
                                .collect();
                            self.compile(&i.bodies[0], &with(&c, &predicate), out)?;
                        }
                    } else {
                        let mut body = Vec::new();
                        self.compile(&i.bodies[0], &predicate, &mut body)?;
                        if body.iter().all(Node::permutation) {
                            out.push(Node::Repeat(n, body));
                        } else {
                            // A non-permutation op bounds every frame within two
                            // consecutive iterations: three iterations exhibit every
                            // frame shape (first, junction, last), each repeating.
                            for _ in 0..n.min(3) {
                                out.extend(body.iter().cloned());
                            }
                        }
                    }
                }
                "qregion" => {} // Hoisted once at function entry.
                "qload" | "qunload" | "qstore" | "qexchm" => {
                    let (_, cells) = self
                        .regions
                        .get(&a[0])
                        .cloned()
                        .ok_or_else(|| Error("§4.6: unknown region".into()))?;
                    let value = if op == "qload" {
                        let w = self.produce(&i.lhs[0], cells.first().map_or(0, Vec::len));
                        out.push(Node::Alloc(w.clone(), vec![false; w.len()]));
                        w
                    } else {
                        self.bundle(&a[2])?
                    };
                    for (index, cell) in cells.iter().enumerate() {
                        let c = with(&self.address(&a[1], index)?, &predicate);
                        for (x, y) in cell.iter().zip(&value) {
                            out.push(match op {
                                "qexchm" => Node::Swap(c.clone(), x.clone(), y.clone()),
                                "qstore" => Node::Flip(with(&[(y.clone(), true)], &c), x.clone()),
                                _ => Node::Flip(with(&[(x.clone(), true)], &c), y.clone()),
                            });
                        }
                    }
                    if op == "qunload" {
                        out.push(Node::Free(value));
                    }
                }
                "oracle_call" | "action_call" | "adj" | "ctl" => {
                    let call = normalize_call(i)?;
                    let mut c = predicate.clone();
                    if let Some(w) = &call.control {
                        c = with(&[(self.scalar(w)?, true)], &c);
                    }
                    let ca = &call.instruction.operands;
                    let callee = self.object.declaration(ca[0].trim_start_matches('@'))?;
                    if call.instruction.op == "oracle_call" {
                        let inputs = list(&ca[2])?
                            .iter()
                            .map(|n| self.bundle(n))
                            .collect::<Result<Vec<_>>>()?
                            .concat();
                        let mut outputs = self.bundle(&ca[3])?;
                        outputs.extend(self.bundle(&ca[4])?);
                        let mut memory = Vec::new();
                        for n in list(&ca[5])? {
                            let (_, cells) = self.regions.get(&n).cloned().ok_or_else(|| {
                                Error("§4.7: oracle mem operand is not a live region".into())
                            })?;
                            memory.extend(cells.into_iter().flatten());
                        }
                        out.push(Node::Opaque {
                            key: format!("oracle:{:?}", (&callee.name, self.classical(&ca[1])?,
                                list(&ca[6])?.iter().map(|n| self.classical(n)).collect::<Result<Vec<_>>>()?)),
                            controls: c,
                            inputs: [inputs, memory].concat(),
                            outputs,
                            assign: false,
                        });
                    } else if callee.kind == "perm_action"
                        && self.object.functions.contains_key(&callee.name)
                    {
                        // Inline the exact permutation (the call graph is acyclic).
                        let f = &self.object.functions[&callee.name];
                        let mut child = Compiler {
                            object: self.object,
                            bundles: BTreeMap::new(),
                            regions: BTreeMap::new(),
                            borrowed: BTreeSet::new(),
                            classical: BTreeMap::new(),
                            generation: self.generation,
                        };
                        for (group, actuals) in [(0, list(&ca[1])?), (1, list(&ca[2])?)] {
                            for (p, n) in f.params[group].iter().zip(actuals) {
                                child.bundles.insert(p.name.clone(), self.bundle(&n)?);
                            }
                        }
                        for (p, n) in f.params[2].iter().zip(list(&ca[3])?) {
                            let r =
                                self.regions.get(&n).cloned().ok_or_else(|| {
                                    Error("§4.7: region operand is not live".into())
                                })?;
                            child.regions.insert(p.name.clone(), r);
                        }
                        for (p, n) in f.params[3].iter().zip(list(&ca[4])?) {
                            child.classical.insert(p.name.clone(), self.classical(&n)?);
                        }
                        let mut body = Vec::new();
                        child.declare_regions(f, &mut body)?;
                        for b in &f.blocks {
                            child.compile(&b.instructions, &c, &mut body)?;
                        }
                        self.generation = child.generation;
                        self.borrowed.extend(child.borrowed);
                        if call.adjoint {
                            body = body.iter().rev().map(Node::adjoint).collect();
                        }
                        out.extend(body);
                    } else {
                        let mut touched = Vec::new();
                        for group in [1, 2] {
                            for n in list(&ca[group])? {
                                touched.extend(self.bundle(&n)?);
                            }
                        }
                        out.push(Node::Other(touched));
                    }
                }
                "qinplace" => {
                    let x = self.bundle(&a[3])?;
                    out.push(Node::Opaque {
                        key: format!("qinplace:{:?}", (&a[0], &a[1], self.classical(&a[2])?)),
                        controls: predicate.clone(),
                        inputs: x.clone(),
                        outputs: x,
                        assign: true,
                    });
                }
                _ if gate(op) => {
                    let (controls, targets, diagonal, other) = gate_operands(i)?;
                    let mut c = predicate.clone();
                    for control in &controls {
                        c = with(&[(self.scalar(control)?, true)], &c);
                    }
                    let targets = targets
                        .iter()
                        .map(|t| self.scalar(t))
                        .collect::<Result<Vec<_>>>()?;
                    if diagonal || other {
                        let mut wires: Vec<String> = c.iter().map(|(w, _)| w.clone()).collect();
                        wires.extend(targets);
                        out.push(Node::Other(wires));
                    } else if op == "swap" {
                        out.push(Node::Swap(c, targets[0].clone(), targets[1].clone()));
                    } else {
                        out.push(Node::Flip(c, targets[0].clone()));
                    }
                }
                _ => {
                    return err(
                        "§4.8",
                        format!("unknown action opcode {op} in borrow analysis"),
                    )
                }
            }
        }
        Ok(())
    }
}

/// Symbolic evaluation state of one frame.
struct Evaluation<'a> {
    vars: BTreeMap<String, u32>,
    state: BTreeMap<String, Poly>,
    opaque: BTreeMap<String, u32>,
    borrowed: &'a BTreeSet<u32>,
}
impl Evaluation<'_> {
    fn var(&mut self, w: &str) -> Poly {
        if let Some(p) = self.state.get(w) {
            return p.clone();
        }
        let id = self.vars.len() as u32 + self.opaque.len() as u32;
        let id = self.vars.entry(w.to_string()).or_insert(id);
        let p = Poly::var(*id);
        self.state.insert(w.to_string(), p.clone());
        p
    }
    fn condition(&mut self, controls: &[(String, bool)]) -> Poly {
        let mut c = Poly::one();
        for (w, polarity) in controls {
            let v = self.var(w);
            c = c.and(&if *polarity { v } else { v.xor(&Poly::one()) });
        }
        c
    }
    fn apply(&mut self, node: &Node) -> Result<()> {
        match node {
            Node::Flip(controls, t) => {
                let c = self.condition(controls);
                let v = self.var(t).xor(&c);
                self.state.insert(t.clone(), v);
            }
            Node::Swap(controls, a, b) => {
                let c = self.condition(controls);
                let (x, y) = (self.var(a), self.var(b));
                let d = c.and(&x.xor(&y));
                self.state.insert(a.clone(), x.xor(&d));
                self.state.insert(b.clone(), y.xor(&d));
            }
            Node::Alloc(wires, image) => {
                for (w, bit) in wires.iter().zip(image) {
                    self.state
                        .insert(w.clone(), if *bit { Poly::one() } else { Poly::zero() });
                }
            }
            Node::Free(wires) => {
                for w in wires {
                    if self.var(w) != Poly::zero() {
                        return err("§5.6", "a wire released inside a borrow frame is not clean");
                    }
                    self.state.remove(w);
                }
            }
            Node::Opaque {
                key,
                controls,
                inputs,
                outputs,
                assign,
            } => {
                let ins: Vec<Poly> = inputs.iter().map(|w| self.var(w)).collect();
                let c = self.condition(controls);
                if ins
                    .iter()
                    .chain(std::iter::once(&c))
                    .any(|p| p.mentions(self.borrowed))
                {
                    return err(
                        "§5.6",
                        format!("{key}: a black-box call whose inputs depend on borrowed wires inside a borrow frame needs a §12 borrow_frames claim"),
                    );
                }
                let olds: Vec<Poly> = outputs.iter().map(|w| self.var(w)).collect();
                for (j, (w, old)) in outputs.iter().zip(olds).enumerate() {
                    // A canonical symbol for output j of the black box at these inputs.
                    let id = format!("{key}#{j}#{:?}", ins);
                    let next = self.vars.len() as u32 + self.opaque.len() as u32;
                    let symbol = Poly::var(*self.opaque.entry(id).or_insert(next));
                    let value = if *assign { symbol } else { old.xor(&symbol) };
                    // Under the control condition c: c*value + (1+c)*old.
                    let merged = c.and(&value).xor(&c.xor(&Poly::one()).and(&old));
                    self.state.insert(w.clone(), merged);
                }
            }
            Node::Repeat(n, body) => {
                // The body as a map from its wires' entry values; M^n by squaring.
                let mut wires = BTreeSet::new();
                for b in body {
                    b.wires(&mut wires);
                }
                let entry: BTreeMap<u32, Poly> = wires
                    .iter()
                    .map(|w| {
                        let p = self.var(w);
                        (self.vars[w], p)
                    })
                    .collect();
                let identity: BTreeMap<String, Poly> = wires
                    .iter()
                    .map(|w| (w.clone(), Poly::var(self.vars[w])))
                    .collect();
                let saved = std::mem::replace(&mut self.state, identity);
                for b in body {
                    self.apply(b)?;
                }
                let body_map = std::mem::replace(&mut self.state, saved);
                let compose = |first: &BTreeMap<String, Poly>,
                               then: &BTreeMap<String, Poly>,
                               vars: &BTreeMap<String, u32>| {
                    let sub: BTreeMap<u32, Poly> = first
                        .iter()
                        .filter_map(|(w, p)| vars.get(w).map(|v| (*v, p.clone())))
                        .collect();
                    then.iter()
                        .map(|(w, p)| (w.clone(), p.substitute(&sub)))
                        .collect::<BTreeMap<_, _>>()
                };
                let mut power: BTreeMap<String, Poly> = body_map;
                let mut result: BTreeMap<String, Poly> = wires
                    .iter()
                    .map(|w| (w.clone(), Poly::var(self.vars[w])))
                    .collect();
                let mut k = *n;
                while k > 0 {
                    if k & 1 == 1 {
                        result = compose(&result, &power, &self.vars);
                    }
                    k >>= 1;
                    if k > 0 {
                        power = compose(&power, &power, &self.vars);
                    }
                }
                for (w, p) in result {
                    self.state.insert(w, p.substitute(&entry));
                }
            }
            Node::Other(_) => return err("§5.6", "non-permutation operation inside a frame"),
        }
        Ok(())
    }
}
fn touches(node: &Node, borrowed: &BTreeSet<String>) -> bool {
    let mut w = BTreeSet::new();
    node.wires(&mut w);
    !w.is_disjoint(borrowed)
}
fn flat_flips(frame: &[Node]) -> Option<Vec<(Vec<String>, String)>> {
    frame
        .iter()
        .map(|n| match n {
            Node::Flip(c, t) if c.iter().all(|(_, p)| *p) => {
                Some((c.iter().map(|(w, _)| w.clone()).collect(), t.clone()))
            }
            _ => None,
        })
        .collect()
}
fn instruction(controls: &[String], target: &str) -> Instruction {
    let (op, operands) = match controls.len() {
        0 => ("x".to_string(), vec![target.to_string()]),
        1 => ("cx".into(), vec![controls[0].clone(), target.into()]),
        2 => (
            "ccx".into(),
            vec![controls[0].clone(), controls[1].clone(), target.into()],
        ),
        _ => (
            "mcx".into(),
            vec![format!("({})", controls.join(", ")), target.into()],
        ),
    };
    Instruction {
        lhs: Vec::new(),
        op,
        operands,
        bodies: Vec::new(),
        line: 0,
    }
}
/// The structural route: the frame is V; W; V^-1; W (§5.6 (i)-(iv)).
fn translation_lemma(frame: &[Node], borrowed: &BTreeSet<String>) -> bool {
    let Some(flips) = flat_flips(frame) else {
        return false;
    };
    if !flips.len().is_multiple_of(2) {
        return false;
    }
    let half = flips.len() / 2;
    for vlen in 1..half {
        let (v, w) = (&flips[..vlen], &flips[vlen..half]);
        let v_inverse: Vec<_> = v.iter().rev().cloned().collect();
        if flips[half..half + vlen] == v_inverse[..] && flips[half + vlen..] == *w {
            let x = v.iter().flat_map(|(c, _)| c.iter().cloned()).collect();
            let to = |s: &[(Vec<String>, String)]| {
                s.iter().map(|(c, t)| instruction(c, t)).collect::<Vec<_>>()
            };
            if validate_translation_lemma(&to(v), &to(w), &x, borrowed).is_ok() {
                return true;
            }
        }
    }
    false
}
/// The exact route: P_F = P' ⊗ I_B on the frame's full domain.
fn factorises(frame: &[Node], borrowed: &BTreeSet<String>) -> Result<()> {
    let mut wires = BTreeSet::new();
    for n in frame {
        n.wires(&mut wires);
    }
    let vars: BTreeMap<String, u32> = wires
        .iter()
        .enumerate()
        .map(|(i, w)| (w.clone(), i as u32))
        .collect();
    let borrowed_vars: BTreeSet<u32> = vars
        .iter()
        .filter(|(w, _)| borrowed.contains(*w))
        .map(|(_, v)| *v)
        .collect();
    let mut e = Evaluation {
        state: vars
            .iter()
            .map(|(w, v)| (w.clone(), Poly::var(*v)))
            .collect(),
        vars,
        opaque: BTreeMap::new(),
        borrowed: &borrowed_vars,
    };
    for n in frame {
        e.apply(n)?;
    }
    for (w, p) in &e.state {
        match e.vars.get(w) {
            Some(v) if borrowed.contains(w) => {
                if *p != Poly::var(*v) {
                    return err(
                        "§5.6",
                        format!("borrow frame does not restore borrowed wire {w}"),
                    );
                }
            }
            _ => {
                if p.mentions(&borrowed_vars) {
                    return err(
                        "§5.6",
                        format!("borrow frame output {w} depends on borrowed entry values"),
                    );
                }
            }
        }
    }
    Ok(())
}
impl Object {
    pub(crate) fn validate_borrow_frames(&self, f: &Function) -> Result<()> {
        fn has_borrow(seq: &[Instruction]) -> bool {
            seq.iter()
                .any(|i| i.op == "qborrow" || i.bodies.iter().any(|b| has_borrow(b)))
        }
        if !f.blocks.iter().any(|b| has_borrow(&b.instructions)) {
            return Ok(());
        }
        let mut compiler = Compiler {
            object: self,
            bundles: BTreeMap::new(),
            regions: BTreeMap::new(),
            borrowed: BTreeSet::new(),
            classical: f.params.iter().flatten()
                .filter(|p| !matches!(p.ty, Type::Wire(_) | Type::Region(_, _)))
                .map(|p| (p.name.clone(), format!("{}:{}", f.name, p.name))).collect(),
            generation: 0,
        };
        for p in f.params.iter().flatten() {
            match p.ty {
                Type::Wire(width) => {
                    compiler.produce(&p.name, width as usize);
                }
                Type::Region(a, w) => {
                    let cells = crate::oracle::power_of_two(a as u128)
                        .and_then(|n| usize::try_from(n).ok())
                        .ok_or_else(|| {
                            Error("host limit: region exceeds host address space".into())
                        })?;
                    let all = (0..cells).map(|_| compiler.fresh(w as usize)).collect();
                    compiler.regions.insert(p.name.clone(), (a, all));
                }
                _ => {}
            }
        }
        let mut nodes = Vec::new();
        compiler.declare_regions(f, &mut nodes)?;
        for block in &f.blocks {
            compiler.compile(&block.instructions, &[], &mut nodes)?;
        }
        let borrowed = compiler.borrowed;
        match self.decide_frames(&nodes, &borrowed) {
            Ok(()) => Ok(()),
            // The library route of §5.6: verified certificate borrow_frames claims.
            Err(e) if self.borrow_frames_certified(f)? => {
                let _ = e;
                Ok(())
            }
            Err(e) => Err(e),
        }
    }
    fn decide_frames(&self, nodes: &[Node], borrowed: &BTreeSet<String>) -> Result<()> {
        // Frames: maximal permutation-only runs, from the first to the last
        // operation touching a borrowed wire.
        let mut start = 0;
        while start < nodes.len() {
            if !nodes[start].permutation() {
                if touches(&nodes[start], borrowed) {
                    return err(
                        "§5.6",
                        "borrowed wire touched by a non-permutation instruction",
                    );
                }
                start += 1;
                continue;
            }
            let end = (start..nodes.len())
                .find(|i| !nodes[*i].permutation())
                .unwrap_or(nodes.len());
            let first = (start..end).find(|i| touches(&nodes[*i], borrowed));
            let last = (start..end).rfind(|i| touches(&nodes[*i], borrowed));
            if let (Some(first), Some(last)) = (first, last) {
                let frame = &nodes[first..=last];
                if !translation_lemma(frame, borrowed) {
                    factorises(frame, borrowed)?;
                }
            }
            start = end;
        }
        Ok(())
    }
}
