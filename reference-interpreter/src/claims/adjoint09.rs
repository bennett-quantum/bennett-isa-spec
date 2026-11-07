//! §12.1 K94: the reference-action adjoint law, not a realisation comparison.
use super::*;
use crate::Instruction;

pub(super) enum StructuralRule {
    TranslationLemma,
    InversePairLibrary,
    AdjointTable,
}
impl StructuralRule {
    pub(super) fn parse(rule: &str) -> Result<Self> {
        match rule {
            "translation_lemma" => Ok(Self::TranslationLemma),
            "inverse_pair_library" => Ok(Self::InversePairLibrary),
            "adjoint_table" => Ok(Self::AdjointTable),
            _ => err("§12.1", format!("unknown structural rule {rule}")),
        }
    }
}

/// Each variant denotes one construction in the §5.3 table. The source
/// validator owns operand, lifetime, version and control side conditions.
enum AdjointStep {
    SelfAdjoint,
    NegateAngle,
    ExchangeGate,
    ExchangeLifetime,
    ExchangePair,
    Call,
    Predicate,
    Conditional,
    Loop,
}
impl AdjointStep {
    fn parse(op: &str) -> Result<Self> {
        match op {
            "x" | "cx" | "ccx" | "mcx" | "swap" | "h" | "cz" | "qxor" | "qexch" | "qstore"
            | "qexchm" => Ok(Self::SelfAdjoint),
            "rx" | "ry" | "rz" => Ok(Self::NegateAngle),
            "s" | "sdg" | "t" | "tdg" => Ok(Self::ExchangeGate),
            "qalloc" | "qfree" | "qborrow" | "qreturn" | "qregion" | "qcopy" | "quncopy"
            | "qload" | "qunload" => Ok(Self::ExchangeLifetime),
            "qinplace" => Ok(Self::ExchangePair),
            "oracle_call" | "action_call" | "adj" | "ctl" => Ok(Self::Call),
            "qpred" | "qunpred" => Ok(Self::Predicate),
            "qif" => Ok(Self::Conditional),
            "qloop" => Ok(Self::Loop),
            _ => err(
                "§12.1 adjoint_table",
                format!("no §5.3 replacement for {op}"),
            ),
        }
    }
}

fn check_steps(seq: &[Instruction]) -> Result<()> {
    for i in seq {
        match AdjointStep::parse(&i.op)? {
            AdjointStep::Call => {
                crate::validation::normalize_call(i)?;
            }
            AdjointStep::Conditional | AdjointStep::Loop => {
                for body in &i.bodies {
                    check_steps(body)?;
                }
            }
            AdjointStep::SelfAdjoint
            | AdjointStep::NegateAngle
            | AdjointStep::ExchangeGate
            | AdjointStep::ExchangeLifetime
            | AdjointStep::ExchangePair
            | AdjointStep::Predicate => {}
        }
    }
    Ok(())
}

impl Object {
    pub(super) fn verify_adjoint_table09(
        &self,
        closure: &str,
        index: usize,
        claim: &Json,
        f: &Function,
        active: &mut BTreeSet<(String, usize)>,
    ) -> Result<ClaimVerdict> {
        crate::metadata_schema::validate_for(crate::IsaVersion::V09, "claim", claim)?;
        if !matches!(f.kind.as_str(), "action" | "perm_action") {
            return err(
                "§12.1 adjoint_table",
                "containing region must be an action or perm_action",
            );
        }
        let invocation = claim.get("invocation")?;
        if invocation.get("args")? != &Json::String("all".into()) {
            self.validate_invocation09(invocation, f)?;
        } else if f.params[3].iter().any(|p| p.ty == Type::Token)
            && invocation.object()?.get("token_aliases") != Some(&Json::String("all".into()))
        {
            return err(
                "§12.3 K91",
                "uniform token arguments require token_aliases:all",
            );
        }
        let mut unavailable = BTreeSet::new();
        let mut reached = BTreeSet::new();
        self.adjoint_source09(f, &mut reached, &mut unavailable)?;
        self.validate_adjoint_source09(&reached)?;
        for sub in claim.get("subclaims")?.array()? {
            let s = usize::try_from(sub.uint()?).map_err(|_| {
                Error::new("§12.1 adjoint_table: subclaim index exceeds usize".into())
            })?;
            if s >= index {
                return err(
                    "§12.1 adjoint_table",
                    "subclaim must be an earlier local index",
                );
            }
            // Check every premise even after an unavailable one: a refutation
            // is a rejection, not an unchecked result hidden by array order.
            match self.verify_claim_guarded(closure, s, active)? {
                ClaimVerdict::Proof => {}
                ClaimVerdict::Evidence { .. }
                | ClaimVerdict::ProofRelativeToImage
                | ClaimVerdict::Unverified(_) => {
                    unavailable.insert(format!(
                        "adjoint_table subclaim {s} is not an established proof"
                    ));
                }
            }
        }
        if unavailable.is_empty() {
            Ok(ClaimVerdict::Proof)
        } else {
            Ok(ClaimVerdict::Unverified(
                unavailable.into_iter().collect::<Vec<_>>().join("; "),
            ))
        }
    }

    fn adjoint_source09(
        &self,
        f: &Function,
        seen: &mut BTreeSet<String>,
        unavailable: &mut BTreeSet<String>,
    ) -> Result<()> {
        if !self
            .functions
            .get(&f.name)
            .is_some_and(|definition| std::ptr::eq(definition, f))
        {
            return err(
                "§12.1 adjoint_table",
                format!("source definition @{} differs from its map key", f.name),
            );
        }
        if !seen.insert(f.name.clone()) {
            return Ok(());
        }
        for block in &f.blocks {
            self.adjoint_dependencies09(f, &block.instructions, seen, unavailable)?;
        }
        Ok(())
    }

    fn adjoint_dependencies09(
        &self,
        f: &Function,
        seq: &[Instruction],
        seen: &mut BTreeSet<String>,
        unavailable: &mut BTreeSet<String>,
    ) -> Result<()> {
        for i in seq {
            for operand in &i.operands {
                if let Some(name) = operand.strip_prefix('@') {
                    // §4.2 lift.addr may name an interface region, not a symbol.
                    if i.op == "lift.addr"
                        && f.params
                            .iter()
                            .flatten()
                            .any(|p| p.name == name && matches!(p.ty, Type::Region(..)))
                    {
                        continue;
                    }
                    if let Some(callee) = self.functions.get(name) {
                        self.adjoint_source09(callee, seen, unavailable)?;
                    } else if self.data.contains_key(name) {
                        seen.insert(name.into());
                    } else {
                        let imports = self.section("imports").ok_or_else(|| {
                            Error::new(format!("§12.1 adjoint_table: undefined symbol @{name}"))
                        })?;
                        imports.get(name)?;
                        seen.insert(name.into());
                        unavailable.insert(format!(
                            "adjoint_table linked definition unavailable: @{name}"
                        ));
                    }
                }
            }
            for body in &i.bodies {
                self.adjoint_dependencies09(f, body, seen, unavailable)?;
            }
        }
        Ok(())
    }

    fn validate_adjoint_source09(&self, reached: &BTreeSet<String>) -> Result<()> {
        // K94 premise 2 is scoped to this dependency closure, not unrelated
        // definitions. The copy selects current fields; no cached lines,
        // parse verdict or pair-proof cache establishes source validity.
        let mut source = self.clone();
        source.functions.retain(|name, _| reached.contains(name));
        source.data.retain(|name, _| reached.contains(name));
        source
            .metadata
            .retain(|section, _| matches!(section.as_str(), "symbols" | "imports" | "layout"));
        let mut symbols = BTreeMap::new();
        for (index, symbol) in self.symbols()? {
            if reached.contains(symbol.get("name")?.string()?) {
                symbols.insert(index.clone(), symbol.clone());
            }
        }
        source
            .metadata
            .insert("symbols".into(), Json::Object(symbols));
        if let Some(imports) = source.metadata.get_mut("imports") {
            let Json::Object(imports) = imports else {
                return err("§12.1 adjoint_table", "imports must be an object");
            };
            imports.retain(|name, _| reached.contains(name));
        }
        let scope = format!("§12.1 adjoint_table source closure {reached:?}");
        source
            .validate_metadata_types()
            .map_err(|e| e.context(&scope))?;
        source
            .validate_metadata_versions()
            .map_err(|e| e.context(&scope))?;
        source.validate_symbols().map_err(|e| e.context(&scope))?;
        source
            .validate_local_schemas()
            .map_err(|e| e.context(&scope))?;
        source
            .validate_call_graph()
            .map_err(|e| e.context(&scope))?;
        for name in source.functions.keys() {
            let f = &self.functions[name];
            let scope = format!("§12.1 adjoint_table source @{name}");
            if matches!(f.kind.as_str(), "action" | "perm_action") {
                for block in &f.blocks {
                    check_steps(&block.instructions).map_err(|e| e.context(&scope))?;
                }
            }
            // Keep the original linked object as validation context: a valid
            // borrow certificate may refer to an independent proof/program
            // outside the source-call closure. Only the selected f is checked.
            self.validate_function(f).map_err(|e| e.context(&scope))?;
        }
        Ok(())
    }
}
