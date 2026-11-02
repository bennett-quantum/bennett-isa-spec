//! Typed, capture-free constructor expansion. Symbol bodies come only from an
//! explicitly supplied closure-hash registry, never from interpreter code.
use crate::{
    json::{self, Json},
    ref_catalog::{index, integer, scalar, Env},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    rc::Rc,
};
mod named_interface;
/// Implementation resource limits, never ISA validity conditions.
pub const SOURCE_SCOPE_LIMIT: usize = 512;
pub const DEFAULT_SOURCE_LEAF_LIMIT: usize = 1_000_000;
pub const HOST_RESOURCE_LIMIT_PREFIX: &str = "HOST_RESOURCE_LIMIT: ";
/// Source context remains outside the closed catalog and positional arguments.
#[derive(Clone, Debug)]
pub struct SourceSymbol {
    pub name: String,
    pub hash: String,
    pub oracle_invocation: Option<Json>,
    pub adjoint: bool,
}
#[derive(Clone, Debug)]
struct SourceCapture {
    oracle_invocation: Option<Json>,
    adjoint: bool,
}
#[derive(Clone)]
// Share immutable lexical snapshots; Context::clone copies only their handles.
struct CapturedArguments {
    symbol: String,
    actuals: Vec<Json>,
    context: Rc<Context>,
    source: Option<SourceCapture>,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcePoint {
    pub symbol: String,
    pub point: String,
    pub iteration: Vec<usize>,
    pub instance: Vec<usize>,
}
impl SourcePoint {
    pub fn pseudo(symbol: &str, point: &str) -> Self {
        Self {
            symbol: symbol.into(),
            point: point.into(),
            iteration: Vec::new(),
            instance: Vec::new(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceEdge {
    pub point: SourcePoint,
    pub call_index: usize,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceStep {
    pub invocation: Vec<usize>,
    pub symbol: String,
    pub step: usize,
    pub reverse: bool,
}
/// Semantic invocation ancestry, including oracle support outside any step.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceInvocation {
    pub invocation: Vec<usize>,
    pub symbol: String,
    pub oracle: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourceLocation {
    pub path: Vec<SourceEdge>,
    pub terminal: SourcePoint,
    pub steps: Vec<SourceStep>,
    pub invocations: Vec<SourceInvocation>,
}
#[derive(Clone, Debug)]
pub struct SourceBody {
    pub body: Json,
    pub symbols: Vec<SourceSymbol>,
    pub symbol: String,
    pub oracle: bool,
    pub points: BTreeMap<String, SourcePoint>,
    pub rows: Vec<SourcePoint>,
}
#[derive(Clone, Debug)]
struct SourceOwner {
    symbol: String,
    oracle: bool,
    points: BTreeMap<String, SourcePoint>,
    rows: Vec<SourcePoint>,
    raw_reverse: bool,
}
#[derive(Clone, Debug)]
struct StepPlan {
    leaves: Vec<Leaf>,
    arguments: BTreeMap<String, Vec<Wire>>,
}
pub trait SourceFactory: std::fmt::Debug + Send + Sync {
    /// A non-emitting K65 carrier. This channel belongs to linked source,
    /// outside the immutable catalog's constructor grammar.
    fn cleanup_recompute(&self, _carrier: &SourceLocation) -> Result<bool, String> {
        Ok(false)
    }
    fn construct(
        &self,
        hash: &str,
        values: &BTreeMap<String, Json>,
        regions: &BTreeMap<String, Json>,
        oracle_invocation: Option<&Json>,
        adjoint: bool,
    ) -> Result<SourceBody, String>;
}
#[derive(Clone)]
struct Handle {
    declared_scratch: bool,
    scratch: bool,
    metadata: (String, usize, String, String),
    live: bool,
}
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
    pub source: Option<std::sync::Arc<SourceLocation>>,
}
#[derive(Clone, Default)]
struct Context {
    ints: Env,
    source_owner: Option<std::sync::Arc<SourceOwner>>,
    source_point: Option<SourcePoint>,
    source_edges: Vec<SourceEdge>,
    source_steps: Vec<SourceStep>,
    source_invocations: Vec<SourceInvocation>,
    source_location: Option<std::sync::Arc<SourceLocation>>,

    records: BTreeMap<String, Vec<Context>>,
    record_schemas: BTreeMap<String, Json>,
    regions: BTreeMap<String, Json>,
    classes: BTreeMap<String, String>,
    handles: BTreeMap<String, Rc<RefCell<Handle>>>,
    wires: BTreeMap<String, Vec<Wire>>,
    wire_owners: BTreeMap<String, Rc<RefCell<Handle>>>,
    angles: BTreeMap<String, u64>,
    bodies: BTreeMap<String, (Rc<Vec<Json>>, Rc<Context>)>,
    symbols: BTreeMap<String, String>,
    events: BTreeMap<String, Json>,
    images: BTreeMap<String, Vec<u8>>,
    templates: BTreeMap<String, (Rc<Vec<Json>>, Rc<Context>)>,
    arguments: BTreeMap<String, CapturedArguments>,
    sources: BTreeMap<String, SourceCapture>,
    invocation_source: Option<SourceCapture>,
    values: BTreeMap<String, (Json, Json)>,
    controls: Vec<Wire>,
    scratch_names: BTreeSet<String>,
    declarations: BTreeMap<String, Json>,
    allocations: BTreeMap<String, (String, usize, String, String)>,
}
impl Context {
    fn location(&mut self) -> Option<std::sync::Arc<SourceLocation>> {
        if self.source_location.is_none() {
            self.source_location = self.source_point.as_ref().map(|point| {
                std::sync::Arc::new(SourceLocation {
                    path: self.source_edges.clone(),
                    terminal: point.clone(),
                    steps: self.source_steps.clone(),
                    invocations: self.source_invocations.clone(),
                })
            });
        }
        self.source_location.clone()
    }
    fn has_name(&self, n: &str) -> bool {
        self.ints.contains_key(n)
            || self.wires.contains_key(n)
            || self.handles.contains_key(n)
            || self.classes.contains_key(n)
            || self.regions.contains_key(n)
            || self.angles.contains_key(n)
            || self.bodies.contains_key(n)
            || self.symbols.contains_key(n)
            || self.events.contains_key(n)
            || self.images.contains_key(n)
            || self.templates.contains_key(n)
            || self.arguments.contains_key(n)
            || self.values.contains_key(n)
            || self.records.contains_key(n)
    }
    fn clear_name(&mut self, n: &str) {
        self.ints.remove(n);
        self.wires.remove(n);
        self.wire_owners.remove(n);
        self.angles.remove(n);
        self.bodies.remove(n);
        self.symbols.remove(n);
        self.sources.remove(n);
        self.events.remove(n);
        self.images.remove(n);
        self.templates.remove(n);
        self.arguments.remove(n);
        self.values.remove(n);
        self.records.remove(n);
        self.record_schemas.remove(n);
        self.regions.remove(n);
        self.classes.remove(n);
        self.handles.remove(n);
        self.allocations.remove(n);
        self.declarations.remove(n);
        self.scratch_names.remove(n);
    }
    fn record_schema(&self, n: &str) -> Result<Json, String> {
        self.record_schemas
            .get(n)
            .cloned()
            .ok_or("unbound record schema".into())
    }
    fn merge(&mut self, b: &Context) {
        self.ints.extend(b.ints.clone());
        self.wires.extend(b.wires.clone());
        self.wire_owners.extend(b.wire_owners.clone());
        self.angles.extend(b.angles.clone());
        self.bodies.extend(b.bodies.clone());
        self.symbols.extend(b.symbols.clone());
        self.sources.extend(b.sources.clone());
        self.events.extend(b.events.clone());
        self.images.extend(b.images.clone());
        self.templates.extend(b.templates.clone());
        self.arguments.extend(b.arguments.clone());
        self.values.extend(b.values.clone());
        self.records.extend(b.records.clone());
        self.record_schemas.extend(b.record_schemas.clone());
        self.regions.extend(b.regions.clone());
        self.classes.extend(b.classes.clone());
        self.handles.extend(b.handles.clone());
    }
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
fn class(j: &Json, c: &Context) -> Result<String, String> {
    let name = if let Some(n) = j.as_str() {
        c.classes.get(n).ok_or("unbound allocation class")?.clone()
    } else {
        keys(j, &["class"])?;
        s(f(j, "class")?)?.to_string()
    };
    if name.is_empty() {
        return Err("empty allocation class".into());
    }
    Ok(name)
}
fn named_wires(n: &str, c: &Context) -> Result<Vec<Wire>, String> {
    if c.wire_owners
        .get(n)
        .is_some_and(|owner| !owner.borrow().live)
    {
        return Err("wire actual's allocation is not live".into());
    }
    if let Some(h) = c.handles.get(n) {
        let h = h.borrow();
        if !h.live {
            return Err("allocation is not live".into());
        }
        return Ok((0..h.metadata.1)
            .map(|i| (h.metadata.0.clone(), i))
            .collect());
    }
    c.wires
        .get(n)
        .cloned()
        .ok_or_else(|| format!("unbound wire {n}"))
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
            if c.wires.contains_key(v) || c.handles.contains_key(v) {
                return named_wires(v, c);
            }
            let (n, i) = index(v, &c.ints)?;
            let ws = named_wires(&n, c)?;
            Ok(vec![ws.get(i).ok_or("wire index out of range")?.clone()])
        }
        Json::Obj(_) => {
            keys(j, &["range", "from", "to"])?;
            let name = s(f(j, "range")?)?;
            let from =
                usize::try_from(scalar(f(j, "from")?, &c.ints)?).map_err(|_| "negative range")?;
            let to =
                usize::try_from(scalar(f(j, "to")?, &c.ints)?).map_err(|_| "negative range")?;
            let ws = named_wires(name, c)?;
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
                    "records" => &["name", "type", "fields", "order"],
                    "region" => &["name", "type", "address", "cell", "role"],
                    "class" | "allocation" => &["name", "type"],
                    "templates" => &["name", "type", "length"],
                    "arguments" => &["name", "type", "symbol"],
                    "value" => &["name", "type", "isa_type"],
                    "integer" | "angle" | "body" | "symbol" | "environment" | "event" | "image" => {
                        &["name", "type"]
                    }
                    _ => return Err("unknown formal type".into()),
                },
            )?;
            if ty == "records" {
                let fields = a(f(p, "fields")?)?;
                validate_formals(fields)?;
                for field in fields {
                    let ty = field
                        .get("type")
                        .and_then(Json::as_str)
                        .unwrap_or("integer");
                    if !matches!(
                        ty,
                        "integer" | "wire" | "region" | "class" | "allocation" | "image"
                    ) {
                        return Err("invalid record field type".into());
                    }
                }
                let mut seen = BTreeSet::new();
                for key in a(f(p, "order")?)? {
                    let n = s(key)?;
                    if !seen.insert(n)
                        || !fields.iter().any(|f| {
                            f.as_str() == Some(n)
                                || (f.get("name").and_then(Json::as_str) == Some(n)
                                    && f.get("type").and_then(Json::as_str) == Some("integer"))
                        })
                    {
                        return Err("record order needs distinct integer fields".into());
                    }
                }
            }
            s(f(p, "name")?)?
        };
        if !names.insert(name) {
            return Err("duplicate formal".into());
        }
    }
    Ok(())
}
fn validate_body(body: &[Json]) -> Result<(), String> {
    validate_body_mode(body, false)
}
fn validate_body_mode(body: &[Json], source: bool) -> Result<(), String> {
    for c in body {
        if c.get("visit").is_some() {
            keys(c, &["visit", "body"])?;
            s(f(c, "visit")?)?;
            validate_body_mode(a(f(c, "body")?)?, source)?;
        } else if c.get("declare").is_some() {
            keys(c, &["declare", "width", "class", "state", "body"])?;
            s(f(c, "declare")?)?;
            validate_body_mode(a(f(c, "body")?)?, source)?;
        } else if c.get("acquire").is_some() || c.get("relinquish").is_some() {
            let key = if c.get("acquire").is_some() {
                "acquire"
            } else {
                "relinquish"
            };
            keys(c, &[key])?;
            s(f(c, key)?)?;
        } else if c.get("image").is_some() {
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
        } else if source && c.get("source_cleanup").is_some() {
            keys(c, &["source_cleanup", "reverse", "recompute", "perm"])?;
            s(f(c, "source_cleanup")?)?;
            f(c, "perm")?
                .as_bool()
                .ok_or("cleanup provenance must be boolean")?;
            validate_body_mode(a(f(c, "reverse")?)?, true)?;
            validate_body_mode(a(f(c, "recompute")?)?, true)?;
        } else if c.get("for").is_some() {
            keys(c, &["for", "from", "to", "body"])?;
            s(f(c, "for")?)?;
            validate_body_mode(a(f(c, "body")?)?, source)?;
        } else if c.get("each").is_some() {
            keys(c, &["each", "in", "body"])?;
            s(f(c, "each")?)?;
            validate_body_mode(a(f(c, "body")?)?, source)?;
        } else if c.get("under").is_some() {
            keys(c, &["under", "body"])?;
            validate_body_mode(a(f(c, "body")?)?, source)?;
        } else if c.get("inverse").is_some() {
            keys(c, &["inverse", "body"])?;
            if f(c, "inverse")?.as_bool() != Some(true) {
                return Err("inverse must be true".into());
            }
            validate_body_mode(a(f(c, "body")?)?, source)?;
        } else if c.get("splice").is_some() {
            keys(c, &["splice"])?;
            s(f(c, "splice")?)?;
        } else {
            return Err(format!("unknown constructor {c}"));
        }
    }
    Ok(())
}
#[derive(Clone, Debug)]
pub struct Expander {
    recipes: BTreeMap<String, Json>,
    source_depth: usize,
    source_leaf_limit: usize,
    source_leaf_count: usize,
    source_allocations: BTreeMap<Vec<usize>, usize>,
    named_interfaces: BTreeMap<String, Vec<String>>,
    source_factory: Option<std::sync::Arc<dyn SourceFactory>>,
    source_ordinals: BTreeMap<std::sync::Arc<SourceLocation>, usize>,
    source_calls: BTreeMap<std::sync::Arc<SourceLocation>, usize>,
    source_serial: usize,
    source_step_plans: BTreeMap<(Vec<usize>, usize), StepPlan>,

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
            source_depth: 0,
            source_leaf_limit: DEFAULT_SOURCE_LEAF_LIMIT,
            source_leaf_count: 0,
            source_allocations: BTreeMap::new(),
            named_interfaces: named_interface::derive(&recipes)?,
            recipes,
            source_factory: None,
            source_ordinals: BTreeMap::new(),
            source_calls: BTreeMap::new(),
            source_serial: 0,
            source_step_plans: BTreeMap::new(),
            registry,
            serial: 0,
            ordinals: BTreeMap::new(),
        })
    }
    /// Reuse a validated catalog with a fresh source-bound registry. Invocation
    /// counters are reset; no captured context or expanded state is shared.
    pub fn with_registry(&self, registry: Registry) -> Result<Self, String> {
        for r in registry.values() {
            validate_formals(a(f(r, "parameters")?)?)?;
            validate_body(a(f(r, "body")?)?)?;
        }
        Ok(Self {
            source_depth: 0,
            source_leaf_limit: self.source_leaf_limit,
            source_leaf_count: 0,
            source_allocations: BTreeMap::new(),
            recipes: self.recipes.clone(),
            named_interfaces: self.named_interfaces.clone(),
            source_factory: self.source_factory.clone(),
            source_ordinals: BTreeMap::new(),
            source_calls: BTreeMap::new(),
            source_serial: 0,
            source_step_plans: BTreeMap::new(),
            registry,
            serial: 0,
            ordinals: BTreeMap::new(),
        })
    }
    pub fn with_source_factory(mut self, factory: std::sync::Arc<dyn SourceFactory>) -> Self {
        self.source_factory = Some(factory);
        self
    }
    /// Bound materialised source leaves, including replayed inverse leaves.
    /// Exceeding this host limit is not an ISA source fault.
    pub fn with_source_leaf_limit(mut self, limit: usize) -> Self {
        self.source_leaf_limit = limit;
        self
    }
    /// Ordered K82 wire names; None denotes a closed typed recipe.
    pub fn named_interface(&self, kind: &str) -> Option<&[String]> {
        self.named_interfaces.get(kind).map(Vec::as_slice)
    }
    pub fn expand(
        &mut self,
        kind: &str,
        args: &[Json],
        bundles: &[(&str, usize)],
    ) -> Result<Vec<Leaf>, String> {
        self.expand_bound(kind, args, bundles, &[])
    }
    pub fn expand_bound(
        &mut self,
        kind: &str,
        args: &[Json],
        bundles: &[(&str, usize)],
        symbols: &[SourceSymbol],
    ) -> Result<Vec<Leaf>, String> {
        // Lexically nested recipes must not depend on an embedding caller's
        // (often 2 MiB) stack. Captures themselves remain shared snapshots.
        std::thread::scope(|scope| {
            std::thread::Builder::new()
                .name("isa-recipe-expansion".into())
                .stack_size(32 * 1024 * 1024)
                .spawn_scoped(scope, || {
                    self.expand_bound_local(kind, args, bundles, symbols)
                })
                .map_err(|e| {
                    format!("{HOST_RESOURCE_LIMIT_PREFIX}cannot allocate expansion stack: {e}")
                })?
                .join()
                .map_err(|_| "recipe expansion worker panicked".to_string())?
        })
    }

    fn expand_bound_local(
        &mut self,
        kind: &str,
        args: &[Json],
        bundles: &[(&str, usize)],
        symbols: &[SourceSymbol],
    ) -> Result<Vec<Leaf>, String> {
        self.source_depth = 0;
        self.source_leaf_count = 0;
        self.source_allocations.clear();
        self.serial = 0;
        self.ordinals.clear();
        self.source_ordinals.clear();
        self.source_calls.clear();
        self.source_serial = 0;
        self.source_step_plans.clear();
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
        for symbol in symbols {
            if !self.registry.contains_key(&symbol.hash) || caller.has_name(&symbol.name) {
                return Err("invalid source symbol binding".into());
            }
            caller
                .symbols
                .insert(symbol.name.clone(), symbol.hash.clone());
            caller.sources.insert(
                symbol.name.clone(),
                SourceCapture {
                    oracle_invocation: symbol.oracle_invocation.clone(),
                    adjoint: symbol.adjoint,
                },
            );
        }
        if let Some(names) = self.named_interface(kind) {
            if names.iter().map(String::as_str).collect::<BTreeSet<_>>()
                != caller
                    .wires
                    .keys()
                    .map(String::as_str)
                    .collect::<BTreeSet<_>>()
            {
                return Err("explicit named-interface map has missing or extra wires".into());
            }
        }
        let mut out = Vec::new();
        self.invoke(kind, args, &caller, &[], false, &mut Vec::new(), &mut out)?;
        let mut live = BTreeMap::new();
        let mut scratch = Vec::new();
        for leaf in &out {
            if let Some((name, _, _, _)) = &leaf.allocation {
                if leaf.gate == "alloc" {
                    if live.insert(name.clone(), leaf.scratch).is_some() {
                        return Err("duplicate live allocation identity".into());
                    }
                    if leaf.scratch {
                        scratch.push(name.clone());
                    }
                } else {
                    let is_scratch = live.remove(name).ok_or("release without allocation")?;
                    if is_scratch && scratch.pop().as_ref() != Some(name) {
                        return Err("release is not reverse allocation order".into());
                    }
                }
            }
        }
        if !scratch.is_empty() {
            return Err("unreleased library scratch".into());
        }

        Ok(out)
    }
    fn bind(&self, r: &Json, args: &[Json], caller: &Context) -> Result<Context, String> {
        self.bind_seeded(r, args, caller, &Env::new())
    }
    fn bind_seeded(
        &self,
        r: &Json,
        args: &[Json],
        caller: &Context,
        seed: &Env,
    ) -> Result<Context, String> {
        let ps = a(f(r, "parameters")?)?;
        if ps.len() != args.len() {
            return Err("parameter arity".into());
        }
        let mut c = Context {
            ints: seed.clone(),
            controls: caller.controls.clone(),
            source_owner: caller.source_owner.clone(),
            source_point: caller.source_point.clone(),
            source_edges: caller.source_edges.clone(),
            source_steps: caller.source_steps.clone(),
            source_invocations: caller.source_invocations.clone(),
            source_location: caller.source_location.clone(),
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
                "class" => {
                    c.classes.insert(name.into(), class(v, caller)?);
                }
                "allocation" => {
                    c.handles.insert(
                        name.into(),
                        caller
                            .handles
                            .get(s(v)?)
                            .cloned()
                            .ok_or("unbound allocation declaration")?,
                    );
                }
                "region" => {
                    let region = if let Some(n) = v.as_str() {
                        caller.regions.get(n).cloned().ok_or("unbound region")?
                    } else {
                        keys(v, &["address", "cell", "role", "origin", "cells"])?;
                        let mut r = v.clone();
                        if let Json::Obj(m) = &mut r {
                            let ws = wires(f(v, "cells")?, caller)?;
                            // Store canonical explicit wires separately from caller names.
                            m.insert(
                                "cells".into(),
                                Json::Arr(
                                    ws.iter()
                                        .map(|(n, i)| Json::Str(format!("{n}[{i}]")))
                                        .collect(),
                                ),
                            );
                        }
                        r
                    };
                    let aa = scalar(f(&region, "address")?, &Env::new())?;
                    let ww = scalar(f(&region, "cell")?, &Env::new())?;
                    if aa < 0
                        || ww <= 0
                        || aa != scalar(f(p, "address")?, &c.ints)?
                        || ww != scalar(f(p, "cell")?, &c.ints)?
                        || s(f(&region, "role")?)? != s(f(p, "role")?)?
                        || !matches!(s(f(&region, "role")?)?, "in" | "mut")
                        || !["local:", "interface:"].iter().any(|prefix| {
                            s(f(&region, "origin").unwrap()).is_ok_and(|v| {
                                v.strip_prefix(prefix).is_some_and(|name| {
                                    name.as_bytes()
                                        .first()
                                        .is_some_and(|b| b.is_ascii_alphabetic() || *b == b'_')
                                        && name.bytes().all(|b| {
                                            b.is_ascii_alphanumeric() || b == b'_' || b == b'.'
                                        })
                                })
                            })
                        })
                    {
                        return Err("region shape/role/origin mismatch".into());
                    }
                    let ws = if let Some(n) = v.as_str() {
                        named_wires(n, caller)?
                    } else {
                        wires(f(v, "cells")?, caller)?
                    };
                    let count = 1usize
                        .checked_shl(u32::try_from(aa).map_err(|_| "region exponent")?)
                        .ok_or("region too large for checker")?;
                    if count.checked_mul(ww as usize) != Some(ws.len()) {
                        return Err("region width mismatch".into());
                    }
                    c.wires.insert(name.into(), ws);
                    c.regions.insert(name.into(), region);
                }
                "records" => {
                    let fields = a(f(p, "fields")?)?;
                    let records = if let Some(n) = v.as_str() {
                        // A forwarded descriptor retains its already validated field bindings.
                        let rs = caller.records.get(n).cloned().ok_or("unbound records")?;
                        // Schemas must match as well as runtime widths.
                        if caller.record_schema(n)?
                            != Json::Arr(vec![f(p, "fields")?.clone(), f(p, "order")?.clone()])
                        {
                            return Err("record schema mismatch".into());
                        }
                        rs
                    } else {
                        let mut result = Vec::new();
                        for record in a(v)? {
                            let names = fields
                                .iter()
                                .map(|p| {
                                    p.as_str()
                                        .or_else(|| p.get("name").and_then(Json::as_str))
                                        .unwrap()
                                })
                                .collect::<Vec<_>>();
                            keys(record, &names)?;
                            let mut widths = c.ints.clone();
                            let mut bound = Context::default();
                            for field in fields {
                                let n = field
                                    .as_str()
                                    .or_else(|| field.get("name").and_then(Json::as_str))
                                    .unwrap();
                                // Width expressions can use outer integers and earlier record integers.
                                let fake = Json::Obj(BTreeMap::from([(
                                    "parameters".into(),
                                    Json::Arr(vec![if field.as_str().is_some() {
                                        Json::Obj(BTreeMap::from([
                                            ("name".into(), field.clone()),
                                            ("type".into(), Json::Str("integer".into())),
                                        ]))
                                    } else {
                                        field.clone()
                                    }]),
                                )]));
                                let mut b = self.bind_seeded(
                                    &fake,
                                    &[f(record, n)?.clone()],
                                    caller,
                                    &widths,
                                )?;
                                b.ints.retain(|key, _| key == n);
                                widths.extend(b.ints.clone());
                                bound.merge(&b);
                            }
                            result.push(bound);
                        }
                        result
                    };
                    let mut previous: Option<Vec<crate::ref_integer::Integer>> = None;
                    for record in &records {
                        let mut widths = c.ints.clone();
                        for field in fields {
                            let n = field
                                .as_str()
                                .or_else(|| field.get("name").and_then(Json::as_str))
                                .unwrap();
                            if field.as_str().is_some()
                                || field.get("type").and_then(Json::as_str) == Some("integer")
                            {
                                widths.insert(
                                    n.into(),
                                    record.ints.get(n).ok_or("missing integer field")?.clone(),
                                );
                            } else if field.get("type").and_then(Json::as_str) == Some("wire")
                                && usize::try_from(scalar(f(field, "width")?, &widths)?).ok()
                                    != Some(named_wires(n, record)?.len())
                            {
                                return Err("forwarded record width mismatch".into());
                            }
                        }
                        let order = a(f(p, "order")?)?
                            .iter()
                            .map(|j| {
                                record
                                    .ints
                                    .get(s(j)?)
                                    .cloned()
                                    .ok_or("unbound order field".into())
                            })
                            .collect::<Result<Vec<_>, String>>()?;
                        if !order.is_empty() && previous.as_ref().is_some_and(|prev| prev >= &order)
                        {
                            return Err("record order not strictly increasing".into());
                        }
                        previous = Some(order);
                    }
                    c.record_schemas.insert(
                        name.into(),
                        Json::Arr(vec![f(p, "fields")?.clone(), f(p, "order")?.clone()]),
                    );
                    c.records.insert(name.into(), records);
                }
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
                        CapturedArguments {
                            symbol: symbol.clone(),
                            actuals: a(v)?.to_vec(),
                            context: Rc::new(caller.clone()),
                            source: c.sources.get(s(f(p, "symbol")?)?).cloned(),
                        }
                    };
                    if &saved.symbol != symbol {
                        return Err("arguments closure mismatch".into());
                    }
                    let r = self.registry.get(symbol).ok_or("unbound closure")?;
                    self.bind(r, &saved.actuals, &saved.context)?;
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
                            validate_body_mode(a(f(t, "body")?)?, self.source_factory.is_some())?;
                        }
                        (Rc::new(ts.to_vec()), Rc::new(caller.clone()))
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
                    c.ints.insert(name.into(), integer(v, &caller.ints)?);
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
                            validate_body_mode(body, self.source_factory.is_some())?;
                            (Rc::new(body.to_vec()), Rc::new(caller.clone()))
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
                    if let Some(context) = caller.sources.get(n) {
                        c.sources.insert(name.into(), context.clone());
                    }
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
        if rev && self.source_factory.is_some() && self.registry.contains_key(kind) {
            // Instantiate the complete source tree in forward order first.
            // Raw inversion preserves its source addresses, call occurrences,
            // scratch identities and ordinals instead of renumbering a reverse
            // traversal or asking an inverse step for a nonexistent plan.
            let mut forward = Vec::new();
            self.invoke(kind, args, caller, path, false, chain, &mut forward)?;
            for mut leaf in forward.into_iter().rev() {
                adjoint_leaf(&mut leaf)?;
                self.account_source_leaf()?;
                out.push(leaf);
            }
            return Ok(());
        }
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
        // A directly entered, empty-formal registry constructor is the
        // caller's script over its explicit bundle map. Semantic `call`
        // invocations always have a nonempty path and bind a closed interface.
        if path.is_empty()
            && self.registry.contains_key(kind)
            && a(f(&r, "parameters")?)?.is_empty()
        {
            c.wires = caller.wires.clone();
        }
        if let Some(names) = self.named_interface(kind) {
            for name in names {
                c.wires.insert(name.clone(), named_wires(name, caller)?);
                if let Some(allocation) = caller.allocations.get(name) {
                    c.allocations.insert(name.clone(), allocation.clone());
                }
                if caller.scratch_names.contains(name) {
                    c.scratch_names.insert(name.clone());
                }
                if let Some(owner) = caller
                    .handles
                    .get(name)
                    .or_else(|| caller.wire_owners.get(name))
                {
                    c.wire_owners.insert(name.clone(), owner.clone());
                    let handle = owner.borrow();
                    c.allocations.insert(name.clone(), handle.metadata.clone());
                    if handle.scratch {
                        c.scratch_names.insert(name.clone());
                    }
                }
            }
        }
        if self.source_factory.is_some() && kind == "step" {
            let step = c
                .source_steps
                .last()
                .filter(|step| step.invocation == path)
                .cloned()
                .ok_or("source step has no bound frame index")?;
            let key = (path.to_vec(), step.step);
            if rev {
                let plan = self
                    .source_step_plans
                    .get(&key)
                    .cloned()
                    .ok_or("inverse source step has no instantiated forward tree")?;
                if plan.arguments != c.wires {
                    return Err("inverse source step changed its bound state".into());
                }
                let mut locations = BTreeMap::new();
                for mut leaf in plan.leaves.into_iter().rev() {
                    adjoint_leaf(&mut leaf)?;
                    if let Some(location) = &leaf.source {
                        let replacement = locations
                            .entry(location.clone())
                            .or_insert_with(|| {
                                let mut location = (**location).clone();
                                for frame in &mut location.steps {
                                    if frame.invocation == path {
                                        frame.reverse = step.reverse;
                                    }
                                }
                                std::sync::Arc::new(location)
                            })
                            .clone();
                        leaf.source = Some(replacement);
                    }
                    self.account_source_leaf()?;
                    out.push(leaf);
                }
                return Ok(());
            }
            let mut leaves = Vec::new();
            chain.push(kind.into());
            self.body(a(f(&r, "body")?)?, &mut c, path, false, chain, &mut leaves)?;
            chain.pop();
            if self
                .source_step_plans
                .insert(
                    key,
                    StepPlan {
                        leaves: leaves.clone(),
                        arguments: c.wires.clone(),
                    },
                )
                .is_some()
            {
                return Err("source step instantiated twice".into());
            }
            out.extend(leaves);
            return Ok(());
        }
        if self.source_factory.is_some()
            && kind == "qregion"
            && c.source_owner.as_ref().is_some_and(|owner| !owner.oracle)
        {
            let owner = c
                .source_owner
                .clone()
                .ok_or("region support has no source action")?;
            let body = a(f(&r, "body")?)?;
            let previous = c.source_point.clone();
            let indices: Vec<_> = if rev {
                (0..body.len()).rev().collect()
            } else {
                (0..body.len()).collect()
            };
            chain.push(kind.into());
            for index in indices {
                c.source_point = if index < 2 {
                    Some(SourcePoint::pseudo(&owner.symbol, "$construct"))
                } else if index > 2 {
                    Some(SourcePoint::pseudo(&owner.symbol, "$release"))
                } else {
                    previous.clone()
                };
                c.source_location = None;
                self.body(&body[index..index + 1], &mut c, path, rev, chain, out)?;
            }
            chain.pop();
            return Ok(());
        }
        if self.source_factory.is_some() && kind == "oracle.construct" {
            let owner = c
                .source_owner
                .clone()
                .filter(|o| o.oracle)
                .ok_or("construction has no source oracle owner")?;
            let body = a(f(&r, "body")?)?;
            let order: Vec<_> = if rev {
                (0..body.len()).rev().collect()
            } else {
                (0..body.len()).collect()
            };
            chain.push(kind.into());
            for index in order {
                let point = if index == 0 {
                    "$construct"
                } else if index == 1 || index + 2 == body.len() {
                    "$step"
                } else if index + 1 == body.len() {
                    "$release"
                } else {
                    "$commit"
                };
                c.source_point = Some(SourcePoint::pseudo(&owner.symbol, point));
                c.source_location = None;
                self.body(&body[index..index + 1], &mut c, path, rev, chain, out)?;
            }
            chain.pop();
            return Ok(());
        }
        chain.push(kind.into());
        if self.registry.contains_key(kind) {
            if let Some(factory) = self.source_factory.clone() {
                let values = c
                    .values
                    .iter()
                    .map(|(k, (v, _))| (k.clone(), v.clone()))
                    .collect();
                let built = factory.construct(
                    kind,
                    &values,
                    &c.regions,
                    caller
                        .invocation_source
                        .as_ref()
                        .and_then(|s| s.oracle_invocation.as_ref()),
                    caller.invocation_source.as_ref().is_some_and(|s| s.adjoint),
                )?;
                validate_body_mode(a(&built.body)?, true)?;
                if built.symbol != kind {
                    return Err("constructed body is bound to a different closure".into());
                }
                c.source_owner = Some(std::sync::Arc::new(SourceOwner {
                    symbol: built.symbol.clone(),
                    oracle: built.oracle,
                    points: built.points,
                    rows: built.rows,
                    raw_reverse: rev,
                }));
                c.source_invocations.push(SourceInvocation {
                    invocation: path.to_vec(),
                    symbol: built.symbol.clone(),
                    oracle: built.oracle,
                });
                c.source_point = if built.oracle {
                    Some(SourcePoint::pseudo(kind, "$construct"))
                } else {
                    None
                };
                c.source_location = None;

                for symbol in built.symbols {
                    if !self.registry.contains_key(&symbol.hash) || c.has_name(&symbol.name) {
                        return Err("invalid constructed source symbol".into());
                    }
                    c.symbols.insert(symbol.name.clone(), symbol.hash);
                    c.sources.insert(
                        symbol.name,
                        SourceCapture {
                            oracle_invocation: symbol.oracle_invocation,
                            adjoint: symbol.adjoint,
                        },
                    );
                }
                self.body(a(&built.body)?, &mut c, path, rev, chain, out)?;
                chain.pop();
                return Ok(());
            }
        }
        self.body(a(f(&r, "body")?)?, &mut c, path, rev, chain, out)?;
        chain.pop();
        Ok(())
    }
    fn allocation_identity(&mut self, path: &[usize]) -> String {
        if self.source_factory.is_none() {
            return format!("@{:?}/{}", path, self.serial);
        }
        // A semantic invocation owns its expansion occurrences. Constructors
        // in a caller's recipe composition do not renumber callee scratch.
        let next = self.source_allocations.entry(path.to_vec()).or_default();
        let identity = format!("@{:?}/{}", path, next);
        *next += 1;
        identity
    }
    fn account_source_leaf(&mut self) -> Result<(), String> {
        if self.source_factory.is_some() {
            if self.source_leaf_count == self.source_leaf_limit {
                return Err(format!("{HOST_RESOURCE_LIMIT_PREFIX}source expansion exceeds this host's materialised leaf budget ({})", self.source_leaf_limit));
            }
            self.source_leaf_count += 1;
        }
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
        let n = if let Some(source) = &l.source {
            self.source_ordinals.entry(source.clone()).or_default()
        } else {
            self.ordinals.entry(l.invocation.clone()).or_default()
        };
        l.ordinal = *n;
        *n += 1;
        self.account_source_leaf()?;
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
        if self.source_depth == SOURCE_SCOPE_LIMIT {
            return Err(format!("{HOST_RESOURCE_LIMIT_PREFIX}source constructor nesting exceeds this host's expansion stack budget"));
        }
        self.source_depth += 1;
        let result = self.body_inner(b, c, path, rev, chain, out);
        self.source_depth -= 1;
        result
    }
    #[allow(clippy::too_many_arguments)]
    fn body_inner(
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
                source: c.location(),
            };
            if let Some(n) = j.get("visit") {
                let mut records = c.records.get(s(n)?).cloned().ok_or("unbound records")?;
                if rev {
                    records.reverse();
                }
                for record in records {
                    let mut local = c.clone();
                    // Fields shadow all captured binding types, just like template formals.
                    for n in record
                        .ints
                        .keys()
                        .chain(record.wires.keys())
                        .chain(record.handles.keys())
                        .chain(record.classes.keys())
                        .chain(record.images.keys())
                    {
                        local.clear_name(n);
                    }
                    local.scratch_names = c.scratch_names.clone();
                    local.merge(&record);
                    if self.source_factory.is_some() && s(n)? == "Frames" {
                        let owner = local
                            .source_owner
                            .as_ref()
                            .ok_or("frame visit has no source owner")?;
                        let index = record
                            .ints
                            .get("index")
                            .and_then(|n| n.to_i64())
                            .and_then(|n| usize::try_from(n).ok())
                            .ok_or("invalid source frame index")?;
                        local.source_steps.push(SourceStep {
                            invocation: path.to_vec(),
                            symbol: owner.symbol.clone(),
                            step: index + 1,
                            reverse: rev ^ owner.raw_reverse,
                        });
                        local.source_location = None;
                    }
                    self.body(a(f(j, "body")?)?, &mut local, path, rev, chain, out)?;
                }
            } else if let Some(n) = j.get("declare") {
                let n = s(n)?;
                if c.has_name(n) {
                    return Err("declaration shadows binding".into());
                }
                let width = usize::try_from(scalar(f(j, "width")?, &c.ints)?)
                    .map_err(|_| "negative declaration width")?;
                let cls = class(f(j, "class")?, c)?;
                let state = s(f(j, "state")?)?;
                if !matches!(state, "clean" | "borrowed") {
                    return Err("invalid allocation state".into());
                }
                let h = Rc::new(RefCell::new(Handle {
                    declared_scratch: c.scratch_names.contains(n),
                    scratch: c.scratch_names.contains(n),
                    metadata: (self.allocation_identity(path), width, cls, state.into()),
                    live: false,
                }));
                c.handles.insert(n.into(), h.clone());
                self.body(a(f(j, "body")?)?, c, path, rev, chain, out)?;
                if h.borrow().live {
                    return Err("declaration exits with live allocation".into());
                }
                c.handles.remove(n);
            } else if j.get("acquire").is_some() || j.get("relinquish").is_some() {
                let acquire = j.get("acquire").is_some() != rev;
                let n = s(j.get("acquire").or_else(|| j.get("relinquish")).unwrap())?;
                let mut h = c
                    .handles
                    .get(n)
                    .ok_or("unbound allocation declaration")?
                    .borrow_mut();
                if acquire {
                    h.scratch = h.declared_scratch || c.scratch_names.contains(n);
                }
                if h.scratch && h.metadata.3 != "clean" {
                    return Err("library scratch must be clean".into());
                }
                leaf.scratch = h.scratch;
                if h.live == acquire {
                    return Err("allocation lifetime mismatch".into());
                }
                h.live = acquire;
                if acquire {
                    h.metadata.0 = self.allocation_identity(path);
                }
                leaf.gate = if acquire { "alloc" } else { "release" }.into();
                leaf.allocation = Some(h.metadata.clone());
                leaf.controls.clear();
                self.emit(leaf, false, out)?;
            } else if let Some(n) = j.get("image") {
                keys(j, &["image", "wires"])?;
                let bytes = c.images.get(s(n)?).ok_or("unbound image")?;
                let ws = wires(f(j, "wires")?, c)?;
                if !bytes.is_empty() && ws.len() != bytes.len() * 8 {
                    return Err("image width mismatch".into());
                }
                let mut indices = (0..bytes.len() * 8).collect::<Vec<_>>();
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
                let mut local = (*captured).clone();
                for p in a(f(t, "parameters")?)? {
                    let name = if let Json::Str(n) = p {
                        n.as_str()
                    } else {
                        s(f(p, "name")?)?
                    };
                    local.clear_name(name);
                }
                local.merge(&bound);
                local.source_steps = bound.source_steps.clone();
                local.source_invocations = bound.source_invocations.clone();
                local.source_edges = bound.source_edges.clone();
                local.source_location = None;
                if self.source_factory.is_some() && matches!(s(n)?, "Rows" | "Next") {
                    let owner = local
                        .source_owner
                        .as_ref()
                        .filter(|o| o.oracle)
                        .ok_or("row template has no source oracle owner")?;
                    local.source_point = Some(
                        owner
                            .rows
                            .get(index)
                            .cloned()
                            .ok_or("source row index has no program point")?,
                    );
                }
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
                    let identity = self.allocation_identity(path);
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
                    if let Some(owner) = c.wire_owners.remove(name) {
                        let mut handle = owner.borrow_mut();
                        if !handle.live {
                            return Err("release of inactive allocation".into());
                        }
                        handle.live = false;
                    }
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
            } else if let Some(n) = j.get("source_cleanup") {
                keys(j, &["source_cleanup", "reverse", "recompute", "perm"])?;
                let factory = self
                    .source_factory
                    .as_ref()
                    .ok_or("source cleanup requires linked source")?;
                let point = if s(n)? == "$current" {
                    c.source_point.clone()
                } else {
                    c.source_owner
                        .as_ref()
                        .and_then(|o| o.points.get(s(n).ok()?))
                        .cloned()
                }
                .ok_or("cleanup carrier has no source point")?;
                let location = c
                    .location()
                    .ok_or("cleanup carrier has no source invocation")?;
                let mut carrier = (*location).clone();
                carrier.terminal = point;
                let recompute = factory.cleanup_recompute(&carrier)?;
                if recompute && f(j, "perm")?.as_bool() != Some(true) {
                    return Err("§6.4: recompute requires permutation provenance".into());
                }
                self.body(
                    a(f(j, if recompute { "recompute" } else { "reverse" })?)?,
                    c,
                    path,
                    rev,
                    chain,
                    out,
                )?;
            } else if let Some(n) = j.get("for") {
                keys(j, &["for", "from", "to", "body"])?;
                let n = s(n)?;
                let from = scalar(f(j, "from")?, &c.ints)?;
                let to = scalar(f(j, "to")?, &c.ints)?;
                if to < from {
                    return Err("reversed range".into());
                }
                let old_point = c.source_point.clone();
                let old_location = c.source_location.clone();
                if self.source_factory.is_some() && n.starts_with("$source-point/") {
                    c.source_point = Some(
                        c.source_owner
                            .as_ref()
                            .and_then(|o| o.points.get(n))
                            .cloned()
                            .ok_or("unbound source program point")?,
                    );
                    c.source_location = None;
                }
                let old = c.ints.get(n).cloned();
                let it: Box<dyn Iterator<Item = i64>> = if rev {
                    Box::new((from..to).rev())
                } else {
                    Box::new(from..to)
                };
                for v in it {
                    c.ints.insert(n.into(), v.into());
                    self.body(a(f(j, "body")?)?, c, path, rev, chain, out)?;
                }
                if let Some(v) = old {
                    c.ints.insert(n.into(), v);
                } else {
                    c.ints.remove(n);
                }
                c.source_point = old_point;
                c.source_location = old_location;
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
                let (args, mut captured) = if let Some(name) = f(j, "args")?.as_str() {
                    let CapturedArguments {
                        symbol: bound_hash,
                        actuals: args,
                        context,
                        source,
                    } = c.arguments.get(name).cloned().ok_or("unbound arguments")?;
                    if bound_hash != hash {
                        return Err("arguments closure mismatch".into());
                    }
                    let mut captured = (*context).clone();
                    captured.invocation_source = source;
                    (args, captured)
                } else {
                    let mut captured = c.clone();
                    captured.invocation_source = c.sources.get(s(n)?).cloned();
                    (a(f(j, "args")?)?.to_vec(), captured)
                };
                captured.controls.extend(c.controls.clone());
                captured.controls.sort();
                captured.controls.dedup();
                captured.source_steps = c.source_steps.clone();
                captured.source_invocations = c.source_invocations.clone();
                captured.source_location = None;
                let mut child = path.to_vec();
                if self.source_factory.is_some() {
                    if let Some(location) = captured.location() {
                        let count = self.source_calls.entry(location.clone()).or_default();
                        let index = *count;
                        *count += 1;
                        captured.source_edges.push(SourceEdge {
                            point: location.terminal.clone(),
                            call_index: index,
                        });
                        captured.source_location = None;
                    }
                    child.push(self.source_serial);
                    self.source_serial += 1;
                } else {
                    child.push(self.serial);
                }
                let mut semantic_chain = chain
                    .iter()
                    .filter(|name| self.registry.contains_key(*name))
                    .cloned()
                    .collect();
                self.invoke(
                    &hash,
                    &args,
                    &captured,
                    &child,
                    rev,
                    &mut semantic_chain,
                    out,
                )?;
            } else if let Some(n) = j.get("splice") {
                keys(j, &["splice"])?;
                let (b, captured) = c.bodies.get(s(n)?).cloned().ok_or("unbound body")?;
                let mut captured = (*captured).clone();
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

/// Invert an already expanded quantum trace in place, preserving its source
/// addresses, ordinals, invocation edges and allocation identities. Invalid
/// adjoints are detected before the trace is changed.
pub fn invert_trace(leaves: &mut [Leaf]) -> Result<(), String> {
    for leaf in leaves.iter() {
        adjoint_gate(&leaf.gate)?;
    }
    leaves.reverse();
    for leaf in leaves {
        adjoint_leaf(leaf)?;
    }
    Ok(())
}
fn adjoint_gate(gate: &str) -> Result<&str, String> {
    Ok(match gate {
        "s" => "sdg",
        "sdg" => "s",
        "t" => "tdg",
        "tdg" => "t",
        "alloc" => "release",
        "release" => "alloc",
        "x" | "h" | "cx" | "cz" | "ccx" | "mcx" | "swap" | "rx" | "ry" | "rz" => gate,
        "classical" => return Err("classical inverse requires retained history".into()),
        _ => return Err(format!("unknown inverse gate {gate}")),
    })
}
fn adjoint_leaf(leaf: &mut Leaf) -> Result<(), String> {
    leaf.gate = adjoint_gate(&leaf.gate)?.into();
    for angle in &mut leaf.angles {
        *angle ^= 1 << 63;
    }
    Ok(())
}
