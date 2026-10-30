//! §5.6 borrowed systems: the translation lemma, and an exact symbolic decision
//! of the frame factorisation P_F = P' ⊗ I_B.
//!
//! A frame is a permutation, so its action on basis states is a vector of
//! Boolean functions. Each is held in algebraic normal form (ANF: an XOR of
//! AND-monomials over the frame's input wires), which is canonical: two ANFs
//! denote the same function iff they are equal. P_F = P' ⊗ I_B iff every
//! borrowed output equals its input variable and no other output mentions a
//! borrowed variable. For Boolean-gate circuits this is a total, exact decision
//! over the frame's full domain (T3), with no wire-count limit; permutation loops use repeated
//! squaring; mixed loops carry facts through every iteration or a proved invariant. Opaque applications retain their full
//! input expressions: composition substitutes recursively into those inputs and
//! interns the resulting application, rather than freezing an ANF variable.
//! This proves identities valid for arbitrary black-box functions; unresolved
//! function-specific identities need a checked claim. Calls whose outputs would
//! depend on borrowed wires cannot be decided this way: they require the
//! §12 borrow_frames claim the specification names, checked when present
//! (src/claims.rs). SPEC-QUESTIONS 20 records this reading.
use crate::{
    err,
    object::list,
    validation::{gate, gate_operands, normalize_call},
    Error, Function, Instruction, Object, Result, Type,
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};

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
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
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
}

/// A compiled operation on named wires.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Node {
    /// Target ^= AND of controls (with polarity).
    Flip(Vec<(String, bool)>, String),
    /// Controlled exchange of two wires.
    Swap(Vec<(String, bool)>, String, String),
    /// The actual `n` repetitions of a structured body. Permutation bodies
    /// compose carried maps AND return obligations by squaring; mixed bodies
    /// transfer gate facts per visit or under a proved invariant. This includes
    /// borrowing inside inlined child calls.
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
    /// A physical non-permutation call boundary, with its full structural touch
    /// set separate from the role-aware transfer. None is an unresolved import.
    Call {
        touched: Vec<String>,
        controls: Vec<(String, bool)>,
        modified: Vec<String>,
        body: Option<Vec<Node>>,
    },
    /// Known gate action, separate from its FULL touched set used by §5.6.
    Quantum {
        key: String,
        inverse: String,
        controls: Vec<(String, bool)>,
        targets: Vec<String>,
        diagonal: bool,
    },
    /// Allocate clean wires; image preparation is represented by explicit flips.
    Alloc(Vec<String>),
    Free(Vec<String>),
    /// Unconditional lifetime boundaries, retained inside Repeat and adjoints.
    Borrow(Vec<String>),
    Return(Vec<String>),
}
impl Node {
    fn wires(&self, out: &mut BTreeSet<String>) {
        match self {
            Self::Call { touched, body, .. } => {
                out.extend(touched.iter().cloned());
                if let Some(body) = body {
                    for n in body {
                        n.wires(out);
                    }
                }
            }
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
            Self::Quantum {
                controls, targets, ..
            } => {
                out.extend(controls.iter().map(|(w, _)| w.clone()));
                out.extend(targets.iter().cloned());
            }
            Self::Other(w) | Self::Alloc(w) | Self::Free(w) | Self::Borrow(w) | Self::Return(w) => {
                out.extend(w.iter().cloned());
            }
        }
    }
    fn permutation(&self) -> bool {
        match self {
            Self::Other(_) | Self::Quantum { .. } | Self::Call { .. } => false,
            Self::Repeat(n, body) => *n == 0 || body.iter().all(Self::permutation),
            _ => true,
        }
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
            Self::Quantum {
                key,
                inverse,
                controls,
                targets,
                diagonal,
            } => Self::Quantum {
                key: inverse.clone(),
                inverse: key.clone(),
                controls: controls.clone(),
                targets: targets.clone(),
                diagonal: *diagonal,
            },
            Self::Call {
                touched,
                controls,
                modified,
                body,
            } => Self::Call {
                touched: touched.clone(),
                controls: controls.clone(),
                modified: modified.clone(),
                body: body
                    .as_ref()
                    .map(|b| b.iter().rev().map(Self::adjoint).collect()),
            },
            Self::Alloc(w) => Self::Free(w.clone()),
            Self::Free(w) => Self::Alloc(w.clone()),
            Self::Borrow(w) => Self::Return(w.clone()),
            Self::Return(w) => Self::Borrow(w.clone()),
            other => other.clone(),
        }
    }
}

impl Node {
    /// Instantiate a cached callee transfer, adding inherited controls only to
    /// gates. Allocation, borrowing, image preparation and release stay unconditional.
    fn instantiate(&self, wires: &BTreeMap<String, String>, context: &[(String, bool)]) -> Self {
        let w = |s: &String| wires[s].clone();
        let ws = |v: &[String]| v.iter().map(w).collect();
        let cs = |c: &[(String, bool)]| {
            let mut out = context.to_vec();
            for (s, p) in c {
                let x = (w(s), *p);
                if !out.contains(&x) {
                    out.push(x);
                }
            }
            out
        };
        match self {
            Self::Flip(c, t) => Self::Flip(cs(c), w(t)),
            Self::Swap(c, a, b) => Self::Swap(cs(c), w(a), w(b)),
            Self::Repeat(n, b) => Self::Repeat(
                *n,
                b.iter().map(|n| n.instantiate(wires, context)).collect(),
            ),
            Self::Quantum {
                key,
                inverse,
                controls,
                targets,
                diagonal,
            } => Self::Quantum {
                key: key.clone(),
                inverse: inverse.clone(),
                controls: cs(controls),
                targets: ws(targets),
                diagonal: *diagonal,
            },
            Self::Opaque {
                key,
                controls,
                inputs,
                outputs,
                assign,
            } => Self::Opaque {
                key: key.clone(),
                controls: cs(controls),
                inputs: ws(inputs),
                outputs: ws(outputs),
                assign: *assign,
            },
            Self::Call {
                touched,
                controls,
                modified,
                body,
            } => Self::Call {
                touched: context
                    .iter()
                    .map(|(w, _)| w.clone())
                    .chain(ws(touched))
                    .collect(),
                controls: cs(controls),
                modified: ws(modified),
                body: body
                    .as_ref()
                    .map(|b| b.iter().map(|n| n.instantiate(wires, context)).collect()),
            },
            Self::Other(v) => Self::Other(ws(v)),
            Self::Alloc(v) => Self::Alloc(ws(v)),
            Self::Free(v) => Self::Free(ws(v)),
            Self::Borrow(v) => Self::Borrow(ws(v)),
            Self::Return(v) => Self::Return(ws(v)),
        }
    }
}
#[derive(Clone)]
struct CallTransfer {
    bundles: BTreeMap<String, Vec<String>>,
    regions: BTreeMap<String, (u64, Vec<Vec<String>>)>,
    body: Vec<Node>,
    borrowed: BTreeSet<String>,
}
type CallCache = Rc<RefCell<BTreeMap<(String, Vec<String>), CallTransfer>>>;

#[derive(Clone)]
struct Trace {
    bundles: BTreeMap<String, Vec<String>>,
    borrowed: BTreeSet<String>,
    nodes: Vec<Node>,
}
struct Compiler<'a> {
    calls: CallCache,
    trace: Option<BTreeMap<usize, Vec<Trace>>>,
    live_borrowed: BTreeSet<String>,
    object: &'a Object,
    bundles: BTreeMap<String, Vec<String>>,
    regions: BTreeMap<String, (u64, Vec<Vec<String>>)>,
    borrowed: BTreeSet<String>,
    classical: BTreeMap<String, String>,
    generation: usize,
}
impl Compiler<'_> {
    fn classical(&self, name: &str) -> Result<String> {
        self.classical
            .get(name)
            .cloned()
            .ok_or_else(|| Error(format!("§3.3: unknown classical binding {name}")))
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
                .ok_or_else(|| Error("host limit: region exceeds host address space".into()))?;
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
            out.push(Node::Alloc(flat.clone()));
            for (bit, wire) in flat.into_iter().enumerate() {
                if image[bit / 8] >> (bit % 8) & 1 == 1 {
                    // §4.6: image gates, like allocation/release, are unconditional.
                    out.push(Node::Flip(Vec::new(), wire));
                }
            }
            self.regions.insert(a[0].clone(), (aw, all));
        }
        Ok(())
    }
    /// Compile the complete §4.6 local-region lifetime, not just its entry.
    /// Reversing these nodes (§5.3) recreates each image before the inverse
    /// body and restores/releases it afterward, even under a call predicate.
    fn compile_function(
        &mut self,
        f: &Function,
        context: &[(String, bool)],
        out: &mut Vec<Node>,
    ) -> Result<()> {
        let mut entry = Vec::new();
        self.declare_regions(f, &mut entry)?;
        let exit: Vec<_> = entry.iter().rev().map(Node::adjoint).collect();
        out.extend(entry);
        for block in &f.blocks {
            self.compile(&block.instructions, context, out)?;
        }
        out.extend(exit);
        Ok(())
    }
    fn call_transfer(
        &mut self,
        f: &Function,
        args: &[String],
        controls: &[(String, bool)],
        adjoint: bool,
    ) -> Result<Vec<Node>> {
        let classical = list(&args[4])?
            .iter()
            .map(|n| self.classical(n))
            .collect::<Result<Vec<_>>>()?;
        let key = (f.name.clone(), classical.clone());
        let cached = self.calls.borrow().get(&key).cloned();
        let transfer = if let Some(t) = cached {
            t
        } else {
            // §5.1 action cycles have already been excluded by SCC validation.
            // Compile each callee once per classical specialization, in canonical
            // wire coordinates, then reuse its per-wire structured transfer.
            let mut child = self.object.borrow_compiler(f)?;
            child.calls = self.calls.clone();
            for (p, v) in f.params[3].iter().zip(classical) {
                child.classical.insert(p.name.clone(), v);
            }
            let bundles = child.bundles.clone();
            let regions = child.regions.clone();
            let mut body = Vec::new();
            child.compile_function(f, &[], &mut body)?;
            let t = CallTransfer {
                bundles,
                regions,
                body,
                borrowed: child.borrowed,
            };
            self.calls.borrow_mut().insert(key, t.clone());
            t
        };
        let mut wires = BTreeMap::new();
        for group in [0, 1] {
            for (p, n) in f.params[group].iter().zip(list(&args[group + 1])?) {
                for (from, to) in transfer.bundles[&p.name].iter().zip(self.bundle(&n)?) {
                    wires.insert(from.clone(), to);
                }
            }
        }
        for (p, n) in f.params[2].iter().zip(list(&args[3])?) {
            for (from, to) in transfer.regions[&p.name]
                .1
                .iter()
                .flatten()
                .zip(self.regions[&n].1.iter().flatten())
            {
                wires.insert(from.clone(), to.clone());
            }
        }
        let mut all = BTreeSet::new();
        for n in &transfer.body {
            n.wires(&mut all);
        }
        for from in all {
            if !wires.contains_key(&from) {
                wires.insert(from, self.fresh(1).remove(0));
            }
        }
        self.borrowed
            .extend(transfer.borrowed.iter().map(|w| wires[w].clone()));
        let body: Vec<_> = transfer
            .body
            .iter()
            .map(|n| n.instantiate(&wires, controls))
            .collect();
        Ok(if adjoint {
            body.iter().rev().map(Node::adjoint).collect()
        } else {
            body
        })
    }
    fn compile(
        &mut self,
        seq: &[Instruction],
        context: &[(String, bool)],
        out: &mut Vec<Node>,
    ) -> Result<()> {
        let mut predicate = context.to_vec();
        for i in seq {
            let start = out.len();
            let snapshot = self.trace.as_ref().map(|_| Trace {
                bundles: self.bundles.clone(),
                borrowed: self.live_borrowed.clone(),
                nodes: Vec::new(),
            });
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
                        self.borrowed.extend(wires.iter().cloned());
                        self.live_borrowed.extend(wires.iter().cloned());
                        out.push(Node::Borrow(wires));
                    } else {
                        out.push(Node::Alloc(wires.clone()));
                    }
                }
                "qfree" => out.push(Node::Free(self.bundle(&a[0])?)),
                "qreturn" => {
                    let wires = self.bundle(&a[0])?;
                    for w in &wires {
                        self.live_borrowed.remove(w);
                    }
                    out.push(Node::Return(wires));
                }
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
                            out.push(Node::Alloc(w.clone()));
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
                    // The static validator has checked the lexical body already.
                    // Lower executed lifetimes once for BOTH names and fact nodes,
                    // as LifetimeWidths does for execution. qregions are hoisted
                    // by compile_function independently of these dynamic visits.
                    if n == 0 {
                        continue;
                    }
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
                        out.push(Node::Repeat(n, body));
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
                        out.push(Node::Alloc(w.clone()));
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
                            key: format!(
                                "oracle:{:?}",
                                (
                                    &callee.name,
                                    self.classical(&ca[1])?,
                                    list(&ca[6])?
                                        .iter()
                                        .map(|n| self.classical(n))
                                        .collect::<Result<Vec<_>>>()?
                                )
                            ),
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
                            calls: self.calls.clone(),
                            trace: None,
                            live_borrowed: BTreeSet::new(),
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
                        child.compile_function(f, &c, &mut body)?;
                        self.generation = child.generation;
                        self.borrowed.extend(child.borrowed);
                        if call.adjoint {
                            body = body.iter().rev().map(Node::adjoint).collect();
                        }
                        out.extend(body);
                    } else {
                        let mut touched: Vec<String> = c.iter().map(|(w, _)| w.clone()).collect();
                        for group in [1, 2] {
                            for n in list(&ca[group])? {
                                touched.extend(self.bundle(&n)?);
                            }
                        }
                        for n in list(&ca[3])? {
                            let (_, cells) = self.regions.get(&n).ok_or_else(|| {
                                Error("§4.7: action mem operand is not a live region".into())
                            })?;
                            touched.extend(cells.iter().flatten().cloned());
                        }
                        let mut modified = Vec::new();
                        for n in list(&ca[1])? {
                            modified.extend(self.bundle(&n)?);
                        }
                        for (p, n) in callee.params[2].iter().zip(list(&ca[3])?) {
                            if p.role.as_deref() == Some("mut") {
                                modified.extend(self.regions[&n].1.iter().flatten().cloned());
                            }
                        }
                        let body = if let Some(f) = self.object.functions.get(&callee.name) {
                            Some(self.call_transfer(f, ca, &c, call.adjoint)?)
                        } else {
                            None
                        };
                        out.push(Node::Call {
                            touched,
                            controls: c,
                            modified,
                            body,
                        });
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
                        let (key, inverse) = match op {
                            "s" => ("s".into(), "sdg".into()),
                            "sdg" => ("sdg".into(), "s".into()),
                            "t" => ("t".into(), "tdg".into()),
                            "tdg" => ("tdg".into(), "t".into()),
                            "rx" | "ry" | "rz" => {
                                if let Ok(angle) = a[0].parse::<f64>() {
                                    (
                                        format!("{op}:{}", angle.to_bits()),
                                        format!("{op}:{}", (-angle).to_bits()),
                                    )
                                } else {
                                    let angle = self.classical(&a[0])?;
                                    (format!("{op}:{angle}"), format!("inverse:{op}:{angle}"))
                                }
                            }
                            _ => (op.into(), op.into()),
                        };
                        out.push(Node::Quantum {
                            key,
                            inverse,
                            controls: c,
                            targets,
                            diagonal,
                        });
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
            if let (Some(mut snapshot), Some(trace)) = (snapshot, &mut self.trace) {
                snapshot.nodes = out[start..].to_vec();
                trace.entry(i.line).or_default().push(snapshot);
            }
        }
        Ok(())
    }
}

/// Extensional identity of one black-box output bit at its symbolic inputs.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Application {
    key: String,
    bit: usize,
    inputs: Vec<Poly>,
}
/// Symbolic evaluation state of one frame.
#[derive(Clone)]
struct Evaluation<'a> {
    opaque_seen: bool,
    obligations: BTreeMap<Poly, String>,
    vars: BTreeMap<String, u32>,
    state: BTreeMap<String, Poly>,
    opaque: BTreeMap<Application, u32>,
    applications: BTreeMap<u32, Application>,
    borrowed: &'a BTreeSet<u32>,
}
impl Evaluation<'_> {
    fn symbol(&mut self, app: Application) -> Result<Poly> {
        self.opaque_seen = true;
        if app.inputs.iter().any(|p| p.mentions(self.borrowed)) {
            return err("§5.6", "composed black-box input depends on borrowed wires; needs a §12 borrow_frames claim");
        }
        let next = self.vars.len() as u32 + self.opaque.len() as u32;
        let id = *self.opaque.entry(app.clone()).or_insert(next);
        self.applications.insert(id, app);
        Ok(Poly::var(id))
    }
    /// Simultaneous substitution, recursively through applications. Inserted
    /// replacement polynomials are already in the destination coordinate system.
    fn substitute(&mut self, p: &Poly, map: &BTreeMap<u32, Poly>) -> Result<Poly> {
        let mut out = Poly::zero();
        for m in &p.0 {
            let mut term = Poly::one();
            for v in m {
                let value = if let Some(value) = map.get(v) {
                    value.clone()
                } else if let Some(mut app) = self.applications.get(v).cloned() {
                    app.inputs = app
                        .inputs
                        .iter()
                        .map(|p| self.substitute(p, map))
                        .collect::<Result<_>>()?;
                    self.symbol(app)?
                } else {
                    Poly::var(*v)
                };
                term = term.and(&value);
            }
            out = out.xor(&term);
        }
        Ok(out)
    }
    fn compose(
        &mut self,
        first: &BTreeMap<String, Poly>,
        then: &BTreeMap<String, Poly>,
    ) -> Result<BTreeMap<String, Poly>> {
        let sub = first
            .iter()
            .filter_map(|(w, p)| self.vars.get(w).map(|v| (*v, p.clone())))
            .collect();
        then.iter()
            .map(|(w, p)| Ok((w.clone(), self.substitute(p, &sub)?)))
            .collect()
    }
    fn transport_obligations(
        &mut self,
        obligations: &BTreeMap<Poly, String>,
        map: &BTreeMap<String, Poly>,
    ) -> Result<BTreeMap<Poly, String>> {
        let sub = map
            .iter()
            .filter_map(|(w, p)| self.vars.get(w).map(|v| (*v, p.clone())))
            .collect();
        obligations
            .iter()
            .map(|(p, message)| Ok((self.substitute(p, &sub)?, message.clone())))
            .collect()
    }
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
            Node::Borrow(wires) => {
                // Every borrowing starts an independent unknown system. A loop
                // summary may reuse its symbolic name only AFTER its return has
                // proved identity and independence on the full entry domain.
                for w in wires {
                    let p = Poly::var(self.vars[w]);
                    self.state.insert(w.clone(), p);
                }
            }
            Node::Return(wires) => {
                let returned: BTreeSet<_> = wires.iter().map(|w| self.vars[w]).collect();
                for w in wires {
                    let difference = self.var(w).xor(&Poly::var(self.vars[w]));
                    self.obligations.insert(
                        difference,
                        format!("qreturn does not restore borrowed wire {w}"),
                    );
                }
                for (w, p) in &self.state {
                    if !wires.contains(w) {
                        // Boolean derivative: zero iff this output is independent
                        // of this lifetime's fresh borrowed entry bit.
                        for v in &returned {
                            let derivative = Poly(
                                p.0.iter()
                                    .filter(|m| m.contains(v))
                                    .map(|m| m.iter().filter(|x| *x != v).copied().collect())
                                    .collect(),
                            );
                            self.obligations.insert(
                                derivative,
                                format!("qreturn output {w} depends on borrowed entry values"),
                            );
                        }
                    }
                }
                for w in wires {
                    self.state.remove(w);
                }
            }
            Node::Alloc(wires) => {
                for w in wires {
                    self.state.insert(w.clone(), Poly::zero());
                }
            }
            Node::Free(wires) => {
                for w in wires {
                    let value = self.var(w);
                    self.obligations.insert(
                        value,
                        "a wire released inside a borrow frame is not clean".into(),
                    );
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
                self.opaque_seen = true;
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
                    let symbol = self.symbol(Application {
                        key: key.clone(),
                        bit: j,
                        inputs: ins.clone(),
                    })?;
                    let value = if *assign { symbol } else { old.xor(&symbol) };
                    // Under the control condition c: c*value + (1+c)*old.
                    let merged = c.and(&value).xor(&c.xor(&Poly::one()).and(&old));
                    self.state.insert(w.clone(), merged);
                }
            }
            Node::Repeat(n, body) => {
                if *n == 0 {
                    return Ok(());
                }
                // Summarize both the carried map AND every internal lifetime
                // obligation. Doubling transports obligations to each reachable
                // entry; known-clean surrounding state is substituted only after
                // composition, without assuming all iterations share that state.
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
                let saved_obligations = std::mem::take(&mut self.obligations);
                for b in body {
                    self.apply(b)?;
                }
                let body_map = std::mem::replace(&mut self.state, saved);
                let mut power: BTreeMap<String, Poly> = body_map;
                let mut power_obligations =
                    std::mem::replace(&mut self.obligations, saved_obligations);
                let mut result_obligations = BTreeMap::new();
                let mut result: BTreeMap<String, Poly> = wires
                    .iter()
                    .map(|w| (w.clone(), Poly::var(self.vars[w])))
                    .collect();
                let mut k = *n;
                while k > 0 {
                    if k & 1 == 1 {
                        result_obligations
                            .extend(self.transport_obligations(&power_obligations, &result)?);
                        result = self.compose(&result, &power)?;
                    }
                    k >>= 1;
                    if k > 0 {
                        power_obligations
                            .extend(self.transport_obligations(&power_obligations, &power)?);
                        power = self.compose(&power, &power)?;
                    }
                }
                for (p, message) in result_obligations {
                    let value = self.substitute(&p, &entry)?;
                    self.obligations.insert(value, message);
                }
                for (w, p) in result {
                    let value = self.substitute(&p, &entry)?;
                    self.state.insert(w, value);
                }
            }
            Node::Other(_) | Node::Quantum { .. } | Node::Call { .. } => {
                return err("§5.6", "non-permutation operation inside a frame")
            }
        }
        Ok(())
    }
}
fn check_permutation_touches(nodes: &[Node], borrowed: &BTreeSet<String>) -> Result<()> {
    for node in nodes {
        if let Node::Repeat(n, body) = node {
            if *n != 0 {
                check_permutation_touches(body, borrowed)?;
            }
        } else if let Node::Call { touched, body, .. } = node {
            if touched.iter().any(|w| borrowed.contains(w)) {
                return err(
                    "§5.6",
                    "borrowed wire touched by a non-permutation instruction",
                );
            }
            if let Some(body) = body {
                check_permutation_touches(body, borrowed)?;
            }
        } else if !node.permutation() && touches(node, borrowed) {
            return err(
                "§5.6",
                "borrowed wire touched by a non-permutation instruction",
            );
        }
    }
    Ok(())
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
enum FrameFailure {
    Invalid(Error),
    NeedsClaim(Error),
}
impl From<Error> for FrameFailure {
    fn from(e: Error) -> Self {
        Self::Invalid(e)
    }
}
/// One causal operation, present on every operand stack. It cancels only when
/// it is on top of ALL its operand stacks, preserving outstanding readers.
/// Saved polynomials preserve correlations; unknown never invents zero.
#[derive(Clone, Debug, PartialEq, Eq)]
struct InverseFact {
    gate: Node,
    before: BTreeMap<String, Poly>,
}
struct Facts<'a> {
    e: Evaluation<'a>,
    borrowed: &'a BTreeSet<String>,
    inverses: BTreeMap<String, Vec<InverseFact>>,
    frame: Vec<Node>,
    unresolved: Option<Error>,
}
impl Facts<'_> {
    fn unknown(&mut self, w: &str) {
        let id = (self.e.vars.len() + self.e.opaque.len()) as u32;
        self.e.vars.insert(format!("unknown_{id}"), id);
        self.e.state.insert(w.into(), Poly::var(id));
    }
    fn forget_inverses(&mut self, touched: &BTreeSet<String>) {
        // An opaque/lifetime touch is a causal barrier. Invalidate the whole
        // connected history, including ancestors reached through another operand.
        let mut affected = touched.clone();
        loop {
            let old = affected.len();
            for w in affected.clone() {
                if let Some(stack) = self.inverses.get(&w) {
                    for fact in stack {
                        fact.gate.wires(&mut affected);
                    }
                }
            }
            if old == affected.len() {
                break;
            }
        }
        for w in affected {
            self.inverses.remove(&w);
        }
    }
    fn reversible(&mut self, node: &Node, touched: &BTreeSet<String>) -> bool {
        let saved = touched
            .iter()
            .next()
            .and_then(|w| self.inverses.get(w))
            .and_then(|stack| stack.last())
            .cloned();
        if let Some(saved) = saved {
            if saved.gate.adjoint() == *node
                && touched
                    .iter()
                    .all(|w| self.inverses.get(w).and_then(|s| s.last()) == Some(&saved))
            {
                for w in touched {
                    let stack = self.inverses.get_mut(w).unwrap();
                    stack.pop();
                    if stack.is_empty() {
                        self.inverses.remove(w);
                    }
                }
                self.e.state.extend(saved.before);
                return true;
            }
        }
        // Basis permutations already retain exact ANFs. Record history only
        // around a quantum loss of information, or above an existing history.
        if !matches!(
            node,
            Node::Quantum {
                diagonal: false,
                ..
            }
        ) && !touched.iter().any(|w| self.inverses.contains_key(w))
        {
            return false;
        }
        let before = touched.iter().map(|w| (w.clone(), self.e.var(w))).collect();
        let fact = InverseFact {
            gate: node.clone(),
            before,
        };
        for w in touched {
            self.inverses
                .entry(w.clone())
                .or_default()
                .push(fact.clone());
        }
        false
    }
    fn flush(&mut self) -> std::result::Result<(), FrameFailure> {
        if self.frame.iter().any(|n| touches(n, self.borrowed)) {
            // The cropped translation lemma remains a sufficient gate-frame
            // proof. Lifetime equations are mandatory even on this route.
            let first = self
                .frame
                .iter()
                .position(|n| touches(n, self.borrowed))
                .unwrap();
            let last = self
                .frame
                .iter()
                .rposition(|n| touches(n, self.borrowed))
                .unwrap();
            let structural = translation_lemma(&self.frame[first..=last], self.borrowed);
            let result = check_factorisation(&self.e, self.borrowed, structural);
            match result {
                Err(FrameFailure::NeedsClaim(e)) => {
                    self.unresolved.get_or_insert(e);
                }
                other => other?,
            }
        }
        self.e.obligations.clear();
        self.frame.clear();
        Ok(())
    }
    fn sequence(&mut self, nodes: &[Node]) -> std::result::Result<(), FrameFailure> {
        for node in nodes {
            if let Node::Repeat(n, body) = node {
                if *n == 0 {
                    continue; // §4.6: no dynamic gate, touch, frame or lifetime.
                }
                let mut wires = BTreeSet::new();
                node.wires(&mut wires);
                if !node.permutation() || wires.iter().any(|w| self.inverses.contains_key(w)) {
                    self.repeat(*n, body)?;
                    continue;
                }
            }
            if let Node::Call {
                controls,
                modified,
                body,
                ..
            } = node
            {
                self.flush()?; // preserve the physical call boundary
                if self.e.condition(controls) != Poly::zero() {
                    if let Some(body) = body {
                        self.sequence(body)?;
                        self.flush()?;
                    } else {
                        self.forget_inverses(&modified.iter().cloned().collect());
                        for w in modified {
                            self.unknown(w);
                        }
                    }
                }
                continue;
            }
            let mut touched = BTreeSet::new();
            node.wires(&mut touched);
            if !node.permutation() {
                self.flush()?;
                if let Node::Quantum {
                    controls,
                    targets,
                    diagonal,
                    ..
                } = node
                {
                    // A known-false predicate disables the gate. Controls are
                    // read-only even when they enable a genuinely quantum gate.
                    if self.e.condition(controls) == Poly::zero() {
                        continue;
                    }
                    if self.reversible(node, &touched) {
                        continue;
                    }
                    if !diagonal {
                        for w in targets {
                            self.unknown(w);
                        }
                    }
                } else {
                    self.forget_inverses(&touched);
                    for w in touched {
                        self.unknown(&w);
                    }
                }
            } else {
                let gate = matches!(node, Node::Flip(..) | Node::Swap(..));
                if gate
                    && self.e.condition(match node {
                        Node::Flip(c, _) | Node::Swap(c, _, _) => c,
                        _ => unreachable!(),
                    }) == Poly::zero()
                {
                    self.frame.push(node.clone());
                    continue;
                }
                let restored = if gate {
                    self.reversible(node, &touched)
                } else {
                    self.forget_inverses(&touched);
                    false
                };
                if let Err(error) = if restored { Ok(()) } else { self.e.apply(node) } {
                    if self.e.opaque_seen {
                        self.unresolved.get_or_insert(error);
                    } else {
                        return Err(FrameFailure::Invalid(error));
                    }
                }
                self.frame.push(node.clone());
            }
        }
        Ok(())
    }
    fn repeat(&mut self, n: u64, body: &[Node]) -> std::result::Result<(), FrameFailure> {
        let mut seen = Vec::new();
        let mut iteration = 0;
        while iteration < n {
            let snapshot = (
                self.e.state.clone(),
                self.e.obligations.clone(),
                self.inverses.clone(),
                self.frame.clone(),
            );
            if let Some((previous, _)) = seen.iter().find(|(_, state)| *state == snapshot) {
                let period = iteration - previous;
                let skip = (n - iteration) / period * period;
                if skip != 0 {
                    iteration += skip;
                    continue;
                }
            }
            // Exact finite traversal is the default. Beyond this work threshold,
            // prove an inductive lattice invariant rather than sampling visits.
            if n - iteration > 1024 && seen.len() >= 1024 {
                return self.invariant(body);
            }
            seen.push((iteration, snapshot));
            self.sequence(body)?;
            iteration += 1;
        }
        Ok(())
    }
    fn invariant(&mut self, body: &[Node]) -> std::result::Result<(), FrameFailure> {
        // Join equal expressions unchanged; disagreement becomes an independent
        // top variable. Every pass removes at least one precise wire fact, or
        // establishes closure. No bound is replaced by a finite sample.
        self.inverses.clear();
        let mut tops: BTreeMap<String, Poly> = BTreeMap::new();
        loop {
            let entry = self.e.state.clone();
            let obligations = self.e.obligations.clone();
            let frame = self.frame.clone();
            self.sequence(body)?;
            self.inverses.clear();
            let mut joined = entry.clone();
            let wires: BTreeSet<_> = entry.keys().chain(self.e.state.keys()).cloned().collect();
            for w in &wires {
                if entry.get(w) != self.e.state.get(w) {
                    let top = if let Some(p) = tops.get(w) {
                        p.clone()
                    } else {
                        self.unknown(w);
                        let p = self.e.state[w].clone();
                        tops.insert(w.clone(), p.clone());
                        p
                    };
                    joined.insert(w.clone(), top);
                }
            }
            if joined == entry && obligations == self.e.obligations && frame == self.frame {
                self.e.state = joined;
                // This state contains every remaining reachable loop entry and
                // exit; body/return proofs above hold for all of them.
                return Ok(());
            }
            self.e.state = joined;
        }
    }
}
fn check_factorisation(
    e: &Evaluation<'_>,
    borrowed: &BTreeSet<String>,
    structural: bool,
) -> std::result::Result<(), FrameFailure> {
    let failure = |error| {
        if e.opaque_seen {
            FrameFailure::NeedsClaim(error)
        } else {
            FrameFailure::Invalid(error)
        }
    };
    for (p, message) in &e.obligations {
        if *p != Poly::zero() {
            return Err(failure(Error(format!("§5.6: {message}"))));
        }
    }
    if structural {
        return Ok(());
    }
    for (w, p) in &e.state {
        match e.vars.get(w) {
            Some(v) if borrowed.contains(w) => {
                if *p != Poly::var(*v) {
                    return Err(failure(Error(format!(
                        "§5.6: borrow frame does not restore borrowed wire {w}"
                    ))));
                }
            }
            _ if p.mentions(e.borrowed) => {
                return Err(failure(Error(format!(
                    "§5.6: borrow frame output {w} depends on borrowed entry values"
                ))));
            }
            _ => {}
        }
    }
    Ok(())
}

impl Object {
    fn borrow_compiler<'a>(&'a self, f: &Function) -> Result<Compiler<'a>> {
        let mut compiler = Compiler {
            calls: Rc::default(),
            trace: None,
            live_borrowed: BTreeSet::new(),
            object: self,
            bundles: BTreeMap::new(),
            regions: BTreeMap::new(),
            borrowed: BTreeSet::new(),
            classical: f
                .params
                .iter()
                .flatten()
                .filter(|p| !matches!(p.ty, Type::Wire(_) | Type::Region(_, _)))
                .map(|p| (p.name.clone(), format!("{}:{}", f.name, p.name)))
                .collect(),
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
        Ok(compiler)
    }
    /// Check a named source frame against the complete compiled circuit,
    /// including inherited predicates, loop structure and physical wire names.
    pub(crate) fn check_translation_frame(
        &self,
        f: &Function,
        endpoints: [&Instruction; 2],
        v: &[&Instruction],
        w: &[&Instruction],
        x: &BTreeSet<String>,
        b: &BTreeSet<String>,
    ) -> Result<()> {
        fn locate<'a>(
            seq: &[&'a Instruction],
            first: &Instruction,
            last: &Instruction,
        ) -> Option<(Vec<&'a Instruction>, usize, usize)> {
            if let (Some(a), Some(b)) = (
                seq.iter().position(|i| std::ptr::eq(*i, first)),
                seq.iter().position(|i| std::ptr::eq(*i, last)),
            ) {
                return Some((seq.to_vec(), a, b));
            }
            seq.iter()
                .flat_map(|i| &i.bodies)
                .find_map(|body| locate(&body.iter().collect::<Vec<_>>(), first, last))
        }
        let root: Vec<_> = f.blocks.iter().flat_map(|b| &b.instructions).collect();
        let (seq, start, end) = locate(&root, endpoints[0], endpoints[1]).ok_or_else(|| {
            Error("§5.6/§12.1: frame endpoints do not delimit one contiguous source circuit".into())
        })?;
        if start > end {
            return err("§5.6/§12.1", "reversed frame endpoints");
        }
        let leading: Vec<_> = v.iter().chain(w).copied().collect();
        if leading.len() > end - start + 1
            || leading
                .iter()
                .zip(&seq[start..])
                .any(|(a, b)| !std::ptr::eq(*a, *b))
        {
            return err(
                "§5.6/§12.1",
                "V and W must name every leading instruction of the frame in order",
            );
        }
        let mut compiler = self.borrow_compiler(f)?;
        compiler.trace = Some(BTreeMap::new());
        compiler.compile_function(f, &[], &mut Vec::new())?;
        let trace = compiler
            .trace
            .ok_or_else(|| Error("§5.6: missing structural trace".into()))?;
        let entries = trace
            .get(&endpoints[0].line)
            .ok_or_else(|| Error("§5.6: frame has no compiled context".into()))?;
        for (context, entry) in entries.iter().enumerate() {
            let resolve = |names: &BTreeSet<String>| -> Result<BTreeSet<String>> {
                let mut out = BTreeSet::new();
                for name in names {
                    let (base, index) = match name.split_once('[') {
                        Some((base, index)) => (
                            base,
                            Some(crate::object::host_count(
                                index.trim_end_matches(']'),
                                "wire index",
                            )? as usize),
                        ),
                        None => (name.as_str(), None),
                    };
                    let wires = entry
                        .bundles
                        .get(base)
                        .ok_or_else(|| Error(format!("§5.6: unknown witness wire {name}")))?;
                    if let Some(i) = index {
                        out.insert(
                            wires
                                .get(i)
                                .ok_or_else(|| {
                                    Error("§5.6: witness wire index out of range".into())
                                })?
                                .clone(),
                        );
                    } else {
                        out.extend(wires.iter().cloned());
                    }
                }
                Ok(out)
            };
            let (x, b) = (resolve(x)?, resolve(b)?);
            if !b.is_subset(&entry.borrowed) || b.is_empty() {
                return err("§5.6/§12.1", "B must identify actual live borrowed wires");
            }
            let get = |i: &Instruction| -> Result<&[Node]> {
                trace
                    .get(&i.line)
                    .and_then(|t| t.get(context))
                    .map(|t| t.nodes.as_slice())
                    .ok_or_else(|| {
                        Error("§5.6: inconsistent structural invocation contexts".into())
                    })
            };
            let collect = |points: &[&Instruction]| -> Result<Vec<Node>> {
                let mut nodes = Vec::new();
                for i in points {
                    nodes.extend_from_slice(get(i)?);
                }
                Ok(nodes)
            };
            let vn = collect(v)?;
            let wn = collect(w)?;
            let frame_points = &seq[start..=end];
            let frame = collect(frame_points)?;
            let expected: Vec<_> = vn
                .iter()
                .cloned()
                .chain(wn.iter().cloned())
                .chain(vn.iter().rev().map(Node::adjoint))
                .chain(wn.iter().cloned())
                .collect();
            if frame != expected {
                return err("§5.6/§12.1", "named frame is not V; W; exact reverse V; W");
            }
            fn flips(nodes: &[Node], out: &mut Vec<Instruction>) -> Result<()> {
                for node in nodes {
                    match node {
                        Node::Flip(c, t) => out.push(instruction(
                            &c.iter().map(|(w, _)| w.clone()).collect::<Vec<_>>(),
                            t,
                        )),
                        Node::Repeat(_, body) => flips(body, out)?,
                        _ => {
                            return err(
                                "§5.6/§12.1",
                                "translation frame contains a non-X-family operation",
                            )
                        }
                    }
                }
                Ok(())
            }
            let (mut vf, mut wf) = (Vec::new(), Vec::new());
            flips(&vn, &mut vf)?;
            flips(&wn, &mut wf)?;
            validate_translation_lemma(&vf, &wf, &x, &b)?;
            let mut touched = BTreeSet::new();
            for node in &frame {
                node.wires(&mut touched);
            }
            if !touched.intersection(&entry.borrowed).all(|w| b.contains(w)) {
                return err(
                    "§5.6/§12.1",
                    "witness omits borrowed wires touched by the frame",
                );
            }
            // First/last borrowed touches delimit the maximal permutation run;
            // lifetime changes and non-permutation operations are boundaries.
            let boundary = |nodes: &[Node]| {
                nodes
                    .iter()
                    .any(|n| !n.permutation() || matches!(n, Node::Borrow(_) | Node::Return(_)))
            };
            if !frame.first().is_some_and(|n| touches(n, &b))
                || !frame.last().is_some_and(|n| touches(n, &b))
            {
                return err("§5.6/§12.1", "frame endpoints must touch B");
            }
            for adjacent in [
                seq[..start].iter().rev().collect::<Vec<_>>(),
                seq[end + 1..].iter().collect(),
            ] {
                for i in adjacent {
                    let nodes = get(i)?;
                    if boundary(nodes) {
                        break;
                    }
                    if nodes.iter().any(|n| touches(n, &b)) {
                        return err("§5.6/§12.1", "named frame omits an adjacent borrowed touch");
                    }
                }
            }
        }
        Ok(())
    }
    /// The §5.6 gate-boundary premise is structural, not an exhaustive tuple
    /// obligation. Check it independently for public certificate verification,
    /// which can also be called on an Object assembled through public fields.
    pub(crate) fn check_borrow_frame_structure(&self, f: &Function) -> Result<()> {
        let mut compiler = self.borrow_compiler(f)?;
        let mut nodes = Vec::new();
        compiler.compile_function(f, &[], &mut nodes)?;
        check_permutation_touches(&nodes, &compiler.borrowed)
    }
    pub(crate) fn validate_borrow_frames(&self, f: &Function) -> Result<()> {
        fn has_borrow(seq: &[Instruction]) -> bool {
            seq.iter()
                .any(|i| i.op == "qborrow" || i.bodies.iter().any(|b| has_borrow(b)))
        }
        if !f.blocks.iter().any(|b| has_borrow(&b.instructions)) {
            return Ok(());
        }
        let mut compiler = self.borrow_compiler(f)?;
        let mut nodes = Vec::new();
        compiler.compile_function(f, &[], &mut nodes)?;
        let borrowed = compiler.borrowed;
        check_permutation_touches(&nodes, &borrowed)?;
        match self.decide_frames(&nodes, &borrowed) {
            Ok(()) => Ok(()),
            // The library route of §5.6: verified certificate borrow_frames claims.
            Err(FrameFailure::NeedsClaim(_)) if self.borrow_frames_certified(f)? => Ok(()),
            Err(FrameFailure::Invalid(e) | FrameFailure::NeedsClaim(e)) => Err(e),
        }
    }
    fn decide_frames(
        &self,
        nodes: &[Node],
        borrowed: &BTreeSet<String>,
    ) -> std::result::Result<(), FrameFailure> {
        check_permutation_touches(nodes, borrowed)?;
        let mut wires = BTreeSet::new();
        for node in nodes {
            node.wires(&mut wires);
        }
        let vars: BTreeMap<String, u32> = wires
            .into_iter()
            .enumerate()
            .map(|(i, w)| (w, i as u32))
            .collect();
        let borrowed_vars = vars
            .iter()
            .filter(|(w, _)| borrowed.contains(*w))
            .map(|(_, v)| *v)
            .collect();
        let e = Evaluation {
            opaque_seen: false,
            obligations: BTreeMap::new(),
            state: vars
                .iter()
                .map(|(w, v)| (w.clone(), Poly::var(*v)))
                .collect(),
            vars,
            opaque: BTreeMap::new(),
            applications: BTreeMap::new(),
            borrowed: &borrowed_vars,
        };
        let mut facts = Facts {
            e,
            borrowed,
            inverses: BTreeMap::new(),
            frame: Vec::new(),
            unresolved: None,
        };
        facts.sequence(nodes)?;
        facts.flush()?;
        match facts.unresolved {
            Some(e) => Err(FrameFailure::NeedsClaim(e)),
            None => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unresolved_earlier_frame_cannot_hide_a_definite_later_nonpermutation() {
        let o = Object::parse(include_str!("../tests/fixtures/inc8.bisa")).unwrap();
        let b = "b".to_string();
        let nodes = [
            Node::Borrow(vec![b.clone()]),
            Node::Opaque {
                key: "unknown".into(),
                controls: vec![],
                inputs: vec![b.clone()],
                outputs: vec!["t".into()],
                assign: false,
            },
            Node::Return(vec![b.clone()]),
            Node::Other(vec![b.clone()]),
        ];
        assert!(matches!(
            o.decide_frames(&nodes, &BTreeSet::from([b])),
            Err(FrameFailure::Invalid(_))
        ));
    }
}
