use super::{expression::binding, ExpressionBindings, Rational};
use crate::{err, json::Json, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExactComplex {
    pub(super) re: Rational,
    pub(super) im: Rational,
}

impl ExactComplex {
    pub fn real(&self) -> &Rational {
        &self.re
    }
    pub fn imaginary(&self) -> &Rational {
        &self.im
    }
    pub(super) fn real_value(re: Rational) -> Self {
        Self {
            re,
            im: Rational::integer(0),
        }
    }
    pub(super) fn conjugate(&self) -> Self {
        Self {
            re: self.re.clone(),
            im: self.im.neg(),
        }
    }
    pub(super) fn add(&self, b: &Self) -> Self {
        Self {
            re: self.re.add(&b.re),
            im: self.im.add(&b.im),
        }
    }
    pub(super) fn sub(&self, b: &Self) -> Self {
        Self {
            re: self.re.sub(&b.re),
            im: self.im.sub(&b.im),
        }
    }
    pub(super) fn mul(&self, b: &Self) -> Self {
        Self {
            re: self.re.mul(&b.re).sub(&self.im.mul(&b.im)),
            im: self.re.mul(&b.im).add(&self.im.mul(&b.re)),
        }
    }
    pub(super) fn div(&self, b: &Self) -> Result<Self> {
        let n = b.re.mul(&b.re).add(&b.im.mul(&b.im));
        let product = self.mul(&b.conjugate());
        Ok(Self {
            re: product.re.div(&n)?,
            im: product.im.div(&n)?,
        })
    }
}

#[derive(Clone, Debug)]
pub enum ModelExpression {
    Rational(Rational),
    Variable(String),
    Apply(ModelOperator, Vec<ModelExpression>),
}

#[derive(Clone, Copy, Debug)]
pub enum ModelOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Sqrt,
    Exp,
    Sin,
    Cos,
    Pi,
    Imaginary,
}

impl ModelExpression {
    pub fn parse(value: &Json, declared: &[&str]) -> Result<Self> {
        if let Json::Array(pair) = value {
            if pair.len() != 2 {
                return err("§8.3", "rational leaf requires two strings");
            }
            return Ok(Self::Rational(Rational::leaf(
                pair[0].string()?,
                pair[1].string()?,
            )?));
        }
        if let Some(var) = value.object()?.get("var") {
            value.fields(&["var"], &[])?;
            let var = var.string()?;
            if !declared.contains(&var) {
                return err("§8.3", format!("undeclared model variable {var}"));
            }
            return Ok(Self::Variable(var.into()));
        }
        value.fields(&["op", "args"], &[])?;
        let (op, count) = match value.get("op")?.string()? {
            "+" => (ModelOperator::Add, 2),
            "-" => (ModelOperator::Subtract, 2),
            "*" => (ModelOperator::Multiply, 2),
            "/" => (ModelOperator::Divide, 2),
            "sqrt" => (ModelOperator::Sqrt, 1),
            "exp" => (ModelOperator::Exp, 1),
            "sin" => (ModelOperator::Sin, 1),
            "cos" => (ModelOperator::Cos, 1),
            "pi" => (ModelOperator::Pi, 0),
            "i" => (ModelOperator::Imaginary, 0),
            _ => return err("§8.3", "unknown exact expression operator"),
        };
        let args = value.get("args")?.array()?;
        if args.len() != count {
            return err("§8.3", "wrong exact expression argument count");
        }
        Ok(Self::Apply(
            op,
            args.iter()
                .map(|v| Self::parse(v, declared))
                .collect::<Result<_>>()?,
        ))
    }

    /// Some means an independently established rational complex value. None
    /// retains a nonrational identity as an outstanding obligation, never zero.
    pub fn exact(&self, bindings: &ExpressionBindings) -> Result<Option<ExactComplex>> {
        use ModelOperator::*;
        let value = match self {
            Self::Rational(n) => ExactComplex::real_value(n.clone()),
            Self::Variable(var) => ExactComplex::real_value(binding(bindings, var)?.clone()),
            Self::Apply(Pi, _) => return Ok(None),
            Self::Apply(Imaginary, _) => ExactComplex {
                re: Rational::integer(0),
                im: Rational::integer(1),
            },
            Self::Apply(op, args) => {
                // Evaluate every child even if an earlier child is nonrational:
                // an unbound variable or exact zero divisor is still invalid.
                let args = args
                    .iter()
                    .map(|a| a.exact(bindings))
                    .collect::<Result<Vec<_>>>()?;
                if matches!(op, Divide)
                    && args[1] == Some(ExactComplex::real_value(Rational::integer(0)))
                {
                    return err("§8.3", "zero divisor in exact model expression");
                }
                let Some(a) = &args[0] else {
                    return Ok(None);
                };
                match op {
                    Add | Subtract | Multiply | Divide => {
                        let Some(b) = &args[1] else {
                            return Ok(None);
                        };
                        match op {
                            Add => a.add(b),
                            Subtract => a.sub(b),
                            Multiply => a.mul(b),
                            Divide => a.div(b)?,
                            _ => unreachable!(),
                        }
                    }
                    Sqrt => {
                        if a.im != Rational::integer(0) {
                            return Ok(None);
                        }
                        let n = a.re.n.abs().sqrt_floor();
                        let d = a.re.d.sqrt_floor();
                        if n.mul(&n) != a.re.n.abs() || d.mul(&d) != a.re.d {
                            return Ok(None);
                        }
                        let root = Rational::reduced(n, d);
                        if a.re < Rational::integer(0) {
                            ExactComplex {
                                re: Rational::integer(0),
                                im: root,
                            }
                        } else {
                            ExactComplex::real_value(root)
                        }
                    }
                    Exp | Sin | Cos if *a == ExactComplex::real_value(Rational::integer(0)) => {
                        ExactComplex::real_value(Rational::integer(if matches!(op, Sin) {
                            0
                        } else {
                            1
                        }))
                    }
                    Exp | Sin | Cos => return Ok(None),
                    Pi | Imaginary => unreachable!("handled constants"),
                }
            }
        };
        Ok(Some(value))
    }
}
