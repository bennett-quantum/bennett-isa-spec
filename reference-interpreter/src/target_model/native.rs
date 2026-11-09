use super::{
    exact_matrix::{dimension, ExactMatrix},
    ExactComplex, ExpressionBindings, IntervalMatrix, ModelExpression, Rational,
};
use crate::{err, json::Json, Result};

/// Concrete §8.1 bindings, obtained from the matched generated operation.
#[derive(Clone, Debug)]
pub struct ModelContext {
    pub own_controls: usize,
    pub total_controls: usize,
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
        Ok(Some(out.controlled(context.total_controls)?))
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
    let basis_map = validate_basis_and_map(model, opcode, context)?;
    let bindings = model_bindings(model.get("parameter_bindings")?, context, epsilon)?;
    let controls = model.get("control_models")?;
    if controls.get("kind")?.string()? != "controlled_U" {
        return Ok(ModelVerification::Unchecked(
            "§8.3: unsupported explicit control-model representation".into(),
        ));
    }
    controls.fields(&["kind", "controls", "disabled"], &[])?;
    if controls.get("controls")?.string()? != "c"
        || controls.get("disabled")?.string()? != "identity"
    {
        return err(
            "§8.3",
            "incorrect controlled_U context/disabled branch binding",
        );
    }
    let data = model.get("data")?;
    if !data.object()?.contains_key("matrix") {
        return Ok(ModelVerification::Unchecked(
            "§8.3: unsupported native data representation".into(),
        ));
    }
    data.fields(&["matrix"], &[])?;
    let Some(base) = reconstruct(data.get("matrix")?, opcode.arity(), &bindings)? else {
        return Ok(ModelVerification::Unchecked(
            "§8.3: exact nonrational model identity not established".into(),
        ));
    };
    base.unitary()?;
    let actual = base
        .permute_basis(&basis_map)?
        .controlled(context.total_controls)?;
    let Some(ideal) = opcode.ideal(context)? else {
        return Ok(ModelVerification::Unchecked(
            "§8.3: exact ideal nonrational identity not established".into(),
        ));
    };
    actual.accuracy(&ideal, epsilon)?;
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
        Some(ideal) => ModelVerification::Checked(ideal.enclose(precision)?),
        None => ModelVerification::Unchecked(
            "§8.3: exact ideal nonrational identity not established".into(),
        ),
    })
}

fn validate_basis_and_map(
    model: &Json,
    opcode: Opcode,
    context: &ModelContext,
) -> Result<Vec<usize>> {
    let arity = opcode.arity();
    if model.get("arity")?.uint()? != arity as u128 {
        return err("§8.3", "model arity disagrees with opcode");
    }
    let expected = if arity == 1 {
        vec!["t"]
    } else {
        vec!["t", "t2"]
    };
    let basis = model
        .get("basis")?
        .array()?
        .iter()
        .map(Json::string)
        .collect::<Result<Vec<_>>>()?;
    if basis.len() != arity
        || expected
            .iter()
            .any(|name| basis.iter().filter(|bit| *bit == name).count() != 1)
    {
        return err("§8.3", "model basis disagrees with logical operands");
    }
    let total = context
        .total_controls
        .checked_add(arity)
        .ok_or_else(|| crate::resource_error("§8.3", "operand count exceeds host range"))?;
    let inherited = context.total_controls - context.own_controls;
    let map = model.get("operand_map")?.array()?;
    if map.len() != total {
        return err(
            "§8.3",
            "operand_map length disagrees with resolved operands",
        );
    }
    for (entry, expected) in map.iter().zip(
        (inherited..context.total_controls)
            .chain(0..inherited)
            .chain(basis.iter().map(|bit| {
                context.total_controls
                    + expected
                        .iter()
                        .position(|name| name == bit)
                        .expect("checked basis")
            })),
    ) {
        if entry.uint()? != expected as u128 {
            return err(
                "§8.3",
                "operand_map disagrees with own/inherited allocation order",
            );
        }
    }
    Ok(basis
        .iter()
        .map(|bit| {
            expected
                .iter()
                .position(|name| name == bit)
                .expect("checked basis")
        })
        .collect())
}

fn model_bindings(
    declarations: &Json,
    context: &ModelContext,
    epsilon: &Rational,
) -> Result<ExpressionBindings> {
    let mut bindings = ExpressionBindings::new();
    for (name, source) in declarations.object()? {
        let source = source.string()?;
        if ["k", "c", "eps", "theta"].contains(&name.as_str()) && name != source {
            return err(
                "§8.3",
                "model parameter/context binding must retain its declared meaning",
            );
        }
        let value = match source {
            "k" => Rational::integer(context.own_controls as i128),
            "c" => Rational::integer(context.total_controls as i128),
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
