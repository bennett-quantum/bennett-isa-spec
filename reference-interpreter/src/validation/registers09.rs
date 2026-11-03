//! Versioned runtime register typing. Width preconditions belong to execution;
//! even an evidently out-of-bounds literal slice must trap without effects.
use super::*;

impl Object {
    pub(super) fn register_instruction09(
        &self,
        f: &Function,
        i: &Instruction,
        types: &BTreeMap<String, Type>,
    ) -> Result<Option<Type>> {
        let a = &i.operands;
        let rd = |name: &str| {
            types
                .get(name)
                .cloned()
                .ok_or_else(|| Error::new(format!("{UNDEFINED} {name}")))
        };
        let arity = |operands, results| {
            if a.len() != operands || i.lhs.len() != results {
                err("§4.8", "register opcode arity mismatch")
            } else {
                Ok(())
            }
        };
        let size = |name: &str, positive: bool| -> Result<Option<u64>> {
            if crate::object::integer(name) {
                let n = crate::object::host_count(name, "qreg width/index")?;
                if positive && n == 0 {
                    return err("§3.3", "literal qreg width must be positive");
                }
                Ok(Some(n))
            } else if rd(name)? == Type::Int(64) {
                Ok(None)
            } else {
                err("§3.3", "runtime qreg width/index requires i64")
            }
        };
        let handle = |name: &str| -> Result<Type> {
            let t = rd(name)?;
            if !t.is_qreg() {
                return err("§3.3", "qreg operand required");
            }
            Ok(t)
        };
        Ok(match i.op.as_str() {
            "qreg.new" => {
                arity(2, 1)?;
                crate::object::literal_name(&a[0])?;
                Some(size(&a[1], true)?.map_or(Type::DynamicQreg, Type::Qreg))
            }
            "qreg.free" => {
                arity(1, 0)?;
                handle(&a[0])?;
                None
            }
            "qreg.slice" => {
                arity(3, 1)?;
                handle(&a[0])?;
                size(&a[1], false)?;
                Some(size(&a[2], true)?.map_or(Type::DynamicQreg, Type::Qreg))
            }
            "qreg.concat" => {
                arity(2, 1)?;
                match (handle(&a[0])?, handle(&a[1])?) {
                    (Type::Qreg(n), Type::Qreg(m)) => {
                        Some(Type::Qreg(n.checked_add(m).ok_or_else(|| {
                            crate::resource_error("§3.3", "qreg width sum exceeds u64")
                        })?))
                    }
                    _ => Some(Type::DynamicQreg),
                }
            }
            "qreg.width" | "qreg.id" => {
                arity(1, 1)?;
                handle(&a[0])?;
                Some(Type::Int(64))
            }
            "qreg.from_id" => {
                arity(1, 1)?;
                if rd(&a[0])? != Type::Int(64) {
                    return err("§3.3", "qreg.from_id requires i64");
                }
                Some(Type::DynamicQreg)
            }
            op if op == "measure" || op.starts_with("measure.") => {
                arity(1, 1)?;
                if f.kind != "cc" {
                    return err("§3.3", "measurement is CC only");
                }
                let ty = handle(&a[0])?;
                let n = if let Some(suffix) = op.strip_prefix("measure.") {
                    let Type::Int(n) = Type::parse(suffix)? else {
                        return err("§3.3", "measure suffix requires iN");
                    };
                    if matches!(ty,Type::Qreg(w) if w!=u64::from(n)) {
                        return err("§3.3", "static measurement width mismatch");
                    }
                    n
                } else if let Type::Qreg(n @ 1..=128) = ty {
                    n as u16
                } else {
                    return err("§3.3", "dynamic qreg measurement requires measure.iN");
                };
                Some(Type::Int(n))
            }
            _ => return err("§4.8", format!("unknown register opcode {}", i.op)),
        })
    }
}
