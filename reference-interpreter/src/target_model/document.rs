use super::{
    document_schema, verify_native_model, CostExpression, ExpressionBindings, ModelContext,
    ModelVerification, Rational,
};
use crate::{err, json::Json, Error, Result};
use std::collections::BTreeMap;

/// Canonical §8 target document and per-entry K86 checker. Entry selection,
/// scheme execution and schedule/certificate coverage are separate obligations;
/// reading this document never establishes an executable certificate.
#[derive(Debug)]
pub struct NativeTarget {
    document: Json,
    hash: String,
    gates: Vec<NativeEntry>,
    tables: BTreeMap<String, Vec<Rational>>,
}

#[derive(Debug)]
struct NativeEntry {
    epsilon: CostExpression,
    costs: BTreeMap<String, CostExpression>,
}

#[derive(Debug)]
pub struct ResolvedNativeEntry {
    pub achieved_epsilon: Rational,
    pub costs: BTreeMap<String, Rational>,
    pub model: ModelVerification,
}

impl NativeTarget {
    pub fn parse(bytes: &str) -> Result<Self> {
        let document = Json::parse(bytes)?;
        if document.canonical() != bytes {
            return err("§8.1", "target bytes must be RFC 8785 canonical JSON");
        }
        document_schema::validate(&document)?;
        let mut tables = BTreeMap::new();
        for (name, values) in document.get("tables")?.object()? {
            tables.insert(
                name.clone(),
                values
                    .array()?
                    .iter()
                    .map(document_schema::number)
                    .collect::<Result<_>>()?,
            );
        }
        let gates = document
            .get("gates")?
            .array()?
            .iter()
            .map(|gate| {
                Ok(NativeEntry {
                    epsilon: CostExpression::parse(gate.get("epsilon")?)?,
                    costs: gate
                        .get("cost")?
                        .object()?
                        .iter()
                        .map(|(name, v)| Ok((name.clone(), CostExpression::parse(v)?)))
                        .collect::<Result<_>>()?,
                })
            })
            .collect::<Result<_>>()?;
        Ok(Self {
            document,
            hash: crate::sha256_hex(bytes.as_bytes()),
            gates,
            tables,
        })
    }
    pub fn hash(&self) -> &str {
        &self.hash
    }

    /// The caller identifies an already selected native entry. This verifies
    /// its resolved control/accuracy context, costs and supplied model, without
    /// claiming that the entry was selected by an object's bound schedule.
    pub fn verify_entry(
        &self,
        index: usize,
        context: &ModelContext,
        precision: usize,
    ) -> Result<ResolvedNativeEntry> {
        let entry = self
            .gates
            .get(index)
            .ok_or_else(|| Error::new("§8.3: native entry index outside target".into()))?;
        let gate = &self.document.get("gates")?.array()?[index];
        let range = gate.get("controls")?;
        let c = Rational::integer(context.total_controls as i128);
        if c < document_schema::number(range.get("min")?)?
            || (!document_schema::unbounded(range.get("max")?)
                && c > document_schema::number(range.get("max")?)?)
        {
            return err("§8.3", "native entry does not cover resolved controls");
        }
        let mut bindings = ExpressionBindings::from([
            ("k".into(), Rational::integer(context.own_controls as i128)),
            ("c".into(), c),
        ]);
        if let Some(eps) = &context.requested_epsilon {
            bindings.insert("eps".into(), eps.clone());
        }
        let epsilon = entry.epsilon.evaluate(&bindings, &self.tables)?;
        if epsilon < Rational::integer(0) {
            return err("§8.1", "negative achieved epsilon");
        }
        if epsilon == Rational::integer(0)
            && (entry.epsilon.uses_eps() || entry.costs.values().any(CostExpression::uses_eps))
        {
            return err(
                "§8.1",
                "zero-epsilon entry must not reference eps or an epsilon index",
            );
        }
        let mut costs = BTreeMap::new();
        for (resource, expr) in &entry.costs {
            let cost = expr.evaluate(&bindings, &self.tables)?;
            if cost < Rational::integer(0) {
                return err("§8.1", format!("negative cost for {resource}"));
            }
            costs.insert(resource.clone(), cost);
        }
        let model = verify_native_model(
            gate.object()?.get("realisation_model"),
            gate.get("op")?.string()?,
            context,
            &epsilon,
            precision,
        )?;
        Ok(ResolvedNativeEntry {
            achieved_epsilon: epsilon,
            costs,
            model,
        })
    }
}
