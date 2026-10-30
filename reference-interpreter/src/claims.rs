//! §12 claim checking: re-execution of exhaustive and sampled claims on the
//! reference semantics, and the syntactic side conditions of structural claims.
//!
//! Encodings the specification leaves open are fixed in SPEC-QUESTIONS 21:
//! a domain element's output tuple is the region's final interface in
//! signature order (oracle: ins, tgt, fault, in-region cells; action: mut
//! bundles, in bundles, region cells in signature order), then the borrowed
//! components; `expected.table` lists one tuple per domain element in
//! enumeration order, each component a canonical decimal (JSON number when
//! at most 2^53 - 1, else string). Interval-arithmetic claims (`numerical`,
//! `channel`) and `expected: "oracle"` (which compares a realisation under a
//! schedule) need the target/schedule executor and are reported as
//! `Unverified` with that reason, never as verified.
use crate::{
    err, json::Json, BitVector, Budget, Error, Function, Object, QuantumBindings, QuantumState,
    Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ClaimVerdict {
    /// Re-executed over D (or the structural rule holds with verified subclaims).
    Proof,
    /// Re-executed over a domain that is not all of D (or sampled).
    Evidence { elements: u128 },
    /// Not checkable by this crate yet; the reason names the missing machinery.
    Unverified(String),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClaimReport {
    pub region: String,
    pub index: usize,
    pub kind: String,
    pub verdict: ClaimVerdict,
}
#[derive(Clone, Debug)]
enum Component {
    Bundle(usize, usize),
    Borrowed(String),
    Cell(usize, usize, usize),
}
struct Domain {
    components: Vec<(Component, usize, BitVector, BitVector)>,
    covers_d: bool,
}
/// ABI slot identity, independent of a function's local spelling. The domain
/// is resolved once against its subject; the same positions bind the reference.
fn interface_slot(f: &Function, name: &str) -> Result<(usize, usize)> {
    f.params
        .iter()
        .enumerate()
        .find_map(|(group, params)| {
            params
                .iter()
                .position(|p| p.name == name)
                .map(|index| (group, index))
        })
        .ok_or_else(|| Error(format!("§12.2: unknown interface component {name}")))
}
fn json_bits(value: &Json, width: usize) -> Result<BitVector> {
    BitVector::parse_unsigned_exact(width, value.uint_text()?)
}
/// Every element of the domain, last component fastest (§12.2).
fn enumerate(domain: &Domain) -> Result<Vec<Vec<BitVector>>> {
    let mut out = Vec::new();
    let mut current: Vec<BitVector> = domain.components.iter().map(|c| c.2.clone()).collect();
    loop {
        out.push(current.clone());
        let mut i = domain.components.len();
        let mut wrapped = true;
        while i > 0 {
            i -= 1;
            if current[i] == domain.components[i].3 {
                current[i] = domain.components[i].2.clone();
            } else {
                current[i] = increment(&current[i])?;
                wrapped = false;
                break;
            }
        }
        if wrapped {
            return Ok(out);
        }
    }
}
fn ones(width: usize) -> Result<BitVector> {
    BitVector::zero(width)?.binary("sub", &BitVector::from_u128(width, 1)?)
}
fn increment(v: &BitVector) -> Result<BitVector> {
    v.binary("add", &BitVector::from_u128(v.width(), 1)?)
}
fn decimal_json(v: &BitVector) -> Json {
    match v.to_u128() {
        Ok(n) if n <= 9_007_199_254_740_991 => Json::Number(n.to_string()),
        _ => Json::String(v.to_decimal()),
    }
}

/// xoshiro256** (Blackman and Vigna), the §12.1 sampler.
pub struct Xoshiro256(pub [u64; 4]);
impl Xoshiro256 {
    pub fn next_word(&mut self) -> u64 {
        let s = &mut self.0;
        let result = s[1].wrapping_mul(5).rotate_left(7).wrapping_mul(9);
        let t = s[1] << 17;
        s[2] ^= s[0];
        s[3] ^= s[1];
        s[1] ^= s[2];
        s[0] ^= s[3];
        s[2] ^= t;
        s[3] = s[3].rotate_left(45);
        result
    }
    /// A uniform value of `width` bits (R = 2^width: no redraws, §12.1).
    pub fn bits(&mut self, width: usize) -> Result<BitVector> {
        let lo = BitVector::zero(width)?;
        let hi = ones(width)?;
        self.draw(width, &lo, &hi)
    }
    /// §12.1: m = ceil(ceil(log2 R)/64) words, least significant first,
    /// reduced modulo 2^ceil(log2 R), redrawn while >= R, plus lo.
    fn draw(&mut self, width: usize, lo: &BitVector, hi: &BitVector) -> Result<BitVector> {
        let range = increment(&hi.binary("sub", lo)?.resize(width + 1, false)?)?;
        let bits = {
            let r_minus_1 = range.binary("sub", &BitVector::from_u128(width + 1, 1)?)?;
            (0..=width)
                .rev()
                .find(|i| r_minus_1.bit(*i).unwrap_or(false))
                .map_or(0, |i| i + 1)
        };
        loop {
            let mut v = BitVector::zero(width + 1)?;
            for word in 0..bits.div_ceil(64) {
                let w = self.next_word();
                for b in 0..64 {
                    let i = word * 64 + b;
                    if i < bits && w >> b & 1 == 1 {
                        v.set(i, true)?;
                    }
                }
            }
            if v.unsigned_cmp(&range)? == std::cmp::Ordering::Less {
                return v
                    .binary("add", &lo.resize(width + 1, false)?)?
                    .resize(width, false);
            }
        }
    }
}

impl Object {
    fn region_function(&self, closure: &str) -> Result<&Function> {
        let identity = self.identity()?;
        let name = identity
            .closure_hashes
            .iter()
            .find_map(|(n, h)| (h.as_deref() == Some(closure)).then_some(n.clone()))
            .ok_or_else(|| Error(format!("§12: no region has closure hash {closure}")))?;
        self.functions
            .get(&name)
            .ok_or_else(|| Error(format!("§12: closure {closure} is not a function")))
    }
    fn domain(&self, f: &Function, d: &Json) -> Result<Domain> {
        let mut components = Vec::new();
        let mut listed = BTreeSet::new();
        let mut full = true;
        for (group, borrowed) in [("bundles", false), ("borrowed", true)] {
            for b in d.get(group)?.array()? {
                let name = b.get("name")?.string()?.to_string();
                let width = usize::try_from(b.get("width")?.uint()?)
                    .map_err(|_| Error("host limit: domain width".into()))?;
                let lo = json_bits(b.get("lo")?, width)?;
                let hi = json_bits(b.get("hi")?, width)?;
                full &= lo.is_zero() && hi == ones(width)?;
                listed.insert(name.clone());
                components.push((
                    if borrowed {
                        Component::Borrowed(name)
                    } else {
                        let (group, index) = interface_slot(f, &name)?;
                        Component::Bundle(group, index)
                    },
                    width,
                    lo,
                    hi,
                ));
            }
        }
        for r in d.get("regions")?.array()? {
            let name = r.get("name")?.string()?.to_string();
            let (group, slot) = interface_slot(f, &name)?;
            let (a, w) = f
                .params
                .iter()
                .flatten()
                .find_map(|p| match p.ty {
                    Type::Region(a, w) if p.name == name => Some((a, w as usize)),
                    _ => None,
                })
                .ok_or_else(|| Error(format!("§12.2: unknown domain region {name}")))?;
            let cells = crate::oracle::power_of_two(a as u128)
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| Error("host limit: region cells".into()))?;
            listed.insert(name.clone());
            match r.get("cells")? {
                Json::String(s) if s == "all" => {
                    for i in 0..cells {
                        components.push((
                            Component::Cell(group, slot, i),
                            w,
                            BitVector::zero(w)?,
                            ones(w)?,
                        ));
                    }
                }
                cs => {
                    full = false;
                    for c in cs.array()? {
                        let i = usize::try_from(c.get("index")?.uint()?)
                            .map_err(|_| Error("§12.2: cell index".into()))?;
                        let lo = json_bits(c.get("lo")?, w)?;
                        let hi = json_bits(c.get("hi")?, w)?;
                        components.push((Component::Cell(group, slot, i), w, lo, hi));
                    }
                }
            }
        }
        // D: every interface bundle and every region in the region's input
        // domain (§5.2, §5.3; tgt/fault included for oracles).
        let required: Vec<&str> = match f.kind.as_str() {
            "oracle" => f.params[..4]
                .iter()
                .flatten()
                .map(|p| p.name.as_str())
                .collect(),
            _ => f.params[..3]
                .iter()
                .flatten()
                .map(|p| p.name.as_str())
                .collect(),
        };
        let covers_d = full && required.iter().all(|n| listed.contains(*n));
        Ok(Domain {
            components,
            covers_d,
        })
    }
    /// Executes region `f` on one domain element; returns the output tuple.
    fn execute_element(
        &self,
        f: &Function,
        element: &[(Component, BitVector)],
        budget: Option<Budget>,
    ) -> Result<Vec<BitVector>> {
        self.execute_element_audited(f, element, budget, None)
    }
    fn execute_element_audited(
        &self,
        f: &Function,
        element: &[(Component, BitVector)],
        budget: Option<Budget>,
        mut audit: Option<&mut Vec<crate::quantum::BorrowObservation>>,
    ) -> Result<Vec<BitVector>> {
        let value = |name: &str, width: usize| -> Result<BitVector> {
            let (group, index) = interface_slot(f, name)?;
            Ok(element
                .iter()
                .find_map(|(c, v)| match c {
                    Component::Bundle(g, j) if (*g, *j) == (group, index) => Some(v.clone()),
                    _ => None,
                })
                .unwrap_or(BitVector::zero(width)?))
        };
        let cell = |region: &str, i: usize, width: usize| -> Result<BitVector> {
            let (group, slot) = interface_slot(f, region)?;
            Ok(element
                .iter()
                .find_map(|(c, v)| match c {
                    Component::Cell(g, s, j) if (*g, *s, *j) == (group, slot, i) => Some(v.clone()),
                    _ => None,
                })
                .unwrap_or(BitVector::zero(width)?))
        };
        let region_dims = |p: &crate::Param| -> Result<(usize, usize)> {
            let Type::Region(a, w) = p.ty else {
                return err("§5.1", "mem slot requires region type");
            };
            let cells = crate::oracle::power_of_two(a as u128)
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| Error("host limit: region cells".into()))?;
            Ok((cells, w as usize))
        };
        if f.kind == "oracle" {
            let budget = budget.ok_or_else(|| Error("§12: oracle claim needs a budget".into()))?;
            let mut input = crate::OracleVectorInput::default();
            for p in &f.params[0] {
                input.ins.push(value(&p.name, p.ty.bits(self.ptr_width)?)?);
            }
            for p in &f.params[3] {
                let (cells, w) = region_dims(p)?;
                let bytes = (0..cells)
                    .map(|i| Ok(cell(&p.name, i, w)?.to_u128()? as u8))
                    .collect::<Result<_>>()?;
                input.regions.insert(p.name.clone(), bytes);
            }
            if !f.params[4].is_empty() {
                return err(
                    "§12",
                    "claim domains carry no classical arguments for this oracle",
                );
            }
            let r = self.run_oracle_vectors(&f.name, &input, budget)?;
            let mut out = input.ins.clone();
            out.push(value(&f.params[1][0].name, r.payload.width())?.binary("xor", &r.payload)?);
            out.push(value(&f.params[2][0].name, r.code.width())?.binary("xor", &r.code)?);
            for p in &f.params[3] {
                let (cells, w) = region_dims(p)?;
                for i in 0..cells {
                    out.push(cell(&p.name, i, w)?);
                }
            }
            return Ok(out);
        }
        if !f.params[3].is_empty() {
            return err(
                "§12",
                "claim domains carry no classical arguments for this action",
            );
        }
        let mut q = QuantumState::new();
        let mut b = QuantumBindings::default();
        let set = |q: &mut QuantumState, wires: &[u64], v: &BitVector| -> Result<()> {
            for (i, w) in wires.iter().enumerate() {
                if v.bit(i)? {
                    q.gate("x", &[*w], None, &[])?;
                }
            }
            Ok(())
        };
        for p in f.params[0].iter().chain(&f.params[1]) {
            let width = p.ty.bits(self.ptr_width)?;
            let wires = q.allocate(width)?;
            set(&mut q, &wires, &value(&p.name, width)?)?;
            b.bundles.insert(p.name.clone(), wires);
        }
        for p in &f.params[2] {
            let (cells, w) = region_dims(p)?;
            let mut all = Vec::new();
            for i in 0..cells {
                let wires = q.allocate(w)?;
                set(&mut q, &wires, &cell(&p.name, i, w)?)?;
                all.push(wires);
            }
            let Type::Region(a, w) = p.ty else {
                return err("§5.1", "mem slot requires region type");
            };
            b.regions.insert(
                p.name.clone(),
                crate::QuantumRegion {
                    address_width: a,
                    cell_width: w,
                    cells: all,
                },
            );
        }
        let mut borrowed = Vec::new();
        for (c, v) in element {
            if let Component::Borrowed(_) = c {
                let wires = q.allocate(v.width())?;
                set(&mut q, &wires, v)?;
                borrowed.push(wires.clone());
                b.borrowed.push(wires);
            }
        }
        if audit.is_some() {
            q.start_borrow_audit()?;
        }
        self.execute_action(&f.name, &mut q, &b, false, &[])?;
        if let Some(audit) = &mut audit {
            audit.extend(q.take_borrow_audit());
        }
        let read = |q: &QuantumState, wires: &[u64]| -> Result<BitVector> {
            let mut v = BitVector::zero(wires.len())?;
            for (i, w) in wires.iter().enumerate() {
                v.set(i, q.read_basis(&[*w])? == 1)?;
            }
            Ok(v)
        };
        let mut out = Vec::new();
        for p in f.params[0].iter().chain(&f.params[1]) {
            out.push(read(&q, &b.bundles[&p.name])?);
        }
        for p in &f.params[2] {
            for c in &b.regions[&p.name].cells {
                out.push(read(&q, c)?);
            }
        }
        for wires in &borrowed {
            out.push(read(&q, wires)?);
        }
        Ok(out)
    }
    fn claim_budget(&self, claim: &Json, f: &Function) -> Result<Option<Budget>> {
        if f.kind != "oracle" {
            return Ok(None);
        }
        let Some(subject) = claim.object()?.get("subject") else {
            return Ok(None);
        };
        let Some(schedule) = self.section("schedule") else {
            return Ok(None);
        };
        if subject.get("schedule_hash")?.string()?
            != crate::sha256_hex(schedule.canonical().as_bytes())
        {
            return Ok(None);
        }
        let b = schedule.get("bindings")?.get("budget")?.array()?;
        Ok(Some(Budget {
            steps: u64::try_from(b[0].uint()?).map_err(|_| Error("§11: steps".into()))?,
            heap: b[1].uint()?,
            stack: b[2].uint()?,
            depth: b[3].uint()?,
        }))
    }
    /// Checks one claim of region `closure`. A false claim is an error; an
    /// uncheckable one is `Unverified` with its reason.
    pub fn verify_claim(&self, closure: &str, index: usize) -> Result<ClaimVerdict> {
        self.verify_claim_guarded(closure, index, &mut BTreeSet::new())
    }
    fn verify_claim_guarded(
        &self,
        closure: &str,
        index: usize,
        active: &mut BTreeSet<(String, usize)>,
    ) -> Result<ClaimVerdict> {
        if !active.insert((closure.to_string(), index)) {
            return err("§12", "cyclic claim reference");
        }
        let certificate = self
            .section("certificate")
            .ok_or_else(|| Error("§12: object has no certificate".into()))?;
        let region = certificate.get("regions")?.get(closure)?;
        let claims = region.get("claims")?.array()?;
        let claim = claims
            .get(index)
            .ok_or_else(|| Error(format!("§12: claim index {index} out of range")))?;
        let f = self.region_function(closure)?;
        let kind = claim.get("kind")?.string()?;
        let verdict = match kind {
            "exhaustive" | "sampled" => {
                let domain = self.domain(f, claim.get("domain")?)?;
                let budget = self.claim_budget(claim, f)?;
                if f.kind == "oracle" && budget.is_none() {
                    return Ok(ClaimVerdict::Unverified(
                        "an oracle claim needs a budget: bind its subject to this object's schedule".into(),
                    ));
                }
                let expected = claim.object()?.get("expected");
                if expected == Some(&Json::String("oracle".into())) {
                    return Ok(ClaimVerdict::Unverified(
                        "expected \"oracle\" compares a realisation under a schedule (target/schedule executor staged)".into(),
                    ));
                }
                let mut elements = Vec::new();
                if kind == "exhaustive" {
                    for lo in &domain.components {
                        if lo.2.unsigned_cmp(&lo.3)? == std::cmp::Ordering::Greater {
                            return err("§12.2", "domain interval with lo > hi");
                        }
                    }
                    elements = enumerate(&domain)?;
                } else {
                    let sampler = claim.get("sampler")?;
                    let seed = sampler
                        .get("seed")?
                        .array()?
                        .iter()
                        .map(|w| {
                            w.string()?
                                .parse::<u64>()
                                .map_err(|_| Error("§12.1: seed".into()))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let mut rng = Xoshiro256([seed[0], seed[1], seed[2], seed[3]]);
                    for _ in 0..sampler.get("count")?.uint()? {
                        elements.push(
                            domain
                                .components
                                .iter()
                                .map(|(_, w, lo, hi)| rng.draw(*w, lo, hi))
                                .collect::<Result<_>>()?,
                        );
                    }
                }
                let table = match expected {
                    Some(e) if e.object().is_ok_and(|o| o.contains_key("table")) => {
                        Some(e.get("table")?.array()?.to_vec())
                    }
                    _ => None,
                };
                let program = match expected {
                    Some(e) if e.object().is_ok_and(|o| o.contains_key("program")) => {
                        Some(self.region_function(e.get("program")?.string()?)?)
                    }
                    _ => None,
                };
                if let Some(t) = &table {
                    if kind == "exhaustive" && t.len() != elements.len() {
                        return err(
                            "§12.1",
                            format!(
                                "expected table has {} rows for {} domain elements",
                                t.len(),
                                elements.len()
                            ),
                        );
                    }
                }
                for (row, values) in elements.iter().enumerate() {
                    let element: Vec<(Component, BitVector)> = domain
                        .components
                        .iter()
                        .map(|c| c.0.clone())
                        .zip(values.iter().cloned())
                        .collect();
                    let out = self.execute_element(f, &element, budget)?;
                    if let Some(t) = &table {
                        let entry = t
                            .get(row)
                            .ok_or_else(|| Error("§12.1: expected table is too short".into()))?;
                        let got: Vec<Json> = out.iter().map(decimal_json).collect();
                        if entry.array()? != got.as_slice() {
                            return err("§12.1", format!("claim {index} of {closure}: element {row} yields {got:?}, the table says {entry:?}"));
                        }
                    }
                    if let Some(p) = program {
                        if p.signature() != f.signature() {
                            return err("§12.1", "expected program has a different signature");
                        }
                        let other = self.execute_element(p, &element, budget)?;
                        if other != out {
                            return err("§12.1", format!("claim {index} of {closure}: element {row} differs from the expected program"));
                        }
                    }
                }
                if kind == "exhaustive" && domain.covers_d {
                    ClaimVerdict::Proof
                } else {
                    ClaimVerdict::Evidence {
                        elements: elements.len() as u128,
                    }
                }
            }
            "structural" => {
                for sub in claim.get("subclaims")?.array()? {
                    let s =
                        usize::try_from(sub.uint()?).map_err(|_| Error("§12: subclaim".into()))?;
                    if self.verify_claim_guarded(closure, s, active)? != ClaimVerdict::Proof {
                        return Ok(ClaimVerdict::Unverified(format!(
                            "subclaim {s} is not a verified proof"
                        )));
                    }
                }
                let witness = claim.get("witness")?;
                match claim.get("rule")?.string()? {
                    "translation_lemma" => {
                        self.check_translation_witness(f, witness)?;
                        ClaimVerdict::Proof
                    }
                    "inverse_pair_library" => {
                        let library = witness.get("library_closure_hash")?.string()?;
                        let i = usize::try_from(witness.get("claim_index")?.uint()?)
                            .map_err(|_| Error("§12: claim index".into()))?;
                        if certificate.get("regions")?.object()?.contains_key(library) {
                            self.verify_claim_guarded(library, i, active)?
                        } else {
                            ClaimVerdict::Unverified(format!(
                                "library {library} is not linked into this certificate"
                            ))
                        }
                    }
                    _ => ClaimVerdict::Unverified(
                        "the adjoint_table rule's side conditions are not specified beyond its subclaims".into(),
                    ),
                }
            }
            _ => ClaimVerdict::Unverified(format!(
                "{kind} claims need the rigorous interval checker (§12.1), staged as deliverable 11"
            )),
        };
        active.remove(&(closure.to_string(), index));
        Ok(verdict)
    }
    fn check_translation_witness(&self, f: &Function, w: &Json) -> Result<()> {
        let hash = &self.identity()?.interface_hashes[&f.name];
        let points = |key: &str| -> Result<Vec<&crate::Instruction>> {
            w.get(key)?
                .array()?
                .iter()
                .map(|p| {
                    let p = p.array()?;
                    if p.len() != 3 || p[0].string()? != hash {
                        return err("§5.6/§12.1", "translation-lemma point outside the region");
                    }
                    let point = self
                        .point(hash, p[1].string()?, p[2].uint()?)
                        .map_err(|e| Error(format!("§5.6/§12.1: {e}")))?;
                    if !std::ptr::eq(point.function, f) {
                        return err("§5.6/§12.1", "point outside subject");
                    }
                    Ok(point.instruction)
                })
                .collect()
        };
        let names = |key: &str| -> Result<BTreeSet<String>> {
            w.get(key)?
                .array()?
                .iter()
                .map(|s| Ok(s.string()?.to_string()))
                .collect()
        };
        let frame = points("frame")?;
        if frame.len() != 2 {
            return err("§5.6/§12.1", "frame needs two endpoints");
        }
        self.check_translation_frame(
            f,
            [frame[0], frame[1]],
            &points("V")?,
            &points("W")?,
            &names("X")?,
            &names("B")?,
        )
    }
    /// Checks every claim of the certificate: false claims are errors, and
    /// the report lists each claim's verdict.
    pub fn verify_certificate(&self) -> Result<Vec<ClaimReport>> {
        let certificate = self
            .section("certificate")
            .ok_or_else(|| Error("§12: object has no certificate".into()))?;
        let mut reports = Vec::new();
        for (closure, region) in certificate.get("regions")?.object()? {
            let name = self.region_function(closure)?.name.clone();
            let f = self.region_function(closure)?;
            for (index, claim) in region.get("claims")?.array()?.iter().enumerate() {
                reports.push(ClaimReport {
                    region: name.clone(),
                    index,
                    kind: claim.get("kind")?.string()?.into(),
                    verdict: self.verify_claim(closure, index)?,
                });
            }
            // A borrow_frames claim must be a proof over D with the §5.6 property.
            for i in region.get("verification")?.get("borrow_frames")?.array()? {
                let i = usize::try_from(i.uint()?).map_err(|_| Error("§12: claim index".into()))?;
                if reports
                    .iter()
                    .any(|r| r.region == name && r.index == i && r.verdict != ClaimVerdict::Proof)
                {
                    return err(
                        "§5.6",
                        format!("borrow_frames claim {i} of @{name} is not a proof over D"),
                    );
                }
                self.check_borrow_independence(f, closure, i)?;
            }
        }
        Ok(reports)
    }
    /// §5.6 library route: a region whose frames the symbolic decision cannot
    /// settle is admitted iff each of its certificate `borrow_frames` claims
    /// is a verified proof over D that restores every borrowed component and
    /// whose other outputs do not depend on the borrowed entry values.
    pub(crate) fn borrow_frames_certified(&self, f: &Function) -> Result<bool> {
        let Some(certificate) = self.section("certificate") else {
            return Ok(false);
        };
        let identity = self.identity()?;
        let Some(Some(closure)) = identity.closure_hashes.get(&f.name) else {
            return Ok(false);
        };
        let Ok(region) = certificate.get("regions").and_then(|r| r.get(closure)) else {
            return Ok(false);
        };
        let indices = region.get("verification")?.get("borrow_frames")?.array()?;
        if indices.is_empty() {
            return Ok(false);
        }
        for i in indices {
            let i = usize::try_from(i.uint()?).map_err(|_| Error("§12: claim index".into()))?;
            if self.verify_claim(closure, i)? != ClaimVerdict::Proof {
                return err(
                    "§5.6",
                    format!("borrow_frames claim {i} is not a verified proof over D"),
                );
            }
            self.check_borrow_independence(f, closure, i)?;
        }
        Ok(true)
    }
    fn check_borrow_independence(&self, f: &Function, closure: &str, index: usize) -> Result<()> {
        let certificate = self
            .section("certificate")
            .ok_or_else(|| Error("§12: object has no certificate".into()))?;
        let claims = certificate
            .get("regions")?
            .get(closure)?
            .get("claims")?
            .array()?;
        let claim = claims
            .get(index)
            .ok_or_else(|| Error("§12: claim index out of range".into()))?;
        let domain = self.domain(f, claim.get("domain")?)?;
        if !domain
            .components
            .iter()
            .any(|c| matches!(c.0, Component::Borrowed(_)))
        {
            return err(
                "§5.6",
                "a borrow_frames claim must range over the borrowed bundles",
            );
        }
        // Group elements by their non-borrowed part: outputs must agree and
        // borrowed outputs must equal borrowed inputs.
        let mut by_rest: BTreeMap<Vec<String>, Vec<String>> = BTreeMap::new();
        let mut at_return = BTreeMap::new();
        for values in enumerate(&domain)? {
            let element: Vec<(Component, BitVector)> = domain
                .components
                .iter()
                .map(|c| c.0.clone())
                .zip(values)
                .collect();
            let mut audit = Vec::new();
            let out = self.execute_element_audited(f, &element, None, Some(&mut audit))?;
            // Each completed lifetime is checked before any future borrowing.
            // Prior checked frames are independent of their returned entries,
            // so reusing a physical pool bit cannot cancel an invalid lifetime.
            for (ordinal, (wires, rest, output)) in audit.into_iter().enumerate() {
                let key = (ordinal, wires, rest);
                if at_return.get(&key).is_some_and(|prior| *prior != output) {
                    return err("§5.6", "qreturn outputs depend on its borrowed entry");
                }
                at_return.insert(key, output);
            }
            let borrowed_in: Vec<String> = element
                .iter()
                .filter(|(c, _)| matches!(c, Component::Borrowed(_)))
                .map(|(_, v)| v.to_decimal())
                .collect();
            let k = borrowed_in.len();
            let split = out
                .len()
                .checked_sub(k)
                .ok_or_else(|| Error("§12: output tuple".into()))?;
            let borrowed_out: Vec<String> =
                out[split..].iter().map(BitVector::to_decimal).collect();
            if borrowed_in != borrowed_out {
                let names: Vec<&str> = element
                    .iter()
                    .filter_map(|(c, _)| match c {
                        Component::Borrowed(n) => Some(n.as_str()),
                        _ => None,
                    })
                    .collect();
                return err(
                    "§5.6",
                    format!("borrowed components {names:?} are not restored"),
                );
            }
            let rest: Vec<String> = element
                .iter()
                .filter(|(c, _)| !matches!(c, Component::Borrowed(_)))
                .map(|(_, v)| v.to_decimal())
                .collect();
            let outputs: Vec<String> = out[..split].iter().map(BitVector::to_decimal).collect();
            if by_rest.get(&rest).is_some_and(|prior| *prior != outputs) {
                return err("§5.6", "outputs depend on borrowed entry values");
            }
            by_rest.insert(rest, outputs);
        }
        Ok(())
    }
}
