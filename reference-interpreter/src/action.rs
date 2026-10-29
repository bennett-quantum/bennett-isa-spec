use crate::{
    err,
    object::list,
    quantum::Wire,
    validation::{gate, normalize_call},
    Argument, Error, Function, Instruction, Object, OracleInput, OracleMachine, QuantumBindings,
    QuantumRegion, QuantumState, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

struct Environment {
    bindings: QuantumBindings,
    widths: BTreeMap<String, usize>,
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
                .ok_or_else(|| Error("§3.3: invalid wire index".into()))?;
            let ws = self
                .bindings
                .bundles
                .get(base)
                .ok_or_else(|| Error(format!("§3.3: unknown live bundle {base}")))?;
            return ws
                .get(index)
                .map(|w| vec![*w])
                .ok_or_else(|| Error("§3.3: wire index out of range".into()));
        }
        self.bindings
            .bundles
            .get(n)
            .cloned()
            .ok_or_else(|| Error(format!("§3.3: unknown live bundle {n}")))
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
        self.widths.insert(n.into(), wires.len());
        self.bindings.bundles.insert(n.into(), wires);
        Ok(())
    }
}
impl Object {
    pub(crate) fn permutation_action_values(
        &self,
        name: &str,
        mutables: &[u128],
        inputs: &[u128],
        regions: &BTreeMap<String, Vec<u8>>,
        args: &[Argument],
        adjoint: bool,
    ) -> Result<Vec<u128>> {
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
                    if i < 128 && v & (1u128 << i) != 0 {
                        q.gate("x", &[*w], None, &[])?;
                    }
                }
                b.bundles.insert(p.name.clone(), wires);
            }
        }
        for p in &f.params[2] {
            let Type::Region(a, w) = p.ty else {
                unreachable!("region")
            };
            let bytes = regions
                .get(&p.name)
                .ok_or_else(|| Error("§4.2: missing lifted action region".into()))?;
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
        let capacity=self.borrow_capacity(name)?;
        let idle=q.allocate(capacity)?;if !idle.is_empty(){b.borrowed.push(idle.clone());}
        self.execute_action(name, &mut q, &b, adjoint, &[])?;
        q.release(&idle)?;
        f.params[0]
            .iter()
            .map(|p| q.read_basis(&b.bundles[&p.name]))
            .collect()
    }
    fn borrow_capacity(&self,root:&str)->Result<usize>{
        fn scan(seq:&[Instruction],count:&mut usize,children:&mut Vec<String>)->Result<()>{for i in seq{if i.op=="qborrow"{*count=count.checked_add(i.operands[1].parse::<usize>().map_err(|_|Error("§5.6: borrow count exceeds host storage".into()))?).ok_or_else(||Error("§5.6: borrow capacity overflow".into()))?;}if let Ok(call)=normalize_call(i){if call.instruction.op=="action_call"{children.push(call.instruction.operands[0].trim_start_matches('@').into());}}for body in &i.bodies{scan(body,count,children)?;}}Ok(())}
        let mut count=0;let mut pending=vec![root.to_string()];let mut visited=BTreeSet::new();while let Some(name)=pending.pop(){if !visited.insert(name.clone()){continue;}let function=self.functions.get(&name).ok_or_else(||Error(format!("§9.3: unresolved action @{name}")))?;for block in &function.blocks{scan(&block.instructions,&mut count,&mut pending)?;}}Ok(count)
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
        let f = self
            .functions
            .get(name)
            .filter(|f| matches!(f.kind.as_str(), "action" | "perm_action"))
            .ok_or_else(|| Error(format!("§5.3: unknown action @{name}")))?;
        let mut used = BTreeSet::new();
        for p in f.params[0].iter().chain(&f.params[1]) {
            let w = bindings
                .bundles
                .get(&p.name)
                .ok_or_else(|| Error(format!("§5.1: missing bundle {}", p.name)))?;
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
                unreachable!("validated region")
            };
            let r = bindings
                .regions
                .get(&p.name)
                .ok_or_else(|| Error("§5.1: missing interface region".into()))?;
            if r.address_width != a
                || r.cell_width != w
                || r.cells.len() as u128 != 1u128 << a
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
                .ok_or_else(|| Error("§5.1: missing action classical argument".into()))?;
            match (&p.ty, arg) {
                (Type::Token, Argument::Token(b)) => self.validate_budget(*b)?,
                (Type::Int(w), Argument::Bits(n)) if *w == 128 || *n < 1u128 << w => {}
                (Type::F64, Argument::Float(a)) if a.is_finite() => {}
                _ => return err("§3.3", "action argument type mismatch"),
            }
        }
        let mut env = Environment {
            bindings: bindings.clone(),
            widths: bindings
                .bundles
                .iter()
                .map(|(n, w)| (n.clone(), w.len()))
                .collect(),
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
            let a = i.operands[1].parse::<u16>().expect("address width");
            let w = i.operands[2].parse::<u16>().expect("cell width");
            let count = usize::try_from(1u128 << a)
                .map_err(|_| Error("§4.6: local region exceeds host address space".into()))?;
            let n = count
                .checked_mul(w as usize)
                .ok_or_else(|| Error("§4.6: region bit size overflow".into()))?;
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
        for b in &f.blocks {
            infer_widths(&b.instructions, &mut env)?;
        }
        let instructions: Vec<_> = f
            .blocks
            .iter()
            .flat_map(|b| b.instructions.iter().cloned())
            .collect();
        self.action_sequence(&instructions, q, &mut env, adjoint, controls)?;
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
        Ok(())
    }
    fn action_sequence(
        &self,
        sequence: &[Instruction],
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
            let a = &i.operands;
            let context = e.controls(inherited);
            let op = i.op.as_str();
            if gate(op) {
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
                    self.action_sequence(&i.bodies[0], q, e, adjoint, &yes)?;
                    let mut no = context.clone();
                    no.push((w, false));
                    self.action_sequence(&i.bodies[1], q, e, adjoint, &no)?;
                }
                "qloop" => {
                    let n = a[0]
                        .parse::<usize>()
                        .map_err(|_| Error("§4.6: loop bound exceeds host index space".into()))?;
                    if a.len() == 1 {
                        for _ in 0..n {
                            self.action_sequence(&i.bodies[0], q, e, adjoint, &context)?;
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
                                self.action_sequence(&i.bodies[0], q, e, true, &c)?;
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
                                self.action_sequence(&i.bodies[0], q, e, false, &c)?;
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
                        let n = e.widths[name];
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
                        let n = e.widths[name];
                        let pool=e.bindings.borrowed.iter().flatten().copied().collect::<Vec<_>>();
                        let indices=(0..pool.len()).filter(|i|!e.borrow_used.contains(i)).take(n).collect::<Vec<_>>();
                        if indices.len()!=n{return err("§5.6","designated idle set lacks requested borrowed wires");}
                        let wires=indices.iter().map(|i|pool[*i]).collect();e.borrow_used.extend(&indices);e.borrowed.insert(name.into(),indices);e.produce(name,wires)?;
                    } else {
                        let indices = e
                            .borrowed
                            .remove(name)
                            .ok_or_else(|| Error("§5.6: unmatched return".into()))?;
                        for index in indices{e.borrow_used.remove(&index);}
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
                        .ok_or_else(|| Error("§4.6: quantum region not found".into()))?;
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
                            .ok_or_else(|| Error("§4.7: missing oracle token".into()))?;
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
                                    .ok_or_else(|| Error("§4.7: missing memory region".into()))?
                                    .clone(),
                            );
                        }
                        let args =
                            list(&ca[6])?
                                .iter()
                                .map(|n| {
                                    e.bindings.args.get(n).cloned().ok_or_else(|| {
                                        Error("§4.7: missing classical argument".into())
                                    })
                                })
                                .collect::<Result<Vec<_>>>()?;
                        self.oracle_quantum(
                            callee, q, &ins, &target, &fault, &regions, &args, token, &c,
                        )?;
                    } else {
                        let mut binding = QuantumBindings {
                            borrowed: vec![e.bindings.borrowed.iter().flatten().enumerate().filter_map(|(i,w)|(!e.borrow_used.contains(&i)).then_some(*w)).collect()],
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
                        .ok_or_else(|| Error("§4.5: qinplace token".into()))?;
                    let gf = &self.functions[g.trim_start_matches('@')];
                    let gif = &self.functions[gi.trim_start_matches('@')];
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
            let mut input = OracleInput::bits(
                &input_positions
                    .iter()
                    .map(|p| read_bits(bits, p))
                    .collect::<Result<Vec<_>>>()?,
            );
            input.args = args.to_vec();
            for (n, cells) in &regions {
                input.regions.insert(
                    n.clone(),
                    cells
                        .iter()
                        .map(|p| read_bits(bits, p).map(|v| v as u8))
                        .collect::<Result<Vec<_>>>()?,
                );
            }
            let mut machine = OracleMachine::new(self, &callee.name, &input, budget)?;
            let before = machine.snapshot();
            for _ in 0..budget.steps {
                machine.forward_step()?;
            }
            let outcome = machine.outcome();
            for _ in 0..budget.steps {
                machine.inverse_step()?;
            }
            if machine.snapshot() != before {
                return err("§5.2", "oracle ancillae did not restore");
            }
            for (i, p) in target.iter().enumerate() {
                if i < 128 {
                    result[*p] ^= outcome.payload & (1u128 << i) != 0;
                }
            }
            for (i, p) in fault.iter().enumerate() {
                if i < 128 {
                    result[*p] ^= outcome.code & (1u128 << i) != 0;
                }
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
fn infer_widths(seq: &[Instruction], e: &mut Environment) -> Result<()> {
    for i in seq {
        match i.op.as_str() {
            "qalloc" | "qborrow" => {
                e.widths.insert(
                    i.lhs[0].clone(),
                    i.operands[1]
                        .parse()
                        .map_err(|_| Error("§4.3: allocation width".into()))?,
                );
            }
            "qcopy" => {
                let n = if i.operands[0].contains('[') {
                    1
                } else {
                    *e.widths
                        .get(&i.operands[0])
                        .ok_or_else(|| Error("§3.3: copy source width".into()))?
                };
                e.widths.insert(i.lhs[0].clone(), n);
            }
            "qload" => {
                let r = e
                    .bindings
                    .regions
                    .get(&i.operands[0])
                    .ok_or_else(|| Error("§4.6: load region width".into()))?;
                e.widths.insert(i.lhs[0].clone(), r.cell_width as usize);
            }
            _ => {}
        }
        for b in &i.bodies {
            infer_widths(b, e)?;
        }
    }
    Ok(())
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
fn read_bits(bits: &[bool], positions: &[usize]) -> Result<u128> {
    if positions.len() > 128 {
        return err("§3.3", "integer evaluation exceeds 128 bits");
    }
    Ok(positions
        .iter()
        .enumerate()
        .fold(0, |n, (i, p)| n | ((bits[*p] as u128) << i)))
}
