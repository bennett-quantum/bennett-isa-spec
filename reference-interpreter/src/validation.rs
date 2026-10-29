//! Operand schemas and static discipline. Dynamic cleanliness is also checked
//! by executors; accepting a text object does not waive those obligations.
use crate::{
    err,
    object::{integer, is_binop, is_div, list, name},
    Error, Function, Instruction, Object, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

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
        if state.bounds != f.bounds {
            return err("§5.3", "declared bounds do not equal loops in text order");
        }
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
                    s.predicates.push(a[0].clone());
                    for body in &i.bodies {
                        self.validate_action_sequence(f, body, s)?;
                    }
                    if s.predicates.pop().as_deref() != Some(&a[0]) {
                        return err("§3.5", "qif body altered predicate stack");
                    }
                }
                "qloop" => {
                    if !i.lhs.is_empty() || !matches!(a.len(), 1 | 3) || i.bodies.len() != 1 {
                        return err("§4.8", "qloop arity/body mismatch");
                    }
                    let n = a[0].parse::<u64>().map_err(|_| {
                        Error("§4.8: loop count must be nonnegative literal".into())
                    })?;
                    s.bounds.push(n);
                    if a.len() == 3 {
                        if a[1] != "while" || s.wires(&a[2])?.len() as u64 != n {
                            return err("§4.6", "qloop selector width mismatch");
                        }
                        s.predicates.push(a[2].clone());
                    }
                    self.validate_action_sequence(f, &i.bodies[0], s)?;
                    if a.len() == 3 && s.predicates.pop().as_deref() != Some(&a[2]) {
                        return err("§3.5", "loop predicate stack mismatch");
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
                    if !name(&a[0]) {
                        return err("§4.8", "invalid wire class");
                    }
                    let n = a[1]
                        .parse::<u16>()
                        .map_err(|_| Error("§3.3: invalid allocation width".into()))?;
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
                        },
                    )?;
                }
                "qfree" | "qreturn" => {
                    arity(1, 0)?;
                    let c = s
                        .created
                        .get(&a[0])
                        .ok_or_else(|| Error("§3.3: release of non-workspace value".into()))?;
                    if c.kind != if op == "qreturn" { "qborrow" } else { "qalloc" } {
                        return err("§5.1", "allocation/release kind mismatch");
                    }
                    if s.wires(&a[0])?
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
                    let c = Creation {
                        kind: op.into(),
                        sources: vec![a[0].clone()],
                        versions: s.snapshot(&wires),
                        controls: s.predicates.clone(),
                    };
                    s.produce(&i.lhs[0], Type::Wire(wires.len() as u16), c)?;
                }
                "quncopy" => {
                    arity(2, 0)?;
                    s.disjoint(&[&a[0], &a[1]])?;
                    let c = s
                        .created
                        .get(&a[1])
                        .ok_or_else(|| Error("§6.2: unmatched quncopy".into()))?;
                    if c.kind != "qcopy"
                        || c.sources != vec![a[0].clone()]
                        || c.versions != s.snapshot(&s.wires(&a[0])?)
                        || c.controls != s.predicates
                        || s.wires(&a[1])?
                            .iter()
                            .any(|w| !s.versions.get(w).is_none_or(Vec::is_empty))
                        || s.wires(&a[0])?.iter().any(|w| s.closed.contains(w))
                    {
                        return err("§6.2", "quncopy outside its computing window");
                    }
                    s.consume(&a[1])?;
                }
                "qregion" => {
                    if !i.lhs.is_empty() || !matches!(a.len(), 3 | 4) || !name(&a[0]) {
                        return err("§4.8", "qregion operand schema");
                    }
                    let aw = a[1]
                        .parse::<u16>()
                        .map_err(|_| Error("§4.8: qregion address width".into()))?;
                    let w = a[2]
                        .parse::<u16>()
                        .map_err(|_| Error("§4.8: qregion cell width".into()))?;
                    if aw > 127 || w == 0 {
                        return err("§3.3", "invalid region dimensions");
                    }
                    if a.len() == 4 {
                        let img = a[3]
                            .strip_prefix('@')
                            .and_then(|n| self.data.get(n))
                            .ok_or_else(|| Error("§4.6: unknown qregion image".into()))?;
                        let bits = (1u128 << aw)
                            .checked_mul(w as u128)
                            .ok_or_else(|| Error("§4.6: image size overflow".into()))?;
                        if bits % 8 != 0 || bits / 8 != img.len() as u128 {
                            return err("§4.6", "image size mismatch");
                        }
                    }
                    if s.types.insert(a[0].clone(), Type::Region(aw, w)).is_some() {
                        return err("§3.3", "region name already defined");
                    }
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
                        s.produce(
                            &i.lhs[0],
                            Type::Wire(w),
                            Creation {
                                kind: op.into(),
                                sources: a.clone(),
                                versions: s.snapshot(&deps),
                                controls: s.predicates.clone(),
                            },
                        )?;
                    } else {
                        if s.wires(&a[2])?.len() != w as usize {
                            return err("§3.3", "memory value width mismatch");
                        }
                        s.disjoint(&[&a[1], &a[2]])?;
                        if op == "qunload" {
                            let c = s
                                .created
                                .get(&a[2])
                                .ok_or_else(|| Error("§6.2: unmatched qunload".into()))?;
                            let mut deps = s.wires(&a[1])?;
                            deps.push(format!("{}[*]", a[0]));
                            if c.kind != "qload"
                                || c.sources != a[..2]
                                || c.controls != s.predicates
                                || c.versions != s.snapshot(&deps)
                            {
                                return err("§6.2", "qunload outside computing window");
                            }
                            s.consume(&a[2])?;
                        } else {
                            if s.inputs.contains(&a[0]) {
                                return err("§5.1", "in region cannot be modified by action");
                            }
                            s.versions
                                .entry(format!("{}[*]", a[0]))
                                .or_default()
                                .push(format!("{}:{}", op, i.line));
                            if op == "qexchm" {
                                s.change(&a[2], format!("{op}:{}", a.join(" ")), false)?;
                            }
                        }
                    }
                }
                "qinplace" => {
                    arity(4, 0)?;
                    for callee in &a[..2] {
                        let callee = callee
                            .strip_prefix('@')
                            .and_then(|n| self.functions.get(n))
                            .filter(|f| f.kind == "oracle")
                            .ok_or_else(|| Error("§4.5: qinplace requires oracle pair".into()))?;
                        if callee.params[0].len() != 1
                            || callee.params[0][0].ty != callee.params[1][0].ty
                        {
                            return err(
                                "§4.5",
                                "qinplace requires equal-width single-input oracles",
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
                    let callee = self
                        .functions
                        .get(call.instruction.operands[0].trim_start_matches('@'));
                    if f.kind == "perm_action" && callee.is_some_and(|c| c.kind == "action") {
                        return err("§5.3", "permutation action calls non-permutation action");
                    }
                    let ca = &call.instruction.operands;
                    let targets = if call.instruction.op == "oracle_call" {
                        vec![ca[3].clone(), ca[4].clone()]
                    } else {
                        list(&ca[1])?
                    };
                    for target in targets {
                        let other = callee.is_some_and(|c| c.kind == "action");
                        s.change(
                            &target,
                            format!("call:{}:{}:{}", call.adjoint, ca.join(" "), i.line),
                            other,
                        )?;
                    }
                    if call.control.is_some() {
                        s.predicates.pop();
                    }
                }
                _ if gate(op) => {
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
                    let version = format!(
                        "{op}:{}:{:?}:{:?}",
                        a.join(" "),
                        s.snapshot(
                            &controls
                                .iter()
                                .flat_map(|w| s.wires(w).unwrap_or_default())
                                .collect::<Vec<_>>()
                        ),
                        s.predicates
                    );
                    for t in targets {
                        s.check_target(&t, diagonal)?;
                        if !diagonal {
                            s.change(&t, version.clone(), other)?;
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
                        s.snapshot(&s.wires(&a[0])?),
                        s.predicates
                    );
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
        let callee = self
            .functions
            .get(n)
            .ok_or_else(|| Error(format!("§9.3: undefined region {n}")))?;
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
                    (Type::Wire(n), Type::Qreg(m)) => *n as u64 == *m,
                    (Type::Region(a, w), Type::Region(b, z)) => a == b && w == z,
                    (Type::Region(a, w), Type::Qreg(n)) => {
                        *a < 64 && (1u128 << *a) * (*w as u128) == *n as u128
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
                    let caller = f
                        .params
                        .iter()
                        .flatten()
                        .find(|p| &p.name == v)
                        .ok_or_else(|| {
                            Error("§3.3: memory call operand is not caller interface".into())
                        })?;
                    if p.role.as_deref() == Some("mut") && caller.role.as_deref() == Some("in") {
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
        for b in &f.blocks {
            for (ordinal, i) in b.instructions.iter().enumerate() {
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
                    if !matches!(t, Type::Int(_)) || !integer(&a[0]) {
                        return err("§4.8", "invalid classical const");
                    }
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
                    same(&a[0], &a[1])?;
                    result = Some(Type::Int(1));
                } else if let Some((cast, suffix)) = op
                    .split_once('.')
                    .filter(|(c, _)| ["zext", "sext", "trunc", "ptrtoint", "load"].contains(c))
                {
                    arity(1, 1)?;
                    let t = Type::parse(suffix)?;
                    let to = t.width(self.ptr_width)?;
                    let from = rd(&a[0])?;
                    if cast == "load" || cast == "ptrtoint" {
                        if from != Type::Ptr {
                            return err("§3.3", "pointer operand required");
                        }
                        if cast == "load" && to % 8 != 0 {
                            return err("§3.3", "load width must be byte multiple");
                        }
                    } else {
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
                            if t.width(self.ptr_width)? % 8 != 0
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
                                let (label, args) = label.split_once('(').ok_or_else(|| {
                                    Error("§4.8: br target needs argument list".into())
                                })?;
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
                            let callee = a[0]
                                .strip_prefix('@')
                                .and_then(|n| self.functions.get(n))
                                .ok_or_else(|| Error("§9.3: undefined classical callee".into()))?;
                            if !matches!(callee.kind.as_str(), "cc" | "r")
                                || op == "uncall" && callee.kind != "r"
                            {
                                return err("§4.8", "classical call tier mismatch");
                            }
                            let vs = if op == "uncall" {
                                arity(3, 0)?;
                                list(&a[1])?
                            } else if a.len() == 2 && a[1].starts_with('(') {
                                list(&a[1])?
                            } else {
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
                                    if types.insert(n.clone(), p.ty.clone()).is_some() {
                                        return err("§3.3", "SSA name defined twice");
                                    }
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
                            for n in &a[1..] {
                                rd(n)?;
                            }
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
                            if service.is_none_or(|s| !s.contains_key(&a[0])) {
                                return err("§10", "hostcall service undeclared");
                            }
                            result = Some(Type::Int(self.ptr_width));
                        }
                        "qreg.new" => {
                            arity(2, 1)?;
                            if f.kind != "cc" {
                                return err("§4.8", "qreg instructions are CC only");
                            }
                            let n = a[1]
                                .parse::<u64>()
                                .map_err(|_| Error("§3.3: qreg literal width".into()))?;
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
                            let lo = a[1]
                                .parse::<u64>()
                                .map_err(|_| Error("§3.3: slice offset".into()))?;
                            let len = a[2]
                                .parse::<u64>()
                                .map_err(|_| Error("§3.3: slice length".into()))?;
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
                    if types.insert(i.lhs[0].clone(), ty).is_some() {
                        return err("§3.3", "SSA definition reused");
                    }
                }
            }
        }
        Ok(())
    }
    pub(crate) fn validate_call_graph(&self) -> Result<()> {
        fn calls<'a>(i: &'a Instruction, output: &mut Vec<&'a str>) {
            if let Ok(c) = normalize_call(i) {
                if matches!(c.instruction.op.as_str(), "action_call" | "oracle_call") {
                    if let Some(n) = i.operands.iter().find_map(|s| s.strip_prefix('@')) {
                        output.push(n);
                    }
                }
            }
            for body in &i.bodies {
                for child in body {
                    calls(child, output);
                }
            }
        }
        fn visit(
            obj: &Object,
            n: &str,
            active: &mut BTreeSet<String>,
            done: &mut BTreeSet<String>,
        ) -> Result<()> {
            if done.contains(n) {
                return Ok(());
            }
            if !active.insert(n.into()) {
                return err("§5.1", "recursive region call graph");
            }
            let f = &obj.functions[n];
            let mut children = Vec::new();
            for i in f.instructions() {
                calls(i, &mut children);
            }
            for child in children {
                if obj.functions.contains_key(child) {
                    visit(obj, child, active, done)?;
                }
            }
            active.remove(n);
            done.insert(n.into());
            Ok(())
        }
        let mut done = BTreeSet::new();
        for f in self
            .functions
            .values()
            .filter(|f| matches!(f.kind.as_str(), "action" | "perm_action" | "oracle"))
        {
            visit(self, &f.name, &mut BTreeSet::new(), &mut done)?;
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
                e.get("count")?.uint()?;
            }
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
        }
        Ok(())
    }
}

#[derive(Clone)]
struct Creation {
    kind: String,
    sources: Vec<String>,
    versions: Vec<(String, Vec<String>)>,
    controls: Vec<String>,
}
struct Discipline {
    types: BTreeMap<String, Type>,
    inputs: BTreeSet<String>,
    predicates: Vec<String>,
    created: BTreeMap<String, Creation>,
    versions: BTreeMap<String, Vec<String>>,
    closed: BTreeSet<String>,
    bounds: Vec<u64>,
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
            closed: BTreeSet::new(),
            bounds: Vec::new(),
        }
    }
    fn wires(&self, n: &str) -> Result<Vec<String>> {
        let (base, index) = if let Some((base, index)) = n.split_once('[') {
            let index = index
                .strip_suffix(']')
                .and_then(|s| s.parse::<u16>().ok())
                .ok_or_else(|| Error("§3.3: invalid indexed wire".into()))?;
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
            .map(|w| (w.clone(), self.versions.get(w).cloned().unwrap_or_default()))
            .collect()
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
        self.wires(n)?;
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
        for w in self.wires(n)? {
            if other {
                self.closed.insert(w.clone());
            }
            let log = self.versions.entry(w).or_default();
            if !other && log.last() == Some(&version) {
                log.pop();
            } else {
                log.push(version.clone());
            }
        }
        Ok(())
    }
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
