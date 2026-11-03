//! Certificate schemas, bindings and references (§9.4, §12).
use crate::{
    err,
    json::Json,
    metadata::{hash, text, variant},
    sha256_hex, Error, Function, Object, Result, Type,
};
use std::collections::{BTreeMap, BTreeSet};

fn contextual<T>(rule: &str, f: impl FnOnce() -> Result<T>) -> Result<T> {
    f().map_err(|e| e.context(rule))
}
fn context_key(key: &str) -> Result<(&str, u128)> {
    let (name, controls) = key
        .split_once(':')
        .ok_or_else(|| Error::new("§12: variant:inherited_controls key required".into()))?;
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
fn frame_width(v: &Json, c: u128) -> Result<u128> {
    // Evaluate with exact rational arithmetic; large intermediates report host
    // capacity rather than changing the declared width.
    fn eval(v: &Json, c: u128) -> Result<(i128, i128)> {
        let capacity =
            || crate::resource_error("§9.4", "frame expression exceeds rational host capacity");
        if matches!(v, Json::Number(_)) {
            let t = v.canonical();
            let (mantissa, exp) = t.split_once('e').map_or(Ok((t.as_str(), 0i32)), |(m, e)| {
                e.parse::<i32>().map(|e| (m, e)).map_err(|_| capacity())
            })?;
            let (whole, fractional) = mantissa.split_once('.').unwrap_or((mantissa, ""));
            let scale = (fractional.len() as i32)
                .checked_sub(exp)
                .ok_or_else(capacity)?;
            let mut num = format!("{whole}{fractional}")
                .parse::<i128>()
                .map_err(|_| capacity())?;
            let den = if scale >= 0 {
                10i128.checked_pow(scale as u32).ok_or_else(capacity)?
            } else {
                num = num
                    .checked_mul(
                        10i128
                            .checked_pow(scale.unsigned_abs())
                            .ok_or_else(capacity)?,
                    )
                    .ok_or_else(capacity)?;
                1
            };
            return Ok((num, den));
        }
        if v.object()?.contains_key("var") {
            return Ok((i128::try_from(c).map_err(|_| capacity())?, 1));
        }
        let args = v
            .get("args")?
            .array()?
            .iter()
            .map(|a| eval(a, c))
            .collect::<Result<Vec<_>>>()?;
        let op = v.get("op")?.string()?;
        let (a, b) = args[0];
        if op == "ceil" {
            return Ok((a.div_euclid(b) + i128::from(a.rem_euclid(b) != 0), 1));
        }
        let (d, e) = args[1];
        let mul = |x: i128, y: i128| x.checked_mul(y).ok_or_else(capacity);
        let (mut n, mut q) = match op {
            "+" => (
                mul(a, e)?.checked_add(mul(d, b)?).ok_or_else(capacity)?,
                mul(b, e)?,
            ),
            "-" => (
                mul(a, e)?.checked_sub(mul(d, b)?).ok_or_else(capacity)?,
                mul(b, e)?,
            ),
            "*" => (mul(a, d)?, mul(b, e)?),
            "/" if d != 0 => (mul(a, e)?, mul(b, d)?),
            "min" | "max" => {
                let less = mul(a, e)? < mul(d, b)?;
                if less == (op == "min") {
                    (a, b)
                } else {
                    (d, e)
                }
            }
            _ => return err("§9.4", "undefined frame expression"),
        };
        if q < 0 {
            n = n.checked_neg().ok_or_else(capacity)?;
            q = q.checked_neg().ok_or_else(capacity)?;
        }
        let (mut x, mut y) = (n.unsigned_abs(), q as u128);
        while y != 0 {
            (x, y) = (y, x % y);
        }
        Ok((n / (x as i128), q / (x as i128)))
    }
    expression(v, &BTreeSet::from(["c"]))?;
    let (n, d) = eval(v, c)?;
    if n < 0 || n % d != 0 {
        return err("§9.4", "frame width must be a nonnegative integer");
    }
    Ok((n / d) as u128)
}
fn frame09(value: &Json, f: &Function, contexts: &[u128]) -> Result<()> {
    let mut ids = BTreeSet::new();
    let mut frames = BTreeSet::new();
    for group in ["slots", "scratch", "history", "borrow", "ancillae"] {
        for record in value.get(group)?.array()? {
            let id = record.get("id")?.string()?;
            if !ids.insert(id) {
                return err("§9.4", "duplicate frame identity");
            }
            if group != "slots" {
                frames.insert(id);
            }
            for c in contexts {
                frame_width(record.get("width")?, *c)?;
            }
        }
    }
    let mut expected = Vec::new();
    let groups: &[(&str, usize)] = match f.kind.as_str() {
        "oracle" => &[("in", 0), ("tgt", 1), ("fault", 2), ("mem", 3)],
        "action" | "perm_action" => &[("mut", 0), ("in", 1), ("mem", 2)],
        _ => &[],
    };
    for (group, index) in groups {
        for (i, p) in f.params[*index].iter().enumerate() {
            let (role, width) = match p.ty {
                Type::Wire(w) => ((*group).to_string(), w as u128),
                Type::Region(a, w) => (
                    format!("mem_{}", p.role.as_deref().unwrap_or("in")),
                    crate::oracle::power_of_two(a as u128)
                        .and_then(|n| n.checked_mul(w as u128))
                        .ok_or_else(|| {
                            crate::resource_error("§9.4", "interface region width exceeds u128")
                        })?,
                ),
                _ => return err("§9.4", "invalid frame interface type"),
            };
            expected.push((format!("{group}:{i}"), role, width));
        }
    }
    let slots = value.get("slots")?.array()?;
    if slots.len() != expected.len() {
        return err("§9.4", "frame interface slot count mismatch");
    }
    for (slot, (id, role, width)) in slots.iter().zip(expected) {
        if slot.get("id")?.string()? != id || slot.get("role")?.string()? != role || width == 0 {
            return err("§9.4", "frame interface identity or role mismatch");
        }
        for c in contexts {
            if frame_width(slot.get("width")?, *c)? != width {
                return err("§9.4", "frame interface width mismatch");
            }
        }
    }
    if let Some(ranges) = value.object()?.get("live_ranges") {
        let mut prior = BTreeMap::<&str, Vec<(u128, u128)>>::new();
        for r in ranges.array()? {
            let id = r.get("frame")?.string()?;
            let (lo, hi) = (r.get("start")?.uint()?, r.get("end")?.uint()?);
            if !frames.contains(id)
                || lo >= hi
                || prior
                    .entry(id)
                    .or_default()
                    .iter()
                    .any(|(a, b)| lo < *b && *a < hi)
            {
                return err("§9.4", "invalid or overlapping non-slot frame live range");
            }
            prior.entry(id).or_default().push((lo, hi));
        }
    }
    Ok(())
}
fn table09(c: &Json, f: &Function) -> Result<()> {
    let expected = c.get("expected")?;
    let Some(table) = expected.object().ok().and_then(|m| m.get("table")) else {
        return Ok(());
    };
    let controls = c.get("subject")?.get("inherited_controls")?.uint()?;
    let widths = f
        .params
        .iter()
        .flatten()
        .filter_map(|p| {
            if let Type::Wire(w) = p.ty {
                Some(w as u128)
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let borrowed = c
        .get("domain")?
        .get("borrowed")?
        .array()?
        .iter()
        .map(|p| p.get("width")?.uint())
        .collect::<Result<Vec<_>>>()?;
    let regions = f
        .params
        .iter()
        .flatten()
        .filter_map(|p| {
            if let Type::Region(a, w) = p.ty {
                Some((a, w))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut before = BTreeSet::new();
    for row in table.array()? {
        if !before.insert(row.get("before")?.canonical()) {
            return err("§12.1 K13", "duplicate table input tuple");
        }
        for key in ["before", "after"] {
            let tuple = row.get(key)?;
            let bundles = tuple.get("bundles")?.array()?;
            if controls
                .checked_add(widths.len() as u128)
                .is_none_or(|n| n != bundles.len() as u128)
            {
                return err("§12.1 K13", "incomplete table bundle tuple");
            }
            for (i, b) in bundles.iter().enumerate() {
                let w = if (i as u128) < controls {
                    1
                } else {
                    widths[i - controls as usize]
                };
                crate::object::unsigned_literal(b.string()?, w)?;
            }
            let values = tuple.get("borrowed")?.array()?;
            if values.len() != borrowed.len() {
                return err("§12.1 K13", "incomplete borrowed tuple");
            }
            for (v, w) in values.iter().zip(&borrowed) {
                crate::object::unsigned_literal(v.string()?, *w)?;
            }
            let cells = tuple.get("regions")?.array()?;
            if cells.len() != regions.len() {
                return err("§12.1 K13", "incomplete region tuple");
            }
            for (r, (a, w)) in cells.iter().zip(&regions) {
                let values = r.array()?;
                if crate::oracle::power_of_two(*a as u128) != Some(values.len() as u128) {
                    return err("§12.1 K13", "incomplete region cells");
                }
                for v in values {
                    crate::object::unsigned_literal(v.string()?, *w as u128)?;
                }
            }
        }
    }
    Ok(())
}

pub(crate) fn invocation_budget(value: &Json) -> Result<crate::Budget> {
    let v = value.array()?;
    let n = v
        .iter()
        .map(|v| {
            v.string()?.parse::<u128>().map_err(|_| {
                crate::resource_error("§12.3 K13", "invocation component exceeds u128")
            })
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(crate::Budget {
        steps: u64::try_from(n[0])
            .map_err(|_| crate::resource_error("§12.3 K13", "invocation steps exceed u64"))?,
        heap: n[1],
        stack: n[2],
        depth: n[3],
    })
}

#[cfg(test)]
fn domain(value: &Json, f: &Function) -> Result<()> {
    domain_for(value, f, crate::IsaVersion::V08, 0)
}
pub(crate) fn domain_for(
    value: &Json,
    f: &Function,
    version: crate::IsaVersion,
    controls: u128,
) -> Result<()> {
    let mut names = BTreeSet::new();
    for group in ["bundles", "borrowed"] {
        let entries = value.get(group)?.array()?;
        if group == "bundles" && (entries.len() as u128) < controls {
            return err("§12.2 K72", "domain omits inherited control positions");
        }
        for (position, b) in entries.iter().enumerate() {
            let inherited = group == "bundles" && (position as u128) < controls;
            let name = text(b.get("name")?)?;
            let label_only =
                inherited || (version == crate::IsaVersion::V09 && group == "borrowed");
            if !label_only && !names.insert((group, name.to_string())) {
                return err("§12.2", "repeated domain component");
            }
            let width = b.get("width")?.uint()?;
            if width == 0 || (inherited && width != 1) {
                return err("§12.2", "zero-width domain component");
            }
            // Admission compares the spelling's magnitude, never allocates width
            // bits for an endpoint. Resource errors retain their own diagnostic.
            let lo = b.get("lo")?.uint_text()?;
            let hi = b.get("hi")?.uint_text()?;
            crate::object::unsigned_literal(lo, width)?;
            crate::object::unsigned_literal(hi, width)?;
            if (lo.len(), lo) > (hi.len(), hi) {
                return err("§12.2", "domain interval with lo > hi");
            }
            if group == "bundles" && !inherited {
                let p = f
                    .params
                    .iter()
                    .flatten()
                    .find(|p| p.name == name && matches!(p.ty, Type::Wire(_)))
                    .ok_or_else(|| Error::new("§12.2: unknown domain interface bundle".into()))?;
                if p.ty.width(32)? as u128 != width {
                    return err("§12.2", "domain bundle width mismatch");
                }
            }
        }
    }
    for region in value.get("regions")?.array()? {
        let name = text(region.get("name")?)?;
        if !names.insert(("regions", name.to_string())) {
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
            .ok_or_else(|| Error::new("§12.2: unknown domain interface region".into()))?;
        if w == 0 {
            return err("§12.2", "zero-width domain cell");
        }
        let cells = region.get("cells")?;
        if cells == &Json::String("all".into()) {
            continue;
        }
        let mut indices = BTreeSet::new();
        for cell in cells.array()? {
            let index = cell.get("index")?.uint()?;
            let lo = cell.get("lo")?.uint_text()?;
            let hi = cell.get("hi")?.uint_text()?;
            crate::object::unsigned_literal(lo, w as u128)?;
            crate::object::unsigned_literal(hi, w as u128)?;
            if crate::oracle::power_of_two(a as u128).is_some_and(|cells| index >= cells)
                || !indices.insert(index)
                || (lo.len(), lo) > (hi.len(), hi)
            {
                return err("§12.2", "invalid domain cell interval");
            }
        }
    }
    Ok(())
}
fn index(value: &Json, count: usize) -> Result<usize> {
    let n = usize::try_from(value.uint()?)
        .map_err(|_| Error::new("§12: claim index exceeds host range".into()))?;
    if n >= count {
        return err("§12", "claim index out of range or cyclic subclaim");
    }
    Ok(n)
}
fn subject(value: &Json) -> Result<()> {
    hash(value.get("closure_hash")?)?;
    if let Some(h) = value.object()?.get("schedule_hash") {
        hash(h)?;
    }
    variant(value.get("variant")?)?;
    value.get("inherited_controls")?.uint()?;
    Ok(())
}
impl Object {
    pub(crate) fn validate_invocation09(&self, v: &Json, f: &Function) -> Result<Vec<u64>> {
        crate::metadata_schema::validate_for(self.isa_version, "invocation09", v)?;
        self.validate_budget(invocation_budget(v.get("budget")?)?)?;
        let args = v.get("args")?.array()?;
        let groups: &[usize] = match f.kind.as_str() {
            "oracle" => &[4],
            "action" | "perm_action" => &[3],
            "lproc" => &[2, 3],
            "cc" | "r" => &[0],
            _ => return err("§12.3 K13", "invocation must bind a function"),
        };
        let params = groups
            .iter()
            .flat_map(|g| &f.params[*g])
            .filter(|p| matches!(p.ty, Type::Int(_) | Type::Ptr | Type::F64 | Type::Token))
            .collect::<Vec<_>>();
        if args.len() != params.len() {
            return err("§12.3 K13", "invocation argument count mismatch");
        }
        for (a, p) in args.iter().zip(&params) {
            match p.ty {
                Type::Int(w) => {
                    crate::metadata_schema::validate_for(
                        self.isa_version,
                        "canonical_decimal09",
                        a,
                    )?;
                    crate::object::unsigned_literal(a.string()?, w as u128)?;
                }
                Type::Ptr => {
                    crate::metadata_schema::validate_for(
                        self.isa_version,
                        "canonical_decimal09",
                        a,
                    )?;
                    crate::object::unsigned_literal(a.string()?, self.ptr_width as u128)?;
                }
                Type::Token => self.validate_budget(invocation_budget(a)?)?,
                Type::F64 => {
                    let t = a.string()?;
                    if t.len() != 18
                        || !t.starts_with("0x")
                        || !t[2..]
                            .bytes()
                            .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
                    {
                        return err("§12.3 K13", "invalid f64 bit pattern");
                    }
                }
                _ => unreachable!(),
            }
        }
        let mut tokens = Vec::new();
        if f.kind == "oracle" {
            tokens.push(Some(invocation_budget(v.get("budget")?)?));
        }
        let mut argument_tokens = 0;
        for (a, p) in args.iter().zip(&params) {
            if p.ty == Type::Token {
                tokens.push(Some(invocation_budget(a)?));
                argument_tokens += 1;
            }
        }
        crate::tokens::alias_binding(
            v.object()?.get("token_aliases"),
            &tokens,
            argument_tokens == 0,
        )
    }
    pub(crate) fn validate_certificate_details(&self, c: &Json) -> Result<()> {
        crate::metadata_schema::validate_for(self.isa_version, "certificate", c)?;
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
            if self.isa_version == crate::IsaVersion::V09 {
                let root = identity
                    .closure_hashes
                    .iter()
                    .find_map(|(n, h)| {
                        (h.as_deref()
                            == Some(b.get("closure_hash").and_then(Json::string).unwrap_or("")))
                        .then_some(n)
                    })
                    .ok_or_else(|| {
                        Error::new("§12.3: certificate root closure is not defined".into())
                    })?;
                let root_function = self.functions.get(root).ok_or_else(|| {
                    Error::new("§12.3: certificate root must name a function, not data".into())
                })?;
                let invocation = b.object()?.get("invocation");
                if self.section("schedule").is_some() && invocation.is_none() {
                    return err("§12.3 K13", "scheduled certificate requires invocation");
                }
                if let Some(v) = invocation {
                    let invocation_aliases = self.validate_invocation09(v, root_function)?;
                    if let Some(schedule) = self.section("schedule") {
                        let sb = schedule.get("bindings")?;
                        let values = v.get("budget")?.array()?;
                        for (actual, wanted) in values.iter().zip(sb.get("budget")?.array()?) {
                            if actual.string()? != wanted.uint_text()? {
                                return err("§12.3 K13", "invocation and schedule budget differ");
                            }
                        }
                        if sha256_hex(v.get("args")?.canonical().as_bytes())
                            != sb.get("params_hash")?.string()?
                        {
                            return err("§12.3 K13", "invocation params hash mismatch");
                        }
                        let implicit = if root_function.kind == "oracle" {
                            invocation_aliases.len() == 1
                        } else {
                            invocation_aliases.is_empty()
                        };
                        let tuples = vec![None; invocation_aliases.len()];
                        let schedule_aliases = crate::tokens::alias_binding(
                            sb.object()?.get("token_aliases"),
                            &tuples,
                            implicit,
                        )?;
                        if invocation_aliases != schedule_aliases {
                            return err(
                                "§12.3 K91",
                                "invocation and schedule token_aliases differ",
                            );
                        }
                    }
                }
            }
            for (closure, r) in c.get("regions")?.object()? {
                hash(&Json::String(closure.clone()))?;
                let name = identity
                    .closure_hashes
                    .iter()
                    .find_map(|(n, h)| (h.as_deref() == Some(closure.as_str())).then_some(n))
                    .ok_or_else(|| Error::new("§12: region closure is not defined".into()))?;
                let f = self
                    .functions
                    .get(name)
                    .ok_or_else(|| Error::new("§12: certificate region is data".into()))?;
                if r.get("kind")?.string()? != f.kind
                    || r.get("signature")? != &f.signature_for(self.isa_version)
                {
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
                if self.isa_version == crate::IsaVersion::V09 {
                    let mut counts = r
                        .get("error")?
                        .object()?
                        .keys()
                        .map(|key| context_key(key).map(|(_, n)| n))
                        .collect::<Result<Vec<_>>>()?;
                    if counts.is_empty() {
                        counts.push(0);
                    }
                    frame09(r.get("frame")?, f, &counts)?;
                } else {
                    frame(r.get("frame")?)?;
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
                    index(pair.get("claim")?, claims.len())?;
                }
                let fault = v.get("fault_rate")?;

                domain_for(fault.get("domain")?, f, self.isa_version, 0)?;
                if self.isa_version == crate::IsaVersion::V09 {
                    self.validate_image_domains09(fault.get("domain")?, f)?;
                }
                for (code, count) in fault.get("faults_by_code")?.object()? {
                    code.parse::<u128>()
                        .map_err(|_| Error::new("§12: invalid fault code key".into()))?;
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
                    .ok_or_else(|| Error::new("§12.3: malformed multiplicity key".into()))?;
                hash(&Json::String(closure.into()))?;
                context_key(context)?;
                n.uint()?;
            }
            Ok(())
        })
    }
    fn validate_claim(&self, c: &Json, f: &Function, position: usize) -> Result<()> {
        crate::metadata_schema::validate_for(self.isa_version, "claim", c)?;
        if self.isa_version == crate::IsaVersion::V09
            && c.object()?.get("expected") == Some(&Json::String("inverse_pair".into()))
        {
            return self.validate_inverse_pair_claim09(f, position, c);
        }
        if self.isa_version == crate::IsaVersion::V09 {
            if let Some(d) = c.object()?.get("domain") {
                self.validate_image_domains09(d, f)?;
            }
            let inverse_pair =
                c.object()?.get("expected") == Some(&Json::String("inverse_pair".into()));
            let subject_schedule = c
                .object()?
                .get("subject")
                .and_then(|s| s.object().ok())
                .and_then(|s| s.get("schedule_hash"));
            if c.object()?.get("expected") == Some(&Json::String("oracle".into()))
                && subject_schedule.is_none()
            {
                return err(
                    "§12.1 K13",
                    "expected:oracle requires subject.schedule_hash",
                );
            }
            if matches!(c.get("kind")?.string()?, "exhaustive" | "sampled") {
                table09(c, f)?;
            }
            if let Some(v) = c.object()?.get("invocation") {
                if v.get("args")? != &Json::String("all".into()) {
                    self.validate_invocation09(v, f)?;
                } else if f.params.iter().flatten().any(|p| p.ty == Type::Token)
                    && v.object()?.get("token_aliases") != Some(&Json::String("all".into()))
                {
                    return err(
                        "§12.3 K91",
                        "uniform token arguments require token_aliases:all",
                    );
                }
            } else if subject_schedule.is_none()
                && !inverse_pair
                && matches!(
                    c.get("kind")?.string()?,
                    "exhaustive" | "sampled" | "structural"
                )
            {
                return err("§12.3 K13", "standalone claim requires invocation");
            }
        }
        // §12.1–12.2: domain/name bindings, executable subjects and acyclic
        // indices need the enclosing function/claim list, beyond the grammar.
        match c.get("kind")?.string()? {
            "exhaustive" => {
                let controls = if self.isa_version == crate::IsaVersion::V09 {
                    c.get("subject")?.get("inherited_controls")?.uint()?
                } else {
                    0
                };
                domain_for(c.get("domain")?, f, self.isa_version, controls)?;
                if c.get("expected")? == &Json::String("oracle".into()) {
                    subject(c.get("subject")?)?;
                }
            }
            "sampled" => {
                let controls = if self.isa_version == crate::IsaVersion::V09 {
                    c.get("subject")?.get("inherited_controls")?.uint()?
                } else {
                    0
                };
                domain_for(c.get("domain")?, f, self.isa_version, controls)?;
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

#[cfg(test)]
mod r20_tests {
    use super::*;
    #[test]
    fn r20_region_domain_extremes() {
        let values = [0u64, 1, (1 << 63) - 1, 1 << 63, u64::MAX - 1, u64::MAX];
        let mut cases = 0;
        for n in values {
            for path in ["A", "W", "index", "lo", "hi"] {
                let (a, w, index, lo, hi) = match path {
                    "A" => (n, 1, 0, 0, 1),
                    "W" => (0, n, 0, 0, 1),
                    "index" => (1, 1, n, 0, 1),
                    "lo" => (0, 1, 0, n, n),
                    _ => (0, 1, 0, 0, n),
                };
                let mut f = Function::from_signature("o", &Json::parse(r#"{"abi":"0.8","args":[],"fault":4,"in":[],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#).unwrap()).unwrap();
                f.params[3].push(crate::Param {
                    name: "r".into(),
                    ty: Type::Region(a, w),
                    role: Some("in".into()),
                });
                let meta = |n: u64| {
                    if n <= 9_007_199_254_740_991 {
                        n.to_string()
                    } else {
                        format!("\"{n}\"")
                    }
                };
                let d = Json::parse(&format!(r#"{{"bundles":[],"borrowed":[],"regions":[{{"name":"r","cells":[{{"index":{},"lo":{},"hi":{}}}]}}]}}"#,meta(index),meta(lo),meta(hi))).unwrap();
                let admit = match path {
                    "A" => true,
                    "W" => n > 0,
                    "index" => n < 2,
                    _ => n < 2,
                };
                let result = std::panic::catch_unwind(|| domain(&d, &f)).expect("no domain panic");
                if admit {
                    result.unwrap();
                } else {
                    let e = result.unwrap_err().to_string();
                    assert!(
                        e.contains('§') && !e.contains("host limit"),
                        "{path}={n}: {e}"
                    );
                }
                cases += 1;
            }
        }
        assert_eq!(cases, 30);
        eprintln!("R20 region domains: 5 paths, {cases} cases");
    }
}

#[cfg(test)]
mod frame09_tests {
    use super::*;
    #[test]
    fn invocation_binds_classical_parameters_and_excludes_results_and_handles() {
        let mut o = Object::parse(include_str!("../tests/fixtures/inc8.bisa")).unwrap();
        o.isa_version = crate::IsaVersion::V09;
        let f=Function::from_signature("entry",&Json::parse(r#"{"abi":"0.9","kind":"cc","tier":"cc","params":["qreg[2]","i8"],"results":["i8"]}"#).unwrap()).unwrap();
        let invocation = Json::parse(r#"{"args":["7"],"budget":["0","0","0","0"]}"#).unwrap();
        o.validate_invocation09(&invocation, &f).unwrap();
        let extra = Json::parse(r#"{"args":["7","8"],"budget":["0","0","0","0"]}"#).unwrap();
        assert!(o.validate_invocation09(&extra, &f).is_err());
    }

    #[test]
    fn published_frame_witness_uses_exact_control_widths_and_disjoint_ranges() {
        let value=Json::parse(r#"{"ancillae":[{"class":"fast","id":"a","scheme":"controlled-x","state":"clean","width":{"var":"c"}}],"borrow":[{"class":"fast","id":"b","width":1}],"history":[{"class":"fast","id":"h","width":3}],"live_ranges":[{"end":2,"frame":"s","start":0},{"end":4,"frame":"s","start":2}],"scratch":[{"class":"fast","id":"s","state":"clean","width":{"args":[2,{"var":"c"}],"op":"+"}}],"slots":[{"class":"fast","id":"mut:0","role":"mut","width":2},{"class":"fast","id":"in:0","role":"in","width":1}]}"#).unwrap();
        let f=Function::from_signature("a", &Json::parse(r#"{"abi":"0.9","tier":"q","kind":"action","args":[],"bounds":[],"mut":[2],"in":[1],"mem":[]}"#).unwrap()).unwrap();
        crate::metadata_schema::validate_for(crate::IsaVersion::V09, "frame", &value).unwrap();
        frame09(&value, &f, &[0, 2, 9]).unwrap();
        assert_eq!(
            frame_width(
                value.get("scratch").unwrap().array().unwrap()[0]
                    .get("width")
                    .unwrap(),
                2
            )
            .unwrap(),
            4
        );
        assert_eq!(
            frame_width(
                value.get("ancillae").unwrap().array().unwrap()[0]
                    .get("width")
                    .unwrap(),
                2
            )
            .unwrap(),
            2
        );
        for (text, want) in [
            ("1e+3", 1000),
            (r#"{"op":"ceil","args":[{"op":"/","args":[5,2]}]}"#, 3),
            (r#"{"op":"max","args":[{"var":"c"},2]}"#, 2),
            (r#"{"op":"min","args":[{"var":"c"},2]}"#, 2),
            (r#"{"op":"*","args":[1.5,2]}"#, 3),
        ] {
            assert_eq!(frame_width(&Json::parse(text).unwrap(), 2).unwrap(), want);
        }
        for text in [
            r#"{"var":"k"}"#,
            r#"{"table":"t","index":"k"}"#,
            r#"{"op":"/","args":[1,0]}"#,
            r#"{"op":"/","args":[1,2]}"#,
            r#"{"op":"-","args":[0,1]}"#,
        ] {
            assert!(
                frame_width(&Json::parse(text).unwrap(), 2).is_err(),
                "{text}"
            );
        }
    }
}
