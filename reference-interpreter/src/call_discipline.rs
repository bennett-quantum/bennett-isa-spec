use crate::{
    err,
    object::list,
    validation::{gate, gate_operands, normalize_call},
    Error, Function, Instruction, Object, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};
impl Object {
    fn interface_targets(
        &self,
        name: &str,
        active: &mut BTreeSet<String>,
    ) -> Result<BTreeSet<(String, u16)>> {
        if !active.insert(name.into()) {
            return err("§5.1", "recursive region graph");
        }
        let f = self.declaration(name)?;
        let mut out = BTreeSet::new();
        let add = |out: &mut BTreeSet<(String, u16)>, operand: &str| -> Result<()> {
            let (base, index) = if let Some((base, index)) = operand.split_once('[') {
                (
                    base,
                    Some(
                        index
                            .trim_end_matches(']')
                            .parse::<u16>()
                            .map_err(|_| Error("§3.3: wire index".into()))?,
                    ),
                )
            } else {
                (operand, None)
            };
            if let Some(p) = f
                .params
                .iter()
                .flatten()
                .find(|p| p.name == base && matches!(p.ty, Type::Wire(_)))
            {
                if let Some(index) = index {
                    out.insert((base.into(), index));
                } else {
                    for bit in 0..p.ty.width(self.ptr_width)? {
                        out.insert((base.into(), bit));
                    }
                }
            }
            Ok(())
        };
        fn flattened<'a>(seq: &'a [Instruction], out: &mut Vec<&'a Instruction>) {
            for i in seq {
                out.push(i);
                for body in &i.bodies {
                    flattened(body, out);
                }
            }
        }
        let mut instructions = Vec::new();
        for b in &f.blocks {
            flattened(&b.instructions, &mut instructions);
        }
        for i in instructions {
            if gate(&i.op) {
                for target in gate_operands(i)?.1 {
                    add(&mut out, &target)?;
                }
            } else {
                match i.op.as_str() {
                    "qxor" => add(&mut out, &i.operands[1])?,
                    "qexch" => {
                        for target in &i.operands {
                            add(&mut out, target)?;
                        }
                    }
                    "qinplace" => add(&mut out, &i.operands[3])?,
                    "qexchm" => add(&mut out, &i.operands[2])?,
                    "oracle_call" | "action_call" | "adj" | "ctl" => {
                        let call = normalize_call(i)?;
                        let a = &call.instruction.operands;
                        if call.instruction.op == "oracle_call" {
                            add(&mut out, &a[3])?;
                            add(&mut out, &a[4])?;
                        } else {
                            let callee = a[0].trim_start_matches('@');
                            let declaration = self.declaration(callee)?;
                            let actuals = [list(&a[1])?, list(&a[2])?];
                            let targets = self.interface_targets(callee, active)?;
                            for (param, index) in targets {
                                for (group,actual) in actuals.iter().enumerate() {
                                    if let Some(position) = declaration.params[group]
                                        .iter()
                                        .position(|p| p.name == param)
                                    {
                                        add(
                                            &mut out,
                                            &format!("{}[{index}]", actual[position]),
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        active.remove(name);
        Ok(out)
    }
    pub(crate) fn validate_inherited_targets(
        &self,
        call: &Instruction,
        context: &[String],
        types: &BTreeMap<String, Type>,
    ) -> Result<()> {
        if context.is_empty() || call.op != "action_call" {
            return Ok(());
        }
        let a = &call.operands;
        let name = a[0].trim_start_matches('@');
        let f = self.declaration(name)?;
        let actuals = [list(&a[1])?, list(&a[2])?];
        let targets = self.interface_targets(name, &mut BTreeSet::new())?;
        for (param, index) in targets {
            for (group,actual) in actuals.iter().enumerate() {
                if let Some(position) = f.params[group].iter().position(|p| p.name == param) {
                    let actual = &actual[position];
                    if !matches!(types.get(actual), Some(Type::Wire(_))) {
                        return err("§3.3", "controlled region operand must be a wire bundle");
                    }
                    if context.contains(&format!("{actual}[{index}]")) {
                        return err(
                            "§3.3",
                            format!("inherited control targets callee wire {param}[{index}]"),
                        );
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn validate_matched_oracle_calls(&self, f: &Function) -> Result<()> {
        let mut rows = Vec::new();
        let mut labels = BTreeMap::new();
        for block in &f.blocks {
            labels.insert(block.name.clone(), rows.len());
            rows.extend(&block.instructions);
        }
        let mut predecessors = vec![Vec::new(); rows.len()];
        for (k, row) in rows.iter().enumerate() {
            let destinations: Vec<usize> = match row.op.as_str() {
                "qjump" => vec![labels[row.operands[0].trim_start_matches('^')]],
                "qbranch" => row.operands[1..]
                    .iter()
                    .map(|s| labels[s.trim_start_matches('^')])
                    .collect(),
                "qjumpr" => (0..rows.len()).collect(),
                "fault" | "lift.ret" | "lift.retval" => Vec::new(),
                _ => {
                    if k + 1 < rows.len() {
                        vec![k + 1]
                    } else {
                        Vec::new()
                    }
                }
            };
            for destination in destinations {
                predecessors[destination].push(k);
            }
        }
        for (k, uncall) in rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.op == "lift.oracle_uncall")
        {
            let ins = list(&uncall.operands[2])?;
            let memory = list(&uncall.operands[5])?;
            let mut work = predecessors[k].clone();
            let mut seen = BTreeSet::new();
            let mut found = false;
            while let Some(point) = work.pop() {
                if !seen.insert(point) {
                    continue;
                }
                let row = rows[point];
                if row.op == "lift.oracle_call" && row.operands == uncall.operands {
                    found = true;
                    continue;
                }
                if row.op == "lift.oracle_uncall" && row.operands == uncall.operands {
                    return err(
                        "§5.5",
                        "oracle_uncall has already consumed its matching invocation",
                    );
                }
                let mut writes = row.lhs.clone();
                if matches!(row.op.as_str(), "lift.oracle_call" | "lift.oracle_uncall") {
                    writes.extend([row.operands[3].clone(), row.operands[4].clone()]);
                }
                if row.op == "lift.perm_call" {
                    writes.extend(list(&row.operands[1])?);
                }
                if writes.iter().any(|w| ins.contains(w)) {
                    return err(
                        "§5.5",
                        "oracle_uncall input was written after the computing call",
                    );
                }
                if !memory.is_empty()
                    && (row.op.starts_with("lift.store.") || row.op == "lift.call")
                {
                    return err("§5.5", "oracle_uncall memory version may have advanced");
                }
                if point == 0 || predecessors[point].is_empty() {
                    return err(
                        "§5.5",
                        "oracle_uncall lacks a matching invocation on an incoming path",
                    );
                }
                work.extend(&predecessors[point]);
            }
            if !found {
                return err("§5.5", "oracle_uncall lacks a matching invocation");
            }
        }
        Ok(())
    }
}
