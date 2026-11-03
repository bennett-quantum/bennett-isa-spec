//! ISA 0.9 runtime register preconditions, checked before ledger/quantum effects.
use super::*;

impl ClassicalMachine<'_> {
    fn register_size09(&self, operand: &str) -> Result<usize> {
        let n = if crate::object::integer(operand) {
            crate::object::host_count(operand, "qreg width/index")? as u128
        } else {
            self.bits(operand)?
        };
        usize::try_from(n)
            .map_err(|_| crate::resource_error("§3.3", "qreg width/index exceeds usize"))
    }
    pub(super) fn register_transfer09(&self, value: &ClassicalValue, ty: &Type) -> Result<()> {
        if !value.matches(ty, self.object.ptr_width) {
            return Err(trap(Error::new("§3.3: transfer value type differs".into())));
        }
        if let ClassicalValue::Qreg(id) = value {
            let width = self.state.quantum.width(*id).map_err(trap)?;
            if matches!(ty,Type::Qreg(n) if *n!=width) {
                return Err(trap(Error::new(
                    "§3.3: dynamic qreg transfer width differs".into(),
                )));
            }
        }
        Ok(())
    }
    pub(super) fn register_step09(
        &mut self,
        instruction: &Instruction,
        undo: &mut Undo,
        output: &mut Option<ClassicalValue>,
        fault: &mut Option<u128>,
        measurement: &mut Option<(u64, Vec<bool>)>,
    ) -> Result<()> {
        let a = &instruction.operands;
        match instruction.op.as_str() {
            "qreg.new" => {
                let class = crate::object::literal_name(&a[0])?;
                let n = self.register_size09(&a[1])?;
                let allocation = self.state.quantum.allocate_class_recorded(n, &class);
                let (id, record) = match allocation {
                    Ok(value) => value,
                    Err(error) if error.kind() == crate::ErrorKind::HostResource => {
                        *fault = Some(3);
                        return Ok(());
                    }
                    Err(error) => return Err(trap(error)),
                };
                undo.registers.push(record);
                *output = Some(ClassicalValue::Qreg(id));
            }
            "qreg.free" => {
                let id = self.qreg(&a[0])?;
                undo.registers
                    .push(self.state.quantum.free_recorded(id).map_err(trap)?);
            }
            "qreg.slice" => {
                let id = self.qreg(&a[0])?;
                let lo = self.register_size09(&a[1])?;
                let len = self.register_size09(&a[2])?;
                let (id, record) = self
                    .state
                    .quantum
                    .slice_recorded(id, lo, len)
                    .map_err(trap)?;
                undo.registers.push(record);
                *output = Some(ClassicalValue::Qreg(id));
            }
            "qreg.concat" => {
                let left = self.qreg(&a[0])?;
                let right = self.qreg(&a[1])?;
                let (id, record) = self
                    .state
                    .quantum
                    .concat_recorded(left, right)
                    .map_err(trap)?;
                undo.registers.push(record);
                *output = Some(ClassicalValue::Qreg(id));
            }
            "qreg.width" | "qreg.id" => {
                let id = self.qreg(&a[0])?;
                let width = self.state.quantum.width(id).map_err(trap)?;
                *output = Some(ClassicalValue::Int(
                    64,
                    if instruction.op == "qreg.id" {
                        id as u128
                    } else {
                        width as u128
                    },
                ));
            }
            "qreg.from_id" => {
                let id = self.bits(&a[0])? as u64;
                let id = self.state.quantum.from_id(id).map_err(trap)?;
                *output = Some(ClassicalValue::Qreg(id));
            }
            op if op == "measure" || op.starts_with("measure.") => {
                let id = self.qreg(&a[0])?;
                let width = self.state.quantum.width(id).map_err(trap)?;
                if let Some(suffix) = op.strip_prefix("measure.") {
                    let Type::Int(expected) = Type::parse(suffix)? else {
                        unreachable!()
                    };
                    if width != u64::from(expected) {
                        return Err(trap(Error::new(
                            "§3.3: runtime measurement width differs".into(),
                        )));
                    }
                }
                // Resolve all preconditions before consuming a queued draw or
                // making the run irreversible.
                let draw = self.measurement_draw();
                let bits = self.state.quantum.measure(id, draw).map_err(trap)?;
                let value = bits
                    .iter()
                    .enumerate()
                    .fold(0, |v, (i, b)| v | ((*b as u128) << i));
                *output = Some(ClassicalValue::Int(bits.len() as u16, value));
                *measurement = Some((id, bits));
            }
            _ => return err("§4.8", "unknown runtime register opcode"),
        }
        Ok(())
    }
}
