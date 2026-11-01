//! Source-derived constructor inputs for the published full-history library.
use crate::{
    machine::DerivedWidths,
    oracle::{operand_width, Program},
    ref_library::{OracleInvocation, Value},
    BitVector, Error, Result,
};
use std::collections::BTreeMap;
fn obj(fields: impl IntoIterator<Item = (&'static str, Value)>) -> Value {
    Value::Obj(fields.into_iter().map(|(k, v)| (k.into(), v)).collect())
}
fn text(v: impl Into<String>) -> Value {
    Value::Str(v.into())
}
fn array(v: impl IntoIterator<Item = Value>) -> Value {
    Value::Arr(v.into_iter().collect())
}
fn number(v: usize) -> Result<Value> {
    if v as u128 >= 1u128 << 53 {
        return Err(crate::resource_error(
            "§11.1",
            "constructor descriptor exceeds exact scalar host capacity",
        ));
    }
    Ok(Value::Num(v as f64))
}
fn invoke(name: &str, args: impl IntoIterator<Item = Value>) -> Value {
    obj([("invoke", text(name)), ("args", array(args))])
}
fn gate(name: &str, wires: impl IntoIterator<Item = Value>) -> Value {
    obj([("gate", text(name)), ("wires", array(wires))])
}
fn inverse(body: Vec<Value>) -> Value {
    obj([("inverse", Value::Bool(true)), ("body", array(body))])
}
fn under(wire: Value, body: Vec<Value>) -> Value {
    obj([("under", wire), ("body", array(body))])
}
fn view(name: &str, lo: usize, w: usize) -> Result<Value> {
    Ok(obj([
        ("range", text(name)),
        ("from", number(lo)?),
        (
            "to",
            number(
                lo.checked_add(w)
                    .ok_or_else(|| crate::resource_error("§11.1", "frame offset overflow"))?,
            )?,
        ),
    ]))
}
fn bit(name: &str, i: usize) -> Value {
    text(format!("{name}[{i}]"))
}
fn formal(name: &str, width: usize) -> Result<Value> {
    Ok(obj([
        ("name", text(name)),
        ("type", text("wire")),
        ("width", number(width)?),
    ]))
}
fn template(params: Vec<Value>, body: Vec<Value>) -> Value {
    obj([("parameters", array(params)), ("body", array(body))])
}
fn literal(wire: &str, value: &BitVector) -> Result<Vec<Value>> {
    let mut body = Vec::new();
    for i in 0..value.width() {
        if value.bit(i)? {
            body.push(gate("x", [bit(wire, i)]));
        }
    }
    Ok(body)
}
fn fixed_successor(p: usize, value: usize) -> Result<Vec<Value>> {
    Ok(vec![invoke(
        "value.literal",
        [number(p)?, number(value)?, text("next")],
    )])
}

pub(crate) fn oracle(
    call: &OracleInvocation<'_>,
) -> Result<bennett_isa_conformance::ref_language::SourceBody> {
    let object = call.object();
    let root = call.function();
    let budget = call.budget();
    let program = Program::new(object, &root.name)?;
    let widths = DerivedWidths::derive(object, &root.name, budget)?;
    let p = widths.p;
    let mut frame_slots = widths.slots.clone();
    if program.rows.iter().any(|(_, i)| i.op == "lift.alloc") {
        frame_slots[1] = frame_slots[1].max(2 * object.ptr_width as usize + 2);
    }
    // R4's complete global-id delivery scan includes every declared width,
    // including locations which this particular return cannot select.
    if program.rows.iter().any(|(_, i)| i.op == "lift.ret") {
        frame_slots[0] = frame_slots[0].max(widths.w_max);
    }
    let slot_total = frame_slots.iter().try_fold(0usize, |n, w| {
        n.checked_add(*w)
            .ok_or_else(|| crate::resource_error("§11.1", "frame slot width overflow"))
    })?;
    let flag_offset = p
        .checked_add(slot_total)
        .ok_or_else(|| crate::resource_error("§11.1", "frame offset overflow"))?;
    let succ_offset = flag_offset
        .checked_add(1)
        .ok_or_else(|| crate::resource_error("§11.1", "frame offset overflow"))?;
    let q = succ_offset
        .checked_add(p)
        .ok_or_else(|| crate::resource_error("§11.1", "frame width overflow"))?;
    let target = root.params[1][0].ty.bits(object.ptr_width)?;
    let fault = root.params[2][0].ty.bits(object.ptr_width)?;
    let mut declarations = vec![
        ("$pc".to_string(), p),
        ("$next".into(), p),
        ("$faulted".into(), 1),
        ("$code".into(), fault),
        ("$result".into(), target),
    ];
    let mut state = Vec::new();
    let mut names = BTreeMap::new();
    let mut preparations = Vec::new();
    let mut location_ids = Vec::new();
    let mut memory_layout = Vec::new();
    let mut images = BTreeMap::new();
    let mut initial = BTreeMap::new();
    let mut addresses = BTreeMap::new();
    let host = |n: u128, what: &str| {
        usize::try_from(n).map_err(|_| {
            crate::resource_error("§11.1", format!("{what} exceeds host descriptor capacity"))
        })
    };
    let ptr = object.ptr_width as usize;
    let mut capacity = 0usize;
    let mut table_fields: [Vec<Value>; 4] = std::array::from_fn(|_| Vec::new());
    if let Some(layout) = object.section("layout") {
        capacity = host(
            crate::oracle::power_of_two(layout.get("A_tab")?.uint()?).ok_or_else(|| {
                crate::resource_error("§11.1", "allocation table exceeds host address space")
            })?,
            "table capacity",
        )?;
        let entry_width = 2 * ptr + 2;
        let table_width = capacity
            .checked_mul(entry_width)
            .ok_or_else(|| crate::resource_error("§11.1", "table width overflow"))?;
        declarations.extend([
            ("$table".into(), table_width),
            ("$heap_used".into(), widths.hc),
            ("$atop".into(), widths.ac),
            ("$sp".into(), widths.sc),
            ("$fp".into(), widths.sc),
            ("$depth".into(), widths.dc),
        ]);
        let stack_count = host(
            crate::oracle::power_of_two(layout.get("S_cells")?.uint()?).ok_or_else(|| {
                crate::resource_error("§11.1", "stack exceeds host address space")
            })?,
            "stack capacity",
        )?;
        declarations.push((
            "$stack".into(),
            stack_count
                .checked_mul(widths.w_cell)
                .ok_or_else(|| crate::resource_error("§11.1", "stack width overflow"))?,
        ));
        let mut entries = Vec::new();
        let mut regions = Vec::new();
        for (name, g) in layout.get("globals")?.object()? {
            let base = host(g.get("base")?.uint()?, "global base")?;
            let size = host(g.get("size")?.uint()?, "global size")?;
            addresses.insert(name.clone(), base);
            entries.push((base, size));
        }
        if !root.params[3].is_empty() {
            for (name, r) in layout.get("interface_regions")?.get(&root.name)?.object()? {
                let param = root.params[3]
                    .iter()
                    .find(|p| p.name == *name)
                    .ok_or_else(|| Error::new("§11.1: unknown mapped region".into()))?;
                let crate::Type::Region(a, 8) = param.ty else {
                    return Err(Error::new(
                        "§11.1: lifted memory interface must have byte cells".into(),
                    ));
                };
                let base = host(r.get("base")?.uint()?, "region base")?;
                let extent = host(
                    crate::oracle::power_of_two(a as u128).ok_or_else(|| {
                        crate::resource_error("§11.1", "region exceeds host address space")
                    })?,
                    "region extent",
                )?;
                addresses.insert(name.clone(), base);
                entries.push((base, extent));
                regions.push((base, extent, name.clone()));
            }
        }
        initial.insert("$atop".to_string(), number(entries.len())?);
        if !entries.is_empty() {
            let mut table = crate::BitVector::zero(table_width)?;
            for (i, (base, size)) in entries.into_iter().enumerate() {
                for (part, v) in [(0, base), (ptr, size)] {
                    for b in 0..ptr.min(usize::BITS as usize) {
                        if v >> b & 1 != 0 {
                            table.set(i * entry_width + part + b, true)?;
                        }
                    }
                }
                table.set(i * entry_width + 2 * ptr, true)?;
                table.set(i * entry_width + 2 * ptr + 1, true)?;
            }
            let decimal = table.to_decimal();
            let scalar = decimal
                .parse::<u64>()
                .ok()
                .filter(|n| *n < (1u64 << 53))
                .map(|n| Value::Num(n as f64))
                .unwrap_or_else(|| text(decimal));
            initial.insert("$table".into(), scalar);
        }
        for i in 0..capacity {
            table_fields[0].push(view("$table", i * entry_width, ptr)?);
            table_fields[1].push(view("$table", i * entry_width + ptr, ptr)?);
            table_fields[2].push(bit("$table", i * entry_width + 2 * ptr));
            table_fields[3].push(bit("$table", i * entry_width + 2 * ptr + 1));
        }
        regions.sort_by_key(|r| r.0);
        let mut partition = Vec::new();
        let mut cursor = 0usize;
        for (base, extent, name) in regions {
            if cursor < base {
                partition.push((cursor, base - cursor, None));
            }
            partition.push((base, extent, Some(name)));
            cursor = base
                .checked_add(extent)
                .ok_or_else(|| crate::resource_error("§11.1", "region end overflow"))?;
        }
        let end = host(
            crate::oracle::power_of_two(layout.get("H")?.uint()?).ok_or_else(|| {
                crate::resource_error("§11.1", "memory exceeds host address space")
            })?,
            "memory extent",
        )?;
        if cursor < end {
            partition.push((cursor, end - cursor, None));
        }
        let width = end
            .checked_mul(8)
            .ok_or_else(|| crate::resource_error("§11.1", "memory width overflow"))?;
        declarations.push(("$memory".into(), width));
        let mut data = crate::reserved(end, "§11.1", "memory image")?;
        data.resize(end, 0u8);
        for (n, g) in layout.get("globals")?.object()? {
            let base = host(g.get("base")?.uint()?, "global base")?;
            let image = &object.data[n];
            data.get_mut(
                base..base
                    .checked_add(image.len())
                    .ok_or_else(|| crate::resource_error("§11.1", "global image end overflow"))?,
            )
            .ok_or_else(|| Error::new("§11.1: global image exceeds memory".into()))?
            .copy_from_slice(image);
        }
        images.insert(
            "$memory".to_string(),
            data.iter().map(|b| format!("{b:02x}")).collect::<String>(),
        );
        for (base, extent, interface) in partition {
            let cells = if let Some(name) = interface {
                text(name)
            } else {
                view("$memory", base * 8, extent * 8)?
            };
            memory_layout.push(obj([
                ("base", number(base)?),
                ("extent", number(extent)?),
                ("cells", cells),
            ]));
        }
    }

    for param in &root.params[0] {
        names.insert((root.name.clone(), param.name.clone()), param.name.clone());
    }
    for (param, actual) in root.params.iter().flatten().zip(call.arguments()) {
        if matches!(param.ty, crate::Type::Int(_) | crate::Type::Ptr) {
            let name = format!("$argument/{}", param.name);
            let width = param.ty.bits(object.ptr_width)?;
            let bits = BitVector::parse_decimal(
                width,
                actual
                    .as_str()
                    .ok_or_else(|| Error::new("§11.1: integer actual must be decimal".into()))?,
            )?;
            names.insert((root.name.clone(), param.name.clone()), name.clone());
            declarations.push((name.clone(), width));
            preparations.push((name, literal("$temporary", &bits)?));
        }
    }
    for (f, _) in &program.rows {
        for loc in f.locations() {
            let key = (f.name.clone(), loc.name.clone());
            if let std::collections::btree_map::Entry::Vacant(e) = names.entry(key) {
                let name = format!("$location/{}/{}", f.name, loc.name);
                declarations.push((name.clone(), loc.ty.bits(object.ptr_width)?));
                location_ids.push((
                    f.name.clone(),
                    loc.name.clone(),
                    loc.ty.bits(object.ptr_width)?,
                ));
                e.insert(name);
            }
        }
    }
    let frame_count = usize::try_from(budget.steps)
        .map_err(|_| crate::resource_error("§11.1", "B exceeds host descriptor capacity"))?;
    // Reserve before materialising the B frame descriptors.
    declarations.try_reserve_exact(frame_count).map_err(|e| {
        crate::resource_error("§11.1", format!("cannot allocate frame descriptors: {e}"))
    })?;
    let mut frames = Vec::new();
    frames.try_reserve_exact(frame_count).map_err(|e| {
        crate::resource_error("§11.1", format!("cannot allocate frame descriptors: {e}"))
    })?;
    for index in 0..frame_count {
        let name = format!("$frame/{index}");
        declarations.push((name.clone(), q));
        frames.push(obj([("index", number(index)?), ("frame", text(name))]));
    }
    let order = |name: &str| match name {
        "$pc" => 0,
        "$next" => 1,
        "$faulted" => 2,
        "$code" => 3,
        "$result" => 4,
        "$heap_used" => 5,
        "$atop" => 6,
        "$sp" => 7,
        "$fp" => 8,
        "$depth" => 9,
        "$memory" => 10,
        "$stack" => 11,
        "$table" => 14,
        _ if name.starts_with("$frame/") => 15,
        _ => 12,
    };
    let mut ordered = declarations.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|(name, _)| order(name));
    let prepared = preparations
        .iter()
        .map(|(name, _)| name)
        .collect::<std::collections::BTreeSet<_>>();
    for (name, width) in ordered {
        if !prepared.contains(name) {
            state.push(obj([
                ("width", number(*width)?),
                (
                    "initial",
                    initial.get(name).cloned().unwrap_or(Value::Num(0.0)),
                ),
                (
                    "I",
                    text(images.get(name).map(String::as_str).unwrap_or("")),
                ),
                ("storage", text(name)),
            ]));
        }
    }
    let fixed = root
        .params
        .iter()
        .flatten()
        .zip(call.arguments())
        .filter(|(p, _)| {
            matches!(
                p.ty,
                crate::Type::Token | crate::Type::Int(_) | crate::Type::Ptr | crate::Type::F64
            )
        })
        .map(|(p, v)| (p.name.clone(), v.clone()))
        .collect::<BTreeMap<_, _>>();
    let closure_hashes = object.identity()?.closure_hashes;
    let mut symbols = Vec::new();
    let mut rows = Vec::new();
    let mut successors = Vec::new();
    for (k, (f, i)) in program.rows.iter().enumerate() {
        let read = |name: &str| {
            names
                .get(&(f.name.clone(), name.into()))
                .cloned()
                .ok_or_else(|| {
                    Error::new(format!("§11.1: no source location {name} in @{}", f.name))
                })
        };
        let width = |name: &str| operand_width(object, f, name).map(|w| w as usize);
        let slot = |index: usize, w: usize| {
            view("frame", p + frame_slots[..index].iter().sum::<usize>(), w)
        };
        let flag = view("frame", flag_offset, 1)?;
        let succ = view("frame", succ_offset, p)?;
        let a = &i.operands;
        let op = i.op.as_str();
        let mut row = Vec::new();
        let mut next = fixed_successor(p, k + 1)?;
        let mut fallible = false;
        if let Some(n) = op.strip_prefix("lift.const.i") {
            let w = crate::object::host_count(n, "constant width")? as usize;
            let value = BitVector::parse_decimal(w, &a[0])?;
            for b in 0..w {
                if value.bit(b)? {
                    row.push(gate("x", [bit("frame", p + b)]));
                }
            }
            let dst = read(&i.lhs[0])?;
            for b in 0..w {
                row.push(gate("swap", [bit("frame", p + b), bit(&dst, b)]));
            }
        } else if let Some(binary) = op
            .strip_prefix("lift.")
            .filter(|b| crate::object::is_binop(b) || crate::object::is_div(b))
        {
            let w = width(&a[0])?;
            let mut args = vec![number(w)?];
            if crate::object::is_div(binary) {
                args.push(number(fault)?);
                fallible = true;
            }
            args.extend([
                text(read(&a[0])?),
                text(read(&a[1])?),
                slot(0, w)?,
                text(read(&i.lhs[0])?),
            ]);
            if fallible {
                args.extend([flag.clone(), text("$faulted"), text("$code")]);
            }
            row.push(invoke(op, args));
        } else if op.starts_with("lift.icmp.") {
            row.push(invoke(
                op,
                [
                    number(width(&a[0])?)?,
                    text(read(&a[0])?),
                    text(read(&a[1])?),
                    slot(0, 1)?,
                    text(read(&i.lhs[0])?),
                ],
            ));
        } else if let Some((memory, n)) = op
            .strip_prefix("lift.")
            .and_then(|s| s.split_once(".i"))
            .filter(|(m, _)| ["load", "store"].contains(m))
        {
            let w = crate::object::host_count(n, "memory width")? as usize;
            let bytes = w / 8;
            let length = (usize::BITS - bytes.leading_zeros()) as usize;
            let mut args = vec![
                number(ptr)?,
                number(length)?,
                number(ptr.max(length) + 1)?,
                number(capacity)?,
                number(widths.ac)?,
                number(fault)?,
                number(w)?,
                number(bytes)?,
                text(read(&a[0])?),
            ];
            args.extend(table_fields.iter().map(|v| array(v.clone())));
            args.extend([
                text("$atop"),
                flag.clone(),
                text("$faulted"),
                text("$code"),
                text(read(if memory == "load" { &i.lhs[0] } else { &a[1] })?),
                slot(0, w)?,
                array(memory_layout.clone()),
            ]);
            row.push(invoke(&format!("lift.{memory}"), args));
            fallible = true;
        } else if let Some((cast, n)) = op
            .strip_prefix("lift.")
            .and_then(|s| s.split_once(".i"))
            .filter(|(c, _)| ["zext", "sext", "trunc"].contains(c))
        {
            let out = crate::object::host_count(n, "cast width")? as usize;
            row.push(invoke(
                &format!("lift.{cast}"),
                [
                    number(width(&a[0])?)?,
                    number(out)?,
                    text(read(&a[0])?),
                    slot(0, out)?,
                    text(read(&i.lhs[0])?),
                ],
            ));
        } else {
            match op {
                "lift.oracle_call" | "lift.oracle_uncall" | "lift.perm_call" => {
                    let oracle = op != "lift.perm_call";
                    let callee = &object.functions[a[0].trim_start_matches('@')];
                    let alias = format!("$source-call/{k}");
                    let context = if oracle {
                        Some(obj([(
                            "token",
                            fixed.get(&a[1]).cloned().ok_or_else(|| {
                                Error::new(format!("§11.1: unresolved fixed source token {}", a[1]))
                            })?,
                        )]))
                    } else {
                        None
                    };
                    symbols.push(bennett_isa_conformance::ref_language::SourceSymbol {
                        name: alias.clone(),
                        hash: closure_hashes[&callee.name].clone().ok_or_else(|| {
                            Error::new("§11.1: unlinked nested source call".into())
                        })?,
                        oracle_invocation: context,
                        adjoint: false,
                    });
                    let mut arguments = Vec::new();
                    if oracle {
                        for n in crate::object::list(&a[2])? {
                            arguments.push(text(read(&n)?));
                        }
                        arguments.extend([text(read(&a[3])?), text(read(&a[4])?)]);
                    } else {
                        for group in [1usize, 2] {
                            for n in crate::object::list(&a[group])? {
                                arguments.push(text(read(&n)?));
                            }
                        }
                    }
                    let mem = if oracle { 5 } else { 3 };
                    for n in crate::object::list(&a[mem])? {
                        arguments.push(text(n));
                    }
                    for n in crate::object::list(&a[mem + 1])? {
                        arguments.push(fixed.get(&n).cloned().ok_or_else(|| {
                            Error::new(format!("§11.1: unresolved fixed source argument {n}"))
                        })?);
                    }
                    row.push(invoke(op, [text(alias), array(arguments)]));
                }
                "lift.call" => {
                    let callee = &object.functions[a[0].trim_start_matches('@')];
                    let actuals = crate::object::list(&a[1])?;
                    let count = actuals.len();
                    let cells = callee.frame_cells();
                    let locations = callee
                        .locations()
                        .iter()
                        .enumerate()
                        .map(|(slot, loc)| {
                            let id = location_ids
                                .iter()
                                .position(|(f, n, _)| f == &callee.name && n == &loc.name)
                                .ok_or_else(|| {
                                    Error::new("§11.1: missing global location id".into())
                                })?;
                            Ok(obj([
                                ("id", number(id)?),
                                ("width", number(loc.ty.bits(object.ptr_width)?)?),
                                ("slot", number(slot)?),
                                (
                                    "location",
                                    text(&names[&(callee.name.clone(), loc.name.clone())]),
                                ),
                            ]))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let captures = actuals
                        .iter()
                        .zip(&callee.params[0])
                        .enumerate()
                        .map(|(j, (actual, param))| {
                            let w = param.ty.bits(object.ptr_width)?;
                            Ok(obj([
                                ("position", number(j)?),
                                ("width", number(w)?),
                                ("source", text(read(actual)?)),
                                (
                                    "parameter",
                                    text(&names[&(callee.name.clone(), param.name.clone())]),
                                ),
                                ("capture", slot(j, w)?),
                            ]))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let id = location_ids
                        .iter()
                        .position(|(ff, n, _)| ff == &f.name && n == &i.lhs[0])
                        .ok_or_else(|| Error::new("§11.1: missing call destination id".into()))?;
                    let physical = host(
                        crate::oracle::power_of_two(
                            object
                                .section("layout")
                                .ok_or_else(|| {
                                    Error::new("§11.1: lproc call requires layout".into())
                                })?
                                .get("S_cells")?
                                .uint()?,
                        )
                        .ok_or_else(|| {
                            crate::resource_error("§11.1", "stack descriptor exceeds host")
                        })?,
                        "stack capacity",
                    )?;
                    let extent = widths
                        .sc
                        .max((usize::BITS - cells.leading_zeros()) as usize)
                        + 1;
                    row.push(invoke(
                        "lift.call",
                        [
                            number(widths.dc)?,
                            number(widths.sc)?,
                            number(extent)?,
                            number(cells)?,
                            number(physical)?,
                            number(widths.w_cell)?,
                            number(p)?,
                            number(widths.did)?,
                            number(fault)?,
                            number(host(budget.depth, "depth limit")?)?,
                            number(host(budget.stack, "stack limit")?)?,
                            number(k + 1)?,
                            number(id)?,
                            text("$depth"),
                            text("$sp"),
                            text("$fp"),
                            text("$stack"),
                            slot(count, widths.sc)?,
                            slot(count + 1, widths.sc)?,
                            slot(count + 2, widths.dc)?,
                            flag.clone(),
                            text("$faulted"),
                            text("$code"),
                            array(locations),
                            array(captures),
                        ],
                    ));
                    next = fixed_successor(p, program.ranges[&callee.name].0)?;
                    fallible = true;
                }
                "lift.ret" => {
                    let locations = f
                        .locations()
                        .iter()
                        .enumerate()
                        .map(|(j, loc)| {
                            let id = location_ids
                                .iter()
                                .position(|(ff, n, _)| ff == &f.name && n == &loc.name)
                                .ok_or_else(|| {
                                    Error::new("§11.1: missing return location id".into())
                                })?;
                            Ok(obj([
                                ("id", number(id)?),
                                ("width", number(loc.ty.bits(object.ptr_width)?)?),
                                ("slot", number(j)?),
                                (
                                    "location",
                                    text(&names[&(f.name.clone(), loc.name.clone())]),
                                ),
                            ]))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let globals = location_ids
                        .iter()
                        .enumerate()
                        .map(|(id, (ff, n, w))| {
                            let local = object.functions[ff]
                                .locations()
                                .iter()
                                .position(|l| &l.name == n)
                                .ok_or_else(|| {
                                    Error::new("§11.1: missing declaration index".into())
                                })?;
                            Ok(obj([
                                ("id", number(id)?),
                                ("width", number(*w)?),
                                ("slot", number(local)?),
                                ("location", text(&names[&(ff.clone(), n.clone())])),
                            ]))
                        })
                        .collect::<Result<Vec<_>>>()?;
                    let physical = host(
                        crate::oracle::power_of_two(
                            object
                                .section("layout")
                                .ok_or_else(|| Error::new("§11.1: return requires layout".into()))?
                                .get("S_cells")?
                                .uint()?,
                        )
                        .ok_or_else(|| {
                            crate::resource_error("§11.1", "stack descriptor exceeds host")
                        })?,
                        "stack capacity",
                    )?;
                    row.push(invoke(
                        "lift.ret",
                        [
                            number(widths.dc)?,
                            number(widths.sc)?,
                            number(physical)?,
                            number(widths.w_cell)?,
                            number(p)?,
                            number(widths.did)?,
                            number(fault)?,
                            number(frame_slots[0])?,
                            number(width(&a[0])?)?,
                            text("$depth"),
                            text("$fp"),
                            text("$stack"),
                            slot(1, widths.sc)?,
                            slot(2, widths.dc)?,
                            flag.clone(),
                            text("$faulted"),
                            text("$code"),
                            text(read(&a[0])?),
                            slot(0, frame_slots[0])?,
                            succ.clone(),
                            array(locations),
                            array(globals),
                        ],
                    ));
                    next = (0..p)
                        .map(|b| gate("cx", [bit("frame", succ_offset + b), bit("next", b)]))
                        .collect();
                    fallible = true;
                }
                "lift.addr" => {
                    let address = *addresses
                        .get(a[0].trim_start_matches('@'))
                        .ok_or_else(|| Error::new("§11.1: unmapped source address".into()))?;
                    row.push(invoke(
                        op,
                        [
                            number(ptr)?,
                            number(address)?,
                            slot(0, ptr)?,
                            text(read(&i.lhs[0])?),
                        ],
                    ));
                }
                "lift.parmov" => {
                    let sources = crate::object::list(&a[0])?;
                    let total = sources
                        .iter()
                        .map(|n| width(n))
                        .collect::<Result<Vec<_>>>()?
                        .iter()
                        .sum::<usize>();
                    let source = array(
                        sources
                            .iter()
                            .map(|n| read(n).map(text))
                            .collect::<Result<Vec<_>>>()?,
                    );
                    let targets = array(
                        i.lhs
                            .iter()
                            .map(|n| read(n).map(text))
                            .collect::<Result<Vec<_>>>()?,
                    );
                    let slots = array(
                        i.lhs
                            .iter()
                            .enumerate()
                            .map(|(j, n)| slot(j, width(n)?))
                            .collect::<Result<Vec<_>>>()?,
                    );
                    row.push(invoke(
                        "lift.parmov",
                        [number(total)?, source, slots, targets],
                    ));
                }
                "lift.alloc" => {
                    let n = width(&a[0])?;
                    let layout = object
                        .section("layout")
                        .ok_or_else(|| Error::new("§11.1: allocation requires layout".into()))?;
                    row.push(invoke(
                        "lift.alloc",
                        [
                            number(ptr)?,
                            number(n)?,
                            number(widths.hc)?,
                            number(widths.ac)?,
                            number(widths.hc.max(n) + 1)?,
                            number(ptr.min(widths.hc))?,
                            number(ptr.min(n))?,
                            number(capacity)?,
                            number(host(layout.get("heap_base")?.uint()?, "heap base")?)?,
                            number(host(budget.heap, "heap limit")?)?,
                            number(fault)?,
                            text(read(&a[0])?),
                            text("$heap_used"),
                            text("$atop"),
                            text("$table"),
                            text(read(&i.lhs[0])?),
                            slot(0, ptr)?,
                            slot(1, 2 * ptr + 2)?,
                            slot(2, widths.hc)?,
                            slot(3, widths.ac)?,
                            flag.clone(),
                            text("$faulted"),
                            text("$code"),
                        ],
                    ));
                    fallible = true;
                }
                "lift.free" | "lift.boundscheck" => {
                    let n = if op == "lift.free" || crate::object::integer(&a[1]) {
                        ptr
                    } else {
                        width(&a[1])?
                    };
                    let length = if op == "lift.free" {
                        text(read(&a[0])?)
                    } else if crate::object::integer(&a[1]) {
                        let name = format!("$length/{k}");
                        declarations.push((name.clone(), n));
                        let v = BitVector::parse_decimal(n, &a[1])?;
                        preparations.push((name.clone(), literal("$temporary", &v)?));
                        text(name)
                    } else {
                        text(read(&a[1])?)
                    };
                    let mut args = vec![
                        number(ptr)?,
                        number(n)?,
                        number(ptr.max(n) + 1)?,
                        number(capacity)?,
                        number(widths.ac)?,
                        number(fault)?,
                        text(read(&a[0])?),
                        length,
                    ];
                    args.extend(table_fields.iter().map(|v| array(v.clone())));
                    args.extend([text("$atop"), flag.clone(), text("$faulted"), text("$code")]);
                    row.push(invoke(op, args));
                    fallible = true;
                }
                "lift.mov" => row.push(invoke(
                    op,
                    [
                        number(width(&a[0])?)?,
                        text(read(&a[0])?),
                        slot(0, width(&a[0])?)?,
                        text(read(&i.lhs[0])?),
                    ],
                )),
                "lift.select" => {
                    let w = width(&a[1])?;
                    row.push(invoke(
                        op,
                        [
                            number(w)?,
                            text(read(&a[1])?),
                            text(read(&a[2])?),
                            text(read(&a[0])?),
                            slot(0, w)?,
                            text(read(&i.lhs[0])?),
                        ],
                    ));
                }
                "lift.retval" => {
                    row.push(invoke(
                        op,
                        [number(target)?, text(read(&a[0])?), text("$result")],
                    ));
                    next = fixed_successor(p, program.halt)?;
                }
                "lift.faultif" => {
                    row.push(gate("cx", [text(read(&a[0])?), flag.clone()]));
                    row.push(gate("cx", [flag.clone(), text("$faulted")]));
                    row.push(under(
                        flag.clone(),
                        literal("$code", &object.fault_literal(fault, &a[1])?)?,
                    ));
                    fallible = true;
                }
                "fault" => {
                    row.push(gate("x", [text("$faulted")]));
                    row.extend(literal("$code", &object.fault_literal(fault, &a[0])?)?);
                    next = fixed_successor(p, program.halt)?;
                }
                "qjump" => next = fixed_successor(p, program.label(&f.name, &a[0])?)?,
                "qbranch" => {
                    row.push(invoke(
                        op,
                        [
                            number(p)?,
                            number(program.label(&f.name, &a[1])?)?,
                            number(program.label(&f.name, &a[2])?)?,
                            text(read(&a[0])?),
                            succ.clone(),
                        ],
                    ));
                    next = Vec::new();
                }
                "qjumpr" => {
                    let (first, end) = program.ranges[&f.name];
                    row.push(invoke(
                        op,
                        [
                            number(p)?,
                            number(fault)?,
                            number(first)?,
                            number(end)?,
                            text(read(&a[0])?),
                            succ.clone(),
                            flag.clone(),
                            text("$faulted"),
                            text("$code"),
                        ],
                    ));
                    next = Vec::new();
                    fallible = true;
                }
                "lift.label" => {
                    let w = width(&i.lhs[0])?;
                    row.push(invoke(
                        op,
                        [
                            number(w)?,
                            number(program.label(&f.name, &a[0])?)?,
                            slot(0, w)?,
                            text(read(&i.lhs[0])?),
                        ],
                    ));
                }
                _ => {
                    return Err(Error::new(format!(
                        "§11.1: source constructor row {op} is not implemented"
                    )))
                }
            }
        }
        // Dynamic successors live in this step's frame and are copied before commit.
        if matches!(op, "qbranch" | "qjumpr") {
            next = (0..p)
                .map(|b| gate("cx", [bit("frame", succ_offset + b), bit("next", b)]))
                .collect();
        }
        if fallible {
            next = vec![
                under(flag.clone(), fixed_successor(p, program.halt)?),
                gate("x", [flag.clone()]),
                under(flag.clone(), next),
                gate("x", [flag]),
            ];
        }
        rows.push(template(vec![formal("frame", q)?], row));
        successors.push(template(
            vec![formal("frame", q)?, formal("next", p)?],
            next,
        ));
    }
    let construct = invoke(
        "oracle.construct",
        [
            number(p)?,
            number(program.halt)?,
            number(q)?,
            number(frame_count)?,
            number(fault)?,
            number(target)?,
            text("$pc"),
            text("$next"),
            text("$faulted"),
            text("$code"),
            text("$result"),
            text(&root.params[1][0].name),
            text(&root.params[2][0].name),
            array(state),
            array(frames),
            array(rows),
            array(successors),
        ],
    );
    let mut setup = Vec::new();
    let mut teardown = Vec::new();
    for (name, mut operations) in preparations {
        for op in &mut operations {
            if let Value::Obj(m) = op {
                if let Some(Value::Arr(w)) = m.get_mut("wires") {
                    for v in w {
                        if let Value::Str(v) = v {
                            *v = v.replace("$temporary", &name);
                        }
                    }
                }
            }
        }
        setup.push(obj([("acquire", text(&name))]));
        setup.extend(operations.clone());
        teardown.insert(0, obj([("relinquish", text(&name))]));
        teardown.insert(0, inverse(operations));
    }
    let source_symbol = call.closure_hash().to_string();
    let mut points = BTreeMap::new();
    points.insert(
        "$source-point/prepare".into(),
        bennett_isa_conformance::ref_language::SourcePoint::pseudo(&source_symbol, "$construct"),
    );
    points.insert(
        "$source-point/release".into(),
        bennett_isa_conformance::ref_language::SourcePoint::pseudo(&source_symbol, "$release"),
    );
    let tag = |name: &str, body: Vec<Value>| {
        obj([
            ("for", text(name)),
            ("from", Value::Num(0.0)),
            ("to", Value::Num(1.0)),
            ("body", array(body)),
        ])
    };
    let mut body = vec![
        tag("$source-point/prepare", setup),
        construct,
        tag("$source-point/release", teardown),
    ];
    for (name, width) in declarations.into_iter().rev() {
        body = vec![obj([
            ("declare", text(name)),
            ("width", number(width)?),
            ("class", obj([("class", text("any"))])),
            ("state", text("clean")),
            ("body", array(body)),
        ])];
    }
    Ok(bennett_isa_conformance::ref_language::SourceBody {
        body: array(body),
        symbol: source_symbol,
        oracle: true,
        points,
        rows: program
            .rows
            .iter()
            .map(|(f, i)| {
                let (block, index) = f
                    .blocks
                    .iter()
                    .find_map(|b| {
                        b.instructions
                            .iter()
                            .position(|v| v.line == i.line)
                            .map(|n| (&b.name, n))
                    })
                    .ok_or_else(|| Error::new("§11.1: row has no source point".into()))?;
                Ok(bennett_isa_conformance::ref_language::SourcePoint {
                    symbol: closure_hashes[&f.name]
                        .clone()
                        .ok_or_else(|| Error::new("§11.1: row symbol is unlinked".into()))?,
                    point: format!("{block}:{index}"),
                    iteration: Vec::new(),
                    instance: Vec::new(),
                })
            })
            .collect::<Result<Vec<_>>>()?,
        symbols,
    })
}

#[derive(Debug)]
pub(crate) struct LinkedSource {
    object: crate::Object,
    names: BTreeMap<String, String>,
}
impl LinkedSource {
    pub fn new(object: crate::Object) -> Result<Self> {
        let names = object
            .identity()?
            .closure_hashes
            .into_iter()
            .filter_map(|(n, h)| h.map(|h| (h, n)))
            .collect();
        Ok(Self { object, names })
    }
}
impl bennett_isa_conformance::ref_language::SourceFactory for LinkedSource {
    fn construct(
        &self,
        hash: &str,
        values: &BTreeMap<String, Value>,
        regions: &BTreeMap<String, Value>,
        context: Option<&Value>,
        adjoint: bool,
    ) -> std::result::Result<bennett_isa_conformance::ref_language::SourceBody, String> {
        let build = || -> Result<_> {
            let name = self
                .names
                .get(hash)
                .ok_or_else(|| Error::new("§11.1: closure is not in linked source".into()))?;
            let f = &self.object.functions[name];
            if f.kind != "oracle" {
                return action::construct(&self.object, f, values, regions, adjoint);
            }
            let context =
                context.ok_or_else(|| Error::new("§11.1 K77: missing oracle_invocation".into()))?;
            let tuple = context
                .get("token")
                .and_then(Value::as_arr)
                .filter(|a| a.len() == 4)
                .ok_or_else(|| Error::new("§11.1 K77: malformed invocation token".into()))?;
            let numbers = tuple
                .iter()
                .map(|v| {
                    v.as_str()
                        .ok_or_else(|| {
                            Error::new("§11.1 K77: token component must be decimal".into())
                        })?
                        .parse::<u128>()
                        .map_err(|_| {
                            crate::resource_error("§11.1 K77", "token exceeds host capacity")
                        })
                })
                .collect::<Result<Vec<_>>>()?;
            let budget = crate::Budget {
                steps: u64::try_from(numbers[0]).map_err(|_| {
                    crate::resource_error("§11.1 K77", "steps exceed host capacity")
                })?,
                heap: numbers[1],
                stack: numbers[2],
                depth: numbers[3],
            };
            let args = f
                .params
                .iter()
                .flatten()
                .map(|p| {
                    if matches!(
                        p.ty,
                        crate::Type::Wire(_) | crate::Type::Reg(_) | crate::Type::Region(..)
                    ) {
                        Ok(text(&p.name))
                    } else {
                        values.get(&p.name).cloned().ok_or_else(|| {
                            Error::new(format!("§11.1: missing fixed value {}", p.name))
                        })
                    }
                })
                .collect::<Result<Vec<_>>>()?;
            let call = OracleInvocation::capture(&self.object, name, Some(context), budget, &args)?;
            oracle(&call)
        };
        build().map_err(|e| e.to_string())
    }
}

mod action;
