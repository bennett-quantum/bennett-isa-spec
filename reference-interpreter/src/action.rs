use crate::{
    err,
    object::list,
    quantum::Wire,
    validation::{gate, normalize_call},
    Argument, Error, Function, Instruction, Object, QuantumBindings, QuantumRegion, QuantumState,
    Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

struct Environment {
    bindings: QuantumBindings,
    predicates: Vec<Wire>,
    borrowed: BTreeMap<String, Vec<usize>>,
    borrow_used: BTreeSet<usize>,
}
impl Environment {
    fn bundle(&self, n: &str) -> Result<Vec<Wire>> {
        if let Some((base, index)) = n.split_once('[') {
            let index = index
                .strip_suffix(']')
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| Error::new("§3.3: invalid wire index".into()))?;
            let ws = self
                .bindings
                .bundles
                .get(base)
                .ok_or_else(|| Error::new(format!("§3.3: unknown live bundle {base}")))?;
            return ws
                .get(index)
                .map(|w| vec![*w])
                .ok_or_else(|| Error::new("§3.3: wire index out of range".into()));
        }
        self.bindings
            .bundles
            .get(n)
            .cloned()
            .ok_or_else(|| Error::new(format!("§3.3: unknown live bundle {n}")))
    }
    fn scalar(&self, n: &str) -> Result<Wire> {
        let w = self.bundle(n)?;
        if w.len() != 1 {
            return err("§3.3", "scalar wire required");
        }
        Ok(w[0])
    }
    fn controls(&self, inherited: &[(Wire, bool)]) -> Vec<(Wire, bool)> {
        let mut c = inherited.to_vec();
        for w in &self.predicates {
            if !c.contains(&(*w, true)) {
                c.push((*w, true));
            }
        }
        c
    }
    fn produce(&mut self, n: &str, wires: Vec<Wire>) -> Result<()> {
        if self.bindings.bundles.contains_key(n) {
            return err("§3.3", "live bundle produced twice");
        }
        self.bindings.bundles.insert(n.into(), wires);
        Ok(())
    }
}
impl Object {
    pub(crate) fn permutation_action_vectors(
        &self,
        name: &str,
        mutables: &[crate::BitVector],
        inputs: &[crate::BitVector],
        regions: &BTreeMap<String, Vec<u8>>,
        args: &[Argument],
        adjoint: bool,
    ) -> Result<Vec<crate::BitVector>> {
        let f = &self.functions[name];
        if f.kind != "perm_action"
            || mutables.len() != f.params[0].len()
            || inputs.len() != f.params[1].len()
            || args.len() != f.params[3].len()
        {
            return err("§4.2", "lift.perm_call interface mismatch");
        }
        let mut q = QuantumState::new();
        let mut b = QuantumBindings::default();
        for (values, ps) in [(mutables, &f.params[0]), (inputs, &f.params[1])] {
            for (v, p) in values.iter().zip(ps) {
                let wires = q.allocate(p.ty.width(self.ptr_width)? as usize)?;
                for (i, w) in wires.iter().enumerate() {
                    if v.bit(i)? {
                        q.gate("x", &[*w], None, &[])?;
                    }
                }
                b.bundles.insert(p.name.clone(), wires);
            }
        }
        for p in &f.params[2] {
            let Type::Region(a, w) = p.ty else {
                return err("§5.1", "mem slot requires region type");
            };
            let bytes = regions
                .get(&p.name)
                .ok_or_else(|| Error::new("§4.2: missing lifted action region".into()))?;
            let mut cells = Vec::new();
            for byte in bytes {
                let cell = q.allocate(w as usize)?;
                for (i, wire) in cell.iter().enumerate() {
                    if i < 8 && byte & (1u8 << i) != 0 {
                        q.gate("x", &[*wire], None, &[])?;
                    }
                }
                cells.push(cell);
            }
            b.regions.insert(
                p.name.clone(),
                QuantumRegion {
                    address_width: a,
                    cell_width: w,
                    cells,
                },
            );
        }
        for (p, v) in f.params[3].iter().zip(args) {
            b.args.insert(p.name.clone(), v.clone());
        }
        let capacity = self.borrow_capacity(name)?;
        let idle = q.allocate(capacity)?;
        if !idle.is_empty() {
            b.borrowed.push(idle.clone());
        }
        self.execute_action(name, &mut q, &b, adjoint, &[])?;
        q.release(&idle)?;
        f.params[0]
            .iter()
            .map(|p| {
                let wires = &b.bundles[&p.name];
                let mut value = crate::BitVector::zero(wires.len())?;
                for (i, wire) in wires.iter().enumerate() {
                    value.set(i, q.read_basis(&[*wire])? != 0)?;
                }
                Ok(value)
            })
            .collect()
    }
    /// The number of borrowed wires an action body's `qborrow` rows can
    /// draw from its caller's idle pool (§5.6), summed over the call closure.
    pub fn borrow_capacity(&self, root: &str) -> Result<usize> {
        fn scan(seq: &[Instruction], count: &mut usize, children: &mut Vec<String>) -> Result<()> {
            for i in seq {
                if i.op == "qborrow" {
                    *count = count
                        .checked_add(i.operands[1].parse::<usize>().map_err(|_| {
                            Error::host_resource(
                                "§5.6: host limit: borrow count exceeds usize".into(),
                            )
                        })?)
                        .ok_or_else(|| {
                            Error::host_resource(
                                "§5.6: host limit: borrow capacity sum exceeds usize".into(),
                            )
                        })?;
                }
                if let Ok(call) = normalize_call(i) {
                    if call.instruction.op == "action_call" {
                        children.push(call.instruction.operands[0].trim_start_matches('@').into());
                    }
                }
                for body in &i.bodies {
                    scan(body, count, children)?;
                }
            }
            Ok(())
        }
        let mut count = 0;
        let mut pending = vec![root.to_string()];
        let mut visited = BTreeSet::new();
        while let Some(name) = pending.pop() {
            if !visited.insert(name.clone()) {
                continue;
            }
            let function = self
                .functions
                .get(&name)
                .ok_or_else(|| Error::new(format!("§9.3: unresolved action @{name}")))?;
            for block in &function.blocks {
                scan(&block.instructions, &mut count, &mut pending)?;
            }
        }
        Ok(count)
    }
    pub fn execute_action(
        &self,
        name: &str,
        q: &mut QuantumState,
        bindings: &QuantumBindings,
        adjoint: bool,
        controls: &[Wire],
    ) -> Result<()> {
        self.ensure_linked()?;
        self.action_internal(
            name,
            q,
            bindings,
            adjoint,
            &controls.iter().map(|w| (*w, true)).collect::<Vec<_>>(),
        )
    }
    fn action_internal(
        &self,
        name: &str,
        q: &mut QuantumState,
        bindings: &QuantumBindings,
        adjoint: bool,
        controls: &[(Wire, bool)],
    ) -> Result<()> {
        let entry_next_wire = q.next_wire_id();
        let f = self
            .functions
            .get(name)
            .filter(|f| matches!(f.kind.as_str(), "action" | "perm_action"))
            .ok_or_else(|| Error::new(format!("§5.3: unknown action @{name}")))?;
        let mut used = BTreeSet::new();
        for p in f.params[0].iter().chain(&f.params[1]) {
            let w = bindings
                .bundles
                .get(&p.name)
                .ok_or_else(|| Error::new(format!("§5.1: missing bundle {}", p.name)))?;
            if w.len() != p.ty.width(self.ptr_width)? as usize {
                return err("§3.3", "action bundle width mismatch");
            }
            for wire in w {
                q.position(*wire)?;
                if !used.insert(wire) {
                    return err("§3.3", "action interface bundles overlap");
                }
            }
        }
        for p in &f.params[2] {
            let Type::Region(a, w) = p.ty else {
                return err("§5.1", "mem slot requires region type");
            };
            let r = bindings
                .regions
                .get(&p.name)
                .ok_or_else(|| Error::new("§5.1: missing interface region".into()))?;
            if r.address_width != a
                || r.cell_width != w
                || crate::oracle::power_of_two(a as u128) != Some(r.cells.len() as u128)
                || r.cells.iter().any(|c| c.len() != w as usize)
            {
                return err("§3.3", "action interface region dimensions mismatch");
            }
            for wire in r.cells.iter().flatten() {
                q.position(*wire)?;
                if !used.insert(wire) {
                    return err("§3.3", "action regions overlap interface");
                }
            }
        }
        for p in &f.params[3] {
            let arg = bindings
                .args
                .get(&p.name)
                .ok_or_else(|| Error::new("§5.1: missing action classical argument".into()))?;
            // §3.3 classical types, ptr included (ptr_width bits).
            crate::oracle::check_argument(self, &p.ty, arg)?;
        }
        if self.section("schedule").is_some() {
            let arguments = f.params[3]
                .iter()
                .map(|p| bindings.args[&p.name].clone())
                .collect::<Vec<_>>();
            self.check_schedule_matches_for_run(name, crate::Budget::steps(0), &arguments)?;
        }
        let mut env = Environment {
            bindings: bindings.clone(),
            predicates: Vec::new(),
            borrowed: BTreeMap::new(),
            borrow_used: BTreeSet::new(),
        };
        let mut regions = Vec::new();
        for b in &f.blocks {
            gather_regions(&b.instructions, &mut regions);
        }
        let mut local = Vec::new();
        for i in regions {
            let a = crate::object::host_count(&i.operands[1], "qregion address width")?;
            let w = crate::object::host_count(&i.operands[2], "qregion cell width")?;
            let count = crate::oracle::power_of_two(a as u128)
                .and_then(|n| usize::try_from(n).ok())
                .ok_or_else(|| {
                    Error::host_resource(
                        "§4.6: host limit: local region exceeds host address space".into(),
                    )
                })?;
            let n = count.checked_mul(w as usize).ok_or_else(|| {
                Error::host_resource("§4.6: host limit: region bit count exceeds usize".into())
            })?;
            let wires = q.allocate(n)?;
            let image = if i.operands.len() == 4 {
                self.data[i.operands[3].trim_start_matches('@')].clone()
            } else {
                vec![0; n.div_ceil(8)]
            };
            for (bit, wire) in wires.iter().enumerate() {
                if image[bit / 8] & (1 << (bit % 8)) != 0 {
                    q.gate("x", &[*wire], None, &[])?;
                }
            }
            let cells = wires.chunks_exact(w as usize).map(|c| c.to_vec()).collect();
            env.bindings.regions.insert(
                i.operands[0].clone(),
                QuantumRegion {
                    address_width: a,
                    cell_width: w,
                    cells,
                },
            );
            local.push((wires, image));
        }
        let mut live_widths = bindings
            .bundles
            .iter()
            .map(|(n, w)| (n.clone(), w.len()))
            .collect();
        let instructions: Vec<_> = f
            .blocks
            .iter()
            .flat_map(|b| b.instructions.iter().cloned())
            .collect();
        let widths = infer_widths(&instructions, &env, &mut live_widths)?;
        self.action_sequence(&instructions, &widths, q, &mut env, adjoint, controls)?;
        if !env.predicates.is_empty() || !env.borrowed.is_empty() {
            return err("§5.1", "action exited with unclosed predicate/borrow frame");
        }
        let interface: BTreeSet<_> = bindings.bundles.keys().collect();
        if env.bindings.bundles.keys().any(|n| !interface.contains(n)) {
            return err("§5.1", "action exited with live workspace");
        }
        for (wires, image) in local.into_iter().rev() {
            for (bit, wire) in wires.iter().enumerate() {
                if image[bit / 8] & (1 << (bit % 8)) != 0 {
                    q.gate("x", &[*wire], None, &[])?;
                }
            }
            q.release(&wires)?;
        }
        // Region-local identities cannot escape through caller-owned interfaces.
        // Reclaim them once all local factors have been explicitly cleaned.
        q.restore_next_wire_id(entry_next_wire)?;
        Ok(())
    }
    fn action_sequence(
        &self,
        sequence: &[Instruction],
        widths: &[LifetimeWidths],
        q: &mut QuantumState,
        e: &mut Environment,
        adjoint: bool,
        inherited: &[(Wire, bool)],
    ) -> Result<()> {
        let indices: Vec<_> = if adjoint {
            (0..sequence.len()).rev().collect()
        } else {
            (0..sequence.len()).collect()
        };
        for index in indices {
            let i = &sequence[index];
            let width = &widths[index];
            let a = &i.operands;
            let context = e.controls(inherited);
            let op = i.op.as_str();
            if gate(op) {
                if !matches!(op, "x" | "cx" | "ccx" | "mcx" | "swap") {
                    q.audit_frame()?;
                }
                let mut wires = Vec::new();
                let mut angle = None;
                if op == "mcx" {
                    for n in list(&a[0])? {
                        wires.push(e.scalar(&n)?);
                    }
                    wires.push(e.scalar(&a[1])?);
                } else if matches!(op, "rx" | "ry" | "rz") {
                    let theta = match a[0].parse::<f64>() {
                        Ok(n) => n,
                        Err(_) => match e.bindings.args.get(&a[0]) {
                            Some(Argument::Float(n)) => *n,
                            _ => return err("§4.4", "angle is not a finite classical f64"),
                        },
                    };
                    angle = Some(if adjoint { -theta } else { theta });
                    wires.push(e.scalar(&a[1])?);
                } else {
                    for n in a {
                        wires.push(e.scalar(n)?);
                    }
                }
                let inverse = if adjoint {
                    match op {
                        "s" => "sdg",
                        "sdg" => "s",
                        "t" => "tdg",
                        "tdg" => "t",
                        _ => op,
                    }
                } else {
                    op
                };
                q.controlled_gate(inverse, &wires, angle, &context)?;
                continue;
            }
            match op {
                "qregion" => {}
                "qpred" | "qunpred" => {
                    let push = (op == "qpred") ^ adjoint;
                    let wire = e.scalar(&a[0])?;
                    if push {
                        e.predicates.push(wire);
                    } else if e.predicates.pop() != Some(wire) {
                        return err("§3.5", "action predicate pop mismatch");
                    }
                }
                "qif" => {
                    let w = e.scalar(&a[0])?;
                    let mut yes = context.clone();
                    yes.push((w, true));
                    let mut no = context.clone();
                    no.push((w, false));
                    // Reverse the complete §4.6 expansion, including its
                    // unconditional lifetime operations (SPEC-QUESTIONS 28).
                    if adjoint {
                        self.action_sequence(&i.bodies[1], &width.bodies[1], q, e, true, &no)?;
                        self.action_sequence(&i.bodies[0], &width.bodies[0], q, e, true, &yes)?;
                    } else {
                        self.action_sequence(&i.bodies[0], &width.bodies[0], q, e, false, &yes)?;
                        self.action_sequence(&i.bodies[1], &width.bodies[1], q, e, false, &no)?;
                    }
                }
                "qloop" => {
                    let n = a[0].parse::<usize>().map_err(|_| {
                        Error::new("§4.6: loop bound exceeds host index space".into())
                    })?;
                    if a.len() == 1 {
                        // body n times (§4.6). The iterated state is exact, so once
                        // it returns to its loop-entry value the remaining iterations
                        // are whole periods and are skipped (X^(2^40) needs 2).
                        let entry = (q.content(), e.bindings.bundles.clone());
                        let mut done = 0usize;
                        while done < n {
                            let iteration = if adjoint { n - 1 - done } else { done };
                            self.action_sequence(
                                &i.bodies[0],
                                width.iteration(iteration),
                                q,
                                e,
                                adjoint,
                                &context,
                            )?;
                            done += 1;
                            // Reverse traversal must execute the transient prefix
                            // separately, even if a steady iteration is identity.
                            let periodic_end = if adjoint { n - width.cycle_start } else { n };
                            if done < periodic_end
                                && !q.auditing_borrows()
                                && (q.content(), e.bindings.bundles.clone()) == entry
                            {
                                done += (periodic_end - done) / done * done;
                            }
                        }
                    } else {
                        let condition = e.bundle(&a[2])?;
                        let flags = q.allocate(n)?;
                        if adjoint {
                            for j in 0..n {
                                let controls = if j == 0 {
                                    vec![condition[j], flags[j]]
                                } else {
                                    vec![flags[j - 1], condition[j], flags[j]]
                                };
                                q.controlled_gate(
                                    if j == 0 { "cx" } else { "ccx" },
                                    &controls,
                                    None,
                                    &context,
                                )?;
                            }
                            for j in (0..n).rev() {
                                let mut c = context.clone();
                                c.push((flags[j], true));
                                self.action_sequence(
                                    &i.bodies[0],
                                    width.iteration(j),
                                    q,
                                    e,
                                    true,
                                    &c,
                                )?;
                            }
                        } else {
                            for j in 0..n {
                                let controls = if j == 0 {
                                    vec![condition[j], flags[j]]
                                } else {
                                    vec![flags[j - 1], condition[j], flags[j]]
                                };
                                q.controlled_gate(
                                    if j == 0 { "cx" } else { "ccx" },
                                    &controls,
                                    None,
                                    &context,
                                )?;
                                let mut c = context.clone();
                                c.push((flags[j], true));
                                self.action_sequence(
                                    &i.bodies[0],
                                    width.iteration(j),
                                    q,
                                    e,
                                    false,
                                    &c,
                                )?;
                            }
                        }
                        for j in (0..n).rev() {
                            let controls = if j == 0 {
                                vec![condition[j], flags[j]]
                            } else {
                                vec![flags[j - 1], condition[j], flags[j]]
                            };
                            q.controlled_gate(
                                if j == 0 { "cx" } else { "ccx" },
                                &controls,
                                None,
                                &context,
                            )?;
                        }
                        q.release(&flags)?;
                    }
                }
                "qalloc" | "qfree" => {
                    let allocate = (op == "qalloc") ^ adjoint;
                    let name = if op == "qalloc" { &i.lhs[0] } else { &a[0] };
                    if allocate {
                        let n = width.produced.ok_or_else(|| {
                            Error::new("§4.3: missing matched lifetime width".into())
                        })?;
                        let w = q.allocate(n)?;
                        e.produce(name, w)?;
                    } else {
                        let w = e.bundle(name)?;
                        q.release(&w)?;
                        e.bindings.bundles.remove(name);
                    }
                }
                "qborrow" | "qreturn" => {
                    let borrow = (op == "qborrow") ^ adjoint;
                    let name = if op == "qborrow" { &i.lhs[0] } else { &a[0] };
                    if borrow {
                        let n = width.produced.ok_or_else(|| {
                            Error::new("§4.3: missing matched lifetime width".into())
                        })?;
                        let pool = e
                            .bindings
                            .borrowed
                            .iter()
                            .flatten()
                            .copied()
                            .collect::<Vec<_>>();
                        let indices = (0..pool.len())
                            .filter(|i| !e.borrow_used.contains(i))
                            .take(n)
                            .collect::<Vec<_>>();
                        if indices.len() != n {
                            return err(
                                "§5.6",
                                "designated idle set lacks requested borrowed wires",
                            );
                        }
                        let wires: Vec<_> = indices.iter().map(|i| pool[*i]).collect();
                        e.borrow_used.extend(&indices);
                        e.borrowed.insert(name.into(), indices);
                        q.audit_borrow(&wires);
                        e.produce(name, wires)?;
                    } else {
                        q.audit_return(&e.bundle(name)?)?;
                        let indices = e
                            .borrowed
                            .remove(name)
                            .ok_or_else(|| Error::new("§5.6: unmatched return".into()))?;
                        for index in indices {
                            e.borrow_used.remove(&index);
                        }
                        e.bindings.bundles.remove(name);
                    }
                }
                "qcopy" | "quncopy" => {
                    let copy = (op == "qcopy") ^ adjoint;
                    let source = e.bundle(&a[0])?;
                    let name = if op == "qcopy" { &i.lhs[0] } else { &a[1] };
                    let destination = if copy {
                        let w = q.allocate(source.len())?;
                        e.produce(name, w.clone())?;
                        w
                    } else {
                        e.bundle(name)?
                    };
                    bundle_xor(q, &source, &destination, &context)?;
                    if !copy {
                        q.release(&destination)?;
                        e.bindings.bundles.remove(name);
                    }
                }
                "qxor" => bundle_xor(q, &e.bundle(&a[0])?, &e.bundle(&a[1])?, &context)?,
                "qexch" => bundle_exchange(q, &e.bundle(&a[0])?, &e.bundle(&a[1])?, &context)?,
                "qload" | "qunload" | "qstore" | "qexchm" => {
                    let region = e
                        .bindings
                        .regions
                        .get(&a[0])
                        .cloned()
                        .ok_or_else(|| Error::new("§4.6: quantum region not found".into()))?;
                    let address = e.bundle(&a[1])?;
                    let load = matches!(op, "qload" | "qunload");
                    let compute = (op == "qload") ^ adjoint;
                    let name = if op == "qload" { &i.lhs[0] } else { &a[2] };
                    let value = if load && compute {
                        let w = q.allocate(region.cell_width as usize)?;
                        e.produce(name, w.clone())?;
                        w
                    } else {
                        e.bundle(name)?
                    };
                    for (index, cell) in region.cells.iter().enumerate() {
                        let mut c = context.clone();
                        for (bit, w) in address.iter().enumerate() {
                            c.push((*w, index & (1usize << bit) != 0));
                        }
                        if op == "qexchm" {
                            bundle_exchange(q, cell, &value, &c)?;
                        } else if load {
                            bundle_xor(q, cell, &value, &c)?;
                        } else {
                            bundle_xor(q, &value, cell, &c)?;
                        }
                    }
                    if load && !compute {
                        q.release(&value)?;
                        e.bindings.bundles.remove(name);
                    }
                }
                "oracle_call" | "action_call" | "adj" | "ctl" => {
                    let call = normalize_call(i)?;
                    let mut c = context.clone();
                    if let Some(w) = call.control {
                        c.push((e.scalar(&w)?, true));
                    }
                    let ca = &call.instruction.operands;
                    let callee = &self.functions[ca[0].trim_start_matches('@')];
                    if call.instruction.op == "oracle_call" {
                        let token = e
                            .bindings
                            .args
                            .get(&ca[1])
                            .and_then(|v| {
                                if let Argument::Token(b) = v {
                                    Some(*b)
                                } else {
                                    None
                                }
                            })
                            .ok_or_else(|| Error::new("§4.7: missing oracle token".into()))?;
                        let ins = list(&ca[2])?
                            .iter()
                            .map(|n| e.bundle(n))
                            .collect::<Result<Vec<_>>>()?;
                        let target = e.bundle(&ca[3])?;
                        let fault = e.bundle(&ca[4])?;
                        let mut regions = BTreeMap::new();
                        for (p, n) in callee.params[3].iter().zip(list(&ca[5])?) {
                            regions.insert(
                                p.name.clone(),
                                e.bindings
                                    .regions
                                    .get(&n)
                                    .ok_or_else(|| {
                                        Error::new("§4.7: missing memory region".into())
                                    })?
                                    .clone(),
                            );
                        }
                        let args = list(&ca[6])?
                            .iter()
                            .map(|n| {
                                e.bindings.args.get(n).cloned().ok_or_else(|| {
                                    Error::new("§4.7: missing classical argument".into())
                                })
                            })
                            .collect::<Result<Vec<_>>>()?;
                        self.oracle_quantum(
                            callee, q, &ins, &target, &fault, &regions, &args, token, &c,
                        )?;
                    } else {
                        let mut binding = QuantumBindings {
                            borrowed: vec![e
                                .bindings
                                .borrowed
                                .iter()
                                .flatten()
                                .enumerate()
                                .filter_map(|(i, w)| (!e.borrow_used.contains(&i)).then_some(*w))
                                .collect()],
                            ..QuantumBindings::default()
                        };
                        for group in 0..2 {
                            for (p, n) in callee.params[group].iter().zip(list(&ca[group + 1])?) {
                                binding.bundles.insert(p.name.clone(), e.bundle(&n)?);
                            }
                        }
                        for (p, n) in callee.params[2].iter().zip(list(&ca[3])?) {
                            binding
                                .regions
                                .insert(p.name.clone(), e.bindings.regions[&n].clone());
                        }
                        for (p, n) in callee.params[3].iter().zip(list(&ca[4])?) {
                            binding
                                .args
                                .insert(p.name.clone(), e.bindings.args[&n].clone());
                        }
                        self.action_internal(
                            &callee.name,
                            q,
                            &binding,
                            adjoint ^ call.adjoint,
                            &c,
                        )?;
                    }
                }
                "qinplace" => {
                    let g = &a[if adjoint { 1 } else { 0 }];
                    let gi = &a[if adjoint { 0 } else { 1 }];
                    let x = e.bundle(&a[3])?;
                    let token = e
                        .bindings
                        .args
                        .get(&a[2])
                        .and_then(|v| {
                            if let Argument::Token(b) = v {
                                Some(*b)
                            } else {
                                None
                            }
                        })
                        .ok_or_else(|| Error::new("§4.5: qinplace token".into()))?;
                    let gf = &self.functions[g.trim_start_matches('@')];
                    let gif = &self.functions[gi.trim_start_matches('@')];
                    // §4.5 requires, under tk, FAULT_NONE on every input and
                    // g∘ginv = ginv∘g = id. Decided here exactly, on the full
                    // n-bit domain at this token's budget, before any state change.
                    self.verify_inverse_pair(gf, gif, token)?;
                    let fw = gf.params[2][0].ty.width(self.ptr_width)? as usize;
                    let t = q.allocate(x.len())?;
                    let fault = q.allocate(fw)?;
                    self.oracle_quantum(
                        gf,
                        q,
                        std::slice::from_ref(&x),
                        &t,
                        &fault,
                        &BTreeMap::new(),
                        &[],
                        token,
                        &context,
                    )?;
                    bundle_exchange(q, &x, &t, &context)?;
                    self.oracle_quantum(
                        gif,
                        q,
                        &[x],
                        &t,
                        &fault,
                        &BTreeMap::new(),
                        &[],
                        token,
                        &context,
                    )?;
                    q.release(&t)?;
                    q.release(&fault)?;
                }
                _ => return err("§4.8", format!("unimplemented Q instruction {op}")),
            }
        }
        Ok(())
    }
    /// §4.5: both oracles are fault-free on every n-bit input under `budget`
    /// and inverse to each other; otherwise the qinplace is not executed.
    pub(crate) fn verify_inverse_pair(
        &self,
        g: &Function,
        ginv: &Function,
        budget: crate::Budget,
    ) -> Result<()> {
        let n = g.params[0][0].ty.bits(self.ptr_width)?;
        let run = |f: &Function, x: &crate::BitVector| -> Result<crate::BitVector> {
            let input = crate::OracleVectorInput {
                ins: vec![x.clone()],
                ..Default::default()
            };
            let r = self.run_oracle_vectors(&f.name, &input, budget)?;
            if !r.code.is_zero() {
                return err(
                    "§4.5",
                    format!(
                        "qinplace oracle @{} faults with code {} on input {} under this token",
                        f.name,
                        r.code.to_decimal(),
                        x.to_decimal()
                    ),
                );
            }
            Ok(r.payload)
        };
        let mut x = crate::BitVector::zero(n)?;
        let one = crate::BitVector::from_u128(n, 1)?;
        loop {
            for (first, second) in [(g, ginv), (ginv, g)] {
                if run(second, &run(first, &x)?)? != x {
                    return err(
                        "§4.5",
                        format!(
                            "@{} is not the inverse of @{} at input {}",
                            second.name,
                            first.name,
                            x.to_decimal()
                        ),
                    );
                }
            }
            x = x.binary("add", &one)?;
            if x.is_zero() {
                return Ok(());
            }
        }
    }
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn oracle_quantum(
        &self,
        callee: &Function,
        q: &mut QuantumState,
        ins: &[Vec<Wire>],
        target: &[Wire],
        fault: &[Wire],
        regions: &BTreeMap<String, QuantumRegion>,
        args: &[Argument],
        budget: crate::Budget,
        controls: &[(Wire, bool)],
    ) -> Result<()> {
        let input_positions = ins
            .iter()
            .map(|w| w.iter().map(|w| q.position(*w)).collect::<Result<Vec<_>>>())
            .collect::<Result<Vec<_>>>()?;
        let target = target
            .iter()
            .map(|w| q.position(*w))
            .collect::<Result<Vec<_>>>()?;
        let fault = fault
            .iter()
            .map(|w| q.position(*w))
            .collect::<Result<Vec<_>>>()?;
        let controls = controls
            .iter()
            .map(|(w, v)| Ok((q.position(*w)?, *v)))
            .collect::<Result<Vec<_>>>()?;
        let regions: BTreeMap<_, _> = regions
            .iter()
            .map(|(n, r)| {
                Ok((
                    n.clone(),
                    r.cells
                        .iter()
                        .map(|c| c.iter().map(|w| q.position(*w)).collect::<Result<Vec<_>>>())
                        .collect::<Result<Vec<_>>>()?,
                ))
            })
            .collect::<Result<_>>()?;
        q.permute(|bits| {
            let mut result = bits.to_vec();
            if controls.iter().any(|(p, v)| bits[*p] != *v) {
                return Ok(result);
            }
            let vectors = input_positions
                .iter()
                .map(|positions| {
                    let mut value = crate::BitVector::zero(positions.len())?;
                    for (i, p) in positions.iter().enumerate() {
                        value.set(i, bits[*p])?;
                    }
                    Ok(value)
                })
                .collect::<Result<Vec<_>>>()?;
            let mut input = crate::OracleVectorInput {
                ins: vectors,
                args: args.to_vec(),
                ..Default::default()
            };
            for (n, cells) in &regions {
                // Oracle interface regions have W = 8 (§7.2): one byte per cell.
                let bytes = cells
                    .iter()
                    .map(|cell| {
                        if cell.len() != 8 {
                            return err("§7.2", "oracle interface region cells must be wire[8]");
                        }
                        Ok(cell
                            .iter()
                            .enumerate()
                            .fold(0u8, |b, (i, p)| b | (u8::from(bits[*p]) << i)))
                    })
                    .collect::<Result<_>>()?;
                input.regions.insert(n.clone(), bytes);
            }
            let mut machine = crate::OracleVectorMachine::new(self, &callee.name, &input, budget)?;
            let entry = machine.snapshot();
            for _ in 0..budget.steps {
                machine.forward_step()?;
            }
            let outcome = machine.outcome()?;
            for _ in 0..budget.steps {
                machine.inverse_step()?;
            }
            if machine.snapshot() != entry || !machine.frames_clean() {
                return err("§5.2", "oracle workspace failed prefix reversal");
            }
            for (i, p) in target.iter().enumerate() {
                result[*p] ^= outcome.payload.bit(i)?;
            }
            for (i, p) in fault.iter().enumerate() {
                result[*p] ^= outcome.code.bit(i)?;
            }
            Ok(result)
        })
    }
}
fn gather_regions<'a>(seq: &'a [Instruction], out: &mut Vec<&'a Instruction>) {
    for i in seq {
        if i.op == "qregion" {
            out.push(i);
        }
        for b in &i.bodies {
            gather_regions(b, out);
        }
    }
}
/// A structured execution plan, indexed by sequence position and dynamic
/// loop entry, never by static source line. Width environments are finite
/// deterministic maps: repeated entries share a cycle, with no trip-count cap.
#[derive(Default)]
struct LifetimeWidths {
    produced: Option<usize>,
    bodies: Vec<Vec<LifetimeWidths>>,
    cycle_start: usize,
}
impl LifetimeWidths {
    fn iteration(&self, index: usize) -> &[LifetimeWidths] {
        let index = if index < self.bodies.len() {
            index
        } else {
            self.cycle_start + (index - self.cycle_start) % (self.bodies.len() - self.cycle_start)
        };
        &self.bodies[index]
    }
}
fn infer_widths(
    seq: &[Instruction],
    e: &Environment,
    live: &mut BTreeMap<String, usize>,
) -> Result<Vec<LifetimeWidths>> {
    let mut plan = Vec::new();
    for i in seq {
        let mut step = LifetimeWidths::default();
        let width = match i.op.as_str() {
            "qalloc" | "qborrow" => Some(
                i.operands[1]
                    .parse()
                    .map_err(|_| Error::new("§4.3: allocation width".into()))?,
            ),
            "qcopy" => Some(if i.operands[0].contains('[') {
                1
            } else {
                *live
                    .get(&i.operands[0])
                    .ok_or_else(|| Error::new("§3.3: copy source width".into()))?
            }),
            "qload" => Some(
                e.bindings
                    .regions
                    .get(&i.operands[0])
                    .ok_or_else(|| Error::new("§4.6: load region width".into()))?
                    .cell_width as usize,
            ),
            _ => None,
        };
        if let Some(width) = width {
            if live.insert(i.lhs[0].clone(), width).is_some() {
                return err("§3.3", "live bundle produced twice");
            }
            step.produced = Some(width);
        }
        let consumed = match i.op.as_str() {
            "qfree" | "qreturn" => Some(&i.operands[0]),
            "quncopy" => Some(&i.operands[1]),
            "qunload" => Some(&i.operands[2]),
            _ => None,
        };
        if let Some(name) = consumed {
            step.produced = Some(
                live.remove(name)
                    .ok_or_else(|| Error::new("§3.3: missing consumed lifetime width".into()))?,
            );
        }
        if i.op == "qloop" {
            let n = i.operands[0]
                .parse::<usize>()
                .map_err(|_| Error::new("§4.6: loop bound exceeds host index space".into()))?;
            let mut seen = BTreeMap::new();
            let mut entries: Vec<BTreeMap<String, usize>> = Vec::new();
            while step.bodies.len() < n {
                if let Some(&start) = seen.get(live) {
                    step.cycle_start = start;
                    let period = step.bodies.len() - start;
                    *live = entries[start + (n - start) % period].clone();
                    break;
                }
                seen.insert(live.clone(), step.bodies.len());
                entries.push(live.clone());
                step.bodies.push(infer_widths(&i.bodies[0], e, live)?);
                step.cycle_start = step.bodies.len();
            }
            // n=0 has no lifetime effects. Lexical qregions were hoisted above.
        } else {
            for body in &i.bodies {
                step.bodies.push(infer_widths(body, e, live)?);
            }
        }
        plan.push(step);
    }
    Ok(plan)
}
fn bundle_xor(
    q: &mut QuantumState,
    source: &[Wire],
    target: &[Wire],
    context: &[(Wire, bool)],
) -> Result<()> {
    if source.len() != target.len() {
        return err("§3.3", "XOR bundle widths differ");
    }
    for (s, t) in source.iter().zip(target) {
        q.controlled_gate("cx", &[*s, *t], None, context)?;
    }
    Ok(())
}
fn bundle_exchange(
    q: &mut QuantumState,
    a: &[Wire],
    b: &[Wire],
    context: &[(Wire, bool)],
) -> Result<()> {
    if a.len() != b.len() {
        return err("§3.3", "exchange bundle widths differ");
    }
    for (s, t) in a.iter().zip(b) {
        q.controlled_gate("swap", &[*s, *t], None, context)?;
    }
    Ok(())
}
