use super::*;
use crate::Function;
use bennett_isa_conformance::json::{self, Json as Value};

impl ClassicalMachine<'_> {
    pub(super) fn preflight_context_frames(
        &self,
        callee: &Function,
        bindings: &QuantumBindings,
        controls: &[crate::quantum::Wire],
        classes: &[Vec<String>],
        adjoint: bool,
        token: Option<(Budget, Option<&crate::BudgetToken>)>,
    ) -> Result<()> {
        let Some(certificate) = self.object.section("certificate") else {
            return Ok(());
        };
        let regions = certificate.get("regions")?.object()?;
        let contextual = regions
            .values()
            .any(|r| r.object().is_ok_and(|m| m.contains_key("frame_contexts")));
        let complete = contextual
            || certificate
                .get("bindings")?
                .object()?
                .contains_key("schedule_hash");
        let identity = self.object.identity()?;
        let closure = identity.closure_hashes[&callee.name]
            .as_deref()
            .ok_or_else(|| Error::new("§12.3: unresolved frame root".into()))?;
        if closure != certificate.get("bindings")?.get("closure_hash")?.string()? {
            if contextual {
                return err(
                    "§12.3",
                    "contextual frame certificate is bound to a different invocation root",
                );
            }
            return Ok(());
        }
        if !complete && !regions.contains_key(closure) {
            return Ok(());
        }
        let arguments =
            callee.params[if callee.kind == "oracle" { 4 } else { 3 }]
                .iter()
                .map(|p| {
                    bindings.args.get(&p.name).cloned().ok_or_else(|| {
                        Error::new("§12.3: missing contextual source argument".into())
                    })
                })
                .collect::<Result<Vec<_>>>()?;
        let resolved = self.object.resolve_invocation_tokens(
            callee,
            &arguments,
            token.map(|t| t.0),
            token.and_then(|t| t.1),
            None,
        )?;
        let values = json::parse(&crate::tokens::typed_arguments(&resolved.arguments).canonical())
            .map_err(|e| Error::new(format!("§12.3: source argument encoding: {e}")))?;
        let values = values
            .as_arr()
            .ok_or_else(|| Error::new("§12.3: source arguments must be an array".into()))?;
        let mut actuals = Vec::new();
        let mut bundles = Vec::new();
        let mut logical = BTreeMap::new();
        let mut next = 0;
        for p in callee.params.iter().flatten() {
            if matches!(p.ty, Type::Wire(_) | Type::Region(..)) {
                let wires = if let Some(w) = bindings.bundles.get(&p.name) {
                    w.clone()
                } else {
                    bindings
                        .regions
                        .get(&p.name)
                        .ok_or_else(|| {
                            Error::new("§12.3: unresolved contextual interface region".into())
                        })?
                        .cells
                        .iter()
                        .flatten()
                        .copied()
                        .collect()
                };
                actuals.push(match p.ty {
                    Type::Region(a, w) => Value::Obj(BTreeMap::from([
                        ("address".into(), Value::Num(a as f64)),
                        ("cell".into(), Value::Num(w as f64)),
                        (
                            "role".into(),
                            Value::Str(p.role.clone().unwrap_or_else(|| "in".into())),
                        ),
                        ("origin".into(), Value::Str(format!("interface:{}", p.name))),
                        ("cells".into(), Value::Str(p.name.clone())),
                    ])),
                    _ => Value::Str(p.name.clone()),
                });
                bundles.push((p.name.clone(), wires.len()));
                for (i, wire) in wires.iter().enumerate() {
                    logical.insert(*wire, (p.name.clone(), i));
                }
            } else {
                actuals.push(values.get(next).cloned().ok_or_else(|| {
                    Error::new("§12.3: contextual argument arity mismatch".into())
                })?);
                next += 1;
            }
        }
        let mut source_controls = Vec::new();
        for (i, wire) in controls.iter().enumerate() {
            let value = if let Some(value) = logical.get(wire) {
                value.clone()
            } else {
                let name = format!("$inherited{i}");
                bundles.push((name.clone(), 1));
                (name, 0)
            };
            source_controls.push(value);
        }
        let borrowed = bundles
            .iter()
            .map(|(name, n)| (name.as_str(), *n))
            .collect::<Vec<_>>();
        let contexts = if let Some((budget, _)) = token {
            let invocation = crate::ref_library::OracleInvocation::capture_with_token_aliases(
                self.object,
                &callee.name,
                None,
                budget,
                &actuals,
                &resolved.aliases,
            )?;
            invocation.source_invocations(
                &self.object.bound_ref_library()?,
                &borrowed,
                &source_controls,
            )?
        } else {
            self.object.source_action_invocations_with_controls(
                &callee.name,
                &actuals,
                &borrowed,
                adjoint,
                Some(&resolved.aliases),
                &source_controls,
            )?
        };
        if complete {
            self.object.select_invocation_frames(&contexts, classes)?;
        } else {
            self.object
                .select_provided_invocation_frames(&contexts, classes)?;
        }
        Ok(())
    }
}
