use crate::{err, json::Json, object::Function, sha256_hex, Error, Object, Result};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug)]
pub struct Identity {
    pub interface_hashes: BTreeMap<String, String>,
    /// None denotes a symbol whose dependency closure still has an unresolved import.
    pub closure_hashes: BTreeMap<String, Option<String>>,
    pub envelopes: BTreeMap<String, String>,
    pub object_hash: String,
    pub scc_members: BTreeMap<String, Vec<String>>,
    pub scc_hashes: BTreeMap<String, String>,
}

/// Symbol reference tokens only: @ inside a quoted string is ordinary text.
fn references(line: &str) -> Vec<(usize, usize, &str)> {
    let bytes = line.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    let mut quoted = false;
    while i < bytes.len() {
        if quoted {
            if bytes[i] == b'\\' {
                i += 2;
                continue;
            }
            if bytes[i] == b'"' {
                quoted = false;
            }
            i += 1;
            continue;
        }
        if bytes[i] == b'"' {
            quoted = true;
            i += 1;
            continue;
        }
        if bytes[i] == b'@'
            && i + 1 < bytes.len()
            && (bytes[i + 1].is_ascii_alphabetic() || bytes[i + 1] == b'_')
        {
            let start = i;
            i += 2;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || matches!(bytes[i], b'_' | b'.'))
            {
                i += 1;
            }
            out.push((start, i, &line[start + 1..i]));
        } else {
            i += 1;
        }
    }
    out
}

/// Classify the operand in its validated canonical instruction context. Only
/// the sole operand of lift.addr may denote this oracle's interface (§4.2).
/// Calls, qregion images and metadata still use ordinary symbol references.
fn symbol_references<'a>(
    line: &'a str,
    function: Option<&Function>,
) -> Vec<(usize, usize, &'a str)> {
    if let Some(f) = function.filter(|f| f.kind == "oracle") {
        if let Some(region) = line
            .split_once(" = ")
            .and_then(|(_, rhs)| rhs.strip_prefix("lift.addr @"))
        {
            if f.params[3].iter().any(|p| p.name == region) {
                return Vec::new();
            }
        }
    }
    references(line)
}

impl Object {
    pub fn identity(&self) -> Result<Identity> {
        let symbols: BTreeMap<_, _> = self
            .symbols()?
            .values()
            .map(|s| Ok((s.get("name")?.string()?.to_string(), s)))
            .collect::<Result<_>>()?;
        let imports = self
            .section("imports")
            .map(Json::object)
            .transpose()?
            .cloned()
            .unwrap_or_default();
        let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for n in symbols.keys() {
            let mut deps = Vec::new();
            if let Some(f) = self.functions.get(n) {
                for line in f.lines.iter().skip(1) {
                    for (_, _, r) in symbol_references(line, Some(f)) {
                        if symbols.contains_key(r) || imports.contains_key(r) {
                            if !deps.iter().any(|s| s == r) {
                                deps.push(r.to_string());
                            }
                        } else {
                            return err("§9.3", format!("unresolved undeclared reference @{r}"));
                        }
                    }
                }
            }
            graph.insert(n.clone(), deps);
        }
        // Tarjan emits dependency SCCs before their users.
        let components = components(&graph);
        // One hashing pass; `imported` gives the hash an import reference takes,
        // None for an import whose closure is unknown (not pinned).
        let pass = |imported: BTreeMap<String, Option<String>>| -> Result<HashPass> {
            let mut hashes: BTreeMap<String, String> = BTreeMap::new();
            let mut known: BTreeMap<String, bool> = BTreeMap::new();
            for (n, h) in &imported {
                if let Some(h) = h {
                    hashes.insert(n.clone(), h.clone());
                }
                known.insert(n.clone(), h.is_some());
            }
            let mut out = HashPass::default();
            for comp in &components {
                let closed = comp.iter().all(|n| {
                    graph[n]
                        .iter()
                        .all(|r| comp.contains(r) || known.get(r).copied().unwrap_or(false))
                });
                let mut envs = Vec::new();
                for n in comp {
                    let env = self.envelope(n, symbols[n], comp, &hashes)?;
                    out.envelopes.insert(n.clone(), env.clone());
                    envs.push(env);
                }
                let recursive = comp.len() > 1 || graph[&comp[0]].contains(&comp[0]);
                let scc = sha256_hex(envs.concat().as_bytes());
                for (i, n) in comp.iter().enumerate() {
                    out.scc_members.insert(n.clone(), comp.clone());
                    out.scc_hashes.insert(n.clone(), scc.clone());
                    let hash = if recursive {
                        sha256_hex(format!("{scc}:{i}").as_bytes())
                    } else {
                        sha256_hex(envs[i].as_bytes())
                    };
                    known.insert(n.clone(), closed);
                    out.closed.insert(n.clone(), closed);
                    hashes.insert(n.clone(), hash.clone());
                    out.hashes.insert(n.clone(), hash);
                }
            }
            Ok(out)
        };
        // Interface hashes: SHA-256(name || 0x00 || JCS signature) for imports.
        let mut fingerprints = BTreeMap::new();
        let mut pins = BTreeMap::new();
        for (n, im) in &imports {
            let mut seed = n.as_bytes().to_vec();
            seed.push(0);
            seed.extend_from_slice(im.get("signature")?.canonical().as_bytes());
            fingerprints.insert(n.clone(), Some(sha256_hex(&seed)));
            // Closure hashes: a pinned import's closure hash is fixed by its pin
            // (§9.3 requires equality at link time), so closures over pinned
            // imports are determined before linking.
            let pin = im
                .object()?
                .get("pinned_closure_hash")
                .map(|p| p.string().map(str::to_string))
                .transpose()?;
            pins.insert(n.clone(), pin);
        }
        let interface = pass(fingerprints)?;
        let closures = pass(pins)?;
        let hashes = interface.hashes;
        let closure: BTreeMap<String, Option<String>> = closures
            .hashes
            .into_iter()
            .map(|(n, h)| {
                let closed = closures.closed[&n];
                (n, closed.then_some(h))
            })
            .collect();
        let envelopes = interface.envelopes;
        // SCC grouping and ordering keys are those of executable (closure) identity.
        let scc_members = closures.scc_members;
        let scc_hashes = closures.scc_hashes;
        let mut canonical = String::new();
        let mut skip = false;
        let mut pending_blank = false;
        for line in self.text.lines() {
            if line.is_empty() {
                pending_blank = true;
                continue;
            }
            if let Some(s) = line.strip_prefix("section ") {
                skip = matches!(s, "schedule" | "certificate" | "coverage");
            }
            if !skip {
                if pending_blank {
                    canonical.push('\n');
                }
                canonical.push_str(line);
                canonical.push('\n');
            }
            pending_blank = false;
        }
        Ok(Identity {
            interface_hashes: hashes,
            closure_hashes: closure,
            envelopes,
            object_hash: sha256_hex(canonical.as_bytes()),
            scc_members,
            scc_hashes,
        })
    }
    pub fn check_hashes(&self) -> Result<()> {
        let identity = self.identity()?;
        for s in self.symbols()?.values() {
            let n = s.get("name")?.string()?;
            if s.get("interface_hash")?.string()? != identity.interface_hashes[n] {
                return err(
                    "§9.2",
                    format!(
                        "interface_hash mismatch for @{n}: expected {}",
                        identity.interface_hashes[n]
                    ),
                );
            }
        }
        Ok(())
    }
    fn envelope(
        &self,
        n: &str,
        symbol: &Json,
        comp: &[String],
        hashes: &BTreeMap<String, String>,
    ) -> Result<String> {
        let kind = symbol.get("kind")?.string()?;
        let mut body = if let Some(f) = self.functions.get(n) {
            f.lines.clone()
        } else {
            vec![format!("{n} {}", hex(&self.data[n]))]
        };
        let mut leaves = Vec::new();
        let mut images = BTreeSet::new();
        if let Some(layout) = self.section("layout") {
            let all = layout_leaves(layout)?;
            match kind {
                "cc" | "r" => leaves = all,
                // Ownership is an original JSON key, never a dotted prefix.
                "oracle" => {
                    leaves = all
                        .into_iter()
                        .filter(|leaf| leaf.keys[0] != "interface_regions" || leaf.keys[1] == n)
                        .collect();
                }
                "lproc" => {
                    let globals: BTreeSet<_> = self.functions[n]
                        .instructions()
                        .filter(|i| i.op == "lift.addr")
                        .filter_map(|i| i.operands[0].strip_prefix('@').map(str::to_owned))
                        .collect();
                    leaves = all
                        .into_iter()
                        .filter(|leaf| leaf.keys[0] == "globals" && globals.contains(&leaf.keys[1]))
                        .collect();
                }
                _ => {}
            }
            for leaf in &leaves {
                if let Json::Object(o) = &leaf.value {
                    if let Some(img) = o.get("image") {
                        images.insert(img.string()?.to_string());
                    }
                }
            }
            if matches!(kind, "oracle" | "cc" | "r") {
                if let Some(leaf) = leaves.iter().find(|leaf| leaf.keys == ["block_image"]) {
                    images.insert(leaf.value.string()?.to_string());
                }
            }
        }
        let mut refs = Vec::new();
        let replace = |line: &str, function: Option<&Function>, refs: &mut Vec<String>| -> String {
            let mut out = String::new();
            let mut previous = 0;
            for (start, end, r) in symbol_references(line, function) {
                out.push_str(&line[previous..start]);
                if let Some(index) = comp.iter().position(|n| n == r) {
                    out.push_str(&format!("@${index}"));
                } else if let Some(hash) = hashes.get(r) {
                    out.push('@');
                    out.push_str(hash);
                    if !refs.contains(hash) {
                        refs.push(hash.clone());
                    }
                } else {
                    out.push_str(&line[start..end]);
                }
                previous = end;
            }
            out.push_str(&line[previous..]);
            out
        };
        if self.functions.contains_key(n) {
            for line in body.iter_mut().skip(1) {
                *line = replace(line, self.functions.get(n), &mut refs);
            }
        }
        let mut lines = vec![
            "isa=0.8".into(),
            format!("ptr_width={}", self.ptr_width),
            symbol.get("signature")?.canonical(),
        ];
        lines.extend(body);
        for img in images {
            let bytes = self
                .data
                .get(&img)
                .ok_or_else(|| Error(format!("§9.2: missing closure data {img}")))?;
            lines.push(replace(
                &format!("data {img} {}", hex(bytes)),
                None,
                &mut refs,
            ));
        }
        for leaf in leaves {
            lines.push(replace(
                &format!("layout {} {}", leaf.path, leaf.value.canonical()),
                None,
                &mut refs,
            ));
        }
        if matches!(kind, "cc" | "r") {
            let mut used = BTreeSet::new();
            for i in self.functions[n].instructions() {
                if i.op == "effect" {
                    if let Some((dev, _)) = i.operands.first().and_then(|s| s.split_once('.')) {
                        used.insert(("models", dev.to_string()));
                    }
                }
                if i.op == "hostcall" {
                    if let Some(service) = i.operands.first() {
                        used.insert(("services", crate::object::literal_name(service)?));
                    }
                }
            }
            let devices = self.section("devices");
            let mut entries = BTreeMap::new();
            for (group, key) in used {
                let entry = devices
                    .map(|d| d.get(group).and_then(Json::object))
                    .transpose()?
                    .and_then(|entries| entries.get(&key));
                if let Some(value) = entry {
                    entries.insert(key, value);
                } else if group == "services"
                    || ["console", "clock", "entropy", "block"].contains(&key.as_str())
                {
                    return err("§9.1", format!("missing declared {group} entry {key}"));
                }
            }
            for (key, v) in entries {
                lines.push(format!("device {key} {}", v.canonical()));
            }
        }
        lines.extend(refs);
        Ok(lines.join("\n") + "\n")
    }
}
#[derive(Default)]
struct HashPass {
    hashes: BTreeMap<String, String>,
    closed: BTreeMap<String, bool>,
    envelopes: BTreeMap<String, String>,
    scc_members: BTreeMap<String, Vec<String>>,
    scc_hashes: BTreeMap<String, String>,
}
pub(crate) fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
/// A dotted rendering is not injective (§16.1 permits dots in NAME).
/// Keep structural identity through selection and emit every leaf once.
struct LayoutLeaf {
    keys: Vec<String>,
    path: String,
    value: Json,
}
fn layout_leaves(layout: &Json) -> Result<Vec<LayoutLeaf>> {
    let mut leaves = Vec::new();
    let mut push = |keys: Vec<String>, value: &Json| {
        leaves.push(LayoutLeaf {
            path: keys.join("."),
            keys,
            value: value.clone(),
        });
    };
    for (k, v) in layout.object()? {
        match k.as_str() {
            "globals" => {
                for (n, e) in v.object()? {
                    push(vec![k.clone(), n.clone()], e);
                }
            }
            "interface_regions" => {
                for (n, rs) in v.object()? {
                    for (r, e) in rs.object()? {
                        push(vec![k.clone(), n.clone(), r.clone()], e);
                    }
                }
            }
            _ => push(vec![k.clone()], v),
        }
    }
    // §9.2 dotted-path order; value-independent tie reading in SPEC-QUESTIONS 31.
    leaves.sort_by(|a, b| a.path.cmp(&b.path).then_with(|| a.keys.cmp(&b.keys)));
    Ok(leaves)
}
fn components(graph: &BTreeMap<String, Vec<String>>) -> Vec<Vec<String>> {
    struct Tarjan<'a> {
        graph: &'a BTreeMap<String, Vec<String>>,
        index: BTreeMap<String, usize>,
        low: BTreeMap<String, usize>,
        stack: Vec<String>,
        on: BTreeSet<String>,
        result: Vec<Vec<String>>,
    }
    fn visit(t: &mut Tarjan<'_>, v: &str) {
        let index = t.index.len();
        t.index.insert(v.into(), index);
        t.low.insert(v.into(), index);
        t.stack.push(v.into());
        t.on.insert(v.into());
        for w in &t.graph[v] {
            if !t.graph.contains_key(w) {
                continue;
            }
            if !t.index.contains_key(w) {
                visit(t, w);
                t.low.insert(v.into(), t.low[v].min(t.low[w]));
            } else if t.on.contains(w) {
                t.low.insert(v.into(), t.low[v].min(t.index[w]));
            }
        }
        if t.low[v] == t.index[v] {
            let mut comp = Vec::new();
            loop {
                let w = t.stack.pop().expect("Tarjan stack");
                t.on.remove(&w);
                comp.push(w.clone());
                if w == v {
                    break;
                }
            }
            comp.sort();
            t.result.push(comp);
        }
    }
    let mut t = Tarjan {
        graph,
        index: BTreeMap::new(),
        low: BTreeMap::new(),
        stack: Vec::new(),
        on: BTreeSet::new(),
        result: Vec::new(),
    };
    for v in graph.keys() {
        if !t.index.contains_key(v) {
            visit(&mut t, v);
        }
    }
    t.result
}
