//! The single recursive admission grammar. `?` marks an optional object key.
//! Clauses apply to every field/leaf in the entry, together with the referenced
//! production's clause. Integers are §9.1 canonical, arbitrary precision;
//! host/range limits belong to semantic consumers, never to tuple components.
//! `data` is the NAME/HEX projection of the textual section (§16.2), not JSON
//! in the object file. Functions retain their separate instruction grammar.
use crate::{err, json::Json, Error, Result};

#[derive(Clone, Copy, Debug)]
pub enum Shape {
    String,
    Text,
    Name,
    Hash,
    Hex,
    Uint,
    U64String,
    Decimal,
    SignedDecimalString,
    Number,
    Null,
    Enum(&'static [&'static str]),
    Object(&'static [(&'static str, &'static str)]),
    Map(&'static str, &'static str),
    Array(&'static str),
    Tuple(&'static [&'static str]),
    Union(&'static [&'static str]),
}
#[derive(Clone, Copy, Debug)]
pub struct Entry {
    pub name: &'static str,
    pub clause: &'static str,
    pub shape: Shape,
}
macro_rules! entry {
    ($name:literal, $clause:literal, $shape:expr) => {
        Entry {
            name: $name,
            clause: $clause,
            shape: $shape,
        }
    };
}
macro_rules! obj {
    ($($key:literal: $ty:literal),* $(,)?) => { Shape::Object(&[$(($key, $ty)),*]) };
}
use Shape::*;
// One named production per line; no procedural field checks outside this table.
#[rustfmt::skip]
pub static TABLE: &[Entry] = &[
    entry!("string", "§9.1", String),
    entry!("text", "§9.1", Text),
    entry!("name", "§9.1/§16.1 NAME", Name),
    entry!("hash", "§9.1/§9.2 SHA-256", Hash),
    entry!("hex", "§9.1/§16.2 HEX", Hex),
    entry!("uint", "§9.1 canonical integer", Uint),
    entry!("seed_word", "§12.1 sampler seed", U64String),
    entry!("decimal", "§9.1/§11/§12.1 decimal", Decimal),
    entry!("endpoint", "§12.1 Y decimal strings", SignedDecimalString),
    entry!("number", "§8.1 expression number", Number),
    entry!("null", "§12.3 absent oracle verification", Null),
    entry!("version", "§9.1/§12.3 version", Enum(&["0.8"])),
    entry!("kind", "§9.1 symbol kind", Enum(&["cc","r","oracle","action","perm_action","lproc","data"])),
    entry!("q", "§9.1 signature tier", Enum(&["q"])),
    entry!("dash", "§9.1 data tier", Enum(&["-"])),
    entry!("data_kind", "§9.1 data signature", Enum(&["data"])),
    entry!("oracle_kind", "§9.1 oracle signature", Enum(&["oracle"])),
    entry!("action_kind", "§9.1 action signature", Enum(&["action","perm_action"])),
    entry!("lproc_kind", "§9.1 lproc signature", Enum(&["lproc"])),
    entry!("classical_kind", "§9.1 classical signature", Enum(&["cc","r"])),
    entry!("tier", "§9.1 classical tier", Enum(&["cc","r"])),
    entry!("strings", "§9.1 string array", Array("string")),
    entry!("texts", "§9.1 identifier array", Array("text")),
    entry!("uints", "§9.1 integer array", Array("uint")),
    entry!("signature", "§9.1 signatures", Union(&["data_sig","oracle_sig","action_sig","lproc_sig","classical_sig"])),
    entry!("data_sig", "§9.1 data signature", obj!("abi":"version","tier":"dash","kind":"data_kind","size":"uint")),
    entry!("oracle_sig", "§9.1 oracle signature", obj!("abi":"version","tier":"q","kind":"oracle_kind","in":"uints","tgt":"uint","fault":"uint","mem":"mem","args":"texts")),
    entry!("action_sig", "§9.1 action signature", obj!("abi":"version","tier":"q","kind":"action_kind","mut":"uints","in":"uints","mem":"mem","args":"texts","bounds":"uints")),
    entry!("lproc_sig", "§9.1 lproc signature", obj!("abi":"version","tier":"q","kind":"lproc_kind","params":"uints","result":"uint")),
    entry!("classical_sig", "§9.1 classical signature", obj!("abi":"version","tier":"tier","kind":"classical_kind","params":"texts","results":"texts")),
    entry!("mem", "§9.1 interface memory", Array("mem_slot")),
    entry!("mem_slot", "§9.1 interface memory", obj!("A":"uint","W":"uint","role":"mem_role")),
    entry!("mem_role", "§9.1/§5.1 interface memory role", Enum(&["in","mut"])),
    entry!("metadata", "§9.1 accepted metadata sections", obj!("symbols?":"symbols","imports?":"imports","devices?":"devices","layout?":"layout","schedule?":"schedule","certificate?":"certificate","coverage?":"coverage")),
    entry!("symbols", "§9.1 symbols", Map("string","symbol")),
    entry!("symbol", "§9.1 symbol", obj!("name":"name","kind":"kind","signature":"signature","interface_hash":"hash")),
    entry!("imports", "§9.1 imports", Map("name","import")),
    entry!("import", "§9.1/§9.3 import", obj!("signature":"signature","pinned_closure_hash?":"hash")),
    entry!("data", "§9.1/§16.2 data lines", Map("name","hex")),
    entry!("devices", "§9.1 devices", obj!("models":"versions_map","services":"versions_map")),
    entry!("versions_map", "§9.1 model/service versions", Map("string","named_device")),
    entry!("named_device", "§9.1 model/service version", obj!("version":"string")),
    entry!("layout", "§9.1 layout", obj!("ptr_width":"uint","H":"uint","A_tab":"uint","S_cells":"uint","null_size":"uint","heap_base":"uint","heap_limit_max":"uint","globals":"globals","interface_regions":"interface_regions","block_image?":"name")),
    entry!("globals", "§9.1 layout globals", Map("string","global")),
    entry!("global", "§9.1 layout global", obj!("base":"uint","size":"uint","image":"name")),
    entry!("interface_regions", "§9.1 layout interface regions", Map("string","regions_map")),
    entry!("regions_map", "§9.1 layout region addresses", Map("string","region_base")),
    entry!("region_base", "§9.1 layout region address", obj!("base":"uint")),
    entry!("schedule", "§11 schedule", obj!("bindings":"schedule_bindings","entries":"schedule_entries")),
    // §17 item 7: omitted expansion binds ref at the object ISA version (0.8).
    // A present descriptor still uses the closed §11 name/version/hash production.
    entry!("schedule_bindings", "§11 bindings/§17 item 7", obj!("closure_hash":"hash","target_hash":"hash","region":"text","variant":"text","inherited_controls":"uint","budget":"budget","params_hash":"hash","epsilon":"decimal","scheduler":"named_version","expansion?":"expansion")),
    entry!("budget", "§11 budget", Tuple(&["uint","uint","uint","uint"])),
    entry!("named_version", "§11 scheduler", obj!("name":"text","version":"text")),
    entry!("expansion", "§11 expansion", obj!("name":"text","version":"text","hash":"hash")),
    entry!("schedule_entries", "§11 entries", Array("schedule_entry")),
    entry!("schedule_entry", "§11 entry", obj!("path":"path","step":"selector","phase":"phase","generated":"selector","kind":"schedule_kind","cleanup?":"cleanup_payload","gate?":"gate_payload","alloc?":"alloc_payload","start?":"start_payload")),
    entry!("schedule_kind", "§11 entry kind", Enum(&["cleanup","gate","alloc","start"])),
    entry!("phase", "§11 execution phase", Enum(&["fwd","rev","*"])),
    entry!("selector", "§11 selector", Union(&["wildcard","uint"])),
    entry!("wildcard", "§11 wildcard selector", Enum(&["*"])),
    entry!("iteration", "§11 iteration selector", Union(&["wildcard","uints"])),
    entry!("path", "§11 invocation path", Array("path_element")),
    entry!("path_element", "§11 invocation path element", obj!("symbol":"hash","point":"text","iteration":"iteration")),
    entry!("cleanup_payload", "§11 cleanup", obj!("realisation":"cleanup_realisation")),
    entry!("cleanup_realisation", "§11/§12.3 cleanup", Enum(&["reverse","recompute"])),
    entry!("gate_payload", "§11 gate/§17 item 9", obj!("realisation":"text","epsilon_share?":"decimal")),
    entry!("alloc_payload", "§11 allocation", obj!("class":"text")),
    entry!("start_payload", "§11 time", obj!("time":"decimal")),
    entry!("domain", "§12.2 domain", obj!("bundles":"bundles","borrowed":"bundles","regions":"domain_regions")),
    entry!("bundles", "§12.2 bundles", Array("bundle")),
    entry!("bundle", "§12.2 bundle", obj!("name":"text","width":"uint","lo":"uint","hi":"uint")),
    entry!("domain_regions", "§12.2 regions", Array("domain_region")),
    entry!("domain_region", "§12.2 region", obj!("name":"text","cells":"cells")),
    entry!("cells", "§12.2 cells", Union(&["all","cell_intervals"])),
    entry!("all", "§12.2 all cells", Enum(&["all"])),
    entry!("cell_intervals", "§12.2 cell intervals", Array("cell_interval")),
    entry!("cell_interval", "§12.2 cell interval", obj!("index":"uint","lo":"uint","hi":"uint")),
    entry!("subject", "§12.1 executable subject", obj!("closure_hash":"hash","variant":"text","inherited_controls":"uint","schedule_hash":"hash")),
    entry!("claims", "§12.1 claims", Array("claim")),
    entry!("claim", "§12.1 claims", Union(&["exhaustive","sampled","structural","numerical","channel"])),
    entry!("label", "§12.1/§12.2 evidence label", Enum(&["proof","evidence"])),
    entry!("exhaustive_kind", "§12.1 exhaustive", Enum(&["exhaustive"])),
    entry!("exhaustive", "§12.1 exhaustive", obj!("kind":"exhaustive_kind","domain":"domain","expected":"expected","label?":"label","subject?":"subject")),
    entry!("expected", "§12.1 exhaustive expected", Union(&["oracle_expected","table_expected","program_expected"])),
    entry!("oracle_expected", "§12.1 expected oracle", Enum(&["oracle"])),
    entry!("table_expected", "§12.1/§12.2 full tuples; SPEC-QUESTIONS 21", obj!("table":"table")),
    entry!("table", "§12.1/§12.2 enumeration order", Array("uints")),
    entry!("program_expected", "§12.1 expected program", obj!("program":"hash")),
    entry!("sampled_kind", "§12.1 sampled", Enum(&["sampled"])),
    entry!("sampled", "§12.1 sampled", obj!("kind":"sampled_kind","domain":"domain","sampler":"sampler","label?":"label")),
    entry!("sampler", "§12.1 sampler/§17 item 13", obj!("algorithm":"algorithm","seed":"seed","count?":"uint")),
    entry!("algorithm", "§12.1 sampler algorithm", Enum(&["xoshiro256**"])),
    entry!("seed", "§12.1 four seed words", Tuple(&["seed_word","seed_word","seed_word","seed_word"])),
    entry!("structural_kind", "§12.1 structural", Enum(&["structural"])),
    entry!("structural", "§12.1 structural rules", Union(&["adjoint_claim","library_claim","translation_claim"])),
    entry!("adjoint_rule", "§12.1 adjoint rule", Enum(&["adjoint_table"])),
    entry!("library_rule", "§12.1 library rule", Enum(&["inverse_pair_library"])),
    entry!("translation_rule", "§12.1 translation rule", Enum(&["translation_lemma"])),
    entry!("adjoint_claim", "§12.1 adjoint rule", obj!("kind":"structural_kind","rule":"adjoint_rule","subclaims":"uints","witness":"empty","label?":"label")),
    entry!("library_claim", "§12.1 library rule", obj!("kind":"structural_kind","rule":"library_rule","subclaims":"uints","witness":"library_witness","label?":"label")),
    entry!("translation_claim", "§12.1 translation rule", obj!("kind":"structural_kind","rule":"translation_rule","subclaims":"uints","witness":"translation_witness","label?":"label")),
    entry!("empty", "§12.1 adjoint witness", obj!()),
    entry!("library_witness", "§12.1 library witness", obj!("library_closure_hash":"hash","claim_index":"uint")),
    entry!("translation_witness", "§12.1 translation witness", obj!("V":"points","W":"points","X":"texts","B":"texts","frame":"translation_frame")),
    entry!("translation_frame", "§12.1 first and last points", Tuple(&["point", "point"])),
    entry!("points", "§12.1 program points", Array("point")),
    entry!("point", "§9.2/§12.1/§13 program point", Tuple(&["hash","text","uint"])),
    entry!("numerical_kind", "§12.1 numerical", Enum(&["numerical"])),
    entry!("channel_kind", "§12.1 channel", Enum(&["channel"])),
    entry!("operator_norm", "§12.1 unitary metric", Enum(&["operator_norm"])),
    entry!("diamond", "§12.1 channel metric", Enum(&["diamond"])),
    entry!("numerical", "§12.1 numerical", obj!("kind":"numerical_kind","subject":"subject","metric":"operator_norm","precision_bits":"uint","bound":"decimal","label?":"label")),
    entry!("channel", "§12.1 channel", obj!("kind":"channel_kind","subject":"subject","metric":"diamond","precision_bits":"uint","bound":"decimal","dual":"dual","label?":"label")),
    entry!("dual", "§12.1 dual witnesses", Array("dual_witness")),
    entry!("dual_witness", "§12.1 dual witness", obj!("Y":"Y","mu":"decimal")),
    entry!("Y", "§12.1 row-major interval matrix", Array("interval")),
    entry!("interval", "§12.1 [re_lo,re_hi,im_lo,im_hi]", Tuple(&["endpoint","endpoint","endpoint","endpoint"])),
    entry!("expression", "§8.1/§9.4 frame expressions", Union(&["number","variable_expr","operator_expr","table_expr"])),
    entry!("variable_expr", "§8.1 variable", obj!("var":"text")),
    entry!("operator_expr", "§8.1 operation", obj!("op":"operator","args":"expressions")),
    entry!("operator", "§8.1 operators", Enum(&["+","-","*","/","max","min","ceil","log2inv_eps"])),
    entry!("expressions", "§8.1 recursive operands", Array("expression")),
    entry!("table_expr", "§8.1 table lookup", obj!("table":"text","index":"table_index")),
    entry!("table_index", "§8.1 table index", Enum(&["k","eps_decade"])),
    entry!("frame", "§9.4 frame", obj!("interface":"frame_slots","scratch":"scratch","retained_history":"expression","borrow":"counts","scheme_ancillae":"counts","live_ranges?":"live_ranges")),
    entry!("frame_slots", "§9.4 interface slots", Array("frame_slot")),
    entry!("frame_slot", "§9.4 interface slot", obj!("role":"frame_role","width":"uint","class":"text","name?":"text")),
    entry!("frame_role", "§9.4 interface role", Enum(&["in","mut","tgt","fault","control"])),
    entry!("scratch", "§9.4 scratch by class", Map("text","scratch_count")),
    entry!("scratch_count", "§9.4 scratch counts", Union(&["expression","clean_dirty"])),
    entry!("clean_dirty", "§9.4 cleanliness counts", obj!("clean":"expression","dirty":"expression")),
    entry!("counts", "§9.4 counts by class", Map("text","expression")),
    entry!("live_ranges", "§9.4 shared-frame live ranges", Array("live_range")),
    entry!("live_range", "§9.4 live range", obj!("frame":"text","start":"uint","end":"uint")),
    entry!("metrics", "§12.3 error metrics", obj!("operator_norm?":"decimal","diamond?":"decimal")),
    entry!("region", "§12.3 region", obj!("kind":"kind","signature":"signature","bounds":"uints","frame":"frame","resources":"resources","gates_used":"gates_used","claims":"claims","verification":"verification","cleanup":"cleanup","error":"errors","definedness_assumptions":"texts")),
    entry!("resources", "§12.3 resources", Map("string","decimal")),
    entry!("gates_used", "§12.3 gates", Map("string","gate_usage")),
    entry!("gate_usage", "§12.3 gate usage", obj!("count":"uint","scheme?":"text","epsilon?":"decimal")),
    entry!("verification", "§12.3 verification indices", obj!("oracle":"optional_index","borrow_frames":"uints","inverse_pairs":"inverse_pairs","unitary":"uints","channel":"uints","fault_rate":"fault_rate")),
    entry!("optional_index", "§12.3 oracle index", Union(&["null","uint"])),
    entry!("inverse_pairs", "§12.3 inverse pairs", Array("inverse_pair")),
    entry!("inverse_pair", "§12.3 inverse pair", obj!("g":"hash","ginv":"hash","claim":"uint")),
    entry!("fault_rate", "§12.3 fault rate", obj!("domain":"domain","faults_by_code":"faults_by_code","undefined_excluded":"uint","layout_dependent_excluded":"uint")),
    entry!("faults_by_code", "§12.3 fault counts", Map("string","uint")),
    entry!("cleanup", "§12.3 cleanup contexts", Map("string","cleanups")),
    entry!("cleanups", "§12.3 cleanups", Array("cleanup_entry")),
    entry!("cleanup_entry", "§12.3 cleanup entry", obj!("path":"path","realisation":"cleanup_realisation")),
    entry!("errors", "§12.3 error contexts", Map("string","error")),
    entry!("error", "§12.3 error maps", obj!("local":"metrics","inclusive":"metrics")),
    entry!("certificate", "§12.3 certificate", obj!("versions":"versions","bindings":"certificate_bindings","producer":"producer","regions":"certificate_regions","composition":"composition")),
    entry!("versions", "§12.3 certificate versions", obj!("isa":"version","object":"version","launch":"version","sched":"version","cert":"version","coverage":"version","target":"version")),
    entry!("certificate_bindings", "§12.3 bindings", obj!("object_hash":"hash","interface_hash":"hash","closure_hash":"hash","reference_program_hash":"hash","dialect_mapping_version":"text","target_hash":"hash","schedule_hash?":"hash")),
    entry!("producer", "§12.3 producer", obj!("name":"text","version":"text","materialisation":"materialisation")),
    entry!("materialisation", "§12.3 materialisation", obj!("llvm_version":"string","pipeline":"string","normaliser_version":"string")),
    entry!("certificate_regions", "§12.3 region closures", Map("hash","region")),
    entry!("composition", "§12.3 composition", obj!("multiplicities":"multiplicities","epsilon_total":"epsilon_total","method":"composition_method")),
    entry!("multiplicities", "§12.3 multiplicities", Map("string","uint")),
    entry!("epsilon_total", "§12.3 total error", obj!("diamond":"decimal")),
    entry!("composition_method", "§12.3 composition method", Enum(&["sum_over_multiplicities"])),
    entry!("coverage", "§13 coverage", obj!("entries":"coverage_entries")),
    entry!("coverage_entries", "§13 entries", Array("coverage_entry")),
    entry!("coverage_entry", "§13 entry", obj!("point":"point","source":"text","classification":"classification","reason":"string","count":"uint","fault_code?":"uint","shim_hash?":"hash")),
    entry!("classification", "§13 classification", Enum(&["lowered","trapped","shimmed"])),
];

pub fn entry(name: &str) -> &'static Entry {
    TABLE
        .iter()
        .find(|e| e.name == name)
        .expect("schema reference must exist")
}
pub fn validate(name: &str, value: &Json) -> Result<()> {
    let e = entry(name);
    check(e.shape, value).map_err(|err| Error::new(format!("§9.1: {} ({name}): {err}", e.clause)))
}
fn digits(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}
fn decimal_text(s: &str) -> bool {
    let (i, f) = s.split_once('.').map_or((s, None), |(i, f)| (i, Some(f)));
    digits(i) && f.is_none_or(digits)
}
fn check(shape: Shape, v: &Json) -> Result<()> {
    match shape {
        String => {
            v.string()?;
        }
        Text => {
            if v.string()?.is_empty() {
                return err("§9.1", "empty metadata identifier");
            }
        }
        Name => {
            if !crate::object::name(v.string()?) {
                return err("§9.1", "invalid NAME");
            }
        }
        Hash | Hex => {
            let s = v.string()?;
            if (matches!(shape, Hash) && s.len() != 64)
                || s.len() % 2 != 0
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            {
                return err("§9.1/§9.2/§16.2", "invalid lowercase hex");
            }
        }
        Uint => {
            v.uint_text()?;
        }
        U64String => {
            let s = v.string()?;
            if !digits(s) || s.parse::<u64>().is_err() {
                return err("§12.1", "seed word must be a u64 decimal string");
            }
        }
        Decimal => match v {
            Json::String(s) if decimal_text(s) => (),
            Json::Number(s)
                if !s.starts_with('-') && s.parse::<f64>().is_ok_and(f64::is_finite) => {}
            _ => return err("§9.1", "expected nonnegative finite decimal"),
        },
        SignedDecimalString => {
            let s = v.string()?;
            if !decimal_text(s.strip_prefix('-').unwrap_or(s)) {
                return err("§12.1", "expected signed finite decimal string");
            }
        }
        Number => {
            if !matches!(v, Json::Number(s) if s.parse::<f64>().is_ok_and(f64::is_finite)) {
                return err("§8.1", "expected finite expression number");
            }
        }
        Null => {
            if v != &Json::Null {
                return err("§12.3", "expected null");
            }
        }
        Enum(values) => {
            if !values.contains(&v.string()?) {
                return err("§9.1", format!("expected one of {values:?}"));
            }
        }
        Object(fields) => {
            let m = v.object()?;
            for (key, ty) in fields {
                let actual = key.strip_suffix('?').unwrap_or(key);
                match m.get(actual) {
                    Some(child) => validate(ty, child).map_err(|e| e.context(actual))?,
                    None if key.ends_with('?') => (),
                    None => return err("§9.1", format!("missing field {actual}")),
                }
            }
            for key in m.keys() {
                if !fields
                    .iter()
                    .any(|(k, _)| k.strip_suffix('?').unwrap_or(k) == key)
                {
                    return err("§9.1", format!("unknown field {key}"));
                }
            }
        }
        Map(key, ty) => {
            for (k, child) in v.object()? {
                validate(key, &Json::String(k.clone()))?;
                validate(ty, child)?;
            }
        }
        Array(ty) => {
            for child in v.array()? {
                validate(ty, child)?;
            }
        }
        Tuple(types) => {
            let a = v.array()?;
            if a.len() != types.len() {
                return err("§9.1", format!("tuple requires {} components", types.len()));
            }
            for (ty, child) in types.iter().zip(a) {
                validate(ty, child)?;
            }
        }
        Union(types) => {
            let mut errors = Vec::new();
            for ty in types {
                match validate(ty, v) {
                    Ok(()) => return Ok(()),
                    Err(e) => errors.push(e.to_string()),
                }
            }
            return err("§9.1", errors.join(" | "));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests;
