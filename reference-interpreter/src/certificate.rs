//! Certificate schemas, bindings and references (§9.4, §12).
use crate::{
    err,
    json::Json,
    metadata::{hash, text, variant},
    sha256_hex, Error, Function, Object, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

fn contextual<T>(rule: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    f().map_err(|e| Error(format!("{rule}: {e}")))
}
fn context_key(key: &str) -> Result<(&str, u128)> {
    let (name, controls) = key
        .split_once(':')
        .ok_or_else(|| Error("§12: variant:inherited_controls key required".into()))?;
    variant(&Json::String(name.into()))?;
    let n = crate::object::host_count(controls, "inherited control count")?;
    Ok((name, n as u128))
}
/// The closed §8.1 AST; frame expressions bind only c.
pub(crate) fn expression(value: &Json, variables: &BTreeSet<&str>) -> Result<()> {
    crate::metadata_schema::validate("expression", value)?;
    match value {
        Json::Number(_) => {}
        Json::Object(fields) => {
            if fields.contains_key("var") {
                if !variables.contains(value.get("var")?.string()?) {
                    return err("§8.1", "expression references an unbound variable");
                }
            } else if fields.contains_key("op") {
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
        for slot in value.get("interface")?.array()? {
            if slot.get("width")?.uint()? == 0 {
                return err("§9.4", "zero-width frame slot");
            }
        }
        let variables = BTreeSet::from(["c"]);
        expression(value.get("retained_history")?, &variables)?;
        for key in ["scratch", "borrow", "scheme_ancillae"] {
            for count in value.get(key)?.object()?.values() {
                if key == "scratch"
                    && count
                        .object()
                        .is_ok_and(|m| m.contains_key("clean") || m.contains_key("dirty"))
                {
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
    let mut names = BTreeSet::new();
    for group in ["bundles", "borrowed"] {
        for b in value.get(group)?.array()? {
            let name = text(b.get("name")?)?;
            if !names.insert(name.to_string()) {
                return err("§12.2", "repeated domain component");
            }
            let width = b.get("width")?.uint()?;
            let bits =
                usize::try_from(width).map_err(|_| Error("host limit: domain width".into()))?;
            if width == 0 {
                return err("§12.2", "zero-width domain component");
            }
            // Bounds are exact integers of any size, within the bundle width.
            let lo = crate::BitVector::parse_unsigned_exact(bits, b.get("lo")?.uint_text()?)
                .map_err(|e| Error(format!("§12.2: domain lo outside bundle width: {e}")))?;
            let hi = crate::BitVector::parse_unsigned_exact(bits, b.get("hi")?.uint_text()?)
                .map_err(|e| Error(format!("§12.2: domain hi outside bundle width: {e}")))?;
            if lo.unsigned_cmp(&hi)? == std::cmp::Ordering::Greater {
                return err("§12.2", "domain interval with lo > hi");
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
            let index = cell.get("index")?.uint()?;
            let bits = usize::try_from(w).map_err(|_| Error("host limit: cell width".into()))?;
            let lo = crate::BitVector::parse_unsigned_exact(bits, cell.get("lo")?.uint_text()?)?;
            let hi = crate::BitVector::parse_unsigned_exact(bits, cell.get("hi")?.uint_text()?)?;
            if crate::oracle::power_of_two(a as u128).is_some_and(|cells| index >= cells)
                || !indices.insert(index)
                || lo.unsigned_cmp(&hi)? == std::cmp::Ordering::Greater
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
    hash(value.get("closure_hash")?)?;
    hash(value.get("schedule_hash")?)?;
    variant(value.get("variant")?)?;
    value.get("inherited_controls")?.uint()?;
    Ok(())
}
impl Object {
    pub(crate) fn validate_certificate_details(&self, c: &Json) -> Result<()> {
        crate::metadata_schema::validate("certificate", c)?;
        contextual("§12", || {
            let b = c.get("bindings")?;
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
                    index(pair.get("claim")?, claims.len())?;
                }
                let fault = v.get("fault_rate")?;

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
                        self.validate_path(cleanup.get("path")?, Some(name))?;
                    }
                }
                for key in r.get("error")?.object()?.keys() {
                    context_key(key)?;
                }
            }
            let composition = c.get("composition")?;
            for (key, n) in composition.get("multiplicities")?.object()? {
                let (closure, context) = key
                    .split_once(':')
                    .ok_or_else(|| Error("§12.3: malformed multiplicity key".into()))?;
                hash(&Json::String(closure.into()))?;
                context_key(context)?;
                n.uint()?;
            }
            Ok(())
        })
    }
    fn validate_claim(&self, c: &Json, f: &Function, position: usize) -> Result<()> {
        crate::metadata_schema::validate("claim", c)?;
        // §12.1–12.2: domain/name bindings, executable subjects and acyclic
        // indices need the enclosing function/claim list, beyond the grammar.
        match c.get("kind")?.string()? {
            "exhaustive" => {
                domain(c.get("domain")?, f)?;
                if c.get("expected")? == &Json::String("oracle".into()) {
                    subject(c.get("subject")?)?;
                }
            }
            "sampled" => {
                domain(c.get("domain")?, f)?;
                if !c
                    .get("sampler")?
                    .get("seed")?
                    .array()?
                    .iter()
                    .any(|w| w.string().is_ok_and(|s| s.bytes().any(|b| b != b'0')))
                {
                    return err("§12.1", "all-zero sampler seed");
                }
            }
            "structural" => {
                for claim in c.get("subclaims")?.array()? {
                    index(claim, position)?;
                }
            }
            "numerical" | "channel" => {
                subject(c.get("subject")?)?;
                if c.get("precision_bits")?.uint()? == 0 {
                    return err("§12.1", "precision must be positive");
                }
            }
            _ => return err("§12.1", "unknown certificate claim kind"),
        }
        Ok(())
    }
}
