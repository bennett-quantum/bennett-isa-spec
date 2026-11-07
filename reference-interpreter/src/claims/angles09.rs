//! K94 source-angle admissibility through positional classical forwarding.
use super::*;
use crate::{object::list, Instruction};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Angle {
    Bits(u64),
    /// A root parameter, quantified over source-admissible invocations. It
    /// imposes finiteness only at a rotation use, not on ordinary f64 data.
    Parameter(usize),
}
type Bindings = BTreeMap<String, Angle>;

enum Operation {
    Rotation,
    RegionCall,
    LiftCall,
    LiftOracle,
    LiftAction,
    Pair,
    Other,
}
impl Operation {
    fn parse(op: &str) -> Self {
        match op {
            "rx" | "ry" | "rz" => Self::Rotation,
            "action_call" | "oracle_call" | "adj" | "ctl" => Self::RegionCall,
            "lift.call" => Self::LiftCall,
            "lift.oracle_call" | "lift.oracle_uncall" => Self::LiftOracle,
            "lift.perm_call" => Self::LiftAction,
            "qinplace" => Self::Pair,
            _ => Self::Other, // Canonical validation already checked the opcode.
        }
    }
}

fn parameters(f: &Function) -> &[crate::Param] {
    &f.params[if f.kind == "oracle" { 4 } else { 3 }]
}
fn root_bindings(f: &Function, invocation: &Json) -> Result<Bindings> {
    let concrete = match invocation.get("args")? {
        Json::Array(values) => Some(values),
        Json::String(scope) if scope == "all" => None,
        _ => return err("§12.1 adjoint_table", "invalid angle invocation scope"),
    };
    let mut bindings = Bindings::new();
    for (index, p) in parameters(f).iter().enumerate() {
        if p.ty != Type::F64 {
            continue;
        }
        let angle = match concrete {
            None => Angle::Parameter(index),
            Some(values) => {
                let text = values
                    .get(index)
                    .ok_or_else(|| {
                        Error::new(format!(
                            "§12.1 adjoint_table: missing angle binding {}",
                            p.name
                        ))
                    })?
                    .string()?;
                let bits = text.strip_prefix("0x").ok_or_else(|| {
                    Error::new("§12.1 adjoint_table: angle binding needs f64 bits".into())
                })?;
                Angle::Bits(u64::from_str_radix(bits, 16).map_err(|_| {
                    Error::new("§12.1 adjoint_table: invalid angle f64 bits".into())
                })?)
            }
        };
        bindings.insert(p.name.clone(), angle);
    }
    Ok(bindings)
}
fn forwarded(callee: &Function, actuals: &[String], caller: &Bindings) -> Result<Bindings> {
    if actuals.len() != parameters(callee).len() {
        return err(
            "§12.1 adjoint_table",
            "angle forwarding argument count mismatch",
        );
    }
    parameters(callee)
        .iter()
        .zip(actuals)
        .filter(|(p, _)| p.ty == Type::F64)
        .map(|(p, actual)| {
            Ok((
                p.name.clone(),
                caller.get(actual).cloned().ok_or_else(|| {
                    Error::new(format!(
                        "§12.1 adjoint_table: missing forwarded f64 {actual}"
                    ))
                })?,
            ))
        })
        .collect()
}
fn rotation(f: &Function, i: &Instruction, bindings: &Bindings) -> Result<()> {
    let operand = &i.operands[0];
    let value = match operand.parse::<f64>() {
        Ok(literal) => Angle::Bits(literal.to_bits()),
        Err(_) => bindings.get(operand).cloned().ok_or_else(|| {
            Error::new(format!(
                "§12.1 adjoint_table: missing rotation argument {operand} in @{}",
                f.name
            ))
        })?,
    };
    match value {
        Angle::Bits(bits) if !f64::from_bits(bits).is_finite() => err(
            "§12.1 adjoint_table / §4.4",
            format!("rotation angle {operand} in @{} must be finite", f.name),
        ),
        Angle::Bits(_) | Angle::Parameter(_) => Ok(()),
    }
}

impl Object {
    pub(super) fn validate_adjoint_angles09(&self, f: &Function, invocation: &Json) -> Result<()> {
        let mut work = vec![(f.name.clone(), root_bindings(f, invocation)?)];
        let mut visited = BTreeSet::new();
        while let Some((name, bindings)) = work.pop() {
            if !visited.insert((name.clone(), bindings.clone())) {
                continue;
            }
            let f = self.functions.get(&name).ok_or_else(|| {
                Error::new(format!(
                    "§12.1 adjoint_table: missing checked definition @{name}"
                ))
            })?;
            let mut sequences = f
                .blocks
                .iter()
                .map(|b| b.instructions.as_slice())
                .collect::<Vec<_>>();
            while let Some(sequence) = sequences.pop() {
                for i in sequence {
                    sequences.extend(i.bodies.iter().map(Vec::as_slice));
                    let a = &i.operands;
                    let calls = match Operation::parse(&i.op) {
                        Operation::Rotation => {
                            rotation(f, i, &bindings)?;
                            Vec::new()
                        }
                        Operation::RegionCall => {
                            let call = crate::validation::normalize_call(i)?;
                            let args = &call.instruction.operands;
                            let index = match call.instruction.op.as_str() {
                                "oracle_call" => 6,
                                "action_call" => 4,
                                _ => {
                                    return err(
                                        "§12.1 adjoint_table",
                                        "invalid normalized angle call",
                                    )
                                }
                            };
                            vec![(args[0].clone(), list(&args[index])?)]
                        }
                        Operation::LiftCall => vec![(
                            a[0].clone(),
                            if a.len() == 2 {
                                Vec::new()
                            } else {
                                list(&a[3])?
                            },
                        )],
                        Operation::LiftOracle => vec![(a[0].clone(), list(&a[6])?)],
                        Operation::LiftAction => vec![(a[0].clone(), list(&a[4])?)],
                        Operation::Pair => {
                            let actuals = if a.len() == 4 {
                                Vec::new()
                            } else {
                                list(&a[6])?
                            };
                            vec![(a[0].clone(), actuals.clone()), (a[1].clone(), actuals)]
                        }
                        Operation::Other => Vec::new(),
                    };
                    for (target, actuals) in calls {
                        let name = target.strip_prefix('@').ok_or_else(|| {
                            Error::new("§12.1 adjoint_table: angle call needs @symbol".into())
                        })?;
                        // Missing linked bodies retain K94's named Unverified
                        // premise from source discovery; never invent their uses.
                        if let Some(callee) = self.functions.get(name) {
                            work.push((name.into(), forwarded(callee, &actuals, &bindings)?));
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
