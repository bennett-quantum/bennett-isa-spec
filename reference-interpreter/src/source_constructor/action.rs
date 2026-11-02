//! Structured source formation, before catalog recipe instantiation.
use super::*;
use bennett_isa_conformance::ref_language::{SourceBody, SourcePoint, SourceSymbol};
#[derive(Clone)]
struct Region {
    cells: String,
    address: usize,
    cell: usize,
    origin: Value,
}
#[derive(Clone, Default)]
struct Env {
    wires: BTreeMap<String, (String, usize)>,
    regions: BTreeMap<String, Region>,
    predicates: Vec<Value>,
    iteration: Vec<usize>,
}
struct Compiler<'a> {
    object: &'a crate::Object,
    values: &'a BTreeMap<String, Value>,
    declarations: Vec<(String, usize, String, String)>,
    symbols: Vec<SourceSymbol>,
    counter: usize,
    hashes: BTreeMap<String, Option<String>>,
    symbol: String,
    points: BTreeMap<String, SourcePoint>,
    locations: BTreeMap<usize, (String, usize)>,
}
impl Env {
    fn wire(&self, name: &str) -> Result<Value> {
        if name == "()" {
            return Ok(array([]));
        }
        if let Some((base, index)) = name.split_once('[') {
            let index = index
                .strip_suffix(']')
                .ok_or_else(|| Error::new("§11.1: malformed wire index".into()))?;
            let (alias, _) = self
                .wires
                .get(base)
                .ok_or_else(|| Error::new(format!("§11.1: unbound source wire {base}")))?;
            return Ok(text(format!("{alias}[{index}]")));
        }
        Ok(text(
            &self
                .wires
                .get(name)
                .ok_or_else(|| Error::new(format!("§11.1: unbound source wire {name}")))?
                .0,
        ))
    }
    fn width(&self, name: &str) -> Result<usize> {
        if name.contains('[') {
            return Ok(1);
        }
        self.wires
            .get(name)
            .map(|v| v.1)
            .ok_or_else(|| Error::new(format!("§11.1: missing source width {name}")))
    }
    fn indexed(&self, name: &str, i: usize) -> Result<Value> {
        if name.contains('[') {
            if i != 0 {
                return Err(Error::new("§11.1: scalar index out of range".into()));
            }
            self.wire(name)
        } else {
            let (alias, w) = self
                .wires
                .get(name)
                .ok_or_else(|| Error::new("§11.1: unknown source wire".into()))?;
            if i >= *w {
                return Err(Error::new("§11.1: source bit out of range".into()));
            }
            Ok(bit(alias, i))
        }
    }
    fn region(&self, name: &str, role: &str) -> Result<Value> {
        let r = self
            .regions
            .get(name)
            .ok_or_else(|| Error::new(format!("§11.1: unknown source region {name}")))?;
        Ok(obj([
            ("address", number(r.address)?),
            ("cell", number(r.cell)?),
            ("role", text(role)),
            ("origin", r.origin.clone()),
            ("cells", text(&r.cells)),
        ]))
    }
}
fn controlled(env: &Env, body: Vec<Value>) -> Vec<Value> {
    if body.is_empty() || env.predicates.is_empty() {
        body
    } else {
        vec![under(array(env.predicates.clone()), body)]
    }
}
impl Compiler<'_> {
    fn declare(&mut self, width: usize, class: &str, state: &str) -> String {
        let name = format!("$source-allocation/{}", self.counter);
        self.counter += 1;
        self.declarations
            .push((name.clone(), width, class.into(), state.into()));
        name
    }
    fn value(&self, name: &str) -> Result<Value> {
        self.values
            .get(name)
            .cloned()
            .ok_or_else(|| Error::new(format!("§11.1: missing fixed argument {name}")))
    }
    fn call(&mut self, i: &crate::Instruction, env: &Env) -> Result<Vec<Value>> {
        let normal = crate::validation::normalize_call(i)?;
        let a = &normal.instruction.operands;
        let oracle = normal.instruction.op == "oracle_call";
        let f = &self.object.functions[a[0].trim_start_matches('@')];
        let alias = format!("$source-call/{}", self.counter);
        self.counter += 1;
        let hash = self.hashes[&f.name]
            .clone()
            .ok_or_else(|| Error::new("§11.1: unlinked call".into()))?;
        let context = if oracle {
            Some(obj([("token", self.value(&a[1])?)]))
        } else {
            None
        };
        self.symbols.push(SourceSymbol {
            name: alias.clone(),
            hash,
            oracle_invocation: context,
            adjoint: normal.adjoint && !oracle,
        });
        let mut args = Vec::new();
        if oracle {
            for n in crate::object::list(&a[2])? {
                args.push(env.wire(&n)?);
            }
            args.extend([env.wire(&a[3])?, env.wire(&a[4])?]);
        } else {
            for group in [1usize, 2] {
                for n in crate::object::list(&a[group])? {
                    args.push(env.wire(&n)?);
                }
            }
        }
        let mem = if oracle { 5 } else { 3 };
        let parameters = if oracle { 3 } else { 2 };
        for (p, n) in f.params[parameters]
            .iter()
            .zip(crate::object::list(&a[mem])?)
        {
            args.push(env.region(&n, p.role.as_deref().unwrap_or("in"))?);
        }
        for n in crate::object::list(&a[mem + 1])? {
            args.push(self.value(&n)?);
        }
        let mut body = vec![invoke(
            if oracle { "oracle_call" } else { "action_call" },
            [text(alias), array(args)],
        )];
        if let Some(control) = normal.control {
            body = vec![under(env.wire(&control)?, body)];
        }
        Ok(controlled(env, body))
    }
    fn sequence(&mut self, seq: &[crate::Instruction], env: &mut Env) -> Result<Vec<Value>> {
        let mut out = Vec::new();
        for i in seq {
            let start = out.len();
            let a = &i.operands;
            let op = i.op.as_str();
            match op {
                "qregion" => {}
                "qpred" => env.predicates.push(env.wire(&a[0])?),
                "qunpred" => {
                    env.predicates
                        .pop()
                        .ok_or_else(|| Error::new("§11.1: predicate stack underflow".into()))?;
                }
                "qalloc" | "qborrow" => {
                    let width =
                        usize::try_from(crate::object::host_count(&a[1], "allocation width")?)
                            .map_err(|_| {
                                crate::resource_error("§11.1", "allocation width exceeds host")
                            })?;
                    let name = self.declare(
                        width,
                        &a[0],
                        if op == "qborrow" { "borrowed" } else { "clean" },
                    );
                    env.wires.insert(i.lhs[0].clone(), (name.clone(), width));
                    out.push(obj([("acquire", text(name))]));
                }
                "qfree" | "qreturn" => {
                    let (name, _) = env
                        .wires
                        .remove(&a[0])
                        .ok_or_else(|| Error::new("§11.1: unknown released value".into()))?;
                    out.push(obj([("relinquish", text(name))]));
                }
                "qcopy" => {
                    let width = env.width(&a[0])?;
                    let name = self.declare(width, "any", "clean");
                    out.push(obj([("acquire", text(&name))]));
                    let body = (0..width)
                        .map(|b| Ok(gate("cx", [env.indexed(&a[0], b)?, bit(&name, b)])))
                        .collect::<Result<Vec<_>>>()?;
                    out.extend(controlled(env, body));
                    env.wires.insert(i.lhs[0].clone(), (name, width));
                }
                "quncopy" => {
                    let (name, width) = env
                        .wires
                        .get(&a[1])
                        .cloned()
                        .ok_or_else(|| Error::new("§11.1: unknown copied value".into()))?;
                    let body = (0..width)
                        .map(|b| Ok(gate("cx", [env.indexed(&a[0], b)?, bit(&name, b)])))
                        .collect::<Result<Vec<_>>>()?;
                    out.extend(controlled(env, body));
                    out.push(obj([("relinquish", text(name))]));
                    env.wires.remove(&a[1]);
                }
                "qxor" | "qexch" => {
                    let width = env.width(&a[0])?;
                    let body = (0..width)
                        .map(|b| {
                            Ok(gate(
                                if op == "qxor" { "cx" } else { "swap" },
                                [env.indexed(&a[0], b)?, env.indexed(&a[1], b)?],
                            ))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    out.extend(controlled(env, body));
                }
                "qif" => {
                    let predicate = env.wire(&a[0])?;
                    let outer = env.predicates.clone();
                    env.predicates.push(predicate.clone());
                    out.extend(self.sequence(&i.bodies[0], env)?);
                    env.predicates = outer.clone();
                    out.extend(controlled(env, vec![gate("x", [predicate.clone()])]));
                    env.predicates.push(predicate.clone());
                    out.extend(self.sequence(&i.bodies[1], env)?);
                    env.predicates = outer;
                    out.extend(controlled(env, vec![gate("x", [predicate])]));
                }
                "qloop" => {
                    let count = usize::try_from(crate::object::host_count(&a[0], "loop count")?)
                        .map_err(|_| crate::resource_error("§11.1", "loop count exceeds host"))?;
                    let _capacity = crate::reserved::<u8>(count, "§11.1", "source loop instances")?;
                    let outer = env.predicates.clone();
                    let mut flags = None;
                    let mut computes = Vec::new();
                    if a.len() == 3 && count > 0 {
                        let name = self.declare(count, "any", "clean");
                        out.push(obj([("acquire", text(&name))]));
                        let first = gate("cx", [env.indexed(&a[2], 0)?, bit(&name, 0)]);
                        out.extend(controlled(env, vec![first.clone()]));
                        computes.push(first);
                        flags = Some(name);
                    }
                    let outer_iteration = env.iteration.clone();
                    for iteration in 0..count {
                        env.iteration.push(iteration);
                        if let Some(name) = &flags {
                            env.predicates.push(bit(name, iteration));
                        }
                        // A unit-count body adapter captures this source instance,
                        // including its immutable oracle contexts, before splicing.
                        let body = self.sequence(&i.bodies[0], env)?;
                        out.push(invoke("qloop", [number(1)?, array(body)]));
                        env.predicates = outer.clone();
                        env.iteration = outer_iteration.clone();
                        if let Some(name) = &flags {
                            if iteration + 1 < count {
                                let next = gate(
                                    "ccx",
                                    [
                                        bit(name, iteration),
                                        env.indexed(&a[2], iteration + 1)?,
                                        bit(name, iteration + 1),
                                    ],
                                );
                                out.extend(controlled(env, vec![next.clone()]));
                                computes.push(next);
                            }
                        }
                    }
                    if let Some(name) = flags {
                        computes.reverse();
                        out.extend(controlled(env, computes));
                        out.push(obj([("relinquish", text(name))]));
                    }
                }
                "qload" | "qunload" | "qstore" | "qexchm" => {
                    let region = env
                        .regions
                        .get(&a[0])
                        .cloned()
                        .ok_or_else(|| Error::new("§11.1: missing source region".into()))?;
                    let count = 1usize
                        .checked_shl(region.address.try_into().map_err(|_| {
                            crate::resource_error("§11.1", "region address width exceeds host")
                        })?)
                        .ok_or_else(|| {
                            crate::resource_error("§11.1", "region extent exceeds host")
                        })?;
                    let address = env.wire(&a[1])?;
                    let name = if op == "qload" {
                        let name = self.declare(region.cell, "any", "clean");
                        out.push(obj([("acquire", text(&name))]));
                        env.wires
                            .insert(i.lhs[0].clone(), (name.clone(), region.cell));
                        name
                    } else {
                        env.wires
                            .get(&a[2])
                            .ok_or_else(|| Error::new("§11.1: missing memory value".into()))?
                            .0
                            .clone()
                    };
                    let operation = match op {
                        "qstore" => "dispatch.store",
                        "qexchm" => "dispatch.exchange",
                        _ => "dispatch.copy",
                    };
                    out.extend(controlled(
                        env,
                        vec![invoke(
                            operation,
                            [
                                number(region.address)?,
                                number(count)?,
                                number(region.cell)?,
                                address,
                                text(&region.cells),
                                text(&name),
                            ],
                        )],
                    ));
                    if op == "qunload" {
                        out.push(obj([("relinquish", text(name))]));
                        env.wires.remove(&a[2]);
                    }
                }
                "qinplace" => {
                    let g = &self.object.functions[a[0].trim_start_matches('@')];
                    let n = env.width(&a[3])?;
                    let fault = g.params[2][0].ty.bits(self.object.ptr_width)?;
                    let t = self.declare(n, "any", "clean");
                    let f = self.declare(fault, "any", "clean");
                    out.extend([obj([("acquire", text(&t))]), obj([("acquire", text(&f))])]);
                    env.wires.insert(t.clone(), (t.clone(), n));
                    env.wires.insert(f.clone(), (f.clone(), fault));
                    let extra = if a.len() > 4 {
                        crate::object::list(&a[4])?
                    } else {
                        Vec::new()
                    };
                    let mut inputs = vec![a[3].clone()];
                    inputs.extend(extra);
                    let mut call = i.clone();
                    call.op = "oracle_call".into();
                    call.operands = vec![
                        a[0].clone(),
                        a[2].clone(),
                        format!("({})", inputs.join(", ")),
                        t.clone(),
                        f.clone(),
                        if a.len() > 5 {
                            a[5].clone()
                        } else {
                            "()".into()
                        },
                        if a.len() > 6 {
                            a[6].clone()
                        } else {
                            "()".into()
                        },
                    ];
                    call.lhs.clear();
                    out.extend(self.call(&call, env)?);
                    let body = (0..n)
                        .map(|b| Ok(gate("swap", [env.indexed(&a[3], b)?, bit(&t, b)])))
                        .collect::<Result<Vec<_>>>()?;
                    out.extend(controlled(env, body));
                    call.operands[0] = a[1].clone();
                    out.extend(self.call(&call, env)?);
                    out.extend([
                        obj([("relinquish", text(&t))]),
                        obj([("relinquish", text(&f))]),
                    ]);
                    env.wires.remove(&t);
                    env.wires.remove(&f);
                }
                "oracle_call" | "action_call" | "ctl" | "adj" => out.extend(self.call(i, env)?),
                _ => {
                    let mut node = if matches!(op, "rx" | "ry" | "rz") {
                        let angle = if let Some(value) = self.values.get(&a[0]) {
                            value.clone()
                        } else {
                            text(format!(
                                "0x{:016x}",
                                a[0].parse::<f64>()
                                    .map_err(|_| Error::new("§11.1: malformed angle".into()))?
                                    .to_bits()
                            ))
                        };
                        obj([
                            ("gate", text(op)),
                            ("wires", array([env.wire(&a[1])?])),
                            ("params", array([angle])),
                        ])
                    } else if op == "mcx" {
                        let mut wires = crate::object::list(&a[0])?
                            .iter()
                            .map(|n| env.wire(n))
                            .collect::<Result<Vec<_>>>()?;
                        wires.push(env.wire(&a[1])?);
                        gate(op, wires)
                    } else if matches!(
                        op,
                        "x" | "h" | "s" | "sdg" | "t" | "tdg" | "cx" | "cz" | "swap" | "ccx"
                    ) {
                        gate(
                            op,
                            a.iter().map(|n| env.wire(n)).collect::<Result<Vec<_>>>()?,
                        )
                    } else {
                        return Err(Error::new(format!(
                            "§11.1: unknown source action operation {op}"
                        )));
                    };
                    // Keep the JSON node's typed angle rather than host float text.
                    out.extend(controlled(
                        env,
                        vec![std::mem::replace(&mut node, Value::Null)],
                    ));
                }
            }
            let (point, _) = self.locations.get(&i.line).cloned().ok_or_else(|| {
                Error::new("§11.1: structured operation has no source point".into())
            })?;
            let tag = format!("$source-point/{}", self.counter);
            self.counter += 1;
            self.points.insert(
                tag.clone(),
                SourcePoint {
                    symbol: self.symbol.clone(),
                    point,
                    iteration: env.iteration.clone(),
                    instance: Vec::new(),
                },
            );
            let nodes = out.split_off(start);
            out.push(obj([
                ("for", text(tag)),
                ("from", Value::Num(0.0)),
                ("to", Value::Num(1.0)),
                ("body", array(nodes)),
            ]));
        }
        Ok(out)
    }
}
fn source_adjoint(body: Vec<Value>, symbols: &mut [SourceSymbol]) -> Result<Vec<Value>> {
    let mut out = Vec::new();
    for mut node in body.into_iter().rev() {
        let Value::Obj(m) = &mut node else {
            return Err(Error::new(
                "§11.1: source operation is not a constructor".into(),
            ));
        };
        if let Some(Value::Str(g)) = m.get_mut("gate") {
            *g = match g.as_str() {
                "s" => "sdg",
                "sdg" => "s",
                "t" => "tdg",
                "tdg" => "t",
                g => g,
            }
            .to_string();
            if let Some(Value::Arr(angles)) = m.get_mut("params") {
                for angle in angles {
                    let Value::Str(t) = angle else {
                        return Err(Error::new("§11.1: angle encoding".into()));
                    };
                    let bits = u64::from_str_radix(
                        t.strip_prefix("0x")
                            .ok_or_else(|| Error::new("§11.1: angle encoding".into()))?,
                        16,
                    )
                    .map_err(|_| Error::new("§11.1: angle encoding".into()))?;
                    *t = format!("0x{:016x}", bits ^ (1 << 63));
                }
            }
        } else if m.contains_key("acquire") {
            let value = m.remove("acquire").expect("key checked");
            m.insert("relinquish".into(), value);
        } else if m.contains_key("relinquish") {
            let value = m.remove("relinquish").expect("key checked");
            m.insert("acquire".into(), value);
        } else if m.contains_key("under") {
            let value = m.remove("body").expect("body constructor");
            let Value::Arr(body) = value else {
                return Err(Error::new("§11.1: constructor body".into()));
            };
            m.insert("body".into(), array(source_adjoint(body, symbols)?));
        } else if m.contains_key("for") {
            let Value::Arr(body) = m.remove("body").expect("source point body") else {
                return Err(Error::new("§11.1: source point body".into()));
            };
            m.insert("body".into(), array(source_adjoint(body, symbols)?));
        } else if let Some(Value::Str(kind)) = m.get("invoke") {
            if matches!(kind.as_str(), "oracle_call" | "action_call") {
                if kind == "action_call" {
                    let alias = m
                        .get("args")
                        .and_then(Value::as_arr)
                        .and_then(|a| a.first())
                        .and_then(Value::as_str)
                        .ok_or_else(|| Error::new("§11.1: source call actual".into()))?;
                    let symbol = symbols
                        .iter_mut()
                        .find(|s| s.name == alias)
                        .ok_or_else(|| Error::new("§11.1: source call binding".into()))?;
                    symbol.adjoint = !symbol.adjoint;
                }
            } else if kind == "qloop" {
                let Some(Value::Arr(args)) = m.get_mut("args") else {
                    return Err(Error::new("§11.1: loop body".into()));
                };
                let Value::Arr(body) = args.remove(1) else {
                    return Err(Error::new("§11.1: loop body".into()));
                };
                args.push(array(source_adjoint(body, symbols)?));
            } else {
                node = inverse(vec![node]);
            }
        } else if m.contains_key("inverse") {
            let Value::Arr(body) = m.remove("body").expect("inverse body") else {
                return Err(Error::new("§11.1: inverse body".into()));
            };
            out.extend(body);
            continue;
        } else {
            return Err(Error::new(
                "§11.1: unknown source operation in adjoint".into(),
            ));
        }
        out.push(node);
    }
    Ok(out)
}
pub(super) fn construct(
    object: &crate::Object,
    f: &crate::Function,
    values: &BTreeMap<String, Value>,
    actual_regions: &BTreeMap<String, Value>,
    adjoint: bool,
) -> Result<SourceBody> {
    let hashes = object.identity()?.closure_hashes;
    let symbol = hashes[&f.name]
        .clone()
        .ok_or_else(|| Error::new("§11.1: source action is unlinked".into()))?;
    let mut locations = BTreeMap::new();
    fn visit(
        seq: &[crate::Instruction],
        block: &str,
        cursor: &mut usize,
        depth: usize,
        out: &mut BTreeMap<usize, (String, usize)>,
    ) {
        for i in seq {
            out.insert(i.line, (format!("{block}:{}", *cursor), depth));
            *cursor += 1;
            for body in &i.bodies {
                visit(
                    body,
                    block,
                    cursor,
                    depth + usize::from(i.op == "qloop"),
                    out,
                );
            }
        }
    }
    for block in &f.blocks {
        visit(&block.instructions, &block.name, &mut 0, 0, &mut locations);
    }
    let mut compiler = Compiler {
        object,
        values,
        declarations: Vec::new(),
        symbols: Vec::new(),
        counter: 0,
        hashes,
        symbol: symbol.clone(),
        points: BTreeMap::new(),
        locations,
    };
    let mut env = Env::default();
    for p in f.params.iter().flatten() {
        match p.ty {
            crate::Type::Wire(w) | crate::Type::Reg(w) => {
                env.wires
                    .insert(p.name.clone(), (p.name.clone(), w as usize));
            }
            crate::Type::Region(a, w) => {
                let origin = actual_regions
                    .get(&p.name)
                    .and_then(|r| r.get("origin"))
                    .cloned()
                    .ok_or_else(|| {
                        Error::new("§11.1: missing bound interface region origin".into())
                    })?;
                env.regions.insert(
                    p.name.clone(),
                    Region {
                        cells: p.name.clone(),
                        address: a as usize,
                        cell: w as usize,
                        origin,
                    },
                );
            }
            _ => {}
        }
    }
    let locals = crate::validation::region_declarations(f);
    for i in &locals {
        let a = &i.operands;
        env.regions.insert(
            a[0].clone(),
            Region {
                cells: a[0].clone(),
                address: crate::object::host_count(&a[1], "address width")? as usize,
                cell: crate::object::host_count(&a[2], "cell width")? as usize,
                origin: text(format!("local:{}", a[0])),
            },
        );
    }
    let mut body = Vec::new();
    for block in &f.blocks {
        body.extend(compiler.sequence(&block.instructions, &mut env)?);
    }
    if !env.predicates.is_empty() {
        return Err(Error::new("§11.1: unclosed source predicates".into()));
    }
    if adjoint {
        body = source_adjoint(body, &mut compiler.symbols)?;
    }
    let local_count = locals.len();
    for i in locals.into_iter().rev() {
        let a = &i.operands;
        let r = &env.regions[&a[0]];
        let count = 1usize
            .checked_shl(
                r.address
                    .try_into()
                    .map_err(|_| crate::resource_error("§11.1", "region width exceeds host"))?,
            )
            .ok_or_else(|| crate::resource_error("§11.1", "region count exceeds host"))?;
        let width = count
            .checked_mul(r.cell)
            .ok_or_else(|| crate::resource_error("§11.1", "region extent overflow"))?;
        let image = if a.len() == 4 {
            object.data[a[3].trim_start_matches('@')]
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        } else {
            String::new()
        };
        body = vec![invoke(
            "qregion",
            [
                number(count)?,
                number(r.cell)?,
                text(image),
                array([template(vec![formal(&a[0], width)?], body)]),
                array([]),
            ],
        )];
    }
    if compiler.declarations.len().saturating_add(local_count)
        >= bennett_isa_conformance::ref_language::SOURCE_SCOPE_LIMIT
    {
        return Err(crate::resource_error(
            "§11.1",
            "source declaration scopes exceed the host expansion stack budget",
        ));
    }
    for (name, width, class, state) in compiler.declarations.into_iter().rev() {
        body = vec![obj([
            ("declare", text(name)),
            ("width", number(width)?),
            ("class", obj([("class", text(class))])),
            ("state", text(state)),
            ("body", array(body)),
        ])];
    }
    Ok(SourceBody {
        body: array(body),
        symbols: compiler.symbols,
        symbol,
        oracle: false,
        points: compiler.points,
        rows: Vec::new(),
    })
}
