//! K91 identity channel. Tuple values retain their K24/K77 encodings.
use super::*;
use bennett_isa_conformance::ref_language::SourceTokenContext;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug)]
pub(crate) struct SourceTokens {
    pub context: Arc<SourceTokenContext>,
    pub handles: BTreeMap<u64, crate::BudgetToken>,
}
fn tuple(value: &Value) -> Result<crate::Budget> {
    let parts = value
        .as_arr()
        .filter(|parts| parts.len() == 4)
        .ok_or_else(|| {
            crate::Error::new("§11.1 K91: token tuple requires four decimal strings".into())
        })?;
    for part in parts {
        let valid = part.as_str().is_some_and(|s| {
            !s.is_empty()
                && s.bytes().all(|b| b.is_ascii_digit())
                && (s.len() == 1 || !s.starts_with('0'))
        });
        if !valid {
            return err("§11.1 K91", "noncanonical token tuple component");
        }
    }
    let bytes = canonical(value).map_err(infeasible)?;
    crate::certificate::invocation_budget(&crate::json::Json::parse(&bytes)?)
}
impl SourceTokens {
    pub fn bind(
        object: &Object,
        function: &crate::Function,
        arguments: &[Value],
        primary: Option<crate::Budget>,
        aliases: Option<&[u64]>,
    ) -> Result<Self> {
        let params = function.params.iter().flatten().collect::<Vec<_>>();
        if params.len() != arguments.len() {
            return err("§11.1 K91", "E_ARG: source argument count differs");
        }
        let args = params
            .iter()
            .zip(arguments)
            .filter(|(p, _)| p.ty == crate::Type::Token)
            .map(|(p, v)| Ok((p.name.clone(), tuple(v)?)))
            .collect::<Result<Vec<_>>>()?;
        let tuples = primary
            .into_iter()
            .chain(args.iter().map(|(_, b)| *b))
            .collect::<Vec<_>>();
        for b in &tuples {
            object.validate_budget(*b)?;
        }
        let implicit;
        let aliases = match aliases {
            Some(labels) => labels,
            None if args.is_empty() => {
                implicit = (0..tuples.len() as u64).collect::<Vec<_>>();
                &implicit
            }
            None => {
                return err(
                    "§11.1 K91",
                    "E_ARG: tuple-only source tokens require token_aliases",
                )
            }
        };
        crate::tokens::validate_aliases(aliases, &tuples)?;
        let mut classes = BTreeMap::new();
        let mut handles = BTreeMap::new();
        let mut ids = Vec::new();
        for (label, b) in aliases.iter().zip(tuples) {
            let token = if let Some(token) = classes.get(label) {
                token
            } else {
                let token = object.token_for(
                    if function.kind == "oracle" {
                        &function.name
                    } else {
                        ""
                    },
                    b,
                    None,
                )?;
                handles.insert(token.id(), token.clone());
                classes.entry(*label).or_insert(token)
            };
            ids.push(token.id());
        }
        let mut ids = ids.into_iter();
        let primary = primary.map(|_| ids.next().unwrap());
        let arguments = args
            .into_iter()
            .map(|(n, _)| (n, ids.next().unwrap()))
            .collect();
        Ok(Self {
            context: Arc::new(SourceTokenContext { primary, arguments }),
            handles,
        })
    }
    pub fn binding(
        &self,
        function: &crate::Function,
        values: &BTreeMap<String, Value>,
        primary: Option<crate::Budget>,
    ) -> Result<bennett_isa_conformance::ref_language::SourceBinding> {
        let params = &function.params[if function.kind == "oracle" { 4 } else { 3 }];
        let args = params
            .iter()
            .map(|p| {
                values
                    .get(&p.name)
                    .cloned()
                    .ok_or_else(|| crate::Error::new(format!("§11.1: unbound argument {}", p.name)))
            })
            .collect::<Result<Vec<_>>>()?;
        let bytes = canonical(&Value::Arr(args)).map_err(infeasible)?;
        let identities = self.context.primary.into_iter().chain(
            params
                .iter()
                .filter(|p| p.ty == crate::Type::Token)
                .map(|p| self.context.arguments[&p.name]),
        );
        let mut labels = BTreeMap::new();
        let token_aliases = identities
            .map(|id| {
                let next = labels.len() as u64;
                *labels.entry(id).or_insert(next)
            })
            .collect();
        let b = primary.unwrap_or(crate::Budget::steps(0));
        Ok(bennett_isa_conformance::ref_language::SourceBinding {
            budget: [u128::from(b.steps), b.heap, b.stack, b.depth],
            params_hash: crate::sha256_hex(bytes.as_bytes()),
            token_aliases,
            inherited_controls: 0,
            variant: "forward".into(),
        })
    }
    pub fn resolve(
        &self,
        object: &Object,
        function: &crate::Function,
        values: &BTreeMap<String, Value>,
        primary: Option<crate::Budget>,
        context: Option<&SourceTokenContext>,
    ) -> Result<Self> {
        let context = context.ok_or_else(|| {
            crate::Error::new("§11.1 K91: missing captured token identity context".into())
        })?;
        let params = function
            .params
            .iter()
            .flatten()
            .filter(|p| p.ty == crate::Type::Token)
            .collect::<Vec<_>>();
        if context.arguments.len() != params.len() || context.primary.is_some() != primary.is_some()
        {
            return err("§11.1 K91", "captured token context has wrong slots");
        }
        let check = |id: u64, b: crate::Budget| -> Result<()> {
            let token = self.handles.get(&id).ok_or_else(|| {
                crate::Error::new("§11.1 K91: uncaptured source token identity".into())
            })?;
            if !token.for_program(object) || token.tuple() != b {
                return err("§11.1 K91", "source identity and tuple disagree");
            }
            Ok(())
        };
        if let Some(b) = primary {
            check(context.primary.unwrap(), b)?;
        }
        for p in params {
            let value = values.get(&p.name).ok_or_else(|| {
                crate::Error::new(format!("§11.1 K91: missing token value {}", p.name))
            })?;
            let id = context.arguments.get(&p.name).ok_or_else(|| {
                crate::Error::new(format!("§11.1 K91: missing token identity {}", p.name))
            })?;
            check(*id, tuple(value)?)?;
        }
        Ok(Self {
            context: Arc::new(context.clone()),
            handles: self.handles.clone(),
        })
    }
}

pub(crate) fn forward(
    function: &crate::Function,
    primary: Option<&str>,
    actuals: &[String],
    identities: &BTreeMap<String, u64>,
) -> Result<Arc<SourceTokenContext>> {
    let get = |name: &str| {
        identities.get(name).copied().ok_or_else(|| {
            crate::Error::new(format!(
                "§11.1 K91: missing forwarded token identity {name}"
            ))
        })
    };
    let group = if function.kind == "oracle" { 4 } else { 3 };
    if function.params[group].len() != actuals.len() {
        return err("§11.1 K91", "forwarded source argument arity differs");
    }
    let arguments = function.params[group]
        .iter()
        .zip(actuals)
        .filter(|(p, _)| p.ty == crate::Type::Token)
        .map(|(p, n)| Ok((p.name.clone(), get(n)?)))
        .collect::<Result<_>>()?;
    Ok(Arc::new(SourceTokenContext {
        primary: primary.map(get).transpose()?,
        arguments,
    }))
}
