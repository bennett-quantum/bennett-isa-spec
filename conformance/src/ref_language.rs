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
}
#[derive(Clone, Default)]
struct Context {
    ints: Env,
    wires: BTreeMap<String, Vec<Wire>>,
    angles: BTreeMap<String, u64>,
    bodies: BTreeMap<String, (Vec<Json>, Box<Context>)>,
    symbols: BTreeMap<String, String>,
    events: BTreeMap<String, Json>,
    controls: Vec<Wire>,
    legacy: bool,
    declarations: BTreeMap<String, Json>,
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
            } else if c.legacy {
                Ok(vec![(n, i)])
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
pub struct Expander {
    recipes: BTreeMap<String, Json>,
    registry: Registry,
    serial: usize,
    ordinals: BTreeMap<Vec<usize>, usize>,
}
impl Expander {
    pub fn new(bytes: &str, registry: Registry) -> Result<Self, String> {
        let d = json::parse(bytes)?;
        keys(&d, &["isa", "name", "version", "recipes"])?;
        for (k, v) in [("isa", "0.9"), ("name", "ref"), ("version", "0.9")] {
            if s(f(&d, k)?)? != v {
                return Err(format!("wrong {k}"));
            }
        }
        let mut recipes = BTreeMap::new();
        let mut previous = None;
        for r in a(f(&d, "recipes")?)? {
            let name = s(f(r, "kind")?)?.to_string();
            if previous.as_ref().is_some_and(|p| p >= &name) {
                return Err("recipe order/duplicate".into());
            }
            previous = Some(name.clone());
            recipes.insert(name, r.clone());
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
        Ok(out)
    }
    fn bind(&self, r: &Json, args: &[Json], caller: &Context) -> Result<Context, String> {
        let ps = a(f(r, "parameters")?)?;
        if ps.len() != args.len() {
            return Err("parameter arity".into());
        }
        let mut c = Context {
            controls: caller.controls.clone(),
            ..Context::default()
        };
        let mut names = BTreeSet::new();
        for (p, v) in ps.iter().zip(args) {
            let (name, ty) = if let Json::Str(n) = p {
                c.legacy = true;
                (n.as_str(), "integer")
            } else {
                (s(f(p, "name")?)?, s(f(p, "type")?)?)
            };
            if !names.insert(name) {
                return Err("duplicate formal".into());
            }
            match ty {
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
                        (a(v)?.to_vec(), Box::new(caller.clone()))
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
            };
            if let Some(g) = j.get("gate") {
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
                let target =
                    if matches!(leaf.gate.as_str(), "cz" | "s" | "sdg" | "t" | "tdg" | "rz") {
                        None
                    } else {
                        leaf.wires.last()
                    };
                if target.is_some_and(|t| leaf.controls.contains(t)) {
                    return Err("control is target".into());
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
                } else {
                    let ws = c
                        .wires
                        .remove(name)
                        .ok_or_else(|| format!("release unbound {name}"))?;
                    let identity = ws
                        .first()
                        .map(|w| w.0.clone())
                        .unwrap_or_else(|| format!("empty/{name}"));
                    leaf.allocation = Some((identity, ws.len(), "any".into(), "clean".into()));
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
                self.invoke(&hash, a(f(j, "args")?)?, c, &child, rev, chain, out)?;
            } else if let Some(n) = j.get("splice") {
                keys(j, &["splice"])?;
                let (b, mut captured) = c.bodies.get(s(n)?).cloned().ok_or("unbound body")?;
                captured.controls.extend(c.controls.clone());
                captured.controls.sort();
                captured.controls.dedup();
                self.body(&b, &mut captured, path, rev, chain, out)?;
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
                leaf.event = Some(c.events.get(s(n)?).cloned().unwrap_or_else(|| j.clone()));
                self.emit(leaf, rev, out)?;
            } else {
                return Err(format!("unknown constructor {j}"));
            }
        }
        Ok(())
    }
}
