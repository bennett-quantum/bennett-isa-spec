//! Symbol resolution and canonical linked objects (§9.3).
use crate::{err, identity::hex, json::Json, Error, Object, Result, SECTIONS};
use std::collections::BTreeMap;

impl Object {
    pub fn link(objects: &[Object]) -> Result<Self> {
        let first = objects
            .first()
            .ok_or_else(|| Error("§9.3: no objects to link".into()))?;
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
            for (section, value) in &object.metadata {
                if matches!(section.as_str(), "symbols" | "imports") {
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
                .ok_or_else(|| Error(format!("§9.3: unresolved import @{name}")))?;
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
                    unreachable!("validated symbol")
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
                    replacements.insert(
                        hash,
                        identity.closure_hashes[&name]
                            .clone()
                            .expect("fully resolved closure"),
                    );
                }
            }
        }
        for section in ["schedule", "certificate", "coverage"] {
            if let Some(value) = result.metadata.get_mut(section) {
                replace_hashes(value, &replacements);
            }
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
fn replace_hashes(value: &mut Json, replacements: &BTreeMap<String, String>) {
    match value {
        Json::String(s) => {
            if let Some(next) = replacements.get(s) {
                *s = next.clone();
            }
        }
        Json::Array(values) => {
            for v in values {
                replace_hashes(v, replacements);
            }
        }
        Json::Object(fields) => {
            let previous = std::mem::take(fields);
            for (key, mut value) in previous {
                replace_hashes(&mut value, replacements);
                let key = replacements.get(&key).cloned().unwrap_or(key);
                fields.insert(key, value);
            }
        }
        _ => {}
    }
}
