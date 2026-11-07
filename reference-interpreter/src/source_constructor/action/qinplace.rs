//! K22 adapters capture the ins/mem/args/token context; the pinned qinplace
//! recipe owns instruction allocation/release order in both orientations.
use super::*;
impl Compiler<'_> {
    pub(super) fn qinplace(&mut self, i: &crate::Instruction, env: &Env) -> Result<Vec<Value>> {
        let a = &i.operands;
        let n = env.width(&a[3])?;
        let f = self.object.functions[a[0].trim_start_matches('@')].params[2][0]
            .ty
            .bits(self.object.ptr_width)?;
        let names = [
            format!("$qinplace/x/{}", self.counter),
            format!("$qinplace/t/{}", self.counter),
            format!("$qinplace/f/{}", self.counter),
        ];
        let mut bound = env.clone();
        bound.predicates.clear();
        for (name, width) in names.iter().zip([n, n, f]) {
            bound.wires.insert(name.clone(), (name.clone(), width));
        }
        // The temporary Instruction passes through the public operand-list
        // parser. Give its private x formal a collision-free lexical alias;
        // keep the actual constructor identity in the environment.
        let mut input_name = format!("qinplace_x_{}", self.counter);
        while bound.wires.contains_key(&input_name) {
            input_name.push('_');
        }
        bound
            .wires
            .insert(input_name.clone(), (names[0].clone(), n));
        let mut inputs = vec![input_name];
        if a.len() == 7 {
            inputs.extend(crate::object::list(&a[4])?);
        }
        let mut call = i.clone();
        call.op = "oracle_call".into();
        call.lhs.clear();
        call.operands = vec![
            a[0].clone(),
            a[2].clone(),
            format!("({})", inputs.join(", ")),
            names[1].clone(),
            names[2].clone(),
            a.get(5).cloned().unwrap_or_else(|| "()".into()),
            a.get(6).cloned().unwrap_or_else(|| "()".into()),
        ];
        let formals = names
            .iter()
            .zip([n, n, f])
            .map(|(name, width)| formal(name, width))
            .collect::<Result<Vec<_>>>()?;
        let g = template(formals.clone(), self.call(&call, &bound)?);
        call.operands[0] = a[1].clone();
        let h = template(formals, self.call(&call, &bound)?);
        Ok(vec![invoke(
            "qinplace",
            [
                number(n)?,
                number(f)?,
                env.wire(&a[3])?,
                array([g]),
                array([h]),
                array(env.predicates.clone()),
            ],
        )])
    }
}
