use crate::{
    err,
    object::list,
    validation::{gate, gate_operands, normalize_call},
    Error, Function, Instruction, Object, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};
/// Complete §5.5 invocation identity; lists retain their operand boundaries.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
struct Invocation {
    callee: String,
    token: String,
    ins: Vec<String>,
    target: String,
    fault: String,
    mem: Vec<String>,
    args: Vec<String>,
}
impl Invocation {
    fn from_row(row: &Instruction) -> Result<Self> {
        let o = &row.operands;
        Ok(Self {
            callee: o[0].clone(),
            token: o[1].clone(),
            ins: list(&o[2])?,
            target: o[3].clone(),
            fault: o[4].clone(),
            mem: list(&o[5])?,
            args: list(&o[6])?,
        })
    }
}
impl Object {
    pub(crate) fn interface_targets(
        &self,
        name: &str,
        active: &mut BTreeSet<String>,
    ) -> Result<BTreeSet<(String, u64)>> {
        if !active.insert(name.into()) {
            return err("§5.1", "recursive region graph");
        }
        let f = self.declaration(name)?;
        let mut out = BTreeSet::new();
        let add = |out: &mut BTreeSet<(String, u64)>, operand: &str| -> Result<()> {
            let (base, index) = if let Some((base, index)) = operand.split_once('[') {
                (
                    base,
                    Some(crate::object::host_count(
                        index.trim_end_matches(']'),
                        "wire index",
                    )?),
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
                                for (group, actual) in actuals.iter().enumerate() {
                                    if let Some(position) = declaration.params[group]
                                        .iter()
                                        .position(|p| p.name == param)
                                    {
                                        add(&mut out, &format!("{}[{index}]", actual[position]))?;
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
            for (group, actual) in actuals.iter().enumerate() {
                if let Some(position) = f.params[group].iter().position(|p| p.name == param) {
                    let actual = &actual[position];
                    if !matches!(types.get(actual), Some(Type::Wire(_) | Type::Qreg(_))) {
                        return err(
                            "§3.3",
                            "controlled region operand must be a wire bundle or qreg",
                        );
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
    /// §5.5 matched cleanup: each `lift.oracle_uncall` pairs with one
    /// outstanding `lift.oracle_call` of identical operands (callee, token,
    /// ins, tgt, fr, mem, args) on EVERY incoming path, with no intervening row
    /// writing its ins or possibly writing a supplied cell. XOR deliveries commute
    /// on outputs only; they still invalidate any invocation reading an output.
    /// Non-delivery writes to tgt/fr also invalidate the invocation.
    /// A forward dataflow over the row CFG: the outstanding multiset meets by
    /// pointwise minimum, address constants (from lift.addr) by agreement.
    pub(crate) fn validate_matched_oracle_calls(&self, f: &Function) -> Result<()> {
        if !f.instructions().any(|i| i.op == "lift.oracle_uncall") {
            return Ok(());
        }
        let mut rows = Vec::new();
        let mut labels = BTreeMap::new();
        for block in &f.blocks {
            labels.insert(block.name.clone(), rows.len());
            rows.extend(&block.instructions);
        }
        let label = |s: &str| -> Result<usize> {
            labels
                .get(s.trim_start_matches('^'))
                .copied()
                .ok_or_else(|| Error(format!("§3.6: unknown label {s}")))
        };
        let mut successors = Vec::new();
        for (k, row) in rows.iter().enumerate() {
            successors.push(match row.op.as_str() {
                "qjump" => vec![label(&row.operands[0])?],
                "qbranch" => vec![label(&row.operands[1])?, label(&row.operands[2])?],
                "qjumpr" => (0..rows.len()).collect(),
                "fault" | "lift.ret" | "lift.retval" => Vec::new(),
                _ => vec![k + 1],
            });
        }
        let regions = self.region_ranges(f)?;
        let addresses = self.address_constants(f)?;
        let stores_in_closure = |callee: &str| -> Result<bool> { self.closure_may_store(callee) };
        #[derive(Clone, PartialEq, Eq)]
        struct State {
            outstanding: BTreeMap<Invocation, u64>,
            constants: BTreeMap<String, u128>,
        }
        let meet = |a: &State, b: &State| State {
            outstanding: a
                .outstanding
                .iter()
                .filter_map(|(k, n)| {
                    let m = (*n).min(b.outstanding.get(k).copied().unwrap_or(0));
                    (m > 0).then(|| (k.clone(), m))
                })
                .collect(),
            constants: a
                .constants
                .iter()
                .filter(|(k, v)| b.constants.get(*k) == Some(v))
                .map(|(k, v)| (k.clone(), *v))
                .collect(),
        };
        let transfer = |row: &Instruction, state: &State| -> Result<State> {
            let mut next = state.clone();
            let op = row.op.as_str();

            // Consume the matched inverse before its delivery invalidates readers.
            // Final validation below checks the pre-row outstanding identity.
            if op == "lift.oracle_uncall" {
                let key = Invocation::from_row(row)?;
                if let Some(n) = next.outstanding.get_mut(&key) {
                    *n -= 1;
                    if *n == 0 {
                        next.outstanding.remove(&key);
                    }
                }
            }
            let delivery = matches!(op, "lift.oracle_call" | "lift.oracle_uncall");
            let mut writes: BTreeSet<String> = row.lhs.iter().cloned().collect();
            if delivery {
                writes.extend([row.operands[3].clone(), row.operands[4].clone()]);
            }
            if op == "lift.perm_call" {
                writes.extend(list(&row.operands[1])?);
            }
            // Cell writes: None = no cell written, Some(None) = unknown address.
            let mut cells: Vec<Option<(u128, u128)>> = Vec::new();
            if let Some(w) = op.strip_prefix("lift.store.i") {
                let n = crate::object::host_count(w, "store width")? as u128 / 8;
                cells.push(state.constants.get(&row.operands[0]).map(|c| (*c, n)));
            }
            if op == "lift.call" && stores_in_closure(row.operands[0].trim_start_matches('@'))? {
                cells.push(None);
            }
            let mut mut_regions = BTreeSet::new();
            if op == "lift.perm_call" {
                let callee = self.declaration(row.operands[0].trim_start_matches('@'))?;
                for (p, n) in callee.params[2].iter().zip(list(&row.operands[3])?) {
                    if p.role.as_deref() == Some("mut") {
                        mut_regions.insert(n);
                    }
                }
            }
            next.outstanding.retain(|k, _| {
                let touches = k.ins.iter().any(|w| writes.contains(w))
                    || (!delivery && [&k.target, &k.fault].iter().any(|w| writes.contains(*w)));
                let clobbers = k.mem.iter().any(|r| {
                    mut_regions.contains(r)
                        || cells.iter().any(|c| match (c, regions.get(r)) {
                            (None, _) | (_, None) => true,
                            (Some((p, n)), Some((base, size))) => {
                                // [p, p+n) meets [base, base+size), computed without wrap.
                                *p < base.saturating_add(*size) && *base < p.saturating_add(*n)
                            }
                        })
                });
                !(touches || clobbers)
            });
            for w in &writes {
                next.constants.remove(w);
            }
            match op {
                "lift.oracle_call" => {
                    *next
                        .outstanding
                        .entry(Invocation::from_row(row)?)
                        .or_default() += 1
                }
                "lift.addr" => {
                    if let Some(base) = addresses.get(row.operands[0].trim_start_matches('@')) {
                        next.constants.insert(row.lhs[0].clone(), *base);
                    }
                }
                _ => {}
            }
            Ok(next)
        };
        let mut entry: Vec<Option<State>> = vec![None; rows.len()];
        entry[0] = Some(State {
            outstanding: BTreeMap::new(),
            constants: BTreeMap::new(),
        });
        let mut work = vec![0usize];
        while let Some(k) = work.pop() {
            let state = entry[k].clone().expect("queued rows have a state");
            let out = transfer(rows[k], &state)?;
            for &s in &successors[k] {
                let merged = match &entry[s] {
                    None => out.clone(),
                    Some(prior) => meet(prior, &out),
                };
                if entry[s].as_ref() != Some(&merged) {
                    entry[s] = Some(merged);
                    work.push(s);
                }
            }
        }
        for (k, row) in rows.iter().enumerate() {
            if row.op != "lift.oracle_uncall" {
                continue;
            }
            let Some(state) = &entry[k] else { continue };
            if !state.outstanding.contains_key(&Invocation::from_row(row)?) {
                return err(
                    "§5.5",
                    format!(
                        "lift.oracle_uncall at line {} has no outstanding matching lift.oracle_call on every path with unchanged ins, tgt, fr and supplied cells",
                        row.line
                    ),
                );
            }
        }
        Ok(())
    }
    /// Base and size of each mapped interface region of an oracle (§7.2).
    fn region_ranges(&self, f: &Function) -> Result<BTreeMap<String, (u128, u128)>> {
        let mut out = BTreeMap::new();
        if f.kind != "oracle" || f.params[3].is_empty() {
            return Ok(out);
        }
        let Some(layout) = self.section("layout") else {
            return Ok(out);
        };
        let mapping = layout.get("interface_regions")?.get(&f.name)?;
        for p in &f.params[3] {
            let Type::Region(a, _) = p.ty else {
                return err("§5.1", "mem slot requires region type");
            };
            let base = mapping.get(&p.name)?.get("base")?.uint()?;
            let size = crate::oracle::power_of_two(a as u128).unwrap_or(u128::MAX);
            out.insert(p.name.clone(), (base, size));
        }
        Ok(out)
    }
    /// lift.addr results known from the layout: globals and this oracle's regions.
    fn address_constants(&self, f: &Function) -> Result<BTreeMap<String, u128>> {
        let mut out: BTreeMap<String, u128> = self
            .region_ranges(f)?
            .into_iter()
            .map(|(n, (base, _))| (n, base))
            .collect();
        if let Some(layout) = self.section("layout") {
            for (n, g) in layout.get("globals")?.object()? {
                out.insert(n.clone(), g.get("base")?.uint()?);
            }
        }
        Ok(out)
    }
    /// Whether an lproc's transitive closure contains a lift.store.
    fn closure_may_store(&self, root: &str) -> Result<bool> {
        let mut pending = vec![root.to_string()];
        let mut seen = BTreeSet::new();
        while let Some(n) = pending.pop() {
            if !seen.insert(n.clone()) {
                continue;
            }
            let Some(f) = self.functions.get(&n) else {
                return Ok(true); // unresolved import: may store
            };
            for i in f.instructions() {
                if i.op.starts_with("lift.store.") {
                    return Ok(true);
                }
                if i.op == "lift.call" {
                    pending.push(i.operands[0].trim_start_matches('@').into());
                }
            }
        }
        Ok(false)
    }
}
