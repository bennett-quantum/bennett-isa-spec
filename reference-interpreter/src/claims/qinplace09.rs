//! Pre-effect K4/K91 coverage over the bound reference invocation tree. Q
//! predicates cannot remove a visit; zero-trip loops have no dynamic visits.
use super::*;
use crate::{Argument, BudgetToken, Instruction};
use pairs09::{args_domain, pair};

struct Invocation {
    name: String,
    args: Vec<Argument>,
    budget: Option<Budget>,
    primary: Option<BudgetToken>,
}
fn qinplace_sites(f: &Function) -> Vec<&Instruction> {
    let mut work = f.instructions().collect::<Vec<_>>();
    let mut sites = Vec::new();
    while let Some(i) = work.pop() {
        if i.op == "qinplace" {
            sites.push(i);
        }
        work.extend(i.bodies.iter().flatten());
    }
    sites
}
fn values(f: &Function, args: &BTreeMap<String, Argument>) -> Result<Vec<Argument>> {
    f.params[if f.kind == "oracle" { 4 } else { 3 }]
        .iter()
        .map(|p| {
            args.get(&p.name)
                .cloned()
                .ok_or_else(|| Error::new(format!("§4.5 K92: unresolved argument {}", p.name)))
        })
        .collect()
}
fn token(args: &BTreeMap<String, Argument>, name: &str) -> Result<BudgetToken> {
    match args.get(name) {
        Some(Argument::DeviceToken(t)) => Ok(t.clone()),
        _ => err(
            "§4.5 K91",
            "qinplace coverage needs an actual token identity",
        ),
    }
}
fn push(
    work: &mut Vec<Invocation>,
    name: &str,
    names: &[String],
    args: &BTreeMap<String, Argument>,
    primary: Option<BudgetToken>,
) -> Result<()> {
    let actuals = names
        .iter()
        .map(|n| {
            args.get(n)
                .cloned()
                .ok_or_else(|| Error::new(format!("§4.5 K92: unresolved call argument {n}")))
        })
        .collect::<Result<_>>()?;
    crate::reserve_more(work, 1, "§4.5 K92", "reference invocation contexts")?;
    work.push(Invocation {
        name: name.trim_start_matches('@').into(),
        args: actuals,
        budget: primary.as_ref().map(BudgetToken::tuple),
        primary,
    });
    Ok(())
}
impl Object {
    fn qinplace_tree_needs_check09(&self) -> Result<bool> {
        let unresolved_imports = self
            .section("imports")
            .map(|imports| imports.object().map(|imports| !imports.is_empty()))
            .transpose()?
            .unwrap_or(false);
        Ok(self.isa_version == crate::IsaVersion::V09
            && (unresolved_imports
                || self
                    .functions
                    .values()
                    .any(|f| !qinplace_sites(f).is_empty())))
    }
    /// Inventory is static (including zero-trip bodies); applicability belongs
    /// to the resolved invocation tree, never to the standalone proof label.
    pub(super) fn check_qinplace_bindings09(
        &self,
    ) -> Result<BTreeMap<String, QinplaceApplicability>> {
        let certificate = self.section("certificate").unwrap();
        let identity = self.identity()?;
        let mut outcomes = BTreeMap::new();
        if !self.qinplace_tree_needs_check09()? {
            return Ok(outcomes);
        }
        for (closure, record) in certificate.get("regions")?.object()? {
            let f = self.region_function(closure)?;
            for site in qinplace_sites(f) {
                let missing_body = site.operands[..2]
                    .iter()
                    .find(|name| !self.functions.contains_key(name.trim_start_matches('@')));
                let hash = |name: &str| -> Result<&str> {
                    let name = name.trim_start_matches('@');
                    if let Some(hash) = identity.closure_hashes.get(name) {
                        return hash.as_deref().ok_or_else(|| {
                            Error::new("§12.3 K92: unresolved qinplace closure".into())
                        });
                    }
                    self.section("imports")
                        .map(|imports| imports.get(name)?.get("pinned_closure_hash")?.string())
                        .transpose()?
                        .ok_or_else(|| {
                            Error::new(format!("§12.3 K92: unresolved qinplace closure @{name}"))
                        })
                };
                let g = hash(&site.operands[0])?;
                let gi = hash(&site.operands[1])?;
                let mut found = false;
                for entry in record.get("verification")?.get("inverse_pairs")?.array()? {
                    let eg = entry.get("g")?.string()?;
                    let ei = entry.get("ginv")?.string()?;
                    if !((eg == g && ei == gi) || (eg == gi && ei == g)) {
                        continue;
                    }
                    found = true;
                    let index = usize::try_from(entry.get("claim")?.uint()?).map_err(|_| {
                        crate::resource_error("§12.3 K92", "claim index exceeds host")
                    })?;
                    let claim = record.get("claims")?.array()?.get(index).ok_or_else(|| {
                        Error::new("§12.3 K92: pair claim index out of range".into())
                    })?;
                    pairs09::pair_claim(claim)?;
                    pairs09::pair_context(self, entry)?;
                    if missing_body.is_some() {
                        continue;
                    }
                    self.inverse_pair_theorem09(closure, index, entry, &mut BTreeSet::new())?;
                    self.verify_claim_guarded(closure, index, &mut BTreeSet::new())?;
                }
                if !found {
                    return err(
                        "§12.3 K92",
                        format!("missing owner-local inverse-pair entry in @{}", f.name),
                    );
                }
                outcomes.insert(
                    closure.clone(),
                    QinplaceApplicability::Unestablished(match missing_body {
                        Some(name) => format!("missing linked body for {name}"),
                        None => "no resolved invocation establishes qinplace applicability".into(),
                    }),
                );
            }
        }
        let bindings = certificate.get("bindings")?;
        let root = bindings.get("closure_hash")?.string()?;
        let mut invocations = Vec::new();
        let mut unresolved = BTreeMap::new();
        if let Some(invocation) = bindings.object()?.get("invocation") {
            invocations.push((root, invocation));
        }
        for (closure, record) in certificate.get("regions")?.object()? {
            for claim in record.get("claims")?.array()? {
                if let Some(invocation) = claim.object()?.get("invocation") {
                    // Parameter-uniform claims keep their universal theorem;
                    // proving applicability requires a separate uniform check.
                    if invocation.get("args")?.array().is_ok() {
                        invocations.push((closure.as_str(), invocation));
                    } else {
                        unresolved.insert(
                            closure.clone(),
                            QinplaceApplicability::Unestablished(
                                "parameter-uniform invocation tree is not checked".into(),
                            ),
                        );
                    }
                }
            }
        }
        let mut checked = BTreeMap::new();
        for (closure, binding) in invocations {
            let f = self.region_function(closure)?;
            if !matches!(f.kind.as_str(), "oracle" | "action" | "perm_action") {
                unresolved.insert(
                    closure.to_string(),
                    QinplaceApplicability::Unestablished(format!(
                        "bound @{} invocation tree is not checked",
                        f.name
                    )),
                );
                continue;
            }
            let invocation = self.decode_invocation09(binding, f)?;
            let group = if f.kind == "oracle" { 4 } else { 3 };
            let args = f.params[group]
                .iter()
                .zip(invocation.args)
                .map(|(p, value)| (p.name.clone(), value))
                .collect();
            match self.qinplace_coverage09(
                f,
                &args,
                Some(invocation.budget),
                invocation.primary.as_ref(),
            ) {
                Ok(owners) if self.section("schedule").is_none() => {
                    for (owner, outcome) in owners {
                        if !matches!(
                            checked.get(&owner),
                            Some(QinplaceApplicability::Unestablished(_))
                        ) {
                            checked.insert(owner, outcome);
                        }
                    }
                }
                Ok(_) => {
                    unresolved.insert(
                        closure.to_string(),
                        QinplaceApplicability::Unestablished(
                            "complete realised invocation tree is not checked".into(),
                        ),
                    );
                }
                Err(error) if error.kind() == crate::ErrorKind::HostResource => {
                    unresolved.insert(
                        closure.to_string(),
                        QinplaceApplicability::Unestablished(error.to_string()),
                    );
                }
                Err(error) => return Err(error),
            }
        }
        outcomes.extend(checked);
        // Root obligations cannot depend on discovering an owner or a claim.
        // An unchecked context also takes precedence over a checked context of
        // the same closure: one invocation cannot discharge another's coverage.
        outcomes.extend(unresolved);
        Ok(outcomes)
    }
    fn check_qinplace_site09(
        &self,
        owner: &Function,
        i: &Instruction,
        args: &BTreeMap<String, Argument>,
    ) -> Result<QinplaceApplicability> {
        let a = &i.operands;
        let primary = token(args, &a[2])?;
        let actuals = if a.len() == 7 {
            crate::object::list(&a[6])?
                .iter()
                .map(|n| {
                    args.get(n)
                        .cloned()
                        .ok_or_else(|| Error::new(format!("§4.5 K4: unresolved pair argument {n}")))
                })
                .collect::<Result<Vec<_>>>()?
        } else {
            Vec::new()
        };
        let identity = self.identity()?;
        let owner_hash = identity.closure_hashes[&owner.name]
            .as_deref()
            .ok_or_else(|| Error::new("§4.5 K92: unresolved owner".into()))?;
        let missing = || {
            Error::new(format!(
                "§4.5 K4/K92: no covering inverse-pair entry in @{}",
                owner.name
            ))
        };
        let record = self
            .section("certificate")
            .and_then(|c| c.get("regions").ok())
            .and_then(|r| r.get(owner_hash).ok())
            .ok_or_else(missing)?;
        let g = identity.closure_hashes[a[0].trim_start_matches('@')]
            .as_deref()
            .ok_or_else(missing)?;
        let gi = identity.closure_hashes[a[1].trim_start_matches('@')]
            .as_deref()
            .ok_or_else(missing)?;
        let mut unresolved = None;
        for entry in record.get("verification")?.get("inverse_pairs")?.array()? {
            let eg = entry.get("g")?.string()?;
            let ei = entry.get("ginv")?.string()?;
            if !((eg == g && ei == gi) || (eg == gi && ei == g)) {
                continue;
            }
            let p = pair(self, entry)?;
            let b = primary.tuple();
            if p.budget.steps > b.steps
                || p.budget.heap > b.heap
                || p.budget.stack > b.stack
                || p.budget.depth > b.depth
            {
                continue;
            }
            let resolved =
                self.resolve_invocation_tokens(p.g, &actuals, Some(b), Some(&primary), None)?;
            if resolved.aliases != p.aliases {
                continue;
            }
            if p.g.params[4].iter().zip(&actuals).any(|(param, value)| {
                param.ty == Type::Token
                    && value.token_budget() != p.tokens.get(&param.name).copied()
            }) {
                continue;
            }
            let index = usize::try_from(entry.get("claim")?.uint()?)
                .map_err(|_| crate::resource_error("§12.3 K4", "claim index exceeds host"))?;
            let Some((claim, domain_function)) =
                self.inverse_pair_theorem09(owner_hash, index, entry, &mut BTreeSet::new())?
            else {
                unresolved = Some("linked inverse-pair theorem is unavailable".into());
                continue;
            };
            let ranges = args_domain(self, domain_function, claim.get("domain")?)?;
            let mut covers = true;
            for (slot, (param, value)) in p.g.params[4].iter().zip(&actuals).enumerate() {
                if param.ty == Type::Token {
                    continue;
                }
                let bits =
                    crate::oracle::argument_bits(self, &param.ty, value)?.ok_or_else(missing)?;
                covers &= if let Some((_, lo, hi)) = ranges.iter().find(|(i, ..)| *i == slot) {
                    bits.unsigned_cmp(lo)? != std::cmp::Ordering::Less
                        && bits.unsigned_cmp(hi)? != std::cmp::Ordering::Greater
                } else {
                    bits.is_zero()
                };
            }
            if !covers {
                continue;
            }
            if self.verify_claim_guarded(owner_hash, index, &mut BTreeSet::new())?
                == ClaimVerdict::Proof
            {
                return Ok(QinplaceApplicability::Established);
            }
            unresolved = Some("entry claim is not an established inverse-pair proof".into());
        }
        match unresolved {
            Some(reason) => Ok(QinplaceApplicability::Unestablished(reason)),
            None => Err(missing()),
        }
    }
    pub(crate) fn precheck_qinplace09(
        &self,
        f: &Function,
        args: &BTreeMap<String, Argument>,
        budget: Option<Budget>,
        primary: Option<&BudgetToken>,
    ) -> Result<()> {
        for outcome in self.qinplace_coverage09(f, args, budget, primary)?.values() {
            if let QinplaceApplicability::Unestablished(reason) = outcome {
                return err(
                    "§4.5 K4/K92",
                    format!("unestablished qinplace applicability: {reason}"),
                );
            }
        }
        Ok(())
    }
    fn qinplace_coverage09(
        &self,
        f: &Function,
        args: &BTreeMap<String, Argument>,
        budget: Option<Budget>,
        primary: Option<&BudgetToken>,
    ) -> Result<BTreeMap<String, QinplaceApplicability>> {
        if !self.qinplace_tree_needs_check09()? {
            return Ok(BTreeMap::new());
        }
        let root = self.identity()?.closure_hashes[&f.name]
            .clone()
            .ok_or_else(|| Error::new("§12.3 K92: unresolved invocation root".into()))?;
        let mut work = vec![Invocation {
            name: f.name.clone(),
            args: values(f, args)?,
            budget,
            primary: primary.cloned(),
        }];
        let mut visited = BTreeSet::new();
        let mut owners = BTreeMap::new();
        while let Some(invocation) = work.pop() {
            let Some(f) = self.functions.get(&invocation.name) else {
                owners.insert(
                    root.clone(),
                    QinplaceApplicability::Unestablished(format!(
                        "missing linked body for @{}",
                        invocation.name
                    )),
                );
                continue;
            };
            let ids = invocation
                .args
                .iter()
                .map(|a| {
                    if let Argument::DeviceToken(t) = a {
                        t.id()
                    } else {
                        0
                    }
                })
                .collect::<Vec<_>>();
            let key = format!(
                "{}:{:?}:{:?}:{:?}:{}",
                f.name,
                invocation.budget,
                invocation.primary.as_ref().map(BudgetToken::id),
                ids,
                crate::tokens::typed_arguments(&invocation.args).canonical()
            );
            if !visited.insert(key) {
                continue;
            }
            if !qinplace_sites(f).is_empty() {
                let closure = self.identity()?.closure_hashes[&f.name]
                    .clone()
                    .ok_or_else(|| Error::new("§12.3 K92: unresolved owner".into()))?;
                owners
                    .entry(closure)
                    .or_insert(QinplaceApplicability::Established);
            }
            let args = f.params[if f.kind == "oracle" { 4 } else { 3 }]
                .iter()
                .zip(&invocation.args)
                .map(|(p, a)| (p.name.clone(), a.clone()))
                .collect::<BTreeMap<_, _>>();
            if f.kind == "oracle" {
                if invocation.budget.is_none_or(|b| b.steps == 0) {
                    continue;
                }
                if let Some(imports) = self.section("imports") {
                    if let Some(name) = imports
                        .object()?
                        .keys()
                        .find(|name| !self.functions.contains_key(*name))
                    {
                        owners.insert(
                            root.clone(),
                            QinplaceApplicability::Unestablished(format!(
                                "missing linked body for @{name}"
                            )),
                        );
                        continue;
                    }
                }
                let program = crate::oracle::Program::new(self, &f.name)?;
                for (activation, i) in &program.rows {
                    if !matches!(
                        i.op.as_str(),
                        "lift.oracle_call" | "lift.oracle_uncall" | "lift.perm_call"
                    ) {
                        continue;
                    }
                    let fixed = program.arguments(activation, &args)?;
                    let a = &i.operands;
                    let oracle = i.op != "lift.perm_call";
                    push(
                        &mut work,
                        &a[0],
                        &crate::object::list(&a[if oracle { 6 } else { 4 }])?,
                        &fixed,
                        if oracle {
                            Some(token(&fixed, &a[1])?)
                        } else {
                            None
                        },
                    )?;
                }
            } else {
                let mut sequences = f
                    .blocks
                    .iter()
                    .map(|b| b.instructions.as_slice())
                    .collect::<Vec<_>>();
                while let Some(sequence) = sequences.pop() {
                    for i in sequence {
                        if i.op == "qloop"
                            && crate::object::host_count(&i.operands[0], "qinplace loop count")?
                                == 0
                        {
                            continue;
                        }
                        for body in &i.bodies {
                            sequences.push(body);
                        }
                        let a = &i.operands;
                        if i.op == "qinplace" {
                            if let Some(name) = a[..2].iter().find(|name| {
                                !self.functions.contains_key(name.trim_start_matches('@'))
                            }) {
                                owners.insert(
                                    root.clone(),
                                    QinplaceApplicability::Unestablished(format!(
                                        "missing linked body for {name}"
                                    )),
                                );
                                continue;
                            }
                            let outcome = self.check_qinplace_site09(f, i, &args)?;
                            if let QinplaceApplicability::Unestablished(_) = outcome {
                                let closure = self.identity()?.closure_hashes[&f.name]
                                    .clone()
                                    .ok_or_else(|| {
                                        Error::new("§12.3 K92: unresolved owner".into())
                                    })?;
                                owners.insert(closure, outcome);
                            }
                            let names = if a.len() == 7 {
                                crate::object::list(&a[6])?
                            } else {
                                Vec::new()
                            };
                            for name in &a[..2] {
                                push(&mut work, name, &names, &args, Some(token(&args, &a[2])?))?;
                            }
                        } else if matches!(
                            i.op.as_str(),
                            "action_call" | "oracle_call" | "adj" | "ctl"
                        ) {
                            let call = crate::validation::normalize_call(i)?;
                            let a = &call.instruction.operands;
                            let oracle = call.instruction.op == "oracle_call";
                            push(
                                &mut work,
                                &a[0],
                                &crate::object::list(&a[if oracle { 6 } else { 4 }])?,
                                &args,
                                if oracle {
                                    Some(token(&args, &a[1])?)
                                } else {
                                    None
                                },
                            )?;
                        }
                    }
                }
            }
        }
        Ok(owners)
    }
}
