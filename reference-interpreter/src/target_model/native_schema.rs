//! Validate the recognized closed grammar before deciding which obligations
//! this checker can establish. Shape checking does not materialise matrices.
use super::{ExpressionBindings, ModelExpression, Rational};
use crate::{err, json::Json, Result};

pub(super) fn size(v: &Json, k: usize, c: usize, d: usize) -> Result<u128> {
    Ok(size_value(v, Some((k, c, d)))?.expect("resolved Size"))
}
// Unselected rows have symbolic dimensions. Still reject every malformed
// production and every shape incompatibility independent of its context.
fn size_value(v: &Json, context: Option<(usize, usize, usize)>) -> Result<Option<u128>> {
    if !matches!(v, Json::Object(_)) {
        return Ok(Some(v.uint()?));
    }
    if let Some(var) = v.object()?.get("var") {
        v.fields(&["var"], &[])?;
        let index = match var.string()? {
            "k" => 0,
            "c" => 1,
            "d" => 2,
            _ => return err("§8.3", "unknown Size variable"),
        };
        return Ok(context.map(|(k, c, d)| [k, c, d][index] as u128));
    }
    v.fields(&["op", "args"], &[])?;
    let op = v.get("op")?.string()?;
    let count = match op {
        "+" | "-" | "*" => 2,
        "pow2" => 1,
        _ => return err("§8.3", "unknown Size operator"),
    };
    let args = v.get("args")?.array()?;
    if args.len() != count {
        return err("§8.3", "wrong Size argument count");
    }
    let a = size_value(&args[0], context)?;
    let b = if count == 2 {
        size_value(&args[1], context)?
    } else {
        Some(0)
    };
    let (Some(a), Some(b)) = (a, b) else {
        return Ok(None);
    };
    if op == "-" && a < b {
        return err("§8.3", "negative Size result");
    }
    let value = match op {
        "+" => a.checked_add(b),
        "-" => a.checked_sub(b),
        "*" => a.checked_mul(b),
        "pow2" => u32::try_from(a).ok().and_then(|a| 1u128.checked_shl(a)),
        _ => unreachable!(),
    };
    value
        .map(Some)
        .ok_or_else(|| crate::resource_error("§8.3", "Size exceeds host range"))
}

pub(super) struct Grammar<'a> {
    /// Unselected explicit rows have no resolved physical context here.
    pub resolve_context: bool,
    pub k: usize,
    pub c: usize,
    pub d: usize,
    pub bindings: &'a ExpressionBindings,
}
type Dim = Option<u128>;
type Shape = (Dim, Dim);
impl Grammar<'_> {
    fn size(&self, v: &Json) -> Result<Dim> {
        size_value(v, self.resolve_context.then_some((self.k, self.c, self.d)))
    }
    fn complex(&self, v: &Json) -> Result<()> {
        let pair = v.array()?;
        if pair.len() != 2 {
            return err("§8.3", "complex requires two expressions");
        }
        let declared = self.bindings.keys().map(String::as_str).collect::<Vec<_>>();
        for expr in pair {
            let parsed = ModelExpression::parse(expr, &declared)?;
            if self.resolve_context {
                parsed.exact(self.bindings)?;
            }
        }
        Ok(())
    }
    fn positive(&self, v: &Json) -> Result<Dim> {
        let n = self.size(v)?;
        if n == Some(0) {
            return err("§8.3", "matrix dimensions must be positive");
        }
        Ok(n)
    }
    pub fn matrix(&self, v: &Json) -> Result<Shape> {
        if let Json::Array(rows) = v {
            if rows.is_empty() {
                return err("§8.3", "empty matrix");
            }
            let columns = rows[0].array()?.len();
            if columns == 0 {
                return err("§8.3", "empty matrix row");
            }
            for row in rows {
                if row.array()?.len() != columns {
                    return err("§8.3", "ragged matrix");
                }
                for value in row.array()? {
                    self.complex(value)?;
                }
            }
            return Ok((Some(rows.len() as u128), Some(columns as u128)));
        }
        let shape = match v.get("kind")?.string()? {
            "identity" => {
                v.fields(&["kind", "dimension"], &[])?;
                let n = self.positive(v.get("dimension")?)?;
                (n, n)
            }
            "zero" => {
                v.fields(&["kind", "rows", "columns"], &[])?;
                (
                    self.positive(v.get("rows")?)?,
                    self.positive(v.get("columns")?)?,
                )
            }
            "diagonal" => {
                v.fields(&["kind", "entries"], &[])?;
                let entries = v.get("entries")?.array()?;
                if entries.is_empty() {
                    return err("§8.3", "empty diagonal");
                }
                for e in entries {
                    self.complex(e)?;
                }
                (Some(entries.len() as u128), Some(entries.len() as u128))
            }
            "adjoint" => {
                v.fields(&["kind", "matrix"], &[])?;
                let (r, c) = self.matrix(v.get("matrix")?)?;
                (c, r)
            }
            "scale" => {
                v.fields(&["kind", "scalar", "matrix"], &[])?;
                self.complex(v.get("scalar")?)?;
                self.matrix(v.get("matrix")?)?
            }
            kind @ ("sum" | "product" | "tensor" | "block_diagonal") => {
                v.fields(&["kind", "matrices"], &[])?;
                let matrices = v.get("matrices")?.array()?;
                if matrices.is_empty() {
                    return err("§8.3", "empty matrix constructor list");
                }
                let mut shape = self.matrix(&matrices[0])?;
                for m in &matrices[1..] {
                    let b = self.matrix(m)?;
                    shape = match kind {
                        "sum" => (equal(shape.0, b.0)?, equal(shape.1, b.1)?),
                        "product" => {
                            equal(shape.1, b.0)?;
                            (shape.0, b.1)
                        }
                        "tensor" => (mul(shape.0, b.0)?, mul(shape.1, b.1)?),
                        "block_diagonal" => (add(shape.0, b.0)?, add(shape.1, b.1)?),
                        _ => return err("§8.3", "incompatible matrix constructor shapes"),
                    };
                }
                shape
            }
            "controlled" => {
                v.fields(&["kind", "controls", "matrix"], &[])?;
                let shape = self.matrix(v.get("matrix")?)?;
                equal(shape.0, shape.1)?;
                let blocks = power_dim(Some(2), self.size(v.get("controls")?)?)?;
                (mul(shape.0, blocks)?, mul(shape.1, blocks)?)
            }
            "tensor_power" => {
                v.fields(&["kind", "count", "matrix"], &[])?;
                let shape = self.matrix(v.get("matrix")?)?;
                let n = self.size(v.get("count")?)?;
                (power_dim(shape.0, n)?, power_dim(shape.1, n)?)
            }
            _ => return err("§8.3", "unknown matrix constructor"),
        };
        Ok(shape)
    }
    pub fn operation(&self, v: &Json, dimension: u128) -> Result<bool> {
        v.fields(&["matrix"], &["representation", "certificates"])?;
        let dimension = self.resolve_context.then_some(dimension);
        self.square(v.get("matrix")?, dimension)?;
        let mut unsupported = matches!(v.get("matrix")?, Json::Object(_));
        if let Some(rep) = v.object()?.get("representation") {
            unsupported = true;
            match rep.get("kind")?.string()? {
                "kraus" => {
                    rep.fields(&["kind", "operators"], &[])?;
                    self.operators(rep.get("operators")?, dimension)?;
                }
                "instrument" => {
                    rep.fields(&["kind", "outcomes"], &[])?;
                    let outcomes = rep.get("outcomes")?.array()?;
                    if outcomes.is_empty() {
                        return err("§8.3", "empty instrument");
                    }
                    let mut previous = None;
                    for outcome in outcomes {
                        outcome.fields(&["label", "operators"], &[])?;
                        let label = outcome.get("label")?.string()?;
                        label_check(label)?;
                        if previous.is_some_and(|p| p >= label) {
                            return err(
                                "§8.3",
                                "instrument labels must be in distinct bytewise order",
                            );
                        }
                        previous = Some(label);
                        self.operators(outcome.get("operators")?, dimension)?;
                    }
                }
                "environment" => {
                    rep.fields(&["kind", "arity", "state", "matrix"], &[])?;
                    let n = power_dim(Some(2), self.size(rep.get("arity")?)?)?;
                    self.square(rep.get("state")?, n)?;
                    self.square(rep.get("matrix")?, mul(dimension, n)?)?;
                }
                _ => return err("§8.3", "unknown representation kind"),
            }
        }
        if let Some(certs) = v.object()?.get("certificates") {
            unsupported = true;
            certs.fields(&[], &["unitarity", "enclosure"])?;
            if let Some(cert) = certs.object()?.get("unitarity") {
                if cert.get("version")?.string()? == "bennett.unitarity/1" {
                    cert.fields(&["version", "gram"], &[])?;
                    self.square(cert.get("gram")?, dimension)?;
                }
            }
            if let Some(cert) = certs.object()?.get("enclosure") {
                if cert.get("version")?.string()? == "bennett.enclosure/1" {
                    cert.fields(&["version", "precision_bits", "matrix"], &[])?;
                    if cert.get("precision_bits")?.uint()? == 0 {
                        return err("§8.3", "zero enclosure precision");
                    }
                    let rows = cert.get("matrix")?.array()?;
                    if dimension.is_some_and(|d| rows.len() as u128 != d) || rows.is_empty() {
                        return err("§8.3", "enclosure shape mismatch");
                    }
                    for row in rows {
                        if row.array()?.len() != rows.len() {
                            return err("§8.3", "enclosure shape mismatch");
                        }
                        for entry in row.array()? {
                            let values = entry.array()?;
                            if values.len() != 4 {
                                return err("§8.3", "enclosure requires four bounds");
                            }
                            let bounds = values
                                .iter()
                                .map(|v| {
                                    let p = v.array()?;
                                    if p.len() != 2 {
                                        return err("§8.3", "invalid rational bound");
                                    }
                                    Rational::leaf(p[0].string()?, p[1].string()?)
                                })
                                .collect::<Result<Vec<_>>>()?;
                            if bounds[0] > bounds[1] || bounds[2] > bounds[3] {
                                return err("§8.3", "reversed enclosure");
                            }
                        }
                    }
                }
            }
        }
        Ok(unsupported)
    }
    fn square(&self, v: &Json, n: Dim) -> Result<()> {
        let shape = self.matrix(v)?;
        equal(shape.0, shape.1)?;
        equal(shape.0, n)?;
        equal(shape.1, n)?;
        Ok(())
    }
    fn operators(&self, v: &Json, n: Dim) -> Result<()> {
        let operators = v.array()?;
        if operators.is_empty() {
            return err("§8.3", "empty operator list");
        }
        for operator in operators {
            self.square(operator, n)?;
        }
        Ok(())
    }
}
pub(super) fn label_check(label: &str) -> Result<()> {
    if label.is_empty() || !label.is_ascii() {
        return err("§8.3", "expected nonempty ASCII label");
    }
    Ok(())
}
fn equal(a: Dim, b: Dim) -> Result<Dim> {
    if matches!((a,b),(Some(a),Some(b)) if a!=b) {
        return err("§8.3", "incompatible matrix shapes");
    }
    Ok(a.or(b))
}
fn add(a: Dim, b: Dim) -> Result<Dim> {
    match (a, b) {
        (Some(a), Some(b)) => a
            .checked_add(b)
            .map(Some)
            .ok_or_else(|| crate::resource_error("§8.3", "matrix shape exceeds host range")),
        _ => Ok(None),
    }
}
fn mul(a: Dim, b: Dim) -> Result<Dim> {
    match (a, b) {
        (Some(a), Some(b)) => a
            .checked_mul(b)
            .map(Some)
            .ok_or_else(|| crate::resource_error("§8.3", "matrix shape exceeds host range")),
        _ => Ok(None),
    }
}
fn power_dim(a: Dim, b: Dim) -> Result<Dim> {
    match (a, b) {
        (Some(a), Some(b)) => Ok(Some(power(a, b)?)),
        (_, Some(0)) => Ok(Some(1)),
        _ => Ok(None),
    }
}
pub(super) fn power(a: u128, b: u128) -> Result<u128> {
    u32::try_from(b)
        .ok()
        .and_then(|b| a.checked_pow(b))
        .ok_or_else(|| crate::resource_error("§8.3", "matrix shape exceeds host range"))
}
