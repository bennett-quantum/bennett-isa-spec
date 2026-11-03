//! Device token identities are not tuple identities. Only validated host/device
//! minting constructs a handle; integer values cannot fabricate one.
use crate::{err, Budget, ClassicalValue, Object, Result};
use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

#[derive(Debug)]
pub(crate) struct TokenNamespace {
    next: AtomicU64,
    runs: AtomicU64,
}
impl TokenNamespace {
    pub fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
            runs: AtomicU64::new(1),
        }
    }
    pub fn run(&self) -> Result<u64> {
        fresh(&self.runs)
    }
}
fn fresh(counter: &AtomicU64) -> Result<u64> {
    counter
        .try_update(Ordering::Relaxed, Ordering::Relaxed, |id| id.checked_add(1))
        .map_err(|_| crate::resource_error("§5.4", "token/run identity namespace exhausted"))
}

#[derive(Clone, Debug)]
pub struct BudgetToken {
    namespace: Arc<TokenNamespace>,
    id: u64,
    tuple: Budget,
    /// None is a host token, valid for the loaded program. A device mint is
    /// tied to one run; its active mint-frame set determines revocation.
    run: Option<u64>,
    oracle: String,
}
impl PartialEq for BudgetToken {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && Arc::ptr_eq(&self.namespace, &other.namespace)
    }
}
impl Eq for BudgetToken {}
impl BudgetToken {
    pub(crate) fn for_program(&self, object: &Object) -> bool {
        Arc::ptr_eq(&self.namespace, &object.tokens)
    }
    pub fn tuple(&self) -> Budget {
        self.tuple
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn validated_oracle(&self) -> &str {
        &self.oracle
    }
    pub(crate) fn valid(
        &self,
        object: &Object,
        run: u64,
        active: &std::collections::BTreeSet<u64>,
    ) -> bool {
        self.for_program(object)
            && self
                .run
                .is_none_or(|owner| owner == run && active.contains(&self.id))
    }
}
impl Object {
    pub(crate) fn bind_entry_tokens(
        &self, args: &[ClassicalValue], aliases: Option<&[u64]>, run: u64,
    ) -> Result<Vec<ClassicalValue>> {
        let tokens = args.iter().filter(|v| v.token_budget().is_some()).collect::<Vec<_>>();
        for value in &tokens {
            self.validate_budget(value.token_budget().unwrap()).map_err(|e| e.context("§10 E_ARG"))?;
            if let ClassicalValue::DeviceToken(token) = value {
                if !token.valid(self,run,&Default::default()) {
                    return err("§10", "E_ARG: token is invalid for this program/run");
                }
            }
        }
        let derived;
        let aliases = if let Some(aliases) = aliases { aliases } else {
            if tokens.iter().any(|v| matches!(v,ClassicalValue::Token(_))) {
                return err("§10 K91", "E_ARG: tuple-only token arguments require token_aliases");
            }
            let mut labels = BTreeMap::new();
            derived = tokens.iter().map(|v| {
                let ClassicalValue::DeviceToken(token) = v else { unreachable!() };
                let next = labels.len() as u64;
                *labels.entry(token.id()).or_insert(next)
            }).collect::<Vec<_>>();
            derived.as_slice()
        };
        let tuples = tokens.iter().map(|v| v.token_budget().unwrap()).collect::<Vec<_>>();
        validate_aliases(aliases,&tuples)?;
        let mut bound: BTreeMap<u64,BudgetToken> = BTreeMap::new();
        let mut handles = BTreeMap::new();
        for (value,label) in tokens.iter().zip(aliases) {
            if let ClassicalValue::DeviceToken(token) = value {
                if bound.get(label).is_some_and(|prior| prior!=token)
                    || handles.get(&token.id()).is_some_and(|prior| prior!=label) {
                    return err("§10 K91", "E_ARG: partition differs from actual token identities");
                }
                bound.insert(*label,token.clone());
                handles.insert(token.id(),*label);
            }
        }
        // Validate the entire request before allocating any anonymous identity.
        for (label,tuple) in aliases.iter().zip(&tuples) {
            if !bound.contains_key(label) {
                let oracle = self.functions.values().find(|f| f.kind=="oracle")
                    .ok_or_else(|| crate::Error::new("§10 K91: E_ARG: tuple token requires a program oracle".into()))?;
                bound.insert(*label,self.token_for(&oracle.name,*tuple,None)?);
            }
        }
        let mut labels = aliases.iter();
        Ok(args.iter().map(|v| if v.token_budget().is_some() {
            ClassicalValue::DeviceToken(bound[labels.next().unwrap()].clone())
        } else { v.clone() }).collect())
    }
    /// Rust adapter for bennett_budget's validated, program-scoped identity.
    pub fn budget_token(&self, oracle: &str, tuple: Budget) -> Result<BudgetToken> {
        if oracle.is_empty() {
            return err("§5.4", "host budget requires an oracle reference");
        }
        self.token_for(oracle, tuple, None)
    }
    pub(crate) fn token_for(
        &self,
        oracle: &str,
        tuple: Budget,
        run: Option<u64>,
    ) -> Result<BudgetToken> {
        if self.isa_version != crate::IsaVersion::V09 {
            return err("§5.4", "device token identities require ISA 0.9");
        }
        if !oracle.is_empty() && self.declaration(oracle)?.kind != "oracle" {
            return err("§5.4", "budget token reference must name an oracle");
        }
        self.validate_budget(tuple)?;
        Ok(BudgetToken {
            namespace: self.tokens.clone(),
            id: fresh(&self.tokens.next)?,
            tuple,
            run,
            oracle: oracle.into(),
        })
    }
}

/// K91's canonical first-occurrence labels and equal-label tuple constraint.
/// Shared by concrete request and certificate/schedule binding validation.
pub(crate) fn validate_aliases(aliases: &[u64], tuples: &[Budget]) -> Result<()> {
    if aliases.len()!=tuples.len() {
        return err("§5.4 K91", "E_ARG: token_aliases arity differs from token list");
    }
    let mut seen = Vec::<Budget>::new();
    for (label,tuple) in aliases.iter().zip(tuples) {
        if *label > seen.len() as u64 {
            return err("§5.4 K91", "E_ARG: token_aliases labels must start at zero without gaps");
        }
        if *label == seen.len() as u64 { seen.push(*tuple); }
        else if seen[*label as usize] != *tuple {
            return err("§5.4 K91", "E_ARG: equal token_aliases labels require equal tuples");
        }
    }
    Ok(())
}
