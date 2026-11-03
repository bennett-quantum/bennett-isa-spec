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
    f: &Function,
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
        name: f.name.clone(),
        args: actuals,
        budget: primary.as_ref().map(BudgetToken::tuple),
        primary,
    });
    Ok(())
}
impl Object {
    fn check_qinplace_site09(
        &self,
        owner: &Function,
        i: &Instruction,
        args: &BTreeMap<String, Argument>,
    ) -> Result<()> {
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
            let claim = record
                .get("claims")?
                .array()?
                .get(index)
                .ok_or_else(|| Error::new("§12.3 K4: pair claim index out of range".into()))?;
            if claim.get("kind")?.string()? != "exhaustive"
                || claim.object()?.get("expected") != Some(&Json::String("inverse_pair".into()))
            {
                return err("§4.5 K4", "unestablished linked inverse-pair proof");
            }
            let ranges = args_domain(self, p.g, claim.get("domain")?)?;
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
            if self.verify_claim(owner_hash, index)? == ClaimVerdict::Proof {
                return Ok(());
            }
        }
        Err(missing())
    }
    pub(crate) fn precheck_qinplace09(
        &self,
        f: &Function,
        args: &BTreeMap<String, Argument>,
        budget: Option<Budget>,
        primary: Option<&BudgetToken>,
    ) -> Result<()> {
        if self.isa_version != crate::IsaVersion::V09
            || !self
                .functions
                .values()
                .any(|f| f.instructions().any(|i| i.op == "qinplace"))
        {
            return Ok(());
        }
        let mut work = vec![Invocation {
            name: f.name.clone(),
            args: values(f, args)?,
            budget,
            primary: primary.cloned(),
        }];
        let mut visited = BTreeSet::new();
        while let Some(invocation) = work.pop() {
            let f = &self.functions[&invocation.name];
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
            let args = f.params[if f.kind == "oracle" { 4 } else { 3 }]
                .iter()
                .zip(&invocation.args)
                .map(|(p, a)| (p.name.clone(), a.clone()))
                .collect::<BTreeMap<_, _>>();
            if f.kind == "oracle" {
                if invocation.budget.is_none_or(|b| b.steps == 0) {
                    continue;
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
                    let callee = &self.functions[a[0].trim_start_matches('@')];
                    push(
                        &mut work,
                        callee,
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
                            self.check_qinplace_site09(f, i, &args)?;
                            let names = if a.len() == 7 {
                                crate::object::list(&a[6])?
                            } else {
                                Vec::new()
                            };
                            for name in &a[..2] {
                                push(
                                    &mut work,
                                    &self.functions[name.trim_start_matches('@')],
                                    &names,
                                    &args,
                                    Some(token(&args, &a[2])?),
                                )?;
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
                                &self.functions[a[0].trim_start_matches('@')],
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
        Ok(())
    }
}
