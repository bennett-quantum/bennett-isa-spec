//! K92 applicability over literal bound constructions, including default and
//! coherently disabled semantic calls. Native scheme leaves introduce no owners.
use super::*;
use crate::ref_library::ResolvedSourceInvocation;
use crate::{Argument, BudgetToken};
use bennett_isa_conformance::json::{self, Json as Value};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum InvocationScope {
    Reference(u128),
    Scheduled,
}
#[derive(Clone, PartialEq, Eq)]
struct CoverageKey {
    object: String,
    certificate: Option<String>,
    schedule: Option<String>,
    root: String,
    args: String,
    budget: Option<Budget>,
    aliases: Vec<u64>,
    scope: InvocationScope,
}
type Owners = BTreeMap<String, QinplaceApplicability>;
// A bounded last-result cache. Identity is the full proof/invocation scope;
// quantum basis inputs are deliberately absent from this universal obligation.
thread_local! {static LAST_COVERAGE:std::cell::RefCell<Option<(CoverageKey,Owners)>>=const {std::cell::RefCell::new(None)};}

fn source_may_reach_site(object: &Object, root: &Function) -> Result<bool> {
    let mut pending = vec![root.name.clone()];
    let mut seen = BTreeSet::new();
    while let Some(name) = pending.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        let Some(function) = object.functions.get(&name) else {
            return Ok(true);
        };
        let mut instructions = function.instructions().collect::<Vec<_>>();
        while let Some(i) = instructions.pop() {
            if i.op == "qinplace" {
                return Ok(true);
            }
            instructions.extend(i.bodies.iter().flatten());
            match i.op.as_str() {
                "call" | "lift.call" | "lift.oracle_call" | "lift.oracle_uncall"
                | "lift.perm_call" => pending.push(i.operands[0].trim_start_matches('@').into()),
                "action_call" | "oracle_call" | "adj" | "ctl" => pending.push(
                    crate::validation::normalize_call(i)?.instruction.operands[0]
                        .trim_start_matches('@')
                        .into(),
                ),
                _ => {}
            }
        }
    }
    Ok(false)
}

struct SourceActuals {
    arguments: Vec<Value>,
    bundles: Vec<(String, usize)>,
}
fn source_actuals(f: &Function, args: &[Argument]) -> Result<SourceActuals> {
    let encoded = json::parse(&crate::tokens::typed_arguments(args).canonical())
        .map_err(|e| Error::new(format!("§12.3: source arguments: {e}")))?;
    let values = encoded
        .as_arr()
        .ok_or_else(|| Error::new("§12.3: source arguments must be array".into()))?;
    let mut actuals = Vec::new();
    let mut bundles = Vec::new();
    let mut next = 0;
    for p in f.params.iter().flatten() {
        match p.ty {
            Type::Wire(width) | Type::Reg(width) => {
                bundles.push((
                    p.name.clone(),
                    usize::try_from(width).map_err(|_| {
                        crate::resource_error("§12.3", "source interface exceeds host")
                    })?,
                ));
                actuals.push(Value::Str(p.name.clone()));
            }
            Type::Region(a, w) => {
                if a >= 1u64 << 53 || w >= 1u64 << 53 {
                    return Err(crate::resource_error(
                        "§12.3",
                        "source region parameters exceed exact constructor scalar range",
                    ));
                }
                let count = crate::oracle::power_of_two(a as u128)
                    .and_then(|n| n.checked_mul(w as u128))
                    .and_then(|n| usize::try_from(n).ok())
                    .ok_or_else(|| crate::resource_error("§12.3", "source region exceeds host"))?;
                bundles.push((p.name.clone(), count));
                actuals.push(Value::Obj(BTreeMap::from([
                    ("address".into(), Value::Num(a as f64)),
                    ("cell".into(), Value::Num(w as f64)),
                    (
                        "role".into(),
                        Value::Str(p.role.clone().unwrap_or_else(|| "in".into())),
                    ),
                    ("origin".into(), Value::Str(format!("interface:{}", p.name))),
                    ("cells".into(), Value::Str(p.name.clone())),
                ])));
            }
            _ => {
                actuals.push(
                    values
                        .get(next)
                        .cloned()
                        .ok_or_else(|| Error::new("§12.3: source argument arity differs".into()))?,
                );
                next += 1;
            }
        }
    }
    Ok(SourceActuals {
        arguments: actuals,
        bundles,
    })
}

fn concrete_arguments(
    object: &Object,
    f: &Function,
    context: &ResolvedSourceInvocation,
) -> Result<BTreeMap<String, Argument>> {
    let mut actuals = Vec::new();
    let params = &f.params[if f.kind == "oracle" { 4 } else { 3 }];
    for p in params {
        let value = context.values.get(&p.name).ok_or_else(|| {
            Error::new(format!(
                "§12.3: missing resolved source argument {}",
                p.name
            ))
        })?;
        let bytes = bennett_isa_conformance::ref_hash::canonical(value)
            .map_err(|e| Error::new(format!("§12.3: source value: {e}")))?;
        let value = Json::parse(&bytes)?;
        actuals.push(match p.ty {
            Type::Token => Argument::Token(crate::certificate::invocation_budget(&value)?),
            Type::F64 => Argument::Float(f64::from_bits(
                u64::from_str_radix(
                    value.string()?.strip_prefix("0x").ok_or_else(|| {
                        Error::new("§12.3: exact source angle binding missing".into())
                    })?,
                    16,
                )
                .map_err(|_| Error::new("§12.3: invalid source angle binding".into()))?,
            )),
            Type::Int(_) | Type::Ptr => Argument::Bits(value.string()?.parse().map_err(|_| {
                crate::resource_error("§12.3", "resolved source argument exceeds host")
            })?),
            _ => return err("§12.3", "invalid classical source argument type"),
        });
    }
    let b = context.binding.budget;
    let budget = Budget {
        steps: u64::try_from(b[0])
            .map_err(|_| crate::resource_error("§12.3", "source budget exceeds host"))?,
        heap: b[1],
        stack: b[2],
        depth: b[3],
    };
    let resolved = object.resolve_invocation_tokens(
        f,
        &actuals,
        (f.kind == "oracle").then_some(budget),
        None,
        Some(&context.binding.token_aliases),
    )?;
    Ok(params
        .iter()
        .zip(resolved.arguments)
        .map(|(p, v)| (p.name.clone(), v))
        .collect())
}

impl Object {
    pub(super) fn qinplace_source_coverage09(
        &self,
        f: &Function,
        args: &BTreeMap<String, Argument>,
        budget: Option<Budget>,
        primary: Option<&BudgetToken>,
        scope: InvocationScope,
    ) -> Result<Owners> {
        if self.isa_version != crate::IsaVersion::V09 {
            return Ok(BTreeMap::new());
        }
        let identity = self.identity()?;
        let root = identity.closure_hashes[&f.name]
            .clone()
            .ok_or_else(|| Error::new("§12.3: unresolved invocation root".into()))?;
        if let Some(imports) = self.section("imports") {
            if let Some(name) = imports
                .object()?
                .keys()
                .find(|n| !self.functions.contains_key(*n))
            {
                return Ok(BTreeMap::from([(
                    root,
                    QinplaceApplicability::Unestablished(format!(
                        "missing linked body for @{name}"
                    )),
                )]));
            }
        }
        let mut reference;
        let source = match scope {
            InvocationScope::Reference(_) => {
                reference = self.clone();
                reference.metadata.remove("schedule");
                &reference
            }
            InvocationScope::Scheduled => self,
        };
        if let Err(error) = source.require_ref_library_binding() {
            return Ok(BTreeMap::from([(
                root,
                QinplaceApplicability::Unestablished(error.to_string()),
            )]));
        }
        if !source_may_reach_site(self, f)? {
            return Ok(BTreeMap::from([(root, QinplaceApplicability::Established)]));
        }
        let actuals = f.params[if f.kind == "oracle" { 4 } else { 3 }]
            .iter()
            .map(|p| {
                args.get(&p.name).cloned().ok_or_else(|| {
                    Error::new(format!("§12.3: unresolved invocation argument {}", p.name))
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let resolved = self.resolve_invocation_tokens(
            f,
            &actuals,
            if f.kind == "oracle" {
                Some(budget.ok_or_else(|| Error::new("§12.3: missing oracle budget".into()))?)
            } else {
                None
            },
            primary,
            None,
        )?;
        let key = CoverageKey {
            object: identity.object_hash,
            certificate: self.section("certificate").map(Json::canonical),
            schedule: self.section("schedule").map(Json::canonical),
            root: root.clone(),
            args: crate::tokens::typed_arguments(&resolved.arguments).canonical(),
            budget,
            aliases: resolved.aliases.clone(),
            scope,
        };
        if let Some(cached) = LAST_COVERAGE.with(|cache| {
            cache
                .borrow()
                .as_ref()
                .filter(|(prior, _)| *prior == key)
                .map(|(_, owners)| owners.clone())
        }) {
            return Ok(cached);
        }
        let actuals = source_actuals(f, &resolved.arguments)?;
        let source_args = actuals.arguments;
        let mut source_bundles = actuals.bundles;
        let control_count = match scope {
            InvocationScope::Reference(c) => c,
            InvocationScope::Scheduled => source
                .section("schedule")
                .ok_or_else(|| Error::new("§12.3: missing scheduled source tree".into()))?
                .get("bindings")?
                .get("inherited_controls")?
                .uint()?,
        };
        let count = usize::try_from(control_count).map_err(|_| {
            crate::resource_error("§12.3", "inherited source controls exceed host range")
        })?;
        let mut controls = crate::reserved(count, "§12.3", "inherited source controls")?;
        for i in 0..count {
            let name = format!("$inherited{i}");
            controls.push((name.clone(), 0));
            source_bundles.push((name, 1));
        }
        let bundles = source_bundles
            .iter()
            .map(|(name, n)| (name.as_str(), *n))
            .collect::<Vec<_>>();
        let contexts = if f.kind == "oracle" {
            crate::ref_library::OracleInvocation::capture_with_token_aliases(
                source,
                &f.name,
                None,
                budget.ok_or_else(|| Error::new("§12.3: missing oracle budget".into()))?,
                &source_args,
                &resolved.aliases,
            )?
            .source_invocations(&source.bound_ref_library()?, &bundles, &controls)?
        } else {
            let variant = if let Some(schedule) = source.section("schedule") {
                let binding = schedule.get("bindings")?;
                if binding.get("closure_hash")?.string()? == root {
                    binding.get("variant")?.string()?
                } else {
                    "forward"
                }
            } else {
                "forward"
            };
            source.source_action_invocations_at(
                &f.name,
                &source_args,
                &bundles,
                &resolved.aliases,
                crate::ref_library::SourceRootRequest {
                    // §4.7 actions carry no separate primary token. An executor's
                    // reference action therefore has the existing zero root tuple.
                    budget: match budget {
                        Some(b) => b,
                        None => Budget {
                            steps: 0,
                            heap: 0,
                            stack: 0,
                            depth: 0,
                        },
                    },
                    variant,
                    controls: &controls,
                },
            )?
        };
        let mut owners = BTreeMap::from([(root, QinplaceApplicability::Established)]);
        for context in contexts {
            let name = identity
                .closure_hashes
                .iter()
                .find_map(|(n, h)| {
                    (h.as_deref() == Some(context.closure_hash.as_str())).then_some(n.clone())
                })
                .ok_or_else(|| Error::new("§12.3: source owner closure missing".into()))?;
            let owner = &self.functions[&name];
            let mut sequences = owner
                .blocks
                .iter()
                .map(|b| b.instructions.as_slice())
                .collect::<Vec<_>>();
            let mut site_args = None;
            while let Some(sequence) = sequences.pop() {
                for instruction in sequence {
                    if instruction.op == "qloop"
                        && crate::object::host_count(
                            &instruction.operands[0],
                            "qinplace loop count",
                        )? == 0
                    {
                        continue;
                    }
                    sequences.extend(instruction.bodies.iter().map(Vec::as_slice));
                    if instruction.op == "qinplace" {
                        if site_args.is_none() {
                            site_args = Some(concrete_arguments(self, owner, &context)?);
                        }
                        let result = self.check_qinplace_site09(
                            owner,
                            instruction,
                            site_args.as_ref().expect("resolved site arguments"),
                        )?;
                        let prior = owners
                            .entry(context.closure_hash.clone())
                            .or_insert(QinplaceApplicability::Established);
                        if matches!(result, QinplaceApplicability::Unestablished(_)) {
                            *prior = result;
                        }
                    }
                }
            }
        }
        if owners
            .values()
            .all(|o| matches!(o, QinplaceApplicability::Established))
        {
            LAST_COVERAGE.with(|cache| *cache.borrow_mut() = Some((key, owners.clone())));
        }
        Ok(owners)
    }
}
