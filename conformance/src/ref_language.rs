//! Typed, capture-free constructor expansion. Symbol bodies come only from an
//! explicitly supplied closure-hash registry, never from interpreter code.
use crate::{
    json::{self, Json},
    ref_catalog::{index, scalar, Env},
};
use std::collections::{BTreeMap, BTreeSet};
pub type Wire = (String, usize);
pub type Registry = BTreeMap<String, Json>;
#[derive(Clone, Debug, PartialEq)]
pub struct Leaf {
    pub invocation: Vec<usize>,
    pub ordinal: usize,
    pub gate: String,
    pub wires: Vec<Wire>,
    pub controls: Vec<Wire>,
    pub angles: Vec<u64>,
    pub allocation: Option<(String, usize, String, String)>,
    pub event: Option<Json>,
    pub scratch: bool,
}
#[derive(Clone, Default)]
struct Context {
    ints: Env,
    wires: BTreeMap<String, Vec<Wire>>,
    angles: BTreeMap<String, u64>,
    bodies: BTreeMap<String, (Vec<Json>, Box<Context>)>,
    symbols: BTreeMap<String, String>,
    events: BTreeMap<String, Json>,
    images: BTreeMap<String, Vec<u8>>,
    templates: BTreeMap<String, (Vec<Json>, Box<Context>)>,
    arguments: BTreeMap<String, (String, Vec<Json>, Box<Context>)>,
    values: BTreeMap<String, (Json, Json)>,
    controls: Vec<Wire>,
    legacy: bool,
    scratch_names: BTreeSet<String>,
    declarations: BTreeMap<String, Json>,
    allocations: BTreeMap<String, (String, usize, String, String)>,
}
fn f<'a>(j: &'a Json, k: &str) -> Result<&'a Json, String> {
    j.get(k).ok_or_else(|| format!("missing {k}"))
}
fn s(j: &Json) -> Result<&str, String> {
    j.as_str().ok_or_else(|| format!("expected string {j}"))
}
fn a(j: &Json) -> Result<&[Json], String> {
    j.as_arr().ok_or_else(|| format!("expected array {j}"))
}
fn keys(j: &Json, names: &[&str]) -> Result<(), String> {
    let Json::Obj(m) = j else {
        return Err("expected constructor object".into());
    };
    if m.keys().map(String::as_str).collect::<BTreeSet<_>>() != names.iter().copied().collect() {
        return Err(format!("closed constructor keys {j}"));
    }
    Ok(())
}
fn angle(j: &Json, c: &Context) -> Result<u64, String> {
    let v = s(j)?;
    if let Some(b) = c.angles.get(v) {
        return Ok(*b);
    }
    if v.len() != 18
        || !v.starts_with("0x")
        || !v[2..]
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("angle needs binary64 hex".into());
    }
    let bits = u64::from_str_radix(&v[2..], 16).map_err(|_| "bad angle")?;
    if !f64::from_bits(bits).is_finite() {
        return Err("nonfinite angle".into());
    }
    Ok(bits)
}
fn value(j: &Json, ty: &Json, c: &Context) -> Result<Json, String> {
    let j = if let Some((v, bound_type)) = j.as_str().and_then(|n| c.values.get(n)) {
        if bound_type != ty {
            return Err("forwarded value type mismatch".into());
        }
        v
    } else {
        j
    };
    fn decimal(j: &Json) -> Result<&str, String> {
        let t = s(j)?;
        if t.is_empty()
            || !t.bytes().all(|b| b.is_ascii_digit())
            || (t.len() > 1 && t.starts_with('0'))
        {
            return Err("noncanonical unsigned decimal".into());
        }
        Ok(t)
    }
    fn fits(t: &str, width: usize) -> bool {
        let mut digits = t.bytes().map(|b| b - b'0').collect::<Vec<_>>();
        let mut bits = 0;
        while digits.iter().any(|d| *d != 0) {
            let mut carry = 0;
            for d in &mut digits {
                let v = carry * 10 + *d;
                *d = v / 2;
                carry = v % 2;
            }
            bits += 1;
            if bits > width {
                return false;
            }
        }
        true
    }
    if let Some(t) = ty.as_str() {
        match t {
            "token" => {
                let parts = a(j)?;
                if parts.len() != 4 {
                    return Err("token needs four components".into());
                }
                for p in parts {
                    decimal(p)?;
                }
            }
            "f64" => {
                let text = s(j)?;
                if text.len() != 18
                    || !text.starts_with("0x")
                    || !text[2..]
                        .bytes()
                        .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                {
                    return Err("value needs binary64 hex".into());
                }
            }
            t if t.starts_with('i') => {
                let width = t[1..]
                    .parse::<usize>()
                    .map_err(|_| "invalid integer value type")?;
                if width == 0 || t[1..].starts_with('0') || !fits(decimal(j)?, width) {
                    return Err("integer value width".into());
                }
            }
            _ => return Err("unknown ISA value type".into()),
        }
    } else {
        keys(ty, &["ptr"])?;
        let width =
            usize::try_from(scalar(f(ty, "ptr")?, &Env::new())?).map_err(|_| "pointer width")?;
        if width == 0 || !fits(decimal(j)?, width) {
            return Err("pointer value width".into());
        }
    }
    Ok(j.clone())
}
fn wires(j: &Json, c: &Context) -> Result<Vec<Wire>, String> {
    match j {
        Json::Arr(v) => {
            let mut out = Vec::new();
            for w in v {
                out.extend(wires(w, c)?)
            }
            Ok(out)
        }
        Json::Str(v) => {
            if let Some(ws) = c.wires.get(v) {
                return Ok(ws.clone());
            }
            let (n, i) = index(v, &c.ints)?;
            if let Some(ws) = c.wires.get(&n) {
                Ok(vec![ws.get(i).ok_or("wire index out of range")?.clone()])
            } else {
                Err(format!("unbound wire {n}"))
            }
        }
        Json::Obj(_) => {
            keys(j, &["range", "from", "to"])?;
            let name = s(f(j, "range")?)?;
            let from =
                usize::try_from(scalar(f(j, "from")?, &c.ints)?).map_err(|_| "negative range")?;
            let to =
                usize::try_from(scalar(f(j, "to")?, &c.ints)?).map_err(|_| "negative range")?;
            let ws = c
                .wires
                .get(name)
                .ok_or_else(|| format!("unbound range {name}"))?;
            Ok(ws.get(from..to).ok_or("wire range out of bounds")?.to_vec())
        }
        _ => Err("invalid wire actual".into()),
    }
}
fn validate_formals(ps: &[Json]) -> Result<(), String> {
    let mut names = BTreeSet::new();
    for p in ps {
        let name = if let Json::Str(n) = p {
            n.as_str()
        } else {
            let ty = s(f(p, "type")?)?;
            keys(
                p,
                match ty {
                    "wire" => &["name", "type", "width"],
                    "templates" => &["name", "type", "length"],
                    "arguments" => &["name", "type", "symbol"],
                    "value" => &["name", "type", "isa_type"],
                    "integer" | "angle" | "body" | "symbol" | "environment" | "event" | "image" => {
                        &["name", "type"]
                    }
                    _ => return Err("unknown formal type".into()),
                },
            )?;
            s(f(p, "name")?)?
        };
        if !names.insert(name) {
            return Err("duplicate formal".into());
        }
    }
    Ok(())
}
fn validate_body(body: &[Json]) -> Result<(), String> {
    for c in body {
        if c.get("image").is_some() {
            keys(c, &["image", "wires"])?;
            s(f(c, "image")?)?;
        } else if c.get("apply").is_some() {
            keys(c, &["apply", "index", "args"])?;
            s(f(c, "apply")?)?;
            a(f(c, "args")?)?;
        } else if c.get("gate").is_some() {
            keys(
                c,
                if c.get("params").is_some() {
                    &["gate", "wires", "params"]
                } else {
                    &["gate", "wires"]
                },
            )?;
            s(f(c, "gate")?)?;
            a(f(c, "wires")?)?;
            if let Some(p) = c.get("params") {
                a(p)?;
            }
        } else if c.get("alloc").is_some() {
            keys(c, &["alloc", "width", "class", "state"])?;
            s(f(c, "alloc")?)?;
            s(f(c, "class")?)?;
            s(f(c, "state")?)?;
        } else if c.get("release").is_some() {
            keys(c, &["release"])?;
            s(f(c, "release")?)?;
        } else if c.get("classical").is_some() {
            keys(c, &["classical", "point"])?;
            s(f(c, "classical")?)?;
            s(f(c, "point")?)?;
        } else if c.get("invoke").is_some() || c.get("call").is_some() {
            let key = if c.get("invoke").is_some() {
                "invoke"
            } else {
                "call"
            };
            keys(c, &[key, "args"])?;
            s(f(c, key)?)?;
            if key == "call" && f(c, "args")?.as_str().is_some() {
                s(f(c, "args")?)?;
            } else {
                a(f(c, "args")?)?;
            }
        } else if c.get("for").is_some() {
            keys(c, &["for", "from", "to", "body"])?;
            s(f(c, "for")?)?;
            validate_body(a(f(c, "body")?)?)?;
        } else if c.get("each").is_some() {
            keys(c, &["each", "in", "body"])?;
            s(f(c, "each")?)?;
            validate_body(a(f(c, "body")?)?)?;
        } else if c.get("under").is_some() {
            keys(c, &["under", "body"])?;
            validate_body(a(f(c, "body")?)?)?;
        } else if c.get("inverse").is_some() {
            keys(c, &["inverse", "body"])?;
            if f(c, "inverse")?.as_bool() != Some(true) {
                return Err("inverse must be true".into());
            }
            validate_body(a(f(c, "body")?)?)?;
        } else if c.get("splice").is_some() {
            keys(c, &["splice"])?;
            s(f(c, "splice")?)?;
        } else {
            return Err(format!("unknown constructor {c}"));
        }
    }
    Ok(())
}
pub struct Expander {
    recipes: BTreeMap<String, Json>,
    registry: Registry,
    serial: usize,
    ordinals: BTreeMap<Vec<usize>, usize>,
}
impl Expander {
    pub fn new(bytes: &str, registry: Registry) -> Result<Self, String> {
        crate::ref_catalog::Catalog::parse(bytes)?;
        let d = json::parse(bytes)?;
        keys(&d, &["isa", "name", "version", "recipes"])?;
        for r in registry.values() {
            validate_formals(a(f(r, "parameters")?)?)?;
            validate_body(a(f(r, "body")?)?)?;
        }
        for (k, v) in [("isa", "0.9"), ("name", "ref"), ("version", "0.9")] {
            if s(f(&d, k)?)? != v {
                return Err(format!("wrong {k}"));
            }
        }
        let mut recipes = BTreeMap::new();
        let mut previous = None;
        for r in a(f(&d, "recipes")?)? {
            validate_body(a(f(r, "body")?)?)?;
            validate_formals(a(f(r, "parameters")?)?)?;
            let name = s(f(r, "kind")?)?.to_string();
            if previous.as_ref().is_some_and(|p| p >= &name) {
                return Err("recipe order/duplicate".into());
            }
            previous = Some(name.clone());
            recipes.insert(name, r.clone());
        }
        // Check declared dependencies through all literal actual syntax too.
        // Resetting the chain at a finite caller splice must never hide a
        // recipe whose declaration recursively recreates that same actual.
        fn collect(j: &Json, deps: &mut BTreeSet<String>) -> Result<(), String> {
            match j {
                Json::Obj(m) => {
                    if let Some(k) = m.get("invoke") {
                        deps.insert(s(k)?.into());
                    }
                    for v in m.values() {
                        collect(v, deps)?;
                    }
                }
                Json::Arr(v) => {
                    for j in v {
                        collect(j, deps)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }
        fn visit(
            k: &str,
            graph: &BTreeMap<String, BTreeSet<String>>,
            active: &mut BTreeSet<String>,
            done: &mut BTreeSet<String>,
        ) -> Result<(), String> {
            if done.contains(k) {
                return Ok(());
            }
            if !active.insert(k.into()) {
                return Err("cyclic declared invoke".into());
            }
            for dep in graph.get(k).ok_or_else(|| format!("unbound recipe {k}"))? {
                visit(dep, graph, active, done)?;
            }
            active.remove(k);
            done.insert(k.into());
            Ok(())
        }
        let mut graph = BTreeMap::new();
        for (k, r) in &recipes {
            let mut deps = BTreeSet::new();
            collect(f(r, "body")?, &mut deps)?;
            graph.insert(k.clone(), deps);
        }
        let mut done = BTreeSet::new();
        for k in graph.keys() {
            visit(k, &graph, &mut BTreeSet::new(), &mut done)?;
        }
        Ok(Self {
            recipes,
            registry,
            serial: 0,
            ordinals: BTreeMap::new(),
        })
    }
    pub fn expand(
        &mut self,
        kind: &str,
        args: &[Json],
        bundles: &[(&str, usize)],
    ) -> Result<Vec<Leaf>, String> {
        self.serial = 0;
        self.ordinals.clear();
        let mut caller = Context::default();
        for (n, w) in bundles {
            if caller
                .wires
                .insert(n.to_string(), (0..*w).map(|i| (n.to_string(), i)).collect())
                .is_some()
            {
                return Err("duplicate bundle".into());
            }
        }
        let mut out = Vec::new();
        self.invoke(kind, args, &caller, &[], false, &mut Vec::new(), &mut out)?;
        let mut lifetimes = Vec::new();
        for leaf in &out {
            if let Some((name, _, _, _)) = &leaf.allocation {
                if leaf.gate == "alloc" {
                    lifetimes.push((name.clone(), leaf.scratch));
                } else if lifetimes.pop().as_ref().map(|v| v.0.as_str()) != Some(name.as_str()) {
                    return Err("release is not reverse allocation order".into());
                }
            }
        }
        if lifetimes.iter().any(|(_, scratch)| *scratch) {
            return Err("unreleased library scratch".into());
        }

        Ok(out)
    }
    fn bind(&self, r: &Json, args: &[Json], caller: &Context) -> Result<Context, String> {
        let ps = a(f(r, "parameters")?)?;
        if ps.len() != args.len() {
            return Err("parameter arity".into());
        }
        let mut c = Context {
            controls: caller.controls.clone(),
            legacy: ps.iter().all(|p| matches!(p, Json::Str(_))),
            ..Context::default()
        };
        if let Some(names) = r.get("scratch") {
            for name in a(names)? {
                c.scratch_names.insert(s(name)?.into());
            }
        }
        validate_formals(ps)?;
        for (p, v) in ps.iter().zip(args) {
            let (name, ty) = if let Json::Str(n) = p {
                (n.as_str(), "integer")
            } else {
                (s(f(p, "name")?)?, s(f(p, "type")?)?)
            };
            match ty {
                "value" => {
                    let ty = f(p, "isa_type")?;
                    c.values
                        .insert(name.into(), (value(v, ty, caller)?, ty.clone()));
                }
                "arguments" => {
                    let symbol = c
                        .symbols
                        .get(s(f(p, "symbol")?)?)
                        .ok_or("arguments require earlier symbol")?;
                    let saved = if let Json::Str(n) = v {
                        caller
                            .arguments
                            .get(n)
                            .cloned()
                            .ok_or("unbound arguments")?
                    } else {
                        (symbol.clone(), a(v)?.to_vec(), Box::new(caller.clone()))
                    };
                    if &saved.0 != symbol {
                        return Err("arguments closure mismatch".into());
                    }
                    let r = self.registry.get(symbol).ok_or("unbound closure")?;
                    self.bind(r, &saved.1, &saved.2)?;
                    c.arguments.insert(name.into(), saved);
                }
                "image" => {
                    let text = s(v)?;
                    let bytes = if let Some(bytes) = caller.images.get(text) {
                        bytes.clone()
                    } else {
                        if text.len() % 2 != 0
                            || !text
                                .bytes()
                                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                        {
                            return Err("image needs lowercase byte hex".into());
                        }
                        (0..text.len())
                            .step_by(2)
                            .map(|i| {
                                u8::from_str_radix(&text[i..i + 2], 16)
                                    .map_err(|_| "bad image".to_string())
                            })
                            .collect::<Result<Vec<_>, _>>()?
                    };
                    c.images.insert(name.into(), bytes);
                }
                "templates" => {
                    let captured = if let Json::Str(n) = v {
                        caller
                            .templates
                            .get(n)
                            .cloned()
                            .ok_or("unbound templates")?
                    } else {
                        let ts = a(v)?;
                        for t in ts {
                            keys(t, &["parameters", "body"])?;
                            validate_formals(a(f(t, "parameters")?)?)?;
                            validate_body(a(f(t, "body")?)?)?;
                        }
                        (ts.to_vec(), Box::new(caller.clone()))
                    };
                    if usize::try_from(scalar(f(p, "length")?, &c.ints)?)
                        .map_err(|_| "negative template length")?
                        != captured.0.len()
                    {
                        return Err("template length mismatch".into());
                    }
                    c.templates.insert(name.into(), captured);
                }
                "integer" => {
                    c.ints.insert(name.into(), scalar(v, &caller.ints)?);
                }
                "angle" => {
                    c.angles.insert(name.into(), angle(v, caller)?);
                }
                "wire" => {
                    let ws = wires(v, caller)?;
                    let width = usize::try_from(scalar(f(p, "width")?, &c.ints)?)
                        .map_err(|_| "negative formal width")?;
                    if width != ws.len() {
                        return Err("formal width mismatch".into());
                    }
                    c.wires.insert(name.into(), ws);
                }
                "environment" => {
                    let ws = wires(v, caller)?;
                    c.controls.extend(ws);
                    c.controls.sort();
                    c.controls.dedup();
                }
                "body" => {
                    let b = if let Json::Str(n) = v {
                        caller.bodies.get(n).cloned().ok_or("unbound body")?
                    } else {
                        {
                            let body = a(v)?;
                            validate_body(body)?;
                            (body.to_vec(), Box::new(caller.clone()))
                        }
                    };
                    c.bodies.insert(name.into(), b);
                }
                "symbol" => {
                    let n = s(v)?;
                    let h = caller.symbols.get(n).map(String::as_str).unwrap_or(n);
                    if h.len() != 64
                        || !h
                            .bytes()
                            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
                    {
                        return Err("symbol needs closure hash".into());
                    }
                    if !self.registry.contains_key(h) {
                        return Err("unbound closure hash".into());
                    }
                    c.symbols.insert(name.into(), h.into());
                }
                "event" => {
                    let event = if let Json::Str(n) = v {
                        caller.events.get(n).cloned().ok_or("unbound event")?
                    } else {
                        v.clone()
                    };
                    keys(&event, &["instruction", "point"])?;
                    s(f(&event, "instruction")?)?;
                    s(f(&event, "point")?)?;
                    c.events.insert(name.into(), event);
                }
                _ => return Err(format!("unknown formal type {ty}")),
            }
        }
        if c.legacy {
            for (n, ws) in &caller.wires {
                c.wires.entry(n.clone()).or_insert_with(|| ws.clone());
            }
        }
        Ok(c)
    }
    #[allow(clippy::too_many_arguments)]
    fn invoke(
        &mut self,
        kind: &str,
        args: &[Json],
        caller: &Context,
        path: &[usize],
        rev: bool,
        chain: &mut Vec<String>,
        out: &mut Vec<Leaf>,
    ) -> Result<(), String> {
        if chain.iter().any(|x| x == kind) {
            return Err("cyclic invoke".into());
        }
        let r = self
            .recipes
            .get(kind)
            .or_else(|| self.registry.get(kind))
            .cloned()
            .ok_or_else(|| format!("unknown recipe {kind}"))?;
        let mut c = self.bind(&r, args, caller)?;
        chain.push(kind.into());
        self.body(a(f(&r, "body")?)?, &mut c, path, rev, chain, out)?;
        chain.pop();
        Ok(())
    }
    fn emit(&mut self, mut l: Leaf, rev: bool, out: &mut Vec<Leaf>) -> Result<(), String> {
        if rev {
            l.gate = match l.gate.as_str() {
                "s" => "sdg",
                "sdg" => "s",
                "t" => "tdg",
                "tdg" => "t",
                "alloc" => "release",
                "release" => "alloc",
                "classical" => return Err("classical inverse requires retained history".into()),
                g => g,
            }
            .into();
            for b in &mut l.angles {
                *b ^= 1 << 63;
            }
        }
        let n = self.ordinals.entry(l.invocation.clone()).or_default();
        l.ordinal = *n;
        *n += 1;
        out.push(l);
        Ok(())
    }
    fn body(
        &mut self,
        b: &[Json],
        c: &mut Context,
        path: &[usize],
        rev: bool,
        chain: &mut Vec<String>,
        out: &mut Vec<Leaf>,
    ) -> Result<(), String> {
        fn declarations(b: &[Json], d: &mut BTreeMap<String, Json>) -> Result<(), String> {
            for j in b {
                if let Some(n) = j.get("alloc") {
                    d.insert(s(n)?.into(), j.clone());
                }
                if let Some(inner) = j.get("body") {
                    declarations(a(inner)?, d)?;
                }
            }
            Ok(())
        }
        declarations(b, &mut c.declarations)?;
        let indices: Vec<_> = if rev {
            (0..b.len()).rev().collect()
        } else {
            (0..b.len()).collect()
        };
        for i in indices {
            let j = &b[i];
            self.serial += 1;
            let mut leaf = Leaf {
                invocation: path.to_vec(),
                ordinal: 0,
                gate: String::new(),
                wires: vec![],
                controls: c.controls.clone(),
                angles: vec![],
                allocation: None,
                event: None,
                scratch: false,
            };
            if let Some(n) = j.get("image") {
                keys(j, &["image", "wires"])?;
                let bytes = c.images.get(s(n)?).ok_or("unbound image")?;
                let ws = wires(f(j, "wires")?, c)?;
                if ws.len() != bytes.len() * 8 {
                    return Err("image width mismatch".into());
                }
                let mut indices = (0..ws.len()).collect::<Vec<_>>();
                if rev {
                    indices.reverse();
                }
                for bit in indices {
                    if bytes[bit / 8] >> (bit % 8) & 1 != 0 {
                        let mut x = leaf.clone();
                        x.gate = "x".into();
                        x.wires = vec![ws[bit].clone()];
                        x.controls.clear();
                        self.emit(x, false, out)?;
                    }
                }
            } else if let Some(n) = j.get("apply") {
                keys(j, &["apply", "index", "args"])?;
                let (ts, captured) = c.templates.get(s(n)?).cloned().ok_or("unbound templates")?;
                let index = usize::try_from(scalar(f(j, "index")?, &c.ints)?)
                    .map_err(|_| "negative template index")?;
                let t = ts.get(index).ok_or("template index out of bounds")?;
                // Actuals resolve at the apply site, free names at capture.
                let bound = self.bind(t, a(f(j, "args")?)?, c)?;
                let mut local = *captured;
                local.ints.extend(bound.ints);
                local.wires.extend(bound.wires);
                local.angles.extend(bound.angles);
                local.bodies.extend(bound.bodies);
                local.symbols.extend(bound.symbols);
                local.events.extend(bound.events);
                local.images.extend(bound.images);
                local.templates.extend(bound.templates);
                local.arguments.extend(bound.arguments);
                local.values.extend(bound.values);
                local.controls.extend(bound.controls);
                local.controls.sort();
                local.controls.dedup();
                self.body(
                    a(f(t, "body")?)?,
                    &mut local,
                    path,
                    rev,
                    &mut Vec::new(),
                    out,
                )?;
            } else if let Some(g) = j.get("gate") {
                keys(
                    j,
                    if j.get("params").is_some() {
                        &["gate", "wires", "params"]
                    } else {
                        &["gate", "wires"]
                    },
                )?;
                leaf.gate = s(g)?.into();
                leaf.wires = wires(f(j, "wires")?, c)?;
                let n = leaf.wires.len();
                let valid = match leaf.gate.as_str() {
                    "x" | "h" | "s" | "sdg" | "t" | "tdg" | "rx" | "ry" | "rz" => n == 1,
                    "cx" | "cz" | "swap" => n == 2,
                    "ccx" => n == 3,
                    "mcx" => n >= 1,
                    _ => false,
                };
                if !valid {
                    return Err("gate arity/name".into());
                }
                if leaf.wires.iter().collect::<BTreeSet<_>>().len() != n {
                    return Err("coincident operands".into());
                }
                if let Some(p) = j.get("params") {
                    leaf.angles = a(p)?
                        .iter()
                        .map(|v| angle(v, c))
                        .collect::<Result<_, _>>()?;
                }
                if leaf.angles.len()
                    != usize::from(matches!(leaf.gate.as_str(), "rx" | "ry" | "rz"))
                {
                    return Err("gate angle arity".into());
                }
                let targets = if leaf.gate == "swap" {
                    leaf.wires.as_slice()
                } else {
                    &leaf.wires[leaf.wires.len() - 1..]
                };
                if targets.iter().any(|w| leaf.controls.contains(w)) {
                    return Err("effective control is a gate target".into());
                }
                leaf.controls.sort();
                leaf.controls.dedup();
                self.emit(leaf, rev, out)?;
            } else if j.get("alloc").is_some() || j.get("release").is_some() {
                keys(
                    j,
                    if j.get("alloc").is_some() {
                        &["alloc", "width", "class", "state"]
                    } else {
                        &["release"]
                    },
                )?;
                let allocating = j.get("alloc").is_some() != rev;
                let name = s(j.get("alloc").or_else(|| j.get("release")).unwrap())?;
                leaf.scratch = c.scratch_names.contains(name);
                if allocating {
                    let declaration = if j.get("alloc").is_some() {
                        j
                    } else {
                        c.declarations
                            .get(name)
                            .ok_or("missing allocation declaration")?
                    };
                    let width = if let Some(w) = declaration.get("width") {
                        usize::try_from(scalar(w, &c.ints)?).map_err(|_| "negative allocation")?
                    } else {
                        return Err("inverse release lacks allocation declaration".into());
                    };
                    if c.wires.contains_key(name) {
                        return Err(format!("allocation shadows live wire {name}"));
                    }
                    let identity = format!("@{:?}/{}", path, self.serial);
                    c.wires.insert(
                        name.into(),
                        (0..width).map(|i| (identity.clone(), i)).collect(),
                    );
                    leaf.allocation = Some((
                        identity,
                        width,
                        s(f(declaration, "class")?)?.into(),
                        s(f(declaration, "state")?)?.into(),
                    ));
                    c.allocations
                        .insert(name.into(), leaf.allocation.clone().unwrap());
                } else {
                    c.wires
                        .remove(name)
                        .ok_or_else(|| format!("release unbound {name}"))?;
                    leaf.allocation = Some(c.allocations.remove(name).ok_or("unmatched release")?);
                }
                leaf.gate = if allocating { "alloc" } else { "release" }.into();
                leaf.controls.clear();
                self.emit(leaf, false, out)?;
            } else if let Some(n) = j.get("each") {
                keys(j, &["each", "in", "body"])?;
                let n = s(n)?;
                let mut ws = wires(f(j, "in")?, c)?;
                if rev {
                    ws.reverse();
                }
                let old = c.wires.get(n).cloned();
                for w in ws {
                    c.wires.insert(n.into(), vec![w]);
                    self.body(a(f(j, "body")?)?, c, path, rev, chain, out)?;
                }
                if let Some(ws) = old {
                    c.wires.insert(n.into(), ws);
                } else {
                    c.wires.remove(n);
                }
            } else if let Some(n) = j.get("for") {
                keys(j, &["for", "from", "to", "body"])?;
                let n = s(n)?;
                let from = scalar(f(j, "from")?, &c.ints)?;
                let to = scalar(f(j, "to")?, &c.ints)?;
                if to < from {
                    return Err("reversed range".into());
                }
                let old = c.ints.get(n).copied();
                let it: Box<dyn Iterator<Item = i64>> = if rev {
                    Box::new((from..to).rev())
                } else {
                    Box::new(from..to)
                };
                for v in it {
                    c.ints.insert(n.into(), v);
                    self.body(a(f(j, "body")?)?, c, path, rev, chain, out)?;
                }
                if let Some(v) = old {
                    c.ints.insert(n.into(), v);
                } else {
                    c.ints.remove(n);
                }
            } else if j.get("inverse").is_some() {
                keys(j, &["inverse", "body"])?;
                if f(j, "inverse")?.as_bool() != Some(true) {
                    return Err("inverse must be true".into());
                }
                self.body(a(f(j, "body")?)?, c, path, !rev, chain, out)?;
            } else if let Some(n) = j.get("invoke") {
                keys(j, &["invoke", "args"])?;
                self.invoke(s(n)?, a(f(j, "args")?)?, c, path, rev, chain, out)?;
            } else if let Some(n) = j.get("call") {
                keys(j, &["call", "args"])?;
                let hash = c.symbols.get(s(n)?).ok_or("unbound symbol")?.clone();
                let mut child = path.to_vec();
                child.push(self.serial);
                if let Some(n) = f(j, "args")?.as_str() {
                    let (bound_hash, args, mut captured) =
                        c.arguments.get(n).cloned().ok_or("unbound arguments")?;
                    if bound_hash != hash {
                        return Err("arguments closure mismatch".into());
                    }
                    captured.controls.extend(c.controls.clone());
                    captured.controls.sort();
                    captured.controls.dedup();
                    self.invoke(&hash, &args, &captured, &child, rev, chain, out)?;
                } else {
                    self.invoke(&hash, a(f(j, "args")?)?, c, &child, rev, chain, out)?;
                }
            } else if let Some(n) = j.get("splice") {
                keys(j, &["splice"])?;
                let (b, mut captured) = c.bodies.get(s(n)?).cloned().ok_or("unbound body")?;
                captured.controls.extend(c.controls.clone());
                captured.controls.sort();
                captured.controls.dedup();
                // A captured actual is finite caller syntax, not a declared
                // dependency of the higher-order callee. Nested qloop(qloop)
                // is legal; recursive invokes within either body still reject.
                self.body(&b, &mut captured, path, rev, &mut Vec::new(), out)?;
            } else if let Some(ws) = j.get("under") {
                keys(j, &["under", "body"])?;
                let old = c.controls.clone();
                c.controls.extend(wires(ws, c)?);
                c.controls.sort();
                c.controls.dedup();
                self.body(a(f(j, "body")?)?, c, path, rev, chain, out)?;
                c.controls = old;
            } else if let Some(n) = j.get("classical") {
                keys(j, &["classical", "point"])?;
                leaf.gate = "classical".into();
                leaf.controls.clear();
                leaf.event = if let Some(event) = c.events.get(s(f(j, "point")?)?) {
                    let instruction = s(f(event, "instruction")?)?;
                    let rhs = instruction
                        .split_once(" = ")
                        .map(|(_, v)| v)
                        .unwrap_or(instruction);
                    let opcode = rhs
                        .split_whitespace()
                        .next()
                        .ok_or("empty classical instruction")?;
                    let expected = s(n)?;
                    let valid = opcode == expected
                        || opcode.strip_prefix(expected).is_some_and(|tail| {
                            tail.starts_with(".i")
                                && tail.len() > 2
                                && !tail[2..].starts_with('0')
                                && tail[2..].bytes().all(|b| b.is_ascii_digit())
                        });
                    if !valid {
                        return Err("bound classical opcode mismatch".into());
                    }
                    Some(event.clone())
                } else {
                    Some(j.clone())
                };
                self.emit(leaf, rev, out)?;
            } else {
                return Err(format!("unknown constructor {j}"));
            }
        }
        Ok(())
    }
}

/// Lower typed permutation leaves to the independently compiled packed-basis
/// machine. Controlled swaps use a balanced CX / multi-control-X / CX sequence.
pub fn permutation(leaves: &[Leaf]) -> Result<Vec<crate::ref_catalog::Leaf>, String> {
    use crate::ref_catalog::{Leaf as OldLeaf, Operation};
    let mut out = Vec::new();
    for l in leaves {
        let mut emit = |operation| {
            out.push(OldLeaf {
                ordinal: l.ordinal,
                operation,
            })
        };
        if let Some((name, w, _, state)) = &l.allocation {
            if state != "clean" {
                return Err("borrowed wires must be explicit witness inputs".into());
            }
            emit(if l.gate == "alloc" {
                Operation::Alloc(name.clone(), *w)
            } else {
                Operation::Release(name.clone(), *w)
            });
        } else if l.gate == "swap" {
            if l.controls.is_empty() {
                emit(Operation::Gate("swap".into(), l.wires.clone()));
            } else {
                let a = l.wires[0].clone();
                let b = l.wires[1].clone();
                emit(Operation::Gate("cx".into(), vec![a.clone(), b.clone()]));
                let mut ws = l.controls.clone();
                ws.push(b.clone());
                ws.sort();
                ws.dedup();
                ws.push(a.clone());
                emit(Operation::Gate("mcx".into(), ws));
                emit(Operation::Gate("cx".into(), vec![a, b]));
            }
        } else if matches!(l.gate.as_str(), "x" | "cx" | "ccx" | "mcx") {
            let mut ws = l.controls.clone();
            ws.extend_from_slice(&l.wires[..l.wires.len() - 1]);
            ws.sort();
            ws.dedup();
            ws.push(l.wires.last().unwrap().clone());
            emit(Operation::Gate("mcx".into(), ws));
        } else {
            return Err(format!("non-permutation leaf {}", l.gate));
        }
    }
    Ok(out)
}
