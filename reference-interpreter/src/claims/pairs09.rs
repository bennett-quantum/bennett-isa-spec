//! K4's entry-bound relation is separate from ordinary K13 tuple replay.
use super::*;
use crate::{Argument, OracleVectorInput, OracleVectorMachine};
pub(super) struct Pair<'a> {
    pub(super) g: &'a Function,
    pub(super) gi: &'a Function,
    pub(super) budget: Budget,
    pub(super) tokens: BTreeMap<String, Budget>,
    pub(super) aliases: Vec<u64>,
}
pub(super) enum PairClaim {
    Exhaustive,
    Library,
}
pub(super) fn pair_claim(claim: &Json) -> Result<PairClaim> {
    match claim.get("kind")?.string()? {
        "exhaustive" if claim.get("expected")? == &Json::String("inverse_pair".into()) => {
            Ok(PairClaim::Exhaustive)
        }
        "structural" if claim.get("rule")?.string()? == "inverse_pair_library" => {
            Ok(PairClaim::Library)
        }
        _ => err(
            "§12.1 K4",
            "entry claim is not an exhaustive or linked-library inverse-pair theorem",
        ),
    }
}
fn entry_budget(value: &Json) -> Result<Budget> {
    let v = value.array()?;
    if v.len() != 4 {
        return err("§12.3 K4", "inverse-pair budget requires four components");
    }
    Ok(Budget {
        steps: u64::try_from(v[0].uint()?)
            .map_err(|_| crate::resource_error("§12.3 K4", "entry steps exceed host u64"))?,
        heap: v[1].uint()?,
        stack: v[2].uint()?,
        depth: v[3].uint()?,
    })
}
pub(super) struct PairContext {
    budget: Budget,
    tokens: BTreeMap<String, Budget>,
}
pub(super) fn pair_context(object: &Object, entry: &Json) -> Result<PairContext> {
    let budget = entry_budget(entry.get("budget")?)?;
    object.validate_budget(budget)?;
    let tokens = entry
        .object()?
        .get("arg_tokens")
        .map(|v| {
            v.object()?
                .iter()
                .map(|(n, b)| Ok((n.clone(), entry_budget(b)?)))
                .collect::<Result<BTreeMap<_, _>>>()
        })
        .transpose()?
        .unwrap_or_default();
    for b in tokens.values() {
        object.validate_budget(*b)?;
    }
    // Formal argument order needs the body; partition shape does not.
    crate::tokens::alias_binding(
        entry.object()?.get("token_aliases"),
        &vec![None; tokens.len() + 1],
        tokens.is_empty(),
    )?;
    Ok(PairContext { budget, tokens })
}
pub(super) fn pair<'a>(object: &'a Object, entry: &Json) -> Result<Pair<'a>> {
    let g = object.region_function(entry.get("g")?.string()?)?;
    let gi = object.region_function(entry.get("ginv")?.string()?)?;
    if g.kind != "oracle"
        || gi.kind != "oracle"
        || g.params[0].is_empty()
        || g.params
            .iter()
            .take(5)
            .map(|ps| ps.iter().map(|p| p.ty.clone()).collect::<Vec<_>>())
            .collect::<Vec<_>>()
            != gi
                .params
                .iter()
                .take(5)
                .map(|ps| ps.iter().map(|p| p.ty.clone()).collect::<Vec<_>>())
                .collect::<Vec<_>>()
        || g.params[0][0].ty != g.params[1][0].ty
    {
        return err(
            "§12.3 K4",
            "inverse-pair oracle signatures are incompatible",
        );
    }
    let PairContext { budget, tokens } = pair_context(object, entry)?;
    let expected = g.params[4]
        .iter()
        .filter(|p| p.ty == Type::Token)
        .map(|p| p.name.as_str())
        .collect::<BTreeSet<_>>();
    if tokens.keys().map(String::as_str).collect::<BTreeSet<_>>() != expected {
        return err(
            "§12.3 K4",
            "arg_tokens must bind exactly the token-typed oracle arguments",
        );
    }
    let mut tuples = vec![Some(budget)];
    tuples.extend(
        g.params[4]
            .iter()
            .filter(|p| p.ty == Type::Token)
            .map(|p| Some(tokens[&p.name])),
    );
    let aliases = crate::tokens::alias_binding(
        entry.object()?.get("token_aliases"),
        &tuples,
        expected.is_empty(),
    )?;
    Ok(Pair {
        g,
        gi,
        budget,
        tokens,
        aliases,
    })
}
pub(super) fn args_domain(
    object: &Object,
    f: &Function,
    domain: &Json,
) -> Result<Vec<(usize, BitVector, BitVector)>> {
    let mut args = Vec::new();
    let mut seen = BTreeSet::new();
    if let Some(ranges) = domain.object()?.get("args") {
        for range in ranges.array()? {
            let name = range.get("name")?.string()?;
            let index = f.params[4]
                .iter()
                .position(|p| p.name == name)
                .ok_or_else(|| Error::new(format!("§12.2 K4: unknown argument {name}")))?;
            let p = &f.params[4][index];
            if p.ty == Type::Token || !seen.insert(index) {
                return err("§12.2 K4", "token/repeated argument in args domain");
            }
            let width = p.ty.bits(object.ptr_width)?;
            if range.get("width")?.uint()? != width as u128 {
                return err("§12.2 K4", "argument domain width differs");
            }
            let lo = json_bits(range.get("lo")?, width)?;
            let hi = json_bits(range.get("hi")?, width)?;
            if lo.unsigned_cmp(&hi)? == std::cmp::Ordering::Greater {
                return err("§12.2 K4", "argument domain has lo > hi");
            }
            args.push((index, lo, hi));
        }
    }
    Ok(args)
}
fn full_quantum_domain(object: &Object, g: &Function, domain: &Json) -> Result<()> {
    crate::certificate::domain_for(domain, g, crate::IsaVersion::V09, 0)?;
    if !domain.get("borrowed")?.array()?.is_empty()
        || domain
            .object()?
            .get("fixed_regions")
            .is_some_and(|v| v.array().is_ok_and(|v| !v.is_empty()))
    {
        return err(
            "§12.2 K4/K85",
            "inverse-pair relation requires unrestricted inputs and memory",
        );
    }
    let mut inputs = BTreeSet::new();
    for b in domain.get("bundles")?.array()? {
        let n = b.get("name")?.string()?;
        let p = g.params[0].iter().find(|p| p.name == n).ok_or_else(|| {
            Error::new("§12.2 K4: inverse-pair bundles are oracle inputs only".into())
        })?;
        let w = p.ty.bits(object.ptr_width)?;
        if !inputs.insert(n)
            || !json_bits(b.get("lo")?, w)?.is_zero()
            || json_bits(b.get("hi")?, w)? != ones(w)?
        {
            return err("§12.2 K4", "inverse-pair input domain must be full");
        }
    }
    if inputs.len() != g.params[0].len() {
        return err("§12.2 K4", "inverse-pair domain omits input slots");
    }
    let mut memory = BTreeSet::new();
    for r in domain.get("regions")?.array()? {
        let name = r.get("name")?.string()?;
        let p = g.params[3]
            .iter()
            .find(|p| p.name == name)
            .ok_or_else(|| Error::new("§12.2 K4: unknown inverse-pair region".into()))?;
        if !memory.insert(name) {
            return err("§12.2 K4", "repeated inverse-pair region");
        }
        let Type::Region(a, w) = p.ty else {
            return err("§12.2 K4", "expected region type");
        };
        if r.get("cells")? != &Json::String("all".into()) {
            let cells = r.get("cells")?.array()?;
            if crate::oracle::power_of_two(a as u128) != Some(cells.len() as u128) {
                return err("§12.2 K4", "inverse-pair domain omits cells");
            }
            for c in cells {
                if !json_bits(c.get("lo")?, w as usize)?.is_zero()
                    || json_bits(c.get("hi")?, w as usize)? != ones(w as usize)?
                {
                    return err("§12.2 K4", "inverse-pair cell domain must be full");
                }
            }
        }
    }
    if memory.len() != g.params[3].len() {
        return err("§12.2 K4", "inverse-pair domain omits regions");
    }
    args_domain(object, g, domain)?;
    Ok(())
}
impl Object {
    /// Resolve the pair theorem, retaining its domain through library reuse.
    /// Every link must bind the same closures and a covered token context.
    pub(super) fn inverse_pair_theorem09(
        &self,
        closure: &str,
        index: usize,
        entry: &Json,
        active: &mut BTreeSet<(String, usize)>,
    ) -> Result<Option<(&Json, &Function)>> {
        let key = (closure.to_string(), index);
        if !active.insert(key.clone()) {
            return err("§12.1 K4", "cyclic inverse-pair library reference");
        }
        let regions = self
            .section("certificate")
            .unwrap()
            .get("regions")?
            .object()?;
        let Some(record) = regions.get(closure) else {
            return Ok(None);
        };
        let claim = record
            .get("claims")?
            .array()?
            .get(index)
            .ok_or_else(|| Error::new("§12.1 K4: library claim index out of range".into()))?;
        let owner = self.region_function(closure)?;
        let result = match pair_claim(claim)? {
            PairClaim::Exhaustive => {
                self.validate_inverse_pair_claim09(owner, index, claim)?;
                Some((claim, pair(self, entry)?.g))
            }
            PairClaim::Library => {
                let witness = claim.get("witness")?;
                let library = witness.get("library_closure_hash")?.string()?;
                let donor_index = usize::try_from(witness.get("claim_index")?.uint()?)
                    .map_err(|_| crate::resource_error("§12.1 K4", "claim index exceeds host"))?;
                pair_context(self, entry)?;
                let identity = self.identity()?;
                let mut pins = BTreeSet::new();
                if let Some(imports) = self.section("imports") {
                    for import in imports.object()?.values() {
                        if let Some(pin) = import.object()?.get("pinned_closure_hash") {
                            pins.insert(pin.string()?);
                        }
                    }
                }
                let mut bodies_available = true;
                for hash in [entry.get("g")?.string()?, entry.get("ginv")?.string()?] {
                    if !identity
                        .closure_hashes
                        .values()
                        .any(|h| h.as_deref() == Some(hash))
                    {
                        if !pins.contains(hash) {
                            return err(
                                "§12.1 K4",
                                "inverse-pair closure is neither local nor imported by pin",
                            );
                        }
                        bodies_available = false;
                    }
                }
                let current = if bodies_available {
                    Some(pair(self, entry)?)
                } else {
                    None
                };
                let current = match (regions.contains_key(library), current) {
                    (true, Some(current)) => current,
                    _ => {
                        active.remove(&key);
                        return Ok(None);
                    }
                };
                let donor = self.region_function(library)?;
                let mut matched = None;
                for donor_entry in self.pair_claim_entries(donor, donor_index)? {
                    let source = pair(self, donor_entry)?;
                    let (g, gi) = (entry.get("g")?, entry.get("ginv")?);
                    let (sg, si) = (donor_entry.get("g")?, donor_entry.get("ginv")?);
                    let same_pair = (g == sg && gi == si) || (g == si && gi == sg);
                    let covered = same_pair
                        && current.budget.steps >= source.budget.steps
                        && current.budget.heap >= source.budget.heap
                        && current.budget.stack >= source.budget.stack
                        && current.budget.depth >= source.budget.depth
                        && current.aliases == source.aliases
                        && current.g.params[4]
                            .iter()
                            .filter(|p| p.ty == Type::Token)
                            .map(|p| current.tokens[&p.name])
                            .eq(source.g.params[4]
                                .iter()
                                .filter(|p| p.ty == Type::Token)
                                .map(|p| source.tokens[&p.name]));
                    if covered {
                        matched = Some(donor_entry);
                        break;
                    }
                }
                let Some(donor_entry) = matched else {
                    return err(
                        "§12.1 K4",
                        "library theorem must cover the same resolved pair and token context",
                    );
                };
                self.inverse_pair_theorem09(library, donor_index, donor_entry, active)?
            }
        };
        active.remove(&key);
        Ok(result)
    }
    pub(super) fn pair_claim_entries(&self, owner: &Function, index: usize) -> Result<Vec<&Json>> {
        let closure = self.identity()?.closure_hashes[&owner.name]
            .clone()
            .ok_or_else(|| Error::new("§12.3 K4: unlinked pair owner".into()))?;
        let record = self
            .section("certificate")
            .ok_or_else(|| Error::new("§12.3 K4: absent certificate".into()))?
            .get("regions")?
            .get(&closure)?;
        let entries = record
            .get("verification")?
            .get("inverse_pairs")?
            .array()?
            .iter()
            .filter(|entry| entry.get("claim").and_then(Json::uint) == Ok(index as u128))
            .collect::<Vec<_>>();
        if entries.is_empty() {
            return err(
                "§12.3 K4",
                "inverse-pair claim has no owner-local entry binding",
            );
        }
        Ok(entries)
    }
    pub(crate) fn validate_inverse_pair_claim09(
        &self,
        owner: &Function,
        index: usize,
        claim: &Json,
    ) -> Result<()> {
        if claim.get("kind")?.string()? != "exhaustive" {
            return err("§12.1 K4", "inverse-pair relation is exhaustive");
        }
        if let Some(subject) = claim.object()?.get("subject") {
            if self.identity()?.closure_hashes[&owner.name].as_deref()
                != Some(subject.get("closure_hash")?.string()?)
            {
                return err("§12.1 K4", "inverse-pair subject differs from its owner");
            }
        }
        for entry in self.pair_claim_entries(owner, index)? {
            let p = pair(self, entry)?;
            full_quantum_domain(self, p.g, claim.get("domain")?)?;
        }
        Ok(())
    }
    pub(super) fn verify_inverse_pair_claim09(
        &self,
        owner: &Function,
        index: usize,
        claim: &Json,
    ) -> Result<ClaimVerdict> {
        self.validate_inverse_pair_claim09(owner, index, claim)?;
        let key = format!(
            "{}:{}:{}:{}",
            self.identity()?.object_hash,
            self.identity()?.closure_hashes[&owner.name]
                .as_deref()
                .unwrap_or(""),
            index,
            crate::sha256_hex(self.section("certificate").unwrap().canonical().as_bytes())
        );
        if self
            .pair_proofs
            .lock()
            .map_err(|_| Error::new("§12.1 K4: proof cache poisoned".into()))?
            .contains(&key)
        {
            return Ok(ClaimVerdict::Proof);
        }
        for entry in self.pair_claim_entries(owner, index)? {
            let p = pair(self, entry)?;
            let mut domain = self.domain(p.g, claim.get("domain")?)?;
            let quantum = domain.components.len();
            let ranges = args_domain(self, p.g, claim.get("domain")?)?;
            for (index, lo, hi) in &ranges {
                domain.components.push((
                    Component::Bundle(4, *index),
                    lo.width(),
                    lo.clone(),
                    hi.clone(),
                ));
            }
            for row in enumerate(&domain)? {
                let mut input = OracleVectorInput {
                    ins: p.g.params[0]
                        .iter()
                        .map(|p| BitVector::zero(p.ty.bits(self.ptr_width)?))
                        .collect::<Result<_>>()?,
                    ..Default::default()
                };
                for param in &p.g.params[3] {
                    let Type::Region(a, _) = param.ty else {
                        return err("§12.2 K4", "expected region");
                    };
                    let count = crate::oracle::power_of_two(a as u128)
                        .and_then(|n| usize::try_from(n).ok())
                        .ok_or_else(|| {
                            crate::resource_error("§12.2 K4", "region capacity exceeds host")
                        })?;
                    let mut bytes = crate::reserved(count, "§12.2 K4", "pair region bytes")?;
                    bytes.resize(count, 0);
                    input.regions.insert(param.name.clone(), bytes);
                }
                for ((component, ..), value) in
                    domain.components[..quantum].iter().zip(&row[..quantum])
                {
                    match component {
                        Component::Bundle(0, slot) => input.ins[*slot] = value.clone(),
                        Component::Cell(3, slot, cell) => {
                            input.regions.get_mut(&p.g.params[3][*slot].name).unwrap()[*cell] =
                                u8::try_from(value.to_u128()?).map_err(|_| {
                                    Error::new("§12.2 K4: oracle region cell exceeds byte".into())
                                })?
                        }
                        _ => return err("§12.2 K4", "unexpected inverse-pair component"),
                    }
                }
                let mut values = p.g.params[4]
                    .iter()
                    .map(|param| {
                        if param.ty == Type::Token {
                            Ok(Argument::Token(p.tokens[&param.name]))
                        } else if param.ty == Type::F64 {
                            Ok(Argument::Float(0.0))
                        } else {
                            Ok(Argument::Bits(0))
                        }
                    })
                    .collect::<Result<Vec<_>>>()?;
                for ((slot, ..), value) in ranges.iter().zip(&row[quantum..]) {
                    values[*slot] = if p.g.params[4][*slot].ty == Type::F64 {
                        Argument::Float(f64::from_bits(value.to_u128()? as u64))
                    } else {
                        Argument::Bits(value.to_u128()?)
                    };
                }
                let resolved = self.resolve_invocation_tokens(
                    p.g,
                    &values,
                    Some(p.budget),
                    None,
                    Some(&p.aliases),
                )?;
                input.args = resolved.arguments;
                let primary = resolved.primary.as_ref();
                let run = |f: &Function, ins: &[BitVector]| -> Result<BitVector> {
                    let mut call = OracleVectorInput {
                        ins: ins.to_vec(),
                        args: input.args.clone(),
                        regions: BTreeMap::new(),
                    };
                    for (a, b) in f.params[3].iter().zip(&p.g.params[3]) {
                        call.regions
                            .insert(a.name.clone(), input.regions[&b.name].clone());
                    }
                    let result =
                        self.run_oracle_vectors_bound(&f.name, &call, p.budget, primary, None, 0)?;
                    let mut machine = OracleVectorMachine::new_bound(
                        self, &f.name, &call, p.budget, primary, None, 0,
                    )?;
                    let before = machine.snapshot();
                    for _ in 0..p.budget.steps {
                        machine.forward_step()?;
                    }
                    let other = machine.outcome()?;
                    if (other.payload, other.code) != (result.payload.clone(), result.code.clone())
                    {
                        return err("§12.1 K4/T3", "inverse-pair independent engines disagree");
                    }
                    for _ in 0..p.budget.steps {
                        machine.inverse_step()?;
                    }
                    if machine.snapshot() != before || !machine.frames_clean() {
                        return err(
                            "§12.1 K4/T3",
                            "inverse-pair oracle workspace is not restored",
                        );
                    }
                    if !result.code.is_zero() {
                        return err("§12.1 K4", "inverse-pair oracle faults at entry budget");
                    }
                    Ok(result.payload)
                };
                let g = run(p.g, &input.ins)?;
                let gi = run(p.gi, &input.ins)?;
                let mut changed = input.ins.clone();
                changed[0] = g;
                if run(p.gi, &changed)? != input.ins[0] {
                    return err("§12.1 K4", "ginv(g(x)) differs from x");
                }
                changed[0] = gi;
                if run(p.g, &changed)? != input.ins[0] {
                    return err("§12.1 K4", "g(ginv(x)) differs from x");
                }
            }
        }
        self.pair_proofs
            .lock()
            .map_err(|_| Error::new("§12.1 K4: proof cache poisoned".into()))?
            .insert(key);
        Ok(ClaimVerdict::Proof)
    }
}
