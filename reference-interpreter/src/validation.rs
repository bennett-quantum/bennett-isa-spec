//! Operand schemas and static discipline. Dynamic cleanliness is also checked
//! by executors; accepting a text object does not waive those obligations.
use crate::{
    err,
    object::{is_binop, is_div, list, name},
    Error, Function, Instruction, Object, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

const UNDEFINED: &str = "§3.3: undefined classical operand";
impl Object {
    pub(crate) fn validate_structured(
        &self,
        f: &Function,
        types: &mut BTreeMap<String, Type>,
        labels: &BTreeSet<&String>,
    ) -> Result<()> {
        if matches!(f.kind.as_str(), "cc" | "r") {
            return self.validate_classical(f, types, labels);
        }
        let mut state = Discipline::new(f, types.clone());
        fn bounds(seq: &[Instruction], out: &mut Vec<u64>) -> Result<()> {
            for i in seq {
                if i.op == "qloop" {
                    let n = i
                        .operands
                        .first()
                        .ok_or_else(|| Error("§4.8: qloop requires a trip count".into()))?;
                    out.push(crate::object::host_count(n, "qloop trip count")?);
                }
                for body in &i.bodies {
                    bounds(body, out)?;
                }
            }
            Ok(())
        }
        let mut declared_bounds = Vec::new();
        for b in &f.blocks {
            bounds(&b.instructions, &mut declared_bounds)?;
        }
        if declared_bounds != f.bounds {
            return err("§5.3", "declared bounds do not equal loops in text order");
        }
        for b in &f.blocks {
            if !b.params.is_empty() {
                return err("§5.1", "action blocks cannot have classical parameters");
            }
            self.validate_action_sequence(f, &b.instructions, &mut state)?;
        }
        if !state.predicates.is_empty() {
            return err("§3.5", "unclosed predicate stack");
        }
        if !state.created.is_empty() {
            return err(
                "§5.1",
                format!(
                    "unreleased workspace {:?}",
                    state.created.keys().collect::<Vec<_>>()
                ),
            );
        }
        if state.local_regions.iter().any(|r| {
            !state
                .versions
                .get(&format!("{r}[*]"))
                .is_none_or(Vec::is_empty)
        }) {
            return err(
                "§5.1",
                "local region has not been explicitly restored to its image",
            );
        }
        self.validate_borrow_frames(f)?;
        Ok(())
    }
    fn validate_action_sequence(
        &self,
        f: &Function,
        seq: &[Instruction],
        s: &mut Discipline,
    ) -> Result<()> {
        for i in seq {
            let op = i.op.as_str();
            let a = &i.operands;
            let arity = |n: usize, lhs: usize| -> Result<()> {
                if a.len() != n || i.lhs.len() != lhs {
                    err("§4.8", format!("{op}: wrong arity at line {}", i.line))
                } else {
                    Ok(())
                }
            };
            let scalar = |s: &Discipline, n: &str| -> Result<()> {
                if s.wires(n)?.len() != 1 {
                    err("§3.3", format!("{n} must name one wire"))
                } else {
                    Ok(())
                }
            };
            if !matches!(op, "qif" | "qloop") && !i.bodies.is_empty() {
                return err("§4.8", "opcode does not take a structured body");
            }
            match op {
                "qif" => {
                    arity(1, 0)?;
                    scalar(s, &a[0])?;
                    if i.bodies.len() != 2 {
                        return err("§4.8", "qif needs two arms");
                    }
                    let outer = s.predicates.clone();
                    for (arm, body) in i.bodies.iter().enumerate() {
                        let mut expected = outer.clone();
                        expected.push(if arm == 0 { a[0].clone() } else { format!("!{}", a[0]) });
                        s.predicates = expected.clone();
                        self.validate_action_sequence(f, body, s)?;
                        if s.predicates != expected {
                            return err("§3.5", "qif body altered predicate stack");
                        }
                    }
                    s.predicates = outer;
                }
                "qloop" => {
                    if !i.lhs.is_empty() || !matches!(a.len(), 1 | 3) || i.bodies.len() != 1 {
                        return err("§4.8", "qloop arity/body mismatch");
                    }
                    let n = crate::object::host_count(&a[0], "qloop trip count")?;
                    if a.len() == 3 && (a[1] != "while" || s.wires(&a[2])?.len() as u64 != n) {
                        return err("§4.6", "qloop selector width mismatch");
                    }
                    if n == 0 {
                        self.validate_action_sequence(f, &i.bodies[0], &mut s.clone())?;
                        continue;
                    }
                    let initial_predicates = s.predicates.clone();
                    let selectors = if a.len() == 3 {
                        s.wires(&a[2])?
                    } else {
                        Vec::new()
                    };
                    let selector_versions = s.snapshot(&selectors);
                    let selector_closures = s.closures(&selectors);
                    let selector_dependencies = s.dependencies(&selectors);
                    if a.len() == 3 && s.is_other(&a[2])? {
                        return err("§4.6", "while selectors require permutation provenance");
                    }
                    let mut seen: Vec<(u64, Discipline)> = Vec::new();
                    let mut iteration = 0;
                    while iteration < n {
                        if selectors.is_empty() {
                            if let Some((prior, _)) = seen.iter().find(|(_, prior)| prior == s) {
                                let period = iteration - prior;
                                let skip = (n - iteration) / period * period;
                                if skip != 0 {
                                    iteration += skip;
                                    continue;
                                }
                            }
                            seen.push((iteration, s.clone()));
                        } else {
                            s.predicates
                                .extend((0..=iteration).map(|k| format!("{}[{k}]", a[2])));
                        }
                        let prior_modified = std::mem::take(&mut s.modified);
                        self.validate_action_sequence(f, &i.bodies[0], s)?;
                        if !s.modified.is_disjoint(&selector_dependencies) {
                            return err("§4.6", "while body writes selector dependencies");
                        }
                        s.modified.extend(prior_modified);
                        let mut expected = initial_predicates.clone();
                        if !selectors.is_empty() {
                            expected.extend((0..=iteration).map(|k| format!("{}[{k}]", a[2])));
                        }
                        if s.predicates != expected {
                            return err("§3.5", "loop body altered predicate stack");
                        }
                        s.predicates = initial_predicates.clone();
                        if s.snapshot(&selectors) != selector_versions
                            || s.closures(&selectors) != selector_closures
                        {
                            return err("§4.6", "while body modifies selector dependencies");
                        }
                        iteration += 1;
                    }
                }
                "qpred" => {
                    arity(1, 0)?;
                    scalar(s, &a[0])?;
                    s.predicates.push(a[0].clone());
                }
                "qunpred" => {
                    arity(1, 0)?;
                    if s.predicates.pop().as_deref() != Some(&a[0]) {
                        return err("§3.5", "unmatched predicate pop");
                    }
                }
                "qalloc" | "qborrow" => {
                    arity(2, 1)?;
                    crate::object::literal_name(&a[0])?;
                    let n = crate::object::host_count(&a[1], "allocation width")?;
                    if n == 0 {
                        return err("§3.3", "allocation width must be positive");
                    }
                    s.produce(
                        &i.lhs[0],
                        Type::Wire(n),
                        Creation {
                            kind: op.into(),
                            sources: Vec::new(),
                            versions: Vec::new(),
                            controls: Vec::new(),
                            closures: Vec::new(),
                        },
                    )?;
                }
                "qfree" | "qreturn" => {
                    arity(1, 0)?;
                    s.check_target(&a[0], false)?;
                    let c = s
                        .created
                        .get(&a[0])
                        .ok_or_else(|| Error("§3.3: release of non-workspace value".into()))?;
                    if c.kind != if op == "qreturn" { "qborrow" } else { "qalloc" } {
                        return err("§5.1", "allocation/release kind mismatch");
                    }
                    if op == "qfree"
                        && s.wires(&a[0])?
                            .iter()
                            .any(|w| !s.versions.get(w).is_none_or(Vec::is_empty))
                    {
                        return err("§6.2", "released value has not been uncomputed");
                    }
                    s.consume(&a[0])?;
                }
                "qcopy" => {
                    arity(1, 1)?;
                    let wires = s.wires(&a[0])?;
                    s.disjoint(&[a[0].as_str()])?;
                    let versions = s.read_snapshot(&wires)?;
                    let c = Creation {
                        kind: op.into(),
                        sources: vec![a[0].clone()],
                        closures: s
                            .closures(&versions.iter().map(|(w, _)| w.clone()).collect::<Vec<_>>()),
                        versions,
                        controls: s.predicates.clone(),
                    };
                    s.produce(&i.lhs[0], Type::Wire(wires.len() as u64), c)?;
                }
                "quncopy" => {
                    arity(2, 0)?;
                    s.check_target(&a[1], false)?;
                    s.disjoint(&[&a[0], &a[1]])?;
                    let c = s
                        .created
                        .get(&a[1])
                        .ok_or_else(|| Error("§6.2: unmatched quncopy".into()))?;
                    if c.kind != "qcopy"
                        || c.sources != vec![a[0].clone()]
                        || c.versions != s.read_snapshot(&s.wires(&a[0])?)?
                        || c.controls != s.predicates
                        || s.wires(&a[1])?
                            .iter()
                            .any(|w| !s.versions.get(w).is_none_or(Vec::is_empty))
                        || s.window_closed(c)
                    {
                        return err("§6.2", "quncopy outside its computing window");
                    }
                    s.consume(&a[1])?;
                }
                "qregion" => {
                    if !i.lhs.is_empty() || !matches!(a.len(), 3 | 4) || !name(&a[0]) {
                        return err("§4.8", "qregion operand schema");
                    }
                    let aw = crate::object::host_count(&a[1], "qregion address width")?;
                    let w = crate::object::host_count(&a[2], "qregion cell width")?;
                    if w == 0 {
                        return err("§3.3", "qregion cell width must be positive");
                    }
                    if a.len() == 4 {
                        let img = a[3]
                            .strip_prefix('@')
                            .ok_or_else(|| Error("§4.6: qregion image must be @data".into()))?;
                        let size = self.data_size(img)?;
                        // 2^A * W bits = 8 * size bytes (§4.6), compared exactly.
                        let bits = crate::oracle::power_of_two(aw as u128)
                            .and_then(|cells| cells.checked_mul(w as u128));
                        if bits.is_none_or(|bits| bits % 8 != 0 || bits / 8 != size) {
                            return err("§4.6", "image size differs from 2^A*W/8 bytes");
                        }
                    }
                    if s.types.insert(a[0].clone(), Type::Region(aw, w)).is_some() {
                        return err("§3.3", "region name already defined");
                    }
                    s.local_regions.insert(a[0].clone());
                }
                "qload" | "qunload" | "qstore" | "qexchm" => {
                    let load = op == "qload";
                    arity(if load { 2 } else { 3 }, if load { 1 } else { 0 })?;
                    let (aw, w) = match s.types.get(&a[0]) {
                        Some(Type::Region(aw, w)) => (*aw, *w),
                        _ => return err("§3.3", "unknown region"),
                    };
                    if s.wires(&a[1])?.len() != aw as usize {
                        return err("§3.3", "memory address width mismatch");
                    }
                    if load {
                        let mut deps = s.wires(&a[1])?;
                        deps.push(format!("{}[*]", a[0]));
                        s.produce(&i.lhs[0], Type::Wire(w), {
                            let versions = s.read_snapshot(&deps)?;
                            Creation {
                                kind: op.into(),
                                sources: a.clone(),
                                closures: s.closures(
                                    &versions.iter().map(|(w, _)| w.clone()).collect::<Vec<_>>(),
                                ),
                                versions,
                                controls: s.predicates.clone(),
                            }
                        })?;
                    } else {
                        if s.wires(&a[2])?.len() != w as usize {
                            return err("§3.3", "memory value width mismatch");
                        }
                        s.disjoint(&[&a[1], &a[2]])?;
                        if op == "qunload" {
                            s.check_target(&a[2], false)?;
                            let c = s
                                .created
                                .get(&a[2])
                                .ok_or_else(|| Error("§6.2: unmatched qunload".into()))?;
                            let mut deps = s.wires(&a[1])?;
                            deps.push(format!("{}[*]", a[0]));
                            if c.kind != "qload"
                                || c.sources != a[..2]
                                || c.controls != s.predicates
                                || c.versions != s.read_snapshot(&deps)?
                                || s.wires(&a[2])?
                                    .iter()
                                    .any(|w| !s.versions.get(w).is_none_or(Vec::is_empty))
                                || s.window_closed(c)
                            {
                                return err("§6.2", "qunload outside computing window");
                            }
                            s.consume(&a[2])?;
                        } else {
                            if s.inputs.contains(&a[0]) {
                                return err("§5.1", "in region cannot be modified by action");
                            }
                            let mut reads = s.wires(&a[1])?;
                            if op == "qstore" {
                                reads.extend(s.wires(&a[2])?);
                            }
                            let version =
                                format!("{op}:{}:{:?}", a.join(" "), s.read_snapshot(&reads)?);
                            s.note_reads(&version, &reads)?;
                            s.update_log(&format!("{}[*]", a[0]), &version, false);
                            *s.cell_epochs.entry(format!("{}[*]", a[0])).or_default() += 1;
                            if op == "qexchm" {
                                s.change(&a[2], version, false)?;
                            }
                        }
                    }
                }
                "qinplace" => {
                    arity(4, 0)?;
                    // §4.5 expansion: t = qalloc any n; f = qalloc any F;
                    // oracle_call @g tk (x) t f; qexch x t; oracle_call @ginv tk (x) t f.
                    // Both are oracles in (n) -> tgt n with no mem or args and one F.
                    let n = s.wires(&a[3])?.len() as u64;
                    let mut fault = None;
                    for callee in &a[..2] {
                        let name = callee
                            .strip_prefix('@')
                            .ok_or_else(|| Error("§4.5: qinplace names @g @ginv".into()))?;
                        let callee = self.declaration(name)?;
                        if callee.kind != "oracle" {
                            return err("§4.5", "qinplace requires an oracle pair");
                        }
                        if callee.params[0].len() != 1
                            || callee.params[0][0].ty != Type::Wire(n)
                            || callee.params[1][0].ty != Type::Wire(n)
                            || !callee.params[3].is_empty()
                            || !callee.params[4].is_empty()
                        {
                            return err(
                                "§4.5",
                                "qinplace oracles map one wire[n] input to a wire[n] target, with no mem or args",
                            );
                        }
                        let f = callee.params[2][0].ty.clone();
                        if fault.get_or_insert(f.clone()) != &f {
                            return err(
                                "§4.5",
                                "@g and @ginv share one fault bundle f: equal fault widths",
                            );
                        }
                    }
                    if s.types.get(&a[2]) != Some(&Type::Token) {
                        return err("§3.3", "qinplace token operand");
                    }
                    s.change(&a[3], format!("qinplace:{}", a.join(" ")), false)?;
                }
                "oracle_call" | "action_call" | "adj" | "ctl" => {
                    let call = normalize_call(i)?;
                    if let Some(control) = call.control.as_ref() {
                        scalar(s, control)?;
                        s.predicates.push(control.clone());
                    }
                    self.validate_region_call(f, &call.instruction, &s.types)?;
                    let context = s
                        .predicates
                        .iter()
                        .map(|p| s.wires(p))
                        .collect::<Result<Vec<_>>>()?
                        .concat();
                    self.validate_inherited_targets(&call.instruction, &context, &s.types)?;
                    let callee =
                        self.declaration(call.instruction.operands[0].trim_start_matches('@'))?;
                    if f.kind == "perm_action" && callee.kind == "action" {
                        return err("§5.3", "permutation action calls non-permutation action");
                    }
                    let ca = &call.instruction.operands;
                    let targets = if call.instruction.op == "oracle_call" {
                        vec![ca[3].clone(), ca[4].clone()]
                    } else {
                        list(&ca[1])?
                    };
                    let other = callee.kind == "action";
                    let inputs = list(&ca[2])?;
                    let mut reads = inputs
                        .iter()
                        .map(|n| s.wires(n))
                        .collect::<Result<Vec<_>>>()?
                        .concat();
                    let mem_index = if call.instruction.op == "oracle_call" {
                        5
                    } else {
                        3
                    };
                    let mems = list(&ca[mem_index])?;
                    let mem_params = &callee.params[if call.instruction.op == "oracle_call" {
                        3
                    } else {
                        2
                    }];
                    for (mem, param) in mems.iter().zip(mem_params) {
                        if param.role.as_deref() == Some("in") {
                            reads.push(format!("{mem}[*]"));
                        }
                    }
                    let version = if call.instruction.op == "oracle_call" {
                        format!(
                            "oracle_call:{}:{:?}:{:?}",
                            ca.join(" "),
                            s.read_snapshot(&reads)?,
                            s.predicates
                        )
                    } else {
                        format!(
                            "call_{}_{}:{}:{:?}:{:?}",
                            if other { "other" } else { "perm" },
                            if call.adjoint { "adj" } else { "fwd" },
                            ca.join(" "),
                            s.read_snapshot(&reads)?,
                            s.predicates
                        )
                    };
                    s.note_reads(&version, &reads)?;
                    for target in targets {
                        s.change(&target, version.clone(), other)?;
                    }
                    for (mem, param) in mems.iter().zip(mem_params) {
                        if param.role.as_deref() == Some("mut") {
                            let cell = format!("{mem}[*]");
                            s.update_log(&cell, &version, false);
                            s.modified.insert(cell.clone());
                            *s.cell_epochs.entry(cell.clone()).or_default() += 1;
                            if other {
                                *s.closed.entry(cell).or_default() += 1;
                            }
                        }
                    }
                    if call.control.is_some() {
                        s.predicates.pop();
                    }
                }
                _ if gate(op) => {
                    if matches!(op, "rx" | "ry" | "rz")
                        && a.first().is_some_and(|angle| angle.parse::<f64>().is_err())
                        && s.types.get(&a[0]) != Some(&Type::F64)
                    {
                        return err("§3.3", "gate angle must be a classical f64 argument");
                    }
                    let (controls, targets, diagonal, other) = gate_operands(i)?;
                    if f.kind == "perm_action" && (diagonal || other) {
                        return err("§5.3", "permutation action contains non-permutation gate");
                    }
                    let all: Vec<_> = controls
                        .iter()
                        .chain(&targets)
                        .map(String::as_str)
                        .collect();
                    for w in &all {
                        scalar(s, w)?;
                    }
                    s.disjoint(&all)?;
                    let mut normalized = a.clone();
                    if matches!(op, "rx" | "ry" | "rz") {
                        if let Ok(angle) = a[0].parse::<f64>() {
                            normalized[0] = format!("#{:016x}", angle.to_bits());
                        }
                    }
                    let version = format!(
                        "{op}:{}:{:?}:{:?}",
                        normalized.join(" "),
                        // Control and effective-control VERSIONS (fable R1-B3): a
                        // flipped predicate wire changes the operation identity.
                        s.read_snapshot(
                            &controls
                                .iter()
                                .map(|w| s.wires(w))
                                .collect::<Result<Vec<_>>>()?
                                .concat()
                        )?,
                        s.predicates
                    );
                    let reads = controls
                        .iter()
                        .map(|w| s.wires(w))
                        .collect::<Result<Vec<_>>>()?
                        .concat();
                    s.note_reads(&version, &reads)?;
                    if diagonal {
                        // A diagonal gate applies a phase that is a function of its
                        // operands' values: an outstanding phase is an entangling
                        // read of every `other` value those values depend on.
                        // `other` operands record the diagonal gate in their own
                        // version log (record_change below); the phase entry covers
                        // the perm operands, whose values may depend on `other` ones.
                        let mut operands = Vec::new();
                        for w in controls.iter().chain(&targets) {
                            if !s.is_other(w)? {
                                operands.extend(s.wires(w)?);
                            }
                        }
                        let mut phase_reads = operands.clone();
                        for p in &s.predicates {
                            phase_reads.extend(s.wires(p)?);
                        }
                        let key = format!(
                            "{op}:{}:{:?}",
                            normalized.join(" "),
                            s.snapshot(&phase_reads)
                        );
                        if !operands.is_empty() {
                            s.note_reads(&key, &operands)?;
                            s.toggle_phase(key);
                        }
                    }
                    for t in targets {
                        s.check_target(&t, diagonal)?;
                        if !diagonal {
                            s.change(&t, version.clone(), other)?;
                        } else if s.is_other(&t)? {
                            s.record_change(&t, version.clone())?;
                        }
                    }
                }
                "qxor" | "qexch" => {
                    arity(2, 0)?;
                    s.disjoint(&[&a[0], &a[1]])?;
                    if s.wires(&a[0])?.len() != s.wires(&a[1])?.len() {
                        return err("§3.3", "bundle widths differ");
                    }
                    s.check_target(&a[1], false)?;
                    let version = format!(
                        "{op}:{}:{:?}:{:?}",
                        a.join(" "),
                        if op == "qxor" {
                            s.read_snapshot(&s.wires(&a[0])?)?
                        } else {
                            s.read_snapshot(&[])?
                        },
                        s.predicates
                    );
                    let reads = if op == "qxor" {
                        s.wires(&a[0])?
                    } else {
                        Vec::new()
                    };
                    s.note_reads(&version, &reads)?;
                    if s.is_other(&a[0])? || (op == "qexch" && s.is_other(&a[1])?) {
                        s.other_versions.insert(version.clone());
                    }
                    s.change(&a[1], version.clone(), false)?;
                    if op == "qexch" {
                        s.check_target(&a[0], false)?;
                        s.change(&a[0], version, false)?;
                    }
                }
                "measure" | "effect" | "br" | "call" | "uncall" | "ret" | "trap" | "hostcall" => {
                    return err("§5.1", "classical instruction inside Q region")
                }
                _ => return err("§4.8", format!("unknown or disallowed action opcode {op}")),
            }
        }
        Ok(())
    }
    pub(crate) fn validate_region_call(
        &self,
        f: &Function,
        i: &Instruction,
        types: &BTreeMap<String, Type>,
    ) -> Result<()> {
        let a = &i.operands;
        let oracle = matches!(
            i.op.as_str(),
            "oracle_call" | "lift.oracle_call" | "lift.oracle_uncall"
        );
        if !i.lhs.is_empty() || a.len() != if oracle { 7 } else { 5 } {
            return err("§4.8", "region call arity mismatch");
        }
        let n = a[0]
            .strip_prefix('@')
            .ok_or_else(|| Error("§4.8: call requires @symbol".into()))?;
        let callee = self.declaration(n)?;
        if oracle && callee.kind != "oracle"
            || !oracle && !matches!(callee.kind.as_str(), "action" | "perm_action")
        {
            return err("§4.8", "region call kind mismatch");
        }
        if i.op == "lift.perm_call" && callee.kind != "perm_action" {
            return err("§5.2", "lifted bodies require exact permutation callees");
        }
        let mut values = Vec::new();
        if oracle {
            if types.get(&a[1]) != Some(&Type::Token) {
                return err("§3.3", "oracle call token must be classical token");
            }
            values.push((list(&a[2])?, &callee.params[0]));
            values.push((vec![a[3].clone()], &callee.params[1]));
            values.push((vec![a[4].clone()], &callee.params[2]));
            values.push((list(&a[5])?, &callee.params[3]));
            values.push((list(&a[6])?, &callee.params[4]));
        } else {
            for (index, ps) in callee.params[..4].iter().enumerate() {
                values.push((list(&a[index + 1])?, ps));
            }
        }
        let lifted = matches!(f.kind.as_str(), "oracle" | "lproc");
        let mut wires = BTreeSet::new();
        for (vs, ps) in values {
            if vs.len() != ps.len() {
                return err("§3.3", "region call operand count differs from signature");
            }
            for (v, p) in vs.iter().zip(ps) {
                let ty = types
                    .get(v)
                    .ok_or_else(|| Error(format!("§3.3: unknown call operand {v}")))?;
                let compatible = match (&p.ty, ty) {
                    (Type::Wire(n), Type::Wire(m)) | (Type::Wire(n), Type::Reg(m)) => n == m,
                    (Type::Wire(n), Type::Qreg(m)) => n == m,
                    (Type::Region(a, w), Type::Region(b, z)) => a == b && w == z,
                    (Type::Region(a, w), Type::Qreg(n)) => {
                        crate::oracle::power_of_two(*a as u128)
                            .and_then(|cells| cells.checked_mul(*w as u128))
                            == Some(u128::from(*n))
                    }
                    _ => &p.ty == ty,
                };
                if !compatible {
                    return err("§3.3", "region call operand type/width mismatch");
                }
                if matches!(
                    ty,
                    Type::Wire(_) | Type::Reg(_) | Type::Qreg(_) | Type::Region(..)
                ) && !wires.insert(v.clone())
                {
                    return err("§3.3", "region interface operands alias");
                }
                if let Type::Region(..) = ty {
                    let caller = f.params.iter().flatten().find(|p| &p.name == v);
                    if p.role.as_deref() == Some("mut")
                        && caller.is_some_and(|p| p.role.as_deref() == Some("in"))
                    {
                        return err("§3.3", "in region passed to mut slot");
                    }
                }
                if lifted
                    && matches!(p.ty, Type::Wire(_))
                    && !f.locations().iter().any(|l| &l.name == v)
                    && p.role.as_deref() != Some("in")
                {
                    return err("§3.7", "callee outputs require writable caller locations");
                }
            }
        }
        Ok(())
    }
    fn validate_classical(
        &self,
        f: &Function,
        types: &mut BTreeMap<String, Type>,
        labels: &BTreeSet<&String>,
    ) -> Result<()> {
        for b in &f.blocks {
            for p in &b.params {
                if types.insert(p.name.clone(), p.ty.clone()).is_some() {
                    return err("§3.3", "duplicate SSA definition");
                }
            }
        }
        // Definitions are typed independently of block text order (codex B11):
        // iterate until every instruction's operand types are known.
        let mut pending: Vec<(usize, usize)> = f
            .blocks
            .iter()
            .enumerate()
            .flat_map(|(b, block)| (0..block.instructions.len()).map(move |i| (b, i)))
            .collect();
        loop {
            let mut retry = Vec::new();
            let mut first_error = None;
            let before = pending.len();
            for (b, ordinal) in pending {
                let block = &f.blocks[b];
                match self.classical_instruction(f, block, ordinal, types, labels) {
                    Ok(defs) => {
                        for (n, ty) in defs {
                            if types.insert(n, ty).is_some() {
                                return err("§3.3", "SSA definition reused");
                            }
                        }
                    }
                    Err(e) if e.0.starts_with(UNDEFINED) => {
                        first_error.get_or_insert(e);
                        retry.push((b, ordinal));
                    }
                    Err(e) => return Err(e),
                }
            }
            if retry.is_empty() {
                break;
            }
            if retry.len() == before {
                return Err(first_error.expect("retry records its error"));
            }
            pending = retry;
        }
        self.validate_handle_views(f, types)
    }
    /// The §4.1/§4.8 schema of one classical instruction; returns its definitions.
    fn classical_instruction(
        &self,
        f: &Function,
        b: &crate::Block,
        ordinal: usize,
        types: &BTreeMap<String, Type>,
        labels: &BTreeSet<&String>,
    ) -> Result<Vec<(String, Type)>> {
        let i = &b.instructions[ordinal];
        let mut defs = Vec::new();
        let op = i.op.as_str();
        let a = &i.operands;
        let arity = |n: usize, lhs: usize| -> Result<()> {
            if a.len() != n || i.lhs.len() != lhs {
                err("§4.8", format!("{op}: wrong arity"))
            } else {
                Ok(())
            }
        };
        let rd = |n: &str| -> Result<Type> {
            types
                .get(n)
                .cloned()
                .ok_or_else(|| Error(format!("§3.3: undefined classical operand {n}")))
        };
        let same = |x: &str, y: &str| -> Result<Type> {
            let t = rd(x)?;
            if t != rd(y)? {
                return err("§3.3", "classical operand types differ");
            }
            Ok(t)
        };
        if !i.bodies.is_empty() {
            return err(
                "§4.8",
                "classical instruction cannot have Q structured body",
            );
        }
        let term = matches!(op, "br" | "ret" | "trap");
        if term != (ordinal + 1 == b.instructions.len()) {
            return err("§3.4", "classical block requires final br/ret/trap");
        }
        let mut result = None;
        if let Some(t) = op.strip_prefix("const.") {
            arity(1, 1)?;
            let t = Type::parse(t)?;
            let Type::Int(n) = t else {
                return err("§4.8", "const requires an iN suffix");
            };
            crate::object::const_literal(&a[0], n as u64)?;
            result = Some(t);
        } else if is_binop(op) || is_div(op) {
            arity(2, 1)?;
            let t = same(&a[0], &a[1])?;
            if !matches!(t, Type::Int(_)) {
                return err("§3.3", "binop requires integers");
            }
            result = Some(t);
        } else if let Some(pred) = op.strip_prefix("icmp.") {
            arity(2, 1)?;
            if !["eq", "ne", "ult", "ule", "slt", "sle"].contains(&pred) {
                return err("§4.8", "unknown comparison");
            }
            if !matches!(same(&a[0], &a[1])?, Type::Int(_) | Type::Ptr) {
                return err("§3.3", "icmp requires integer or pointer bit-vectors");
            }
            result = Some(Type::Int(1));
        } else if let Some((cast, suffix)) = op
            .split_once('.')
            .filter(|(c, _)| ["zext", "sext", "trunc", "ptrtoint", "load"].contains(c))
        {
            arity(1, 1)?;
            // §4.8: every suffix of these opcodes is iN / iM.
            let t = Type::parse(suffix)?;
            if !matches!(t, Type::Int(_)) {
                return err("§4.8", format!("{op}: the result suffix must be iN"));
            }
            let to = t.width(self.ptr_width)?;
            let from = rd(&a[0])?;
            if cast == "load" || cast == "ptrtoint" {
                if from != Type::Ptr {
                    return err("§3.3", "pointer operand required");
                }
                if cast == "load" && !to.is_multiple_of(8) {
                    return err("§3.3", "load.iN requires N in {8, 16, ..., 128}");
                }
            } else {
                if !matches!(from, Type::Int(_)) {
                    return err("§3.3", "zext/sext/trunc require an iN operand");
                }
                let w = from.width(self.ptr_width)?;
                if if cast == "trunc" { to >= w } else { to <= w } {
                    return err("§3.3", "cast direction mismatch");
                }
            }
            result = Some(t);
        } else {
            match op {
                "select" => {
                    arity(3, 1)?;
                    if rd(&a[0])? != Type::Int(1) {
                        return err("§3.3", "select requires i1 condition");
                    }
                    result = Some(same(&a[1], &a[2])?);
                }
                "inttoptr" => {
                    arity(1, 1)?;
                    if !matches!(rd(&a[0])?, Type::Int(_)) {
                        return err("§3.3", "inttoptr requires integer");
                    }
                    result = Some(Type::Ptr);
                }
                "alloc" => {
                    arity(1, 1)?;
                    if !matches!(rd(&a[0])?, Type::Int(_)) {
                        return err("§3.3", "alloc requires integer length");
                    }
                    result = Some(Type::Ptr);
                }
                "free" => {
                    arity(1, 0)?;
                    if rd(&a[0])? != Type::Ptr {
                        return err("§3.3", "free requires pointer");
                    }
                }
                _ if op.starts_with("store.") => {
                    arity(2, 0)?;
                    let t = Type::parse(&op[6..])?;
                    if !matches!(t, Type::Int(_)) {
                        return err("§4.8", "store requires an iN suffix");
                    }
                    if !t.width(self.ptr_width)?.is_multiple_of(8)
                        || rd(&a[0])? != Type::Ptr
                        || rd(&a[1])? != t
                    {
                        return err("§3.3", "store operand type mismatch");
                    }
                }
                "br" => {
                    if !i.lhs.is_empty() || !matches!(a.len(), 1 | 3) {
                        return err("§4.8", "br operand count");
                    }
                    if a.len() == 3 && rd(&a[0])? != Type::Int(1) {
                        return err("§3.3", "br predicate must be i1");
                    }
                    for branch in if a.len() == 1 { &a[..] } else { &a[1..] } {
                        let label = branch
                            .strip_prefix('^')
                            .ok_or_else(|| Error("§4.8: br target must be label".into()))?;
                        let (label, args) = label
                            .split_once('(')
                            .ok_or_else(|| Error("§4.8: br target needs argument list".into()))?;
                        if !labels.iter().any(|l| l.as_str() == label) {
                            return err("§3.4", "unknown branch label");
                        }
                        let vs = list(&format!("({args}"))?;
                        let block = f
                            .blocks
                            .iter()
                            .find(|b| b.name == label)
                            .expect("label checked");
                        if vs.len() != block.params.len() {
                            return err("§3.3", "branch argument count mismatch");
                        }
                        for (v, p) in vs.iter().zip(&block.params) {
                            if rd(v)? != p.ty {
                                return err("§3.3", "branch parameter type mismatch");
                            }
                        }
                    }
                }
                "ret" => {
                    if !i.lhs.is_empty() || a.len() != f.params[1].len() {
                        return err("§4.8", "return arity mismatch");
                    }
                    for (v, p) in a.iter().zip(&f.params[1]) {
                        if rd(v)? != p.ty {
                            return err("§3.3", "return type mismatch");
                        }
                    }
                }
                "trap" => arity(0, 0)?,
                "call" | "uncall" => {
                    if a.is_empty() {
                        return err("§4.8", "missing callee");
                    }
                    let callee = self.declaration(
                        a[0].strip_prefix('@')
                            .ok_or_else(|| Error("§4.8: call requires @symbol".into()))?,
                    )?;
                    // §3.1: CC calls CC or R functions, R calls R functions
                    // (SPEC-QUESTIONS 19); uncall follows the same graph.
                    if !matches!(callee.kind.as_str(), "cc" | "r") {
                        return err("§4.8", "call/uncall requires a CC or R function");
                    }
                    if f.kind == "r" && callee.kind != "r" {
                        return err("§3.1", "an R function calls only R functions and Q regions");
                    }
                    // §4.1 prints `results = call @f args` (bare operands) and
                    // `uncall @f (args) (results)`; one spelling each (SPEC-QUESTIONS 18).
                    let vs = if op == "uncall" {
                        arity(3, 0)?;
                        list(&a[1])?
                    } else {
                        if a[1..].iter().any(|v| !name(v)) {
                            return err(
                                "§16.2",
                                "call arguments are bare names: results = call @f a b",
                            );
                        }
                        a[1..].to_vec()
                    };
                    if vs.len() != callee.params[0].len() {
                        return err("§3.3", "call argument count");
                    }
                    for (v, p) in vs.iter().zip(&callee.params[0]) {
                        if rd(v)? != p.ty {
                            return err("§3.3", "call argument type");
                        }
                    }
                    if op == "call" {
                        if i.lhs.len() != callee.params[1].len() {
                            return err("§3.3", "call result count");
                        }
                        for (n, p) in i.lhs.iter().zip(&callee.params[1]) {
                            defs.push((n.clone(), p.ty.clone()));
                        }
                    } else {
                        let rs = list(&a[2])?;
                        if rs.len() != callee.params[1].len() {
                            return err("§3.3", "uncall result count");
                        }
                        for (v, p) in rs.iter().zip(&callee.params[1]) {
                            if rd(v)? != p.ty {
                                return err("§3.3", "uncall result type");
                            }
                        }
                    }
                }
                "effect" => {
                    if f.kind != "r" {
                        return err("§4.8", "effect is R only");
                    }
                    if !i.lhs.is_empty() || a.is_empty() {
                        return err("§4.8", "effect schema");
                    }
                    let operands = a[1..].iter().map(|n| rd(n)).collect::<Result<Vec<_>>>()?;
                    self.validate_effect(&a[0], &operands)?;
                }
                "hostcall" => {
                    arity(3, 1)?;
                    if f.kind != "cc"
                        || rd(&a[1])? != Type::Ptr
                        || !matches!(rd(&a[2])?, Type::Int(_))
                    {
                        return err("§4.8", "hostcall tier/type mismatch");
                    }
                    let service = self
                        .section("devices")
                        .and_then(|d| d.get("services").ok())
                        .and_then(|s| s.object().ok());
                    let service_name = crate::object::literal_name(&a[0])?;
                    if service.is_none_or(|s| !s.contains_key(&service_name)) {
                        return err("§10", "hostcall service undeclared");
                    }
                    result = Some(Type::Int(self.ptr_width));
                }
                "qreg.new" => {
                    arity(2, 1)?;
                    if f.kind != "cc" {
                        return err("§4.8", "qreg instructions are CC only");
                    }
                    crate::object::literal_name(&a[0])?;
                    let n = crate::object::host_count(&a[1], "qreg size")?;
                    if n == 0 {
                        return err("§3.3", "qreg size must be positive");
                    }
                    result = Some(Type::Qreg(n));
                }
                "qreg.free" => {
                    arity(1, 0)?;
                    if f.kind != "cc" || !matches!(rd(&a[0])?, Type::Qreg(_)) {
                        return err("§3.3", "qreg.free operand/tier");
                    }
                }
                "qreg.slice" => {
                    arity(3, 1)?;
                    let Type::Qreg(n) = rd(&a[0])? else {
                        return err("§3.3", "slice needs qreg");
                    };
                    let lo = crate::object::host_count(&a[1], "slice offset")?;
                    let len = crate::object::host_count(&a[2], "slice length")?;
                    if f.kind != "cc" || lo.checked_add(len).is_none_or(|end| end > n) {
                        return err("§3.3", "invalid slice");
                    }
                    result = Some(Type::Qreg(len));
                }
                "qreg.concat" => {
                    arity(2, 1)?;
                    let (Type::Qreg(n), Type::Qreg(m)) = (rd(&a[0])?, rd(&a[1])?) else {
                        return err("§3.3", "concat needs qregs");
                    };
                    if f.kind != "cc" || a[0] == a[1] {
                        return err("§3.3", "concat tier/alias");
                    }
                    result = Some(Type::Qreg(
                        n.checked_add(m)
                            .ok_or_else(|| Error("§3.3: qreg size overflow".into()))?,
                    ));
                }
                "measure" => {
                    arity(1, 1)?;
                    let Type::Qreg(n) = rd(&a[0])? else {
                        return err("§3.3", "measure needs qreg");
                    };
                    if f.kind != "cc" || n == 0 || n > 128 {
                        return err("§3.3", "measure tier/width");
                    }
                    result = Some(Type::Int(n as u16));
                }
                "oracle_call" | "action_call" | "adj" | "ctl" => {
                    let call = normalize_call(i)?;
                    self.validate_region_call(f, &call.instruction, types)?;
                }
                _ => return err("§4.8", format!("unknown classical opcode {op}")),
            }
        }
        if let Some(ty) = result {
            defs.push((i.lhs[0].clone(), ty));
        }
        Ok(defs)
    }
    /// §7.3 device operation table; an unmodelled device is legal text whose
    /// execution faults FAULT_EFFECT_UNMODELLED (§17 item 3).
    fn validate_effect(&self, operation: &str, operands: &[Type]) -> Result<()> {
        let (device, op) = operation
            .split_once('.')
            .ok_or_else(|| Error("§7.3: effect names dev.op".into()))?;
        let int = |t: &Type| matches!(t, Type::Int(_));
        let ptr = |t: &Type| *t == Type::Ptr;
        let shape: &[fn(&Type) -> bool] = match (device, op) {
            ("console", "write") | ("entropy", "read") => &[ptr, int],
            ("clock", "read") => &[ptr],
            ("block", "read" | "write") => &[int, ptr],
            ("console" | "clock" | "entropy" | "block", _) => {
                return err("§7.3", format!("device {device} has no operation {op}"))
            }
            _ => return Ok(()),
        };
        if operands.len() != shape.len() || !shape.iter().zip(operands).all(|(ok, t)| ok(t)) {
            return err(
                "§7.3",
                format!(
                    "{device}.{op} takes {} ptr/len operands of the §7.3 table",
                    shape.len()
                ),
            );
        }
        let declared = self
            .section("devices")
            .and_then(|d| d.get("models").ok())
            .and_then(|m| m.object().ok())
            .is_some_and(|m| m.contains_key(device));
        if !declared {
            return err(
                "§7.3",
                format!("device model {device} is used but not declared"),
            );
        }
        Ok(())
    }
    pub(crate) fn validate_metadata(&self) -> Result<()> {
        if let Some(d) = self.section("devices") {
            d.fields(&["models", "services"], &[])?;
            for group in ["models", "services"] {
                for (n, v) in d.get(group)?.object()? {
                    v.fields(&["version"], &[])?;
                    if group == "models"
                        && (!["console", "clock", "entropy", "block"].contains(&n.as_str())
                            || v.get("version")?.string()? != "0.8")
                    {
                        return err("§7.3", "unsupported device model/version");
                    }
                }
            }
        }
        if let Some(c) = self.section("coverage") {
            c.fields(&["entries"], &[])?;
            for e in c.get("entries")?.array()? {
                e.fields(
                    &["point", "source", "classification", "reason", "count"],
                    &["fault_code", "shim_hash"],
                )?;
                if !["lowered", "trapped", "shimmed"].contains(&e.get("classification")?.string()?)
                {
                    return err("§13", "unknown coverage classification");
                }
            }
            self.validate_coverage_details(c)?;
        }
        if let Some(s) = self.section("schedule") {
            s.fields(&["bindings", "entries"], &[])?;
            let b = s.get("bindings")?;
            b.fields(
                &[
                    "closure_hash",
                    "target_hash",
                    "region",
                    "variant",
                    "inherited_controls",
                    "budget",
                    "params_hash",
                    "epsilon",
                    "scheduler",
                ],
                &["expansion"],
            )?;
            b.get("scheduler")?.fields(&["name", "version"], &[])?;
            if let Some(e) = b.object()?.get("expansion") {
                e.fields(&["name", "version", "hash"], &[])?;
            }
            if b.get("budget")?.array()?.len() != 4 {
                return err("§11", "schedule budget needs four values");
            }
            for e in s.get("entries")?.array()? {
                e.fields(
                    &["path", "step", "phase", "generated", "kind"],
                    &["cleanup", "gate", "alloc", "start"],
                )?;
                let k = e.get("kind")?.string()?;
                match k {
                    "cleanup" => {
                        let c = e.get(k)?;
                        c.fields(&["realisation"], &[])?;
                        if !["reverse", "recompute"].contains(&c.get("realisation")?.string()?) {
                            return err("§6.4", "unsupported cleanup realisation");
                        }
                    }
                    "gate" => e.get(k)?.fields(&["realisation", "epsilon_share"], &[])?,
                    "alloc" => {
                        e.get(k)?.fields(&["class"], &[])?;
                        if e.get(k)?.get("class")?.string()? == "any" {
                            return err("§11", "schedule requires concrete class");
                        }
                    }
                    "start" => e.get(k)?.fields(&["time"], &[])?,
                    _ => return err("§11", "unknown schedule entry kind"),
                };
                for p in e.get("path")?.array()? {
                    p.fields(&["symbol", "point", "iteration"], &[])?;
                }
            }
            self.validate_schedule_details(s)?;
        }
        if let Some(c) = self.section("certificate") {
            c.fields(
                &["versions", "bindings", "producer", "regions", "composition"],
                &[],
            )?;
            c.get("bindings")?.fields(
                &[
                    "object_hash",
                    "interface_hash",
                    "closure_hash",
                    "reference_program_hash",
                    "dialect_mapping_version",
                    "target_hash",
                ],
                &["schedule_hash"],
            )?;
            c.get("producer")?
                .fields(&["name", "version", "materialisation"], &[])?;
            c.get("producer")?
                .get("materialisation")?
                .fields(&["llvm_version", "pipeline", "normaliser_version"], &[])?;
            c.get("composition")?
                .fields(&["multiplicities", "epsilon_total", "method"], &[])?;
            for r in c.get("regions")?.object()?.values() {
                r.fields(
                    &[
                        "kind",
                        "signature",
                        "bounds",
                        "frame",
                        "resources",
                        "gates_used",
                        "claims",
                        "verification",
                        "cleanup",
                        "error",
                        "definedness_assumptions",
                    ],
                    &[],
                )?;
                r.get("verification")?.fields(
                    &[
                        "oracle",
                        "borrow_frames",
                        "inverse_pairs",
                        "unitary",
                        "channel",
                        "fault_rate",
                    ],
                    &[],
                )?;
            }
            self.validate_certificate_details(c)?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Creation {
    kind: String,
    sources: Vec<String>,
    versions: Vec<(String, Vec<String>)>,
    controls: Vec<String>,
    /// Closure counts of the dependency wires at the computation.
    closures: Vec<(String, u64)>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
struct Discipline {
    types: BTreeMap<String, Type>,
    inputs: BTreeSet<String>,
    predicates: Vec<String>,
    created: BTreeMap<String, Creation>,
    versions: BTreeMap<String, Vec<String>>,
    /// Window-closing events per wire (non-diagonal gates, non-permutation
    /// calls): a window opens at its computation (§6.2), so cleanups compare
    /// counts snapshotted at the computation rather than a permanent set.
    closed: BTreeMap<String, u64>,
    /// Outstanding diagonal operations (phase functions of their operands).
    phases: Vec<String>,
    reads: BTreeMap<String, Vec<(String, Vec<String>)>>,
    other_versions: BTreeSet<String>,
    local_regions: BTreeSet<String>,
    cell_epochs: BTreeMap<String, u128>,
    modified: BTreeSet<String>,
}
impl Discipline {
    fn new(f: &Function, types: BTreeMap<String, Type>) -> Self {
        Self {
            types,
            inputs: f.params[1]
                .iter()
                .chain(
                    f.params[2]
                        .iter()
                        .filter(|p| p.role.as_deref() == Some("in")),
                )
                .map(|p| p.name.clone())
                .collect(),
            predicates: Vec::new(),
            created: BTreeMap::new(),
            versions: BTreeMap::new(),
            closed: BTreeMap::new(),
            phases: Vec::new(),
            reads: BTreeMap::new(),
            other_versions: BTreeSet::new(),
            local_regions: BTreeSet::new(),
            cell_epochs: BTreeMap::new(),
            modified: BTreeSet::new(),
        }
    }
    fn wires(&self, n: &str) -> Result<Vec<String>> {
        let n = n.strip_prefix('!').unwrap_or(n);
        let (base, index) = if let Some((base, index)) = n.split_once('[') {
            let index = crate::object::host_count(
                index
                    .strip_suffix(']')
                    .ok_or_else(|| Error("§16.2: invalid indexed wire".into()))?,
                "wire index",
            )?;
            (base, Some(index))
        } else {
            (n, None)
        };
        let w = match self.types.get(base) {
            Some(Type::Wire(n)) => *n,
            _ => return err("§3.3", format!("unknown/live wire {n}")),
        };
        if let Some(i) = index {
            if i >= w {
                return err("§3.3", "wire index out of range");
            }
            Ok(vec![format!("{base}[{i}]")])
        } else {
            Ok((0..w).map(|i| format!("{base}[{i}]")).collect())
        }
    }
    fn disjoint(&self, operands: &[&str]) -> Result<()> {
        let mut seen = BTreeSet::new();
        for o in operands {
            for w in self.wires(o)? {
                if !seen.insert(w) {
                    return err("§3.3", "overlapping wire operands");
                }
            }
        }
        Ok(())
    }
    fn snapshot(&self, wires: &[String]) -> Vec<(String, Vec<String>)> {
        wires
            .iter()
            .map(|w| {
                let mut log = self.versions.get(w).cloned().unwrap_or_default();
                if let Some(epoch) = self.cell_epochs.get(w) {
                    log.push(format!("cell_version:{epoch}"));
                }
                (w.clone(), log)
            })
            .collect()
    }
    fn read_snapshot(&self, wires: &[String]) -> Result<Vec<(String, Vec<String>)>> {
        let mut reads = wires.to_vec();
        for p in &self.predicates {
            reads.extend(self.wires(p)?);
        }
        reads.sort();
        reads.dedup();
        Ok(self.snapshot(&reads))
    }
    /// Diagonal operations commute: an inverse anywhere in the outstanding
    /// list cancels (same operand versions are part of the key).
    fn toggle_phase(&mut self, key: String) {
        match operation_inverse(&key).and_then(|inv| self.phases.iter().position(|p| *p == inv)) {
            Some(i) => {
                self.phases.remove(i);
            }
            None => self.phases.push(key),
        }
    }
    fn closures(&self, wires: &[String]) -> Vec<(String, u64)> {
        wires
            .iter()
            .map(|w| (w.clone(), self.closed.get(w).copied().unwrap_or(0)))
            .collect()
    }
    /// §6.2: a non-diagonal gate or non-permutation call on a dependency
    /// since the computation closed the window.
    fn window_closed(&self, c: &Creation) -> bool {
        c.closures
            .iter()
            .any(|(w, n)| self.closed.get(w).copied().unwrap_or(0) != *n)
    }
    fn dependencies(&self, wires: &[String]) -> BTreeSet<String> {
        let mut out = BTreeSet::new();
        let mut pending = wires.to_vec();
        while let Some(wire) = pending.pop() {
            if !out.insert(wire.clone()) {
                continue;
            }
            if let Some(c) = self.created.get(wire.split('[').next().expect("wire name")) {
                pending.extend(c.versions.iter().map(|(w, _)| w.clone()));
            }
            for v in self.versions.get(&wire).into_iter().flatten() {
                if let Some(reads) = self.reads.get(v) {
                    pending.extend(reads.iter().map(|(w, _)| w.clone()));
                }
            }
        }
        out
    }
    /// Records what an operation read: its operands, the effective control
    /// context and, transitively, every wire their current versions depend on
    /// (§6.1 dependency sets are unions of the read operands' sets). An
    /// entangling read through an intermediate value therefore still blocks
    /// the adjoint of an `other` value it read (found by the §6 fuzzer:
    /// `h d0; ccx d0 p d1; cx d1 t; ccx d0 p d1; h d0` leaves d0 entangled with t).
    fn note_reads(&mut self, version: &str, wires: &[String]) -> Result<()> {
        let direct = self.read_snapshot(wires)?;
        let mut all: Vec<String> = direct.iter().map(|(w, _)| w.clone()).collect();
        all.extend(self.dependencies(&all));
        all.sort();
        all.dedup();
        let snapshot = self.snapshot(&all);
        self.reads.insert(version.into(), snapshot);
        Ok(())
    }
    fn produce(&mut self, n: &str, ty: Type, c: Creation) -> Result<()> {
        if self.types.contains_key(n) {
            return err("§3.3", "linear name produced twice");
        }
        self.types.insert(n.into(), ty);
        self.created.insert(n.into(), c);
        Ok(())
    }
    fn consume(&mut self, n: &str) -> Result<()> {
        let consumed = self.wires(n)?;
        for wire in &consumed {
            self.versions.remove(wire);
            self.closed.remove(wire);
            self.modified.remove(wire);
        }
        // A cleaned value no longer carries information: records that read it
        // keep its transitive dependencies (note_reads) but not its name, which
        // a later value may reuse.
        for reads in self.reads.values_mut() {
            reads.retain(|(w, _)| !consumed.contains(w));
        }
        self.types.remove(n);
        self.created.remove(n);
        Ok(())
    }
    fn check_target(&self, n: &str, diagonal: bool) -> Result<()> {
        let wires = self.wires(n)?;
        for p in &self.predicates {
            if self.wires(p)?.iter().any(|w| wires.contains(w)) {
                return err("§3.3", "effective control aliases target");
            }
        }
        let base = n.split('[').next().expect("name");
        if !diagonal && self.inputs.contains(base) {
            return err("§5.1", "instruction changes an in wire");
        }
        Ok(())
    }
    fn change(&mut self, n: &str, version: String, other: bool) -> Result<()> {
        self.check_target(n, false)?;
        let context = self.read_snapshot(&[])?;
        // Keep the dependency metadata under the complete operation identity.
        // (The context snapshot itself contains earlier ":context" suffixes, so
        // the operation key is taken before formatting, never re-split.)
        let original = version;
        let version = format!("{original}:context{context:?}");
        let reads = self.reads.get(&original).cloned().unwrap_or(context);
        self.reads.insert(version.clone(), reads);
        if self.other_versions.contains(&original) {
            self.other_versions.insert(version.clone());
        }
        let needs_adjoint = other || self.is_other(n)?;
        for w in self.wires(n)? {
            if other {
                *self.closed.entry(w.clone()).or_default() += 1;
            }
            let blocked = needs_adjoint
                && (self.phases.iter().any(|v| {
                    self.reads
                        .get(v)
                        .is_some_and(|reads| reads.iter().any(|(source, _)| source == &w))
                }) || self.versions.iter().any(|(target, log)| {
                    target != &w
                        && log.iter().any(|v| {
                            self.reads
                                .get(v)
                                .is_some_and(|reads| reads.iter().any(|(source, _)| source == &w))
                        })
                }) || self
                    .created
                    .values()
                    .any(|c| c.versions.iter().any(|(source, _)| source == &w)));
            self.update_log(&w, &version, blocked);
        }
        Ok(())
    }
    fn update_log(&mut self, wire: &str, version: &str, blocked: bool) {
        self.modified.insert(wire.into());
        let log = self.versions.entry(wire.into()).or_default();
        if !blocked
            && log
                .last()
                .is_some_and(|prior| operation_inverse(prior).as_deref() == Some(version))
        {
            log.pop();
        } else {
            log.push(version.into());
        }
        if log.is_empty() {
            self.versions.remove(wire);
        }
    }
    fn is_other(&self, n: &str) -> Result<bool> {
        Ok(self.wires(n)?.iter().any(|w| {
            self.versions.get(w).is_some_and(|log| {
                log.iter().any(|v| {
                    self.other_versions.contains(v) || {
                        ["h:", "rx:", "ry:", "call_other_"]
                            .iter()
                            .any(|p| v.starts_with(p))
                    }
                })
            })
        }))
    }
    fn record_change(&mut self, n: &str, version: String) -> Result<()> {
        for wire in self.wires(n)? {
            let log = self.versions.entry(wire).or_default();
            if log
                .last()
                .is_some_and(|p| operation_inverse(p).as_deref() == Some(&version))
            {
                log.pop();
            } else {
                log.push(version.clone());
            }
        }
        Ok(())
    }
}
fn operation_inverse(version: &str) -> Option<String> {
    let (op, rest) = version.split_once(':')?;
    let inverse = match op {
        "s" => "sdg",
        "sdg" => "s",
        "t" => "tdg",
        "tdg" => "t",
        "call_other_fwd" => "call_other_adj",
        "call_other_adj" => "call_other_fwd",
        "call_perm_fwd" => "call_perm_adj",
        "call_perm_adj" => "call_perm_fwd",
        "x" | "cx" | "ccx" | "mcx" | "swap" | "h" | "cz" | "qxor" | "qexch" | "qexchm"
        | "qstore" | "oracle_call" => op,
        "rx" | "ry" | "rz" => {
            let (angle, tail) = rest.split_once(' ')?;
            let bits = u64::from_str_radix(angle.strip_prefix('#')?, 16).ok()?;
            return Some(format!("{op}:#{:016x} {tail}", bits ^ (1u64 << 63)));
        }
        _ => return None,
    };
    Some(format!("{inverse}:{rest}"))
}
pub(crate) fn gate(op: &str) -> bool {
    [
        "x", "h", "s", "sdg", "t", "tdg", "cx", "ccx", "mcx", "swap", "cz", "rx", "ry", "rz",
    ]
    .contains(&op)
}
pub(crate) fn gate_operands(i: &Instruction) -> Result<(Vec<String>, Vec<String>, bool, bool)> {
    let a = &i.operands;
    let op = i.op.as_str();
    let n = match op {
        "x" | "h" | "s" | "sdg" | "t" | "tdg" => 1,
        "ccx" => 3,
        _ => 2,
    };
    if !i.lhs.is_empty() || a.len() != n {
        return err("§4.8", "gate arity mismatch");
    }
    let diagonal = matches!(op, "s" | "sdg" | "t" | "tdg" | "cz" | "rz");
    let other = matches!(op, "h" | "rx" | "ry");
    let (controls, targets) = match op {
        "cx" => (vec![a[0].clone()], vec![a[1].clone()]),
        "ccx" => (a[..2].to_vec(), vec![a[2].clone()]),
        "mcx" => (list(&a[0])?, vec![a[1].clone()]),
        "swap" | "cz" => (Vec::new(), a.clone()),
        "rx" | "ry" | "rz" => {
            if let Ok(x) = a[0].parse::<f64>() {
                if !x.is_finite() || !a[0].contains(['.', 'e', 'E']) {
                    return err("§16.1", "angle must be finite FLOAT");
                }
            } else if !name(&a[0]) {
                return err("§4.4", "angle must be literal or classical argument");
            }
            (Vec::new(), vec![a[1].clone()])
        }
        _ => (Vec::new(), vec![a[0].clone()]),
    };
    Ok((controls, targets, diagonal, other))
}
pub(crate) struct NormalCall {
    pub instruction: Instruction,
    pub control: Option<String>,
    pub adjoint: bool,
}
pub(crate) fn normalize_call(i: &Instruction) -> Result<NormalCall> {
    let mut out = i.clone();
    let mut control = None;
    let mut adjoint = false;
    loop {
        match out.op.as_str() {
            "adj" => {
                adjoint = !adjoint;
                if out.operands.is_empty() {
                    return err("§4.7", "missing adjoint call");
                }
                out.op = out.operands.remove(0);
            }
            "ctl" => {
                if control.is_some() || out.operands.len() < 2 {
                    return err("§4.7", "invalid controlled call");
                }
                control = Some(out.operands.remove(0));
                out.op = out.operands.remove(0);
            }
            _ => break,
        }
    }
    if !matches!(out.op.as_str(), "oracle_call" | "action_call") {
        return err("§4.7", "adj/ctl requires a region call");
    }
    Ok(NormalCall {
        instruction: out,
        control,
        adjoint,
    })
}
