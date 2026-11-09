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
        if !certificate
            .get("regions")?
            .object()?
            .values()
            .any(|r| r.object().is_ok_and(|m| m.contains_key("frame_contexts")))
        {
            return Ok(());
        }
        let identity = self.object.identity()?;
        if identity.closure_hashes[&callee.name].as_deref()
            != Some(certificate.get("bindings")?.get("closure_hash")?.string()?)
        {
            return err(
                "§12.3",
                "contextual frame certificate is bound to a different invocation root",
            );
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
                actuals.push(Value::Str(p.name.clone()));
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
        self.object.select_invocation_frames(&contexts, classes)?;
        Ok(())
    }
}
