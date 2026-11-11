use super::native_schema::{self, Grammar};
use super::{
    exact_matrix::{dimension, ExactMatrix},
    ExactComplex, ExpressionBindings, IntervalMatrix, ModelExpression, Rational,
};
use crate::{err, json::Json, Result};
use std::collections::BTreeSet;

/// Resolved physical identities, including the independently resolved §12.1
/// allocation order. Own/effective lists retain their logical occurrences.
#[derive(Clone, Debug)]
pub struct ModelOperands {
    pub own_controls: Vec<(String, usize)>,
    pub effective_controls: Vec<(String, usize)>,
    pub targets: Vec<(String, usize)>,
    pub allocation_order: Vec<(String, usize)>,
}
impl ModelOperands {
    fn controls(&self) -> Vec<(String, usize)> {
        self.own_controls
            .iter()
            .chain(&self.effective_controls)
            .cloned()
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    fn validate(&self, own: usize, total: usize, arity: usize) -> Result<()> {
        if self.own_controls.len() != own
            || self.effective_controls.len().checked_add(own) != Some(total)
            || self.targets.len() != arity
        {
            return err(
                "§8.3",
                "physical operands disagree with matching control/target counts",
            );
        }
        let controls = self.controls().into_iter().collect::<BTreeSet<_>>();
        let targets = self.targets.iter().cloned().collect::<BTreeSet<_>>();
        let all = controls.union(&targets).cloned().collect::<BTreeSet<_>>();
        let allocation = self
            .allocation_order
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        if targets.len() != arity
            || !controls.is_disjoint(&targets)
            || all != allocation
            || allocation.len() != self.allocation_order.len()
        {
            return err(
                "§8.3",
                "physical operands alias or allocation order is not a permutation",
            );
        }
        let inherited = self
            .effective_controls
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        if self.allocation_order[..inherited.len()]
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>()
            != inherited
        {
            return err(
                "§8.3",
                "allocation order must place inherited controls first",
            );
        }
        Ok(())
    }
}

/// Concrete §8.1 bindings, obtained from the matched generated operation.
#[derive(Clone, Debug)]
pub struct ModelContext {
    pub own_controls: usize,
    pub total_controls: usize,
    pub operands: ModelOperands,
    pub requested_epsilon: Option<Rational>,
    pub theta_bits: Option<u64>,
}

#[derive(Debug)]
pub enum ModelVerification {
    Checked(IntervalMatrix),
    Unchecked(String),
}

#[derive(Clone, Copy)]
enum Opcode {
    X,
    Cx,
    Ccx,
    Mcx,
    H,
    S,
    Sdg,
    T,
    Tdg,
    Cz,
    Swap,
    Rx,
    Ry,
    Rz,
}
impl Opcode {
    fn parse(s: &str) -> Result<Self> {
        Ok(match s {
            "x" => Self::X,
            "cx" => Self::Cx,
            "ccx" => Self::Ccx,
            "mcx" => Self::Mcx,
            "h" => Self::H,
            "s" => Self::S,
            "sdg" => Self::Sdg,
            "t" => Self::T,
            "tdg" => Self::Tdg,
            "cz" => Self::Cz,
            "swap" => Self::Swap,
            "rx" => Self::Rx,
            "ry" => Self::Ry,
            "rz" => Self::Rz,
            _ => return err("§4.4/§8.3", "unknown native opcode"),
        })
    }
    fn arity(self) -> usize {
        if matches!(self, Self::Cz | Self::Swap) {
            2
        } else {
            1
        }
    }
    fn validate(self, context: &ModelContext) -> Result<()> {
        let expected = match self {
            Self::Cx => 1,
            Self::Ccx => 2,
            Self::Mcx => context.own_controls,
            _ => 0,
        };
        if context.total_controls < context.own_controls || expected != context.own_controls {
            return err("§8.1/§8.3", "incorrect own/total control binding");
        }
        context
            .operands
            .validate(context.own_controls, context.total_controls, self.arity())?;
        let parametric = matches!(self, Self::Rx | Self::Ry | Self::Rz);
        if parametric != context.theta_bits.is_some() {
            return err("§8.3", "incorrect angle parameter binding");
        }
        if let Some(bits) = context.theta_bits {
            Rational::binary64(bits)?;
        }
        if context
            .requested_epsilon
            .as_ref()
            .is_some_and(|eps| *eps < Rational::integer(0))
        {
            return err("§8.1", "negative requested epsilon");
        }
        Ok(())
    }
    fn ideal(self, context: &ModelContext) -> Result<Option<ExactMatrix>> {
        let n = dimension(self.arity())?;
        let mut out = ExactMatrix::identity(n)?;
        match self {
            Self::X | Self::Cx | Self::Ccx | Self::Mcx => {
                out = ExactMatrix::zeros(2)?;
                out.entries[1] = ExactComplex::real_value(Rational::integer(1));
                out.entries[2] = ExactComplex::real_value(Rational::integer(1));
            }
            Self::S | Self::Sdg => {
                out.entries[3] = ExactComplex {
                    re: Rational::integer(0),
                    im: Rational::integer(if matches!(self, Self::S) { 1 } else { -1 }),
                }
            }
            Self::Cz => out.entries[15] = ExactComplex::real_value(Rational::integer(-1)),
            Self::Swap => {
                out.entries[5] = ExactComplex::real_value(Rational::integer(0));
                out.entries[10] = ExactComplex::real_value(Rational::integer(0));
                out.entries[6] = ExactComplex::real_value(Rational::integer(1));
                out.entries[9] = ExactComplex::real_value(Rational::integer(1));
            }
            Self::Rx | Self::Ry | Self::Rz
                if Rational::binary64(context.theta_bits.expect("validated angle"))?
                    == Rational::integer(0) => {}
            Self::H | Self::T | Self::Tdg | Self::Rx | Self::Ry | Self::Rz => return Ok(None),
        }
        Ok(Some(out.controlled(context.operands.controls().len())?))
    }
}

/// K86's supplied-model obligations, independently of any numerical/channel
/// claim. Unsupported identities are explicit unchecked results. No positive
/// epsilon context ever falls back to its ideal opcode when the model is absent.
pub fn verify_native_model(
    model: Option<&Json>,
    opcode: &str,
    context: &ModelContext,
    achieved_epsilon: &Rational,
    precision: usize,
) -> Result<ModelVerification> {
    let result = verify(
        model,
        Opcode::parse(opcode)?,
        context,
        achieved_epsilon,
        precision,
    );
    match result {
        Err(e) if e.kind() == crate::ErrorKind::HostResource => {
            Ok(ModelVerification::Unchecked(e.to_string()))
        }
        other => other,
    }
}

fn verify(
    model: Option<&Json>,
    opcode: Opcode,
    context: &ModelContext,
    epsilon: &Rational,
    precision: usize,
) -> Result<ModelVerification> {
    context_checks(opcode, context, epsilon, precision)?;
    let Some(model) = model else {
        if *epsilon != Rational::integer(0) {
            return Ok(ModelVerification::Unchecked(
                "§8.3: missing positive-epsilon native model".into(),
            ));
        }
        return enclose_ideal(opcode, context, precision);
    };
    if model.get("version")?.string()? != "bennett.native-model/1" {
        return Ok(ModelVerification::Unchecked(
            "§8.3: unsupported native-model version".into(),
        ));
    }
    model.fields(
        &[
            "version",
            "kind",
            "arity",
            "basis",
            "parameter_bindings",
            "control_models",
            "data",
            "operand_map",
        ],
        &[],
    )?;
    if model.get("kind")?.string()? != "unitary" {
        return err("§8.3", "native model kind must be unitary");
    }
    let bindings = model_bindings(model.get("parameter_bindings")?, context, epsilon)?;
    let d = context.operands.controls().len();
    let grammar = Grammar {
        resolve_context: true,
        k: context.own_controls,
        c: context.total_controls,
        d,
        bindings: &bindings,
    };
    let controls = model.get("control_models")?;
    let explicit = match controls.get("kind")?.string()? {
        "controlled_U" => {
            controls.fields(&["kind", "controls", "disabled"], &[])?;
            if controls.get("disabled")?.string()? != "identity" {
                return err("§8.3", "incorrect disabled branch binding");
            }
            false
        }
        "explicit" => {
            controls.fields(&["kind", "controls"], &[])?;
            true
        }
        _ => return err("§8.3", "unknown control-model kind"),
    };
    if controls.get("controls")?.string()? != "c" {
        return err("§8.3", "incorrect control context binding");
    }
    let full_map = validate_basis_and_map(model, opcode, context, explicit)?;
    let data = model.get("data")?;
    if explicit {
        data.fields(&["contexts"], &[])?;
        let rows = data.get("contexts")?.array()?;
        if rows.is_empty() {
            return err("§8.3", "empty explicit contexts");
        }
        let mut previous = None;
        let mut selected = 0;
        for row in rows {
            row.fields(&["controls", "data"], &[])?;
            let range = row.get("controls")?;
            range.fields(&["min", "max"], &[])?;
            let min = range.get("min")?.uint()?;
            let max = if range.get("max")? == &Json::String("unbounded".into()) {
                u128::MAX
            } else {
                range.get("max")?.uint()?
            };
            if min > max || previous.is_some_and(|p| p >= min) {
                return err("§8.3", "overlapping or reversed explicit ranges");
            }
            previous = Some(max);
            let reached =
                min <= context.total_controls as u128 && context.total_controls as u128 <= max;
            selected += usize::from(reached);
            let row_grammar = Grammar {
                resolve_context: reached,
                ..grammar
            };
            row_grammar.operation(
                row.get("data")?,
                native_schema::power(2, (d + opcode.arity()) as u128)?,
            )?;
        }
        if selected != 1 {
            return err("§8.3", "missing explicit context selection");
        }
        return Ok(ModelVerification::Unchecked(
            "§8.3: unsupported explicit control-model verification".into(),
        ));
    }
    let unsupported = grammar.operation(data, native_schema::power(2, opcode.arity() as u128)?)?;
    if matches!(data.get("matrix")?, Json::Object(_)) {
        return Ok(ModelVerification::Unchecked(
            "§8.3: unsupported matrix constructor, representation or certificate verification"
                .into(),
        ));
    }
    let Some(base) = reconstruct(data.get("matrix")?, opcode.arity(), &bindings)? else {
        return Ok(ModelVerification::Unchecked(
            "§8.3: exact nonrational model identity not established".into(),
        ));
    };
    base.unitary()?;
    let actual = base.controlled(d)?.permute_basis(&full_map)?;
    let Some(ideal) = opcode.ideal(context)? else {
        return Ok(ModelVerification::Unchecked(
            "§8.3: exact ideal nonrational identity not established".into(),
        ));
    };
    // The ideal starts in D + logical target order, independently of the
    // model's target basis; both are then placed in allocation order.
    let ideal_map = context
        .operands
        .controls()
        .iter()
        .chain(context.operands.targets.iter())
        .map(|wire| {
            context
                .operands
                .allocation_order
                .iter()
                .position(|w| w == wire)
                .expect("validated physical operands")
        })
        .collect::<Vec<_>>();
    let ideal = ideal.permute_basis(&ideal_map)?;
    actual.accuracy(&ideal, epsilon)?;
    if unsupported {
        return Ok(ModelVerification::Unchecked(
            "§8.3: unsupported representation or certificate verification".into(),
        ));
    }
    Ok(ModelVerification::Checked(actual.enclose(precision)?))
}

fn context_checks(
    opcode: Opcode,
    context: &ModelContext,
    epsilon: &Rational,
    precision: usize,
) -> Result<()> {
    opcode.validate(context)?;
    if precision == 0 {
        return err("§12.1", "precision_bits must be positive");
    }
    if *epsilon < Rational::integer(0) {
        return err("§8.1", "negative achieved epsilon");
    }
    if *epsilon > Rational::integer(0)
        && context
            .requested_epsilon
            .as_ref()
            .is_none_or(|eps| epsilon > eps)
    {
        return err("§8.1", "achieved epsilon exceeds or lacks requested share");
    }
    Ok(())
}

fn enclose_ideal(
    opcode: Opcode,
    context: &ModelContext,
    precision: usize,
) -> Result<ModelVerification> {
    Ok(match opcode.ideal(context)? {
        Some(ideal) => {
            let map = context
                .operands
                .controls()
                .iter()
                .chain(&context.operands.targets)
                .map(|w| {
                    context
                        .operands
                        .allocation_order
                        .iter()
                        .position(|a| a == w)
                        .expect("validated operands")
                })
                .collect::<Vec<_>>();
            ModelVerification::Checked(ideal.permute_basis(&map)?.enclose(precision)?)
        }
        None => ModelVerification::Unchecked(
            "§8.3: exact ideal nonrational identity not established".into(),
        ),
    })
}

fn validate_basis_and_map(
    model: &Json,
    opcode: Opcode,
    context: &ModelContext,
    explicit: bool,
) -> Result<Vec<usize>> {
    let d = context.operands.controls();
    let arity = opcode.arity() + if explicit { d.len() } else { 0 };
    if native_schema::size(
        model.get("arity")?,
        context.own_controls,
        context.total_controls,
        d.len(),
    )? != arity as u128
    {
        return err("§8.3", "model arity disagrees with opcode");
    }
    let targets = if opcode.arity() == 1 {
        vec!["t".to_owned()]
    } else {
        vec!["t".to_owned(), "t2".to_owned()]
    };
    let expected = if explicit {
        (0..d.len())
            .map(|i| format!("c[{i}]"))
            .chain(targets.clone())
            .collect::<Vec<_>>()
    } else {
        targets.clone()
    };
    let basis = match model.get("basis")? {
        Json::Array(a) => a
            .iter()
            .map(|v| Ok(v.string()?.to_owned()))
            .collect::<Result<Vec<_>>>()?,
        template if explicit => {
            template.fields(&["controls", "targets"], &[])?;
            if template.get("controls")?.string()? != "c" {
                return err("§8.3", "invalid Basis control binding");
            }
            (0..d.len())
                .map(|i| Ok(format!("c[{i}]")))
                .chain(
                    template
                        .get("targets")?
                        .array()?
                        .iter()
                        .map(|v| Ok(v.string()?.to_owned())),
                )
                .collect::<Result<Vec<_>>>()?
        }
        _ => return err("§8.3", "template Basis requires explicit controls"),
    };
    if basis.len() != arity
        || expected
            .iter()
            .any(|n| basis.iter().filter(|b| *b == n).count() != 1)
    {
        return err("§8.3", "model basis disagrees with logical operands");
    }
    let basis_map = basis
        .iter()
        .map(|b| expected.iter().position(|n| n == b).expect("checked basis"))
        .collect::<Vec<_>>();
    let complete = if explicit {
        basis_map.clone()
    } else {
        (0..d.len())
            .chain(basis_map.iter().map(|i| d.len() + i))
            .collect()
    };
    let physical = d
        .iter()
        .chain(&context.operands.targets)
        .collect::<Vec<_>>();
    let map = complete
        .iter()
        .map(|i| {
            context
                .operands
                .allocation_order
                .iter()
                .position(|w| w == physical[*i])
                .expect("validated operands")
        })
        .collect::<Vec<_>>();
    match model.get("operand_map")? {
        Json::Array(entries) => {
            if entries.len() != map.len() {
                return err(
                    "§8.3",
                    "operand_map length disagrees with distinct physical operands",
                );
            }
            for (v, expected) in entries.iter().zip(&map) {
                if v.uint()? != *expected as u128 {
                    return err(
                        "§8.3",
                        "operand_map disagrees with resolved physical identities",
                    );
                }
            }
        }
        generated => {
            generated.fields(&["kind"], &[])?;
            if generated.get("kind")?.string()? != "entry_to_allocation" {
                return err("§8.3", "unknown operand_map kind");
            }
        }
    }
    Ok(map)
}

fn model_bindings(
    declarations: &Json,
    context: &ModelContext,
    epsilon: &Rational,
) -> Result<ExpressionBindings> {
    let mut bindings = ExpressionBindings::new();
    for (name, source) in declarations.object()? {
        native_schema::label_check(name)?;
        let source = source.string()?;
        if ["k", "c", "d", "eps", "theta"].contains(&name.as_str()) && name != source {
            return err(
                "§8.3",
                "model parameter/context binding must retain its declared meaning",
            );
        }
        let value = match source {
            "k" => Rational::integer(context.own_controls as i128),
            "c" => Rational::integer(context.total_controls as i128),
            "d" => Rational::integer(context.operands.controls().len() as i128),
            "eps" if *epsilon > Rational::integer(0) => {
                context.requested_epsilon.clone().expect("validated share")
            }
            "theta" if context.theta_bits.is_some() => {
                Rational::binary64(context.theta_bits.expect("angle present"))?
            }
            _ => return err("§8.3", format!("unbound model context variable {source}")),
        };
        bindings.insert(name.clone(), value);
    }
    Ok(bindings)
}

fn reconstruct(
    matrix: &Json,
    arity: usize,
    bindings: &ExpressionBindings,
) -> Result<Option<ExactMatrix>> {
    let size = dimension(arity)?;
    let rows = matrix.array()?;
    if rows.len() != size {
        return err("§8.3", "matrix dimension disagrees with resolved basis");
    }
    let declared = bindings.keys().map(String::as_str).collect::<Vec<_>>();
    let mut entries = crate::reserved(size * size, "§8.3", "model entries")?;
    for row in rows {
        let row = row.array()?;
        if row.len() != size {
            return err("§8.3", "matrix dimension disagrees with resolved basis");
        }
        for entry in row {
            let pair = entry.array()?;
            if pair.len() != 2 {
                return err("§8.3", "complex matrix entry requires real/imaginary pair");
            }
            let re = ModelExpression::parse(&pair[0], &declared)?.exact(bindings)?;
            let im = ModelExpression::parse(&pair[1], &declared)?.exact(bindings)?;
            entries.push(match (re, im) {
                (Some(re), Some(im)) => Some(re.add(&ExactComplex {
                    re: im.im.neg(),
                    im: im.re,
                })),
                _ => None,
            });
        }
    }
    Ok(entries
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .map(|entries| ExactMatrix { size, entries }))
}
