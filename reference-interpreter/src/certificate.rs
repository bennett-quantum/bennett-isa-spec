//! Certificate schemas, bindings and references (§9.4, §12).
use crate::{
    err,
    json::Json,
    metadata::{hash, nonnegative, text},
    sha256_hex, Error, Function, Object, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

fn contextual<T>(rule: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    f().map_err(|e| Error(format!("{rule}: {e}")))
}
fn decimal(value: &Json) -> Result<&str> {
    let s = match value {
        Json::String(s) | Json::Number(s) => s,
        _ => return err("§12", "decimal required"),
    };
    if s.is_empty() {
        return err("§12", "empty decimal");
    }
    nonnegative(&Json::String(s.strip_prefix('-').unwrap_or(s).into()))?;
    Ok(s)
}
fn metric_map(value: &Json) -> Result<()> {
    value.fields(&[], &["operator_norm", "diamond"])?;
    for v in value.object()?.values() {
        nonnegative(v)?;
    }
    Ok(())
}
fn context_key(key: &str) -> Result<(&str, u128)> {
    let (variant, controls) = key
        .rsplit_once(':')
        .ok_or_else(|| Error("§12: variant:inherited_controls key required".into()))?;
    if variant.is_empty() {
        return err("§12", "empty variant");
    }
    Ok((
        variant,
        controls
            .parse()
            .map_err(|_| Error("§12: invalid inherited control count".into()))?,
    ))
}
/// The closed §8.1 AST; frame expressions bind only c.
pub(crate) fn expression(value: &Json, variables: &BTreeSet<&str>) -> Result<()> {
    match value {
        Json::Number(_) => {
            decimal(value)?;
        }
        Json::Object(fields) => {
            if fields.contains_key("var") {
                value.fields(&["var"], &[])?;
                if !variables.contains(value.get("var")?.string()?) {
                    return err("§8.1", "expression references an unbound variable");
                }
            } else if fields.contains_key("op") {
                value.fields(&["op", "args"], &[])?;
                let arity = match value.get("op")?.string()? {
                    "+" | "-" | "*" | "/" | "max" | "min" => 2,
                    "ceil" => 1,
                    "log2inv_eps" => 0,
                    _ => return err("§8.1", "unknown expression operator"),
                };
                let args = value.get("args")?.array()?;
                if args.len() != arity {
                    return err("§8.1", "expression arity mismatch");
                }
                if value.get("op")?.string()? == "log2inv_eps" && !variables.contains("eps") {
                    return err("§8.1", "epsilon is unbound");
                }
                for a in args {
                    expression(a, variables)?;
                }
            } else {
                value.fields(&["table", "index"], &[])?;
                text(value.get("table")?)?;
                let index = value.get("index")?.string()?;
                if (index == "k" && !variables.contains("k"))
                    || (index == "eps_decade" && !variables.contains("eps"))
                    || !["k", "eps_decade"].contains(&index)
                {
                    return err("§8.1", "unbound table index");
                }
            }
        }
        _ => return err("§8.1", "expression must be a number or closed AST"),
    }
    Ok(())
}
fn frame(value: &Json) -> Result<()> {
    contextual("§9.4", || {
        value.fields(
            &[
                "interface",
                "scratch",
                "retained_history",
                "borrow",
                "scheme_ancillae",
            ],
            &["live_ranges"],
        )?;
        for slot in value.get("interface")?.array()? {
            slot.fields(&["role", "width", "class"], &["name"])?;
            if !["in", "mut", "tgt", "fault", "control"].contains(&text(slot.get("role")?)?) {
                return err("§9.4", "invalid interface role");
            }
            if slot.get("width")?.uint()? == 0 {
                return err("§9.4", "zero-width frame slot");
            }
            text(slot.get("class")?)?;
            if let Some(name) = slot.object()?.get("name") {
                text(name)?;
            }
        }
        let variables = BTreeSet::from(["c"]);
        expression(value.get("retained_history")?, &variables)?;
        for key in ["scratch", "borrow", "scheme_ancillae"] {
            for (class, count) in value.get(key)?.object()? {
                if class.is_empty() {
                    return err("§9.4", "empty frame class");
                }
                if key == "scratch"
                    && count
                        .object()
                        .is_ok_and(|m| m.contains_key("clean") || m.contains_key("dirty"))
                {
                    count.fields(&["clean", "dirty"], &[])?;
                    expression(count.get("clean")?, &variables)?;
                    expression(count.get("dirty")?, &variables)?;
                } else {
                    expression(count, &variables)?;
                }
            }
        }
        if let Some(ranges) = value.object()?.get("live_ranges") {
            let mut grouped: BTreeMap<String, Vec<(u128, u128)>> = BTreeMap::new();
            for r in ranges.array()? {
                r.fields(&["frame", "start", "end"], &[])?;
                let id = text(r.get("frame")?)?.to_string();
                let lo = r.get("start")?.uint()?;
                let hi = r.get("end")?.uint()?;
                if lo >= hi {
                    return err("§9.4", "empty or inverted shared-frame live range");
                }
                let prior = grouped.entry(id).or_default();
                if prior.iter().any(|(a, b)| lo < *b && *a < hi) {
                    return err("§9.4", "shared frame has overlapping live ranges");
                }
                prior.push((lo, hi));
            }
        }
        Ok(())
    })
}
fn domain(value: &Json, f: &Function) -> Result<()> {
    value.fields(&["bundles", "borrowed", "regions"], &[])?;
    let mut names = BTreeSet::new();
    for group in ["bundles", "borrowed"] {
        for b in value.get(group)?.array()? {
            b.fields(&["name", "width", "lo", "hi"], &[])?;
            let name = text(b.get("name")?)?;
            if !names.insert(name.to_string()) {
                return err("§12.2", "repeated domain component");
            }
            let width = b.get("width")?.uint()?;
            let lo = b.get("lo")?.uint()?;
            let hi = b.get("hi")?.uint()?;
            if width == 0 || lo > hi || (width < 128 && hi >= 1u128 << (width as u32)) {
                return err("§12.2", "domain interval outside bundle width");
            }
            if group == "bundles" {
                let p = f
                    .params
                    .iter()
                    .flatten()
                    .find(|p| p.name == name && matches!(p.ty, Type::Wire(_)))
                    .ok_or_else(|| Error("§12.2: unknown domain interface bundle".into()))?;
                if p.ty.width(32)? as u128 != width {
                    return err("§12.2", "domain bundle width mismatch");
                }
            }
        }
    }
    for region in value.get("regions")?.array()? {
        region.fields(&["name", "cells"], &[])?;
        let name = text(region.get("name")?)?;
        if !names.insert(name.to_string()) {
            return err("§12.2", "repeated domain region");
        }
        let (a, w) = f
            .params
            .iter()
            .flatten()
            .find_map(|p| {
                if p.name == name {
                    if let Type::Region(a, w) = p.ty {
                        Some((a, w))
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .ok_or_else(|| Error("§12.2: unknown domain interface region".into()))?;
        let cells = region.get("cells")?;
        if cells == &Json::String("all".into()) {
            continue;
        }
        let mut indices = BTreeSet::new();
        for cell in cells.array()? {
            cell.fields(&["index", "lo", "hi"], &[])?;
            let index = cell.get("index")?.uint()?;
            let lo = cell.get("lo")?.uint()?;
            let hi = cell.get("hi")?.uint()?;
            if (a < 128 && index >= 1u128 << a)
                || !indices.insert(index)
                || lo > hi
                || (w < 128 && hi >= 1u128 << w)
            {
                return err("§12.2", "invalid domain cell interval");
            }
        }
    }
    Ok(())
}
fn index(value: &Json, count: usize) -> Result<usize> {
    let n = usize::try_from(value.uint()?)
        .map_err(|_| Error("§12: claim index exceeds host range".into()))?;
    if n >= count {
        return err("§12", "claim index out of range or cyclic subclaim");
    }
    Ok(n)
}
fn subject(value: &Json) -> Result<()> {
    value.fields(
        &[
            "closure_hash",
            "variant",
            "inherited_controls",
            "schedule_hash",
        ],
        &[],
    )?;
    hash(value.get("closure_hash")?)?;
    hash(value.get("schedule_hash")?)?;
    text(value.get("variant")?)?;
    value.get("inherited_controls")?.uint()?;
    Ok(())
}
impl Object {
    pub(crate) fn validate_certificate_details(&self, c: &Json) -> Result<()> {
        contextual("§12", || {
            let versions = c.get("versions")?;
            versions.fields(
                &[
                    "isa", "object", "launch", "sched", "cert", "coverage", "target",
                ],
                &[],
            )?;
            for version in versions.object()?.values() {
                if version.string()? != "0.8" {
                    return err("§12", "certificate version mismatch");
                }
            }
            let b = c.get("bindings")?;
            for key in [
                "object_hash",
                "interface_hash",
                "closure_hash",
                "reference_program_hash",
                "target_hash",
            ] {
                hash(b.get(key)?)?;
            }
            text(b.get("dialect_mapping_version")?)?;
            let identity = self.identity()?;
            if b.get("object_hash")?.string()? != identity.object_hash {
                return err("§12", "certificate object hash mismatch");
            }
            let interface = hash(b.get("interface_hash")?)?;
            if !identity.interface_hashes.values().any(|h| h == interface) {
                return err("§12", "certificate interface hash binding mismatch");
            }
            if !identity.closure_hashes.values().any(|h| {
                h.as_deref() == Some(b.get("closure_hash").and_then(Json::string).unwrap_or(""))
            }) {
                return err("§12", "certificate closure hash binding mismatch");
            }
            if let Some(schedule_hash) = b.object()?.get("schedule_hash") {
                hash(schedule_hash)?;
                if let Some(s) = self.section("schedule") {
                    if schedule_hash.string()? != sha256_hex(s.canonical().as_bytes()) {
                        return err("§12", "certificate schedule hash mismatch");
                    }
                }
            }
            let producer = c.get("producer")?;
            text(producer.get("name")?)?;
            text(producer.get("version")?)?;
            for value in producer.get("materialisation")?.object()?.values() {
                value.string()?;
            }
            for (closure, r) in c.get("regions")?.object()? {
                hash(&Json::String(closure.clone()))?;
                let name = identity
                    .closure_hashes
                    .iter()
                    .find_map(|(n, h)| (h.as_deref() == Some(closure.as_str())).then_some(n))
                    .ok_or_else(|| Error("§12: region closure is not defined".into()))?;
                let f = self
                    .functions
                    .get(name)
                    .ok_or_else(|| Error("§12: certificate region is data".into()))?;
                if r.get("kind")?.string()? != f.kind || r.get("signature")? != &f.signature() {
                    return err("§12", "region signature mismatch");
                }
                let bounds = r
                    .get("bounds")?
                    .array()?
                    .iter()
                    .map(Json::uint)
                    .collect::<Result<Vec<_>>>()?;
                if bounds != f.bounds.iter().map(|n| *n as u128).collect::<Vec<_>>() {
                    return err("§12", "certificate bounds mismatch");
                }
                frame(r.get("frame")?)?;
                for value in r.get("resources")?.object()?.values() {
                    nonnegative(value)?;
                }
                for value in r.get("gates_used")?.object()?.values() {
                    value.fields(&["count"], &["scheme", "epsilon"])?;
                    value.get("count")?.uint()?;
                    if let Some(s) = value.object()?.get("scheme") {
                        text(s)?;
                    }
                    if let Some(e) = value.object()?.get("epsilon") {
                        nonnegative(e)?;
                    }
                }
                let claims = r.get("claims")?.array()?;
                for (position, claim) in claims.iter().enumerate() {
                    self.validate_claim(claim, f, position)?;
                }
                let v = r.get("verification")?;
                if v.get("oracle")? != &Json::Null {
                    index(v.get("oracle")?, claims.len())?;
                }
                for group in ["borrow_frames", "unitary", "channel"] {
                    for claim in v.get(group)?.array()? {
                        index(claim, claims.len())?;
                    }
                }
                for pair in v.get("inverse_pairs")?.array()? {
                    pair.fields(&["g", "ginv", "claim"], &[])?;
                    hash(pair.get("g")?)?;
                    hash(pair.get("ginv")?)?;
                    index(pair.get("claim")?, claims.len())?;
                }
                let fault = v.get("fault_rate")?;
                fault.fields(
                    &[
                        "domain",
                        "faults_by_code",
                        "undefined_excluded",
                        "layout_dependent_excluded",
                    ],
                    &[],
                )?;
                domain(fault.get("domain")?, f)?;
                for (code, count) in fault.get("faults_by_code")?.object()? {
                    code.parse::<u128>()
                        .map_err(|_| Error("§12: invalid fault code key".into()))?;
                    count.uint()?;
                }
                fault.get("undefined_excluded")?.uint()?;
                fault.get("layout_dependent_excluded")?.uint()?;
                for (key, cleanups) in r.get("cleanup")?.object()? {
                    context_key(key)?;
                    for cleanup in cleanups.array()? {
                        cleanup.fields(&["path", "realisation"], &[])?;
                        self.validate_path(cleanup.get("path")?, Some(name))?;
                        if !["reverse", "recompute"]
                            .contains(&cleanup.get("realisation")?.string()?)
                        {
                            return err("§12", "invalid cleanup realisation");
                        }
                    }
                }
                for (key, error) in r.get("error")?.object()? {
                    context_key(key)?;
                    error.fields(&["local", "inclusive"], &[])?;
                    metric_map(error.get("local")?)?;
                    metric_map(error.get("inclusive")?)?;
                }
                for assumption in r.get("definedness_assumptions")?.array()? {
                    text(assumption)?;
                }
            }
            let composition = c.get("composition")?;
            if composition.get("method")?.string()? != "sum_over_multiplicities" {
                return err("§12.3", "unknown composition method");
            }
            for (key, n) in composition.get("multiplicities")?.object()? {
                let (closure, context) = key
                    .split_once(':')
                    .ok_or_else(|| Error("§12.3: malformed multiplicity key".into()))?;
                hash(&Json::String(closure.into()))?;
                context_key(context)?;
                n.uint()?;
            }
            composition
                .get("epsilon_total")?
                .fields(&["diamond"], &[])?;
            nonnegative(composition.get("epsilon_total")?.get("diamond")?)?;
            Ok(())
        })
    }
    fn validate_claim(&self, c: &Json, f: &Function, position: usize) -> Result<()> {
        match c.get("kind")?.string()? {
            "exhaustive" => {
                c.fields(&["kind", "domain", "expected"], &["subject", "label"])?;
                domain(c.get("domain")?, f)?;
                let expected = c.get("expected")?;
                if expected == &Json::String("oracle".into()) {
                    subject(c.get("subject")?)?;
                } else {
                    let e = expected.object()?;
                    if e.contains_key("table") {
                        expected.fields(&["table"], &[])?;
                        expected.get("table")?.array()?;
                    } else {
                        expected.fields(&["program"], &[])?;
                        hash(expected.get("program")?)?;
                    }
                }
            }
            "sampled" => {
                c.fields(&["kind", "domain", "sampler"], &["label"])?;
                domain(c.get("domain")?, f)?;
                let sampler = c.get("sampler")?;
                sampler.fields(&["algorithm", "seed", "count"], &[])?;
                if sampler.get("algorithm")?.string()? != "xoshiro256**" {
                    return err("§12.1", "unsupported sampler algorithm");
                }
                let seed = sampler.get("seed")?.array()?;
                if seed.len() != 4 {
                    return err("§12.1", "xoshiro seed must have four words");
                }
                let mut nonzero = false;
                for word in seed {
                    let n = word.string()?.parse::<u64>().map_err(|_| {
                        Error("§12.1: seed words must be u64 decimal strings".into())
                    })?;
                    nonzero |= n != 0;
                }
                if !nonzero {
                    return err("§12.1", "all-zero sampler seed");
                }
                sampler.get("count")?.uint()?;
            }
            "structural" => {
                c.fields(&["kind", "rule", "subclaims", "witness"], &["label"])?;
                for claim in c.get("subclaims")?.array()? {
                    index(claim, position)?;
                }
                let witness = c.get("witness")?;
                match c.get("rule")?.string()? {
                    "adjoint_table" => witness.fields(&[], &[])?,
                    "inverse_pair_library" => {
                        witness.fields(&["library_closure_hash", "claim_index"], &[])?;
                        hash(witness.get("library_closure_hash")?)?;
                        witness.get("claim_index")?.uint()?;
                    }
                    "translation_lemma" => {
                        witness.fields(&["V", "W", "X", "B", "frame"], &[])?;
                        for group in ["V", "W", "frame"] {
                            for point in witness.get(group)?.array()? {
                                let p = point.array()?;
                                if p.len() != 3 {
                                    return err(
                                        "§12",
                                        "structural point must be a program-point tuple",
                                    );
                                }
                                hash(&p[0])?;
                                text(&p[1])?;
                                p[2].uint()?;
                            }
                        }
                        for group in ["X", "B"] {
                            for wire in witness.get(group)?.array()? {
                                text(wire)?;
                            }
                        }
                        if witness.get("frame")?.array()?.len() != 2 {
                            return err("§12", "frame needs first and last points");
                        }
                    }
                    _ => return err("§12", "unknown structural rule"),
                }
            }
            "numerical" | "channel" => {
                let channel = c.get("kind")?.string()? == "channel";
                let required = if channel {
                    vec![
                        "kind",
                        "subject",
                        "metric",
                        "precision_bits",
                        "bound",
                        "dual",
                    ]
                } else {
                    vec!["kind", "subject", "metric", "precision_bits", "bound"]
                };
                c.fields(&required, &["label"])?;
                subject(c.get("subject")?)?;
                if c.get("metric")?.string()? != if channel { "diamond" } else { "operator_norm" } {
                    return err("§12.1", "claim kind and metric mismatch");
                }
                if c.get("precision_bits")?.uint()? == 0 {
                    return err("§12.1", "precision must be positive");
                }
                nonnegative(c.get("bound")?)?;
                if channel {
                    for witness in c.get("dual")?.array()? {
                        witness.fields(&["Y", "mu"], &[])?;
                        nonnegative(witness.get("mu")?)?;
                        for entry in witness.get("Y")?.array()? {
                            let entry = entry.array()?;
                            if entry.len() != 4 {
                                return err(
                                    "§12.1",
                                    "interval complex entry requires four endpoints",
                                );
                            }
                            for endpoint in entry {
                                decimal(endpoint)?;
                            }
                        }
                    }
                }
            }
            _ => return err("§12.1", "unknown certificate claim kind"),
        }
        if let Some(label) = c.object()?.get("label") {
            if !["proof", "evidence"].contains(&label.string()?) {
                return err("§12", "unknown claim label");
            }
        }
        Ok(())
    }
}
