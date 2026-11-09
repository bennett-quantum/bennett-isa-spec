use super::Rational;
use crate::{err, json::Json, Error, Result};
use std::collections::BTreeMap;

pub type ExpressionBindings = BTreeMap<String, Rational>;

#[derive(Clone, Debug)]
pub enum CostExpression {
    Number(Rational),
    Variable(String),
    Apply(CostOperator, Vec<CostExpression>),
    Table(String, TableIndex),
}

#[derive(Clone, Copy, Debug)]
pub enum CostOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Maximum,
    Minimum,
    Ceiling,
    Log2InverseEps,
}

#[derive(Clone, Copy, Debug)]
pub enum TableIndex {
    Controls,
    EpsDecade,
}

impl CostExpression {
    /// The closed §8.1 grammar; contextual availability is checked at evaluation.
    pub fn parse(value: &Json) -> Result<Self> {
        if let Json::Number(n) = value {
            return Ok(Self::Number(Rational::decimal(n)?));
        }
        let obj = value.object()?;
        if let Some(var) = obj.get("var") {
            value.fields(&["var"], &[])?;
            let var = var.string()?;
            if !["k", "c", "eps", "a", "w", "i", "theta"].contains(&var) {
                return err("§8.1", format!("unknown variable {var}"));
            }
            return Ok(Self::Variable(var.into()));
        }
        if let Some(table) = obj.get("table") {
            value.fields(&["table", "index"], &[])?;
            let index = match value.get("index")?.string()? {
                "k" => TableIndex::Controls,
                "eps_decade" => TableIndex::EpsDecade,
                _ => return err("§8.1", "invalid table index"),
            };
            return Ok(Self::Table(table.string()?.into(), index));
        }
        value.fields(&["op", "args"], &[])?;
        let (op, count) = match value.get("op")?.string()? {
            "+" => (CostOperator::Add, 2),
            "-" => (CostOperator::Subtract, 2),
            "*" => (CostOperator::Multiply, 2),
            "/" => (CostOperator::Divide, 2),
            "max" => (CostOperator::Maximum, 2),
            "min" => (CostOperator::Minimum, 2),
            "ceil" => (CostOperator::Ceiling, 1),
            "log2inv_eps" => (CostOperator::Log2InverseEps, 0),
            _ => return err("§8.1", "unknown expression operator"),
        };
        let args = value.get("args")?.array()?;
        if args.len() != count {
            return err("§8.1", "wrong expression argument count");
        }
        Ok(Self::Apply(
            op,
            args.iter().map(Self::parse).collect::<Result<_>>()?,
        ))
    }

    pub fn uses_eps(&self) -> bool {
        match self {
            Self::Variable(var) => var == "eps",
            Self::Apply(CostOperator::Log2InverseEps, _)
            | Self::Table(_, TableIndex::EpsDecade) => true,
            Self::Apply(_, args) => args.iter().any(Self::uses_eps),
            _ => false,
        }
    }

    pub fn evaluate(
        &self,
        bindings: &ExpressionBindings,
        tables: &BTreeMap<String, Vec<Rational>>,
    ) -> Result<Rational> {
        use CostOperator::*;
        match self {
            Self::Number(n) => Ok(n.clone()),
            Self::Variable(var) => binding(bindings, var).cloned(),
            Self::Table(name, index) => {
                let index = match index {
                    TableIndex::Controls => binding(bindings, "k")?.clone(),
                    TableIndex::EpsDecade => inverse_eps_index(bindings, 10)?,
                };
                let index = host_index(&index)?;
                tables
                    .get(name)
                    .and_then(|t| t.get(index))
                    .cloned()
                    .ok_or_else(|| {
                        Error::new(format!("§8.1: table {name} index {index} outside array"))
                    })
            }
            Self::Apply(Log2InverseEps, _) => inverse_eps_index(bindings, 2),
            Self::Apply(op, args) => {
                let a = args[0].evaluate(bindings, tables)?;
                if matches!(op, Ceiling) {
                    return Ok(a.ceil());
                }
                let b = args[1].evaluate(bindings, tables)?;
                Ok(match op {
                    Add => a.add(&b),
                    Subtract => a.sub(&b),
                    Multiply => a.mul(&b),
                    Divide => a.div(&b)?,
                    Maximum => a.max(b),
                    Minimum => a.min(b),
                    Ceiling | Log2InverseEps => unreachable!("handled unary/zero-arity operators"),
                })
            }
        }
    }
}

pub(super) fn binding<'a>(bindings: &'a ExpressionBindings, var: &str) -> Result<&'a Rational> {
    bindings
        .get(var)
        .ok_or_else(|| Error::new(format!("§8.1/§8.3: unbound variable {var}")))
}

pub(super) fn host_index(value: &Rational) -> Result<usize> {
    let (n, d) = value.pair();
    if d != "1" || n.starts_with('-') {
        return err("§8.1", "index must be a nonnegative integer");
    }
    n.parse()
        .map_err(|_| crate::resource_error("§8.1", "index exceeds host range"))
}

fn inverse_eps_index(bindings: &ExpressionBindings, base: i128) -> Result<Rational> {
    let mut eps = binding(bindings, "eps")?.clone();
    let one = Rational::integer(1);
    if eps <= Rational::integer(0) || eps > one {
        return err("§8.1", "epsilon index requires 0 < eps <= 1");
    }
    let mut index = Rational::integer(0);
    while eps < one {
        eps = eps.mul(&Rational::integer(base));
        index = index.add(&one);
    }
    Ok(index)
}
