//! Symbol resolution and canonical linked objects (§9.3).
use crate::{err, identity::hex, json::Json, Error, Object, Result, SECTIONS};
use std::collections::BTreeMap;

impl Object {
    pub fn link(objects: &[Object]) -> Result<Self> {
        let first = objects
            .first()
            .ok_or_else(|| Error::new("§9.3: no objects to link".into()))?;
        let mut result = Self {
            text: String::new(),
            ptr_width: first.ptr_width,
            metadata: BTreeMap::new(),
            functions: BTreeMap::new(),
            function_order: Vec::new(),
            data: BTreeMap::new(),
        };
        let mut definitions: BTreeMap<String, Json> = BTreeMap::new();
        let mut imports: Vec<(String, Json)> = Vec::new();
        let mut known_closures: BTreeMap<String, String> = BTreeMap::new();
        let mut old_identities = Vec::new();
        let mut certificates: Vec<Json> = Vec::new();
        for object in objects {
            object.check_hashes()?;
            if object.ptr_width != result.ptr_width {
                return err("§9.3", "pointer widths differ");
            }
            let identity = object.identity()?;
            for (name, hash) in &identity.closure_hashes {
                if let Some(hash) = hash {
                    if known_closures.get(name).is_some_and(|prior| prior != hash) {
                        return err("§9.3", format!("unequal closure definitions for @{name}"));
                    }
                    known_closures.insert(name.clone(), hash.clone());
                }
            }
            old_identities.push(identity);
            for symbol in object.symbols()?.values() {
                let name = symbol.get("name")?.string()?;
                if let Some(prior) = definitions.get(name) {
                    if prior.get("signature")? != symbol.get("signature")?
                        || prior.get("kind")? != symbol.get("kind")?
                    {
                        return err("§9.3", format!("incompatible duplicate @{name}"));
                    }
                } else {
                    definitions.insert(name.into(), symbol.clone());
                }
            }
            for (name, function) in &object.functions {
                if result
                    .functions
                    .get(name)
                    .is_some_and(|prior| prior.lines != function.lines)
                {
                    return err("§9.3", format!("conflicting function bodies for @{name}"));
                }
                result.functions.insert(name.clone(), function.clone());
            }
            for (name, bytes) in &object.data {
                if result.data.get(name).is_some_and(|prior| prior != bytes) {
                    return err("§9.3", format!("conflicting data @{name}"));
                }
                result.data.insert(name.clone(), bytes.clone());
            }
            if let Some(entries) = object.section("imports") {
                for (name, value) in entries.object()? {
                    imports.push((name.clone(), value.clone()));
                }
            }
            if let Some(c) = object.section("certificate") {
                certificates.push(c.clone());
            }
            for (section, value) in &object.metadata {
                if matches!(section.as_str(), "symbols" | "imports" | "certificate") {
                    continue;
                }
                match result.metadata.get_mut(section) {
                    None => {
                        result.metadata.insert(section.clone(), value.clone());
                    }
                    Some(prior) => merge_metadata(prior, value, section)?,
                }
            }
        }
        for (name, import) in &imports {
            let definition = definitions
                .get(name)
                .ok_or_else(|| Error::new(format!("§9.3: unresolved import @{name}")))?;
            if import.get("signature")? != definition.get("signature")? {
                return err("§9.3", format!("import signature mismatch for @{name}"));
            }
        }
        result.function_order = result.functions.keys().cloned().collect();
        result.metadata.insert(
            "symbols".into(),
            Json::Object(
                definitions
                    .values()
                    .cloned()
                    .enumerate()
                    .map(|(i, s)| (i.to_string(), s))
                    .collect(),
            ),
        );
        // Resolution changes envelopes which used unresolved-name fingerprints.
        // Recompute the graph before rebinding any external metadata.
        let identity = result.identity()?;
        for (name, import) in &imports {
            if let Some(pin) = import.object()?.get("pinned_closure_hash") {
                if identity.closure_hashes[name].as_deref() != Some(pin.string()?) {
                    return err("§9.3", format!("pinned closure mismatch for @{name}"));
                }
            }
        }
        if let Some(Json::Object(symbols)) = result.metadata.get_mut("symbols") {
            for symbol in symbols.values_mut() {
                let n = symbol.get("name")?.string()?.to_string();
                let Json::Object(fields) = symbol else {
                    return err("§9.1", "symbol entries are objects");
                };
                fields.insert(
                    "interface_hash".into(),
                    Json::String(identity.interface_hashes[&n].clone()),
                );
            }
        }
        let mut replacements = BTreeMap::new();
        for old in old_identities {
            for (name, hash) in old.interface_hashes {
                replacements.insert(hash, identity.interface_hashes[&name].clone());
            }
            for (name, hash) in old.closure_hashes {
                if let Some(hash) = hash {
                    let next = identity.closure_hashes[&name].clone().ok_or_else(|| {
                        Error::new(format!("§9.3: @{name} is still unresolved after linking"))
                    })?;
                    replacements.insert(hash, next);
                }
            }
        }
        let old_schedule = result
            .metadata
            .get("schedule")
            .map(|s| crate::sha256_hex(s.canonical().as_bytes()));
        for section in ["schedule", "coverage"] {
            if let Some(value) = result.metadata.get_mut(section) {
                replace_hashes(value, &replacements)?;
            }
        }
        // A rebound schedule has a new schedule_hash (§9.2): rebind references.
        if let (Some(old), Some(new)) = (
            old_schedule,
            result
                .metadata
                .get("schedule")
                .map(|s| crate::sha256_hex(s.canonical().as_bytes())),
        ) {
            replacements.insert(old, new);
        }
        for c in &mut certificates {
            replace_hashes(c, &replacements)?;
        }
        if !certificates.is_empty() {
            result
                .metadata
                .insert("certificate".into(), compose_certificates(&certificates)?);
        }
        result.text = result.render()?;
        let object_hash = result.identity()?.object_hash;
        if let Some(Json::Object(cert)) = result.metadata.get_mut("certificate") {
            if let Some(Json::Object(bindings)) = cert.get_mut("bindings") {
                bindings.insert("object_hash".into(), Json::String(object_hash));
            }
        }
        result.text = result.render()?;
        let linked = Self::parse(&result.text)?;
        linked.check_hashes()?;
        Ok(linked)
    }
    pub(crate) fn render(&self) -> Result<String> {
        let mut text=format!("bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width={}\n",self.ptr_width);
        for section in SECTIONS {
            let body = match section {
                "functions" => {
                    if self.functions.is_empty() {
                        continue;
                    }
                    self.function_order
                        .iter()
                        .map(|n| self.functions[n].lines.join("\n"))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                "data" => {
                    if self.data.is_empty() {
                        continue;
                    }
                    self.data
                        .iter()
                        .map(|(n, b)| format!("{n} {}", hex(b)))
                        .collect::<Vec<_>>()
                        .join("\n")
                }
                _ => match self.section(section) {
                    Some(value) => value.canonical(),
                    None => continue,
                },
            };
            text.push_str(&format!("\nsection {section}\n{body}\n"));
        }
        Ok(text)
    }
}
fn merge_metadata(destination: &mut Json, source: &Json, path: &str) -> Result<()> {
    if destination == source {
        return Ok(());
    }
    if path == "coverage.entries" {
        let (Json::Array(a), Json::Array(b)) = (destination, source) else {
            return err("§9.3", "coverage entries must be arrays");
        };
        for e in b {
            if !a.contains(e) {
                a.push(e.clone());
            }
        }
        return Ok(());
    }
    let recursive = matches!(
        path,
        "layout"
            | "layout.globals"
            | "layout.interface_regions"
            | "devices"
            | "devices.models"
            | "devices.services"
            | "coverage"
    ) || path.starts_with("layout.interface_regions.")
        && path.split('.').count() == 3;
    if !recursive {
        return err("§9.3", format!("conflicting metadata at {path}"));
    }
    let Json::Object(target) = destination else {
        return err("§9.3", format!("metadata object required at {path}"));
    };
    for (key, value) in source.object()? {
        if let Some(prior) = target.get_mut(key) {
            merge_metadata(prior, value, &format!("{path}.{key}"))?;
        } else {
            target.insert(key.clone(), value.clone());
        }
    }
    Ok(())
}
fn replace_hashes(value: &mut Json, replacements: &BTreeMap<String, String>) -> Result<()> {
    match value {
        Json::String(s) => {
            if let Some(next) = replacements.get(s) {
                *s = next.clone();
            }
        }
        Json::Array(values) => {
            for v in values {
                replace_hashes(v, replacements)?;
            }
        }
        Json::Object(fields) => {
            let previous = std::mem::take(fields);
            for (key, mut value) in previous {
                replace_hashes(&mut value, replacements)?;
                // Composite keys "<hash>:<variant>:<controls>" rebind their hash.
                let key = match replacements.get(&key) {
                    Some(next) => next.clone(),
                    None => match key.split_once(':') {
                        Some((h, rest)) if replacements.contains_key(h) => {
                            format!("{}:{rest}", replacements[h])
                        }
                        _ => key,
                    },
                };
                if let Some(prior) = fields.insert(key.clone(), value.clone()) {
                    if prior != value {
                        return err(
                            "§9.3",
                            format!("two metadata keys rebind to {key} with different values"),
                        );
                    }
                }
            }
        }
        _ => {}
    }
    Ok(())
}
/// §9.3 linking performs certificate composition (§12.3): the region records
/// of every certificate are united (one record per closure hash, equal where
/// they meet), multiplicities are united (equal where they meet), and
/// epsilon_total is recomputed by the sum over multiplicities from the local
/// error bounds, operator norms converting to diamond by x2 (SPEC-QUESTIONS 22).
/// The composed certificate binds to the first certificate's root region.
fn compose_certificates(certificates: &[Json]) -> Result<Json> {
    let first = &certificates[0];
    for c in &certificates[1..] {
        if c.get("versions")? != first.get("versions")? {
            return err("§9.3", "certificates disagree on versions");
        }
        for key in ["dialect_mapping_version", "target_hash"] {
            if c.get("bindings")?.get(key)? != first.get("bindings")?.get(key)? {
                return err("§9.3", format!("certificates bind different {key} values"));
            }
        }
        if c.get("composition")?.get("method")? != first.get("composition")?.get("method")? {
            return err("§12.3", "certificates use different composition methods");
        }
    }
    let mut regions = BTreeMap::new();
    let mut multiplicities = BTreeMap::new();
    for c in certificates {
        for (k, r) in c.get("regions")?.object()? {
            if regions.get(k).is_some_and(|prior: &Json| prior != r) {
                return err(
                    "§9.3",
                    format!("conflicting certificate records for region {k}"),
                );
            }
            regions.insert(k.clone(), r.clone());
        }
        for (k, n) in c.get("composition")?.get("multiplicities")?.object()? {
            if multiplicities.get(k).is_some_and(|prior: &Json| prior != n) {
                return err("§12.3", format!("conflicting multiplicities for {k}"));
            }
            multiplicities.insert(k.clone(), n.clone());
        }
    }
    let local = |region: &str, key: &str| -> Result<crate::exact::Decimal> {
        let Some(r) = regions.get(region) else {
            return err(
                "§12.3",
                format!("multiplicity names region {region} with no record"),
            );
        };
        let Some(e) = r.get("error")?.object()?.get(key) else {
            return Ok(crate::exact::Decimal::zero());
        };
        let l = e.get("local")?.object()?;
        let text = |v: &Json| -> Result<String> {
            Ok(match v {
                Json::Number(s) | Json::String(s) => s.clone(),
                _ => return err("§12.3", "error bound must be a decimal"),
            })
        };
        if let Some(d) = l.get("diamond") {
            return crate::exact::Decimal::parse(&text(d)?)
                .ok_or_else(|| Error::new("§12.3: invalid diamond bound".into()));
        }
        if let Some(o) = l.get("operator_norm") {
            return Ok(crate::exact::Decimal::parse(&text(o)?)
                .ok_or_else(|| Error::new("§12.3: invalid operator_norm bound".into()))?
                .mul_int(2));
        }
        Ok(crate::exact::Decimal::zero())
    };
    let mut total = crate::exact::Decimal::zero();
    for (k, n) in &multiplicities {
        let (region, key) = k
            .split_once(':')
            .ok_or_else(|| Error::new("§12.3: malformed multiplicity key".into()))?;
        total = total.add(&local(region, key)?.mul_int(n.uint()?));
    }
    let root = first.get("bindings")?.get("closure_hash")?.string()?;
    if let Some(r) = regions.get(root) {
        for key in r.get("error")?.object()?.keys() {
            total = total.add(&local(root, key)?);
        }
    }
    let mut out = first.clone();
    let Json::Object(fields) = &mut out else {
        return err("§12", "certificate must be an object");
    };
    fields.insert("regions".into(), Json::Object(regions));
    let total_text = total.to_text();
    let total_json = if total_text.contains('.') || total_text.len() > 15 {
        Json::String(total_text)
    } else {
        Json::Number(total_text)
    };
    fields.insert(
        "composition".into(),
        Json::Object(BTreeMap::from([
            (
                "epsilon_total".into(),
                Json::Object(BTreeMap::from([("diamond".into(), total_json)])),
            ),
            (
                "method".into(),
                first.get("composition")?.get("method")?.clone(),
            ),
            ("multiplicities".into(), Json::Object(multiplicities)),
        ])),
    );
    Ok(out)
}
