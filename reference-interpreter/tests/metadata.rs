mod support;
use bennett_isa_ref::{json::Json, Object};
use support::json;

const INC: &str = include_str!("fixtures/inc8.bisa");
const HASH: &str = "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93";
fn schedule() -> Json {
    json(&format!(
        r#"{{"bindings":{{"budget":[3,0,0,0],"closure_hash":"{HASH}","epsilon":0,"expansion":{{"hash":"{}","name":"ref","version":"0.8"}},"inherited_controls":0,"params_hash":"{}","region":"inc8","scheduler":{{"name":"test","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[]}}"#,
        "1".repeat(64),
        "2".repeat(64),
        "3".repeat(64)
    ))
}
fn with(section: &str, value: &Json) -> String {
    format!("{INC}\nsection {section}\n{}\n", value.canonical())
}
fn set(value: &mut Json, key: &str, next: Json) {
    let Json::Object(fields) = value else {
        panic!("object")
    };
    fields.insert(key.into(), next);
}
fn reject(value: &Json, section: &str, rule: &str) {
    let e = match Object::parse(&with(section, value)) {
        Err(e) => e,
        Ok(_) => panic!("malformed {section} accepted"),
    };
    assert!(e.to_string().contains(rule), "{e}");
}
fn entry(point: &str, iteration: &str) -> Json {
    json(&format!(
        r#"{{"generated":"*","kind":"gate","path":[{{"iteration":{iteration},"point":"{point}","symbol":"{HASH}"}}],"phase":"fwd","step":1,"gate":{{"epsilon_share":0,"realisation":"native"}}}}"#
    ))
}
#[test]
fn schedule_requires_typed_bindings_selectors_and_unique_maxima() {
    Object::parse(&with("schedule", &schedule())).unwrap();
    for (key, value) in [
        ("closure_hash", json("\"bad\"")),
        ("inherited_controls", json("-1")),
        ("budget", json("[3,-1,0,0]")),
        ("epsilon", json("-0.1")),
    ] {
        let mut s = schedule();
        let Json::Object(fields) = &mut s else {
            unreachable!()
        };
        set(fields.get_mut("bindings").unwrap(), key, value);
        reject(&s, "schedule", "§11");
    }
    let mut s = schedule();
    let Json::Object(fields) = &mut s else {
        unreachable!()
    };
    let Json::Object(bindings) = fields.get_mut("bindings").unwrap() else {
        unreachable!()
    };
    bindings.remove("expansion");
    reject(&s, "schedule", "§11");
    for (key, value) in [
        ("phase", json("\"backwards\"")),
        ("step", json("0")),
        ("generated", json("-1")),
        (
            "gate",
            json(r#"{"epsilon_share":-1,"realisation":"native"}"#),
        ),
    ] {
        let mut e = entry("entry:0", "[]");
        set(&mut e, key, value);
        let mut s = schedule();
        set(&mut s, "entries", Json::Array(vec![e]));
        reject(&s, "schedule", "§11");
    }
    let e = entry("entry:0", "[]");
    let mut s = schedule();
    set(&mut s, "entries", Json::Array(vec![e.clone(), e]));
    reject(&s, "schedule", "§11");
}

#[test]
fn schedule_paths_bind_real_program_points_and_loop_depths() {
    let mut valid = schedule();
    set(
        &mut valid,
        "entries",
        Json::Array(vec![entry("entry:0", "[]")]),
    );
    Object::parse(&with("schedule", &valid)).unwrap();
    for e in [
        entry("missing:0", "[]"),
        entry("entry:3", "[]"),
        entry("entry:0", "[0]"),
    ] {
        let mut s = schedule();
        set(&mut s, "entries", Json::Array(vec![e]));
        reject(&s, "schedule", "§11");
    }
    let mut e = entry("entry:0", "[]");
    let Json::Object(fields) = &mut e else {
        unreachable!()
    };
    let Json::Array(path) = fields.get_mut("path").unwrap() else {
        unreachable!()
    };
    set(
        &mut path[0],
        "symbol",
        json(&format!("\"{}\"", "f".repeat(64))),
    );
    let mut s = schedule();
    set(&mut s, "entries", Json::Array(vec![e]));
    reject(&s, "schedule", "§11");
}

#[test]
fn coverage_points_and_classification_payloads_are_checked() {
    let coverage = json(&format!(
        r#"{{"entries":[{{"classification":"lowered","count":1,"point":["{HASH}","entry",0],"reason":"direct","source":"dialect.add"}}]}}"#
    ));
    Object::parse(&with("coverage", &coverage)).unwrap();
    for (key, value) in [
        ("point", json(&format!(r#"["{HASH}","entry",99]"#))),
        ("source", json("4")),
        ("count", json("-1")),
        ("classification", json("\"trapped\"")),
        ("shim_hash", json("\"bad\"")),
    ] {
        let mut c = coverage.clone();
        let Json::Object(fields) = &mut c else {
            unreachable!()
        };
        let Json::Array(entries) = fields.get_mut("entries").unwrap() else {
            unreachable!()
        };
        set(&mut entries[0], key, value);
        reject(&c, "coverage", "§13");
    }
}

fn certificate() -> Json {
    let object = Object::parse(INC).unwrap();
    let identity = object.identity().unwrap();
    let signature = object.symbols().unwrap()["0"]
        .get("signature")
        .unwrap()
        .canonical();
    json(&format!(
        r#"{{"versions":{{"isa":"0.8","object":"0.8","launch":"0.8","sched":"0.8","cert":"0.8","coverage":"0.8","target":"0.8"}},"bindings":{{"object_hash":"{}","interface_hash":"{HASH}","closure_hash":"{HASH}","reference_program_hash":"{HASH}","dialect_mapping_version":"test.1","target_hash":"{}"}},"producer":{{"name":"tests","version":"1","materialisation":{{"llvm_version":"none","pipeline":"none","normaliser_version":"1"}}}},"regions":{{"{HASH}":{{"kind":"oracle","signature":{signature},"bounds":[],"frame":{{"interface":[{{"role":"in","width":8,"class":"any"}},{{"role":"tgt","width":8,"class":"any"}},{{"role":"fault","width":4,"class":"any"}}],"scratch":{{}},"retained_history":0,"borrow":{{}},"scheme_ancillae":{{}}}},"resources":{{}},"gates_used":{{}},"claims":[],"verification":{{"oracle":null,"borrow_frames":[],"inverse_pairs":[],"unitary":[],"channel":[],"fault_rate":{{"domain":{{"bundles":[],"borrowed":[],"regions":[]}},"faults_by_code":{{}},"undefined_excluded":0,"layout_dependent_excluded":0}}}},"cleanup":{{}},"error":{{}},"definedness_assumptions":[]}}}},"composition":{{"multiplicities":{{}},"epsilon_total":{{"diamond":0}},"method":"sum_over_multiplicities"}}}}"#,
        identity.object_hash,
        "3".repeat(64)
    ))
}
fn region_mut(c: &mut Json) -> &mut Json {
    let Json::Object(fields) = c else {
        unreachable!()
    };
    let Json::Object(regions) = fields.get_mut("regions").unwrap() else {
        unreachable!()
    };
    regions.get_mut(HASH).unwrap()
}
#[test]
fn certificate_claim_domains_samplers_and_references_are_typed() {
    Object::parse(&with("certificate", &certificate())).unwrap();
    let domain =
        r#"{"bundles":[{"name":"x","width":8,"lo":0,"hi":255}],"borrowed":[],"regions":[]}"#;
    let sampled = json(&format!(
        r#"{{"kind":"sampled","domain":{domain},"sampler":{{"algorithm":"xoshiro256**","seed":["1","0","0","0"],"count":10}}}}"#
    ));
    let exhaustive = json(&format!(
        r#"{{"kind":"exhaustive","domain":{domain},"expected":{{"table":[]}}}}"#
    ));
    let structural =
        json(r#"{"kind":"structural","rule":"adjoint_table","subclaims":[],"witness":{}}"#);
    for valid in [sampled.clone(), exhaustive, structural] {
        let mut c = certificate();
        set(region_mut(&mut c), "claims", Json::Array(vec![valid]));
        Object::parse(&with("certificate", &c)).unwrap();
    }
    let invalids = [
        json(r#"{"kind":"unproved"}"#),
        json(&format!(
            r#"{{"kind":"sampled","domain":{domain},"sampler":{{"algorithm":"xoshiro256**","seed":["0","0","0","0"],"count":10}}}}"#
        )),
        json(&format!(
            r#"{{"kind":"sampled","domain":{domain},"sampler":{{"algorithm":"mt19937","seed":["1","0","0","0"],"count":10}}}}"#
        )),
        json(
            r#"{"kind":"exhaustive","domain":{"bundles":[{"name":"x","width":8,"lo":1,"hi":256}],"borrowed":[],"regions":[]},"expected":{"table":[]}}"#,
        ),
        json(r#"{"kind":"structural","rule":"adjoint_table","subclaims":[0],"witness":{}}"#),
    ];
    for invalid in invalids {
        let mut c = certificate();
        set(region_mut(&mut c), "claims", Json::Array(vec![invalid]));
        reject(&c, "certificate", "§12");
    }
    let mut c = certificate();
    let r = region_mut(&mut c);
    let Json::Object(fields) = r else {
        unreachable!()
    };
    set(fields.get_mut("verification").unwrap(), "oracle", json("0"));
    reject(&c, "certificate", "§12");
}

#[test]
fn certificate_subjects_errors_frame_expressions_and_bindings_are_checked() {
    let subject = format!(
        r#"{{"closure_hash":"{HASH}","variant":"forward","inherited_controls":0,"schedule_hash":"{}"}}"#,
        "4".repeat(64)
    );
    let numerical = json(&format!(
        r#"{{"kind":"numerical","subject":{subject},"metric":"operator_norm","precision_bits":128,"bound":"0.001"}}"#
    ));
    let mut c = certificate();
    set(
        region_mut(&mut c),
        "claims",
        Json::Array(vec![numerical.clone()]),
    );
    Object::parse(&with("certificate", &c)).unwrap();
    for (key, value) in [
        ("metric", json("\"diamond\"")),
        ("precision_bits", json("0")),
        ("bound", json("\"-0.1\"")),
        ("subject", json("{}")),
    ] {
        let mut n = numerical.clone();
        set(&mut n, key, value);
        let mut c = certificate();
        set(region_mut(&mut c), "claims", Json::Array(vec![n]));
        reject(&c, "certificate", "§12");
    }
    let mut c = certificate();
    let Json::Object(r) = region_mut(&mut c) else {
        unreachable!()
    };
    set(
        r.get_mut("frame").unwrap(),
        "scratch",
        json(r#"{"any":{"var":"k"}}"#),
    );
    reject(&c, "certificate", "§9.4");
    let mut c = certificate();
    set(
        region_mut(&mut c),
        "error",
        json(r#"{"forward:0":{"local":{"operator_norm":-1},"inclusive":{"operator_norm":0}}}"#),
    );
    reject(&c, "certificate", "§12");
    let mut c = certificate();
    let Json::Object(fields) = &mut c else {
        unreachable!()
    };
    set(
        fields.get_mut("bindings").unwrap(),
        "object_hash",
        json(&format!("\"{}\"", "f".repeat(64))),
    );
    reject(&c, "certificate", "§12");
}

#[test]
fn r13_service_versions_require_strings() {
    let valid = support::r13_text("service-0.8");
    support::r13_check("service-0.8", "inc8");
    for v in ["7", "false", "null", "[]", "{}"] {
        let invalid = valid.replace(r#""version":"0.8""#, &format!(r#""version":{v}"#));
        assert!(Object::parse(&invalid)
            .unwrap_err()
            .to_string()
            .contains("§9.1"));
    }
    // Services have their own arbitrary string versions, not the model's 0.8 rule.
    let arbitrary = valid.replace(r#""version":"0.8""#, r#""version":"vendor.2026""#);
    Object::parse(&arbitrary).unwrap().check_hashes().unwrap();
}

// One generated family: visit every field (including container fields and
// array elements) in the accepted schema exemplars. The type alternatives
// below come from §§9.1/9.4/11/12/13, not from the interpreter's result.
fn r13_kind(v: &Json) -> u8 {
    match v {
        Json::Number(_) => 1,
        Json::String(_) => 2,
        Json::Bool(_) => 4,
        Json::Null => 8,
        Json::Array(_) => 16,
        Json::Object(_) => 32,
    }
}
fn r13_types(section: &str, path: &[String], value: &Json) -> u8 {
    let key = path.last().map(String::as_str).unwrap_or("");
    let parent = path.iter().rev().nth(1).map(String::as_str).unwrap_or("");
    if section == "certificate" {
        if key == "oracle" && parent == "verification" {
            return 1 | 2 | 8;
        }
        if key == "expected" {
            return 2 | 32;
        }
        if key == "cells" {
            return 2 | 16;
        }
        if ["retained_history", "clean", "dirty"].contains(&key)
            || ["scratch", "borrow", "scheme_ancillae"].contains(&parent)
            || (parent == "args" && path.iter().any(|p| p == "retained_history"))
        {
            return 1 | 32;
        }
        if ["bound", "mu", "epsilon", "operator_norm", "diamond"].contains(&key)
            || parent == "resources"
            || (path.iter().any(|p| p == "Y") && !matches!(value, Json::Array(_)))
        {
            return 1 | 2;
        }
        // Canonical integers have number and decimal-string representations.
        if parent == "multiplicities" {
            return 1 | 2;
        }
    }
    if section == "schedule" {
        if key == "iteration" {
            return 2 | 16;
        }
        if ["step", "generated", "epsilon", "epsilon_share", "time"].contains(&key) {
            return 1 | 2;
        }
    }
    if matches!(value, Json::Number(_)) {
        return 1 | 2;
    }
    r13_kind(value)
}
fn r13_integer(section: &str, path: &[String], value: &Json) -> bool {
    let key = path.last().map(String::as_str).unwrap_or("");
    let parent = path.iter().rev().nth(1).map(String::as_str).unwrap_or("");
    if section == "certificate"
        && (path.iter().any(|p| p == "Y")
            || [
                "bound",
                "mu",
                "epsilon",
                "operator_norm",
                "diamond",
                "retained_history",
                "clean",
                "dirty",
            ]
            .contains(&key)
            || ["resources", "borrow", "scheme_ancillae"].contains(&parent)
            || path.iter().any(|p| p == "retained_history"))
    {
        return false;
    }
    if section == "schedule" && ["epsilon", "epsilon_share", "time"].contains(&key) {
        return false;
    }
    matches!(value, Json::Number(_)) || (section == "certificate" && parent == "multiplicities")
}
fn r13_nodes(v: &Json, path: &mut Vec<String>, out: &mut Vec<(Vec<String>, Json)>) {
    out.push((path.clone(), v.clone()));
    match v {
        Json::Object(m) => {
            for (k, v) in m {
                path.push(k.clone());
                r13_nodes(v, path, out);
                path.pop();
            }
        }
        Json::Array(a) => {
            for (k, v) in a.iter().enumerate() {
                path.push(k.to_string());
                r13_nodes(v, path, out);
                path.pop();
            }
        }
        _ => {}
    }
}
fn r13_replace(v: &mut Json, path: &[String], next: Json) {
    if path.is_empty() {
        *v = next;
        return;
    }
    let child = match v {
        Json::Object(m) => m.get_mut(&path[0]).unwrap(),
        Json::Array(a) => &mut a[path[0].parse::<usize>().unwrap()],
        _ => panic!("container"),
    };
    r13_replace(child, &path[1..], next);
}
#[test]
fn r13_generated_metadata_schema_type_family() {
    let corpus = json(include_str!("support/r13_schema_cases.json"));
    let mut negatives = 0;
    let mut fields = 0;
    let mut positives = 0;
    let mut failures = Vec::new();
    for case in corpus.array().unwrap() {
        let name = case.get("name").unwrap().string().unwrap();
        let base = case.get("object").unwrap().string().unwrap();
        let o = Object::parse(base).unwrap_or_else(|e| panic!("positive {name}: {e}"));
        o.check_hashes()
            .unwrap_or_else(|e| panic!("positive pin {name}: {e}"));
        positives += 1;
        for (section, v) in &o.metadata {
            let mut nodes = Vec::new();
            r13_nodes(v, &mut Vec::new(), &mut nodes);
            for (path, value) in nodes {
                fields += 1;
                let types = r13_types(section, &path, &value);
                let mut wrong = ["7", r#""wrong""#, "false", "null", "[]", "{}"]
                    .iter()
                    .map(|s| json(s))
                    .filter(|v| types & r13_kind(v) == 0)
                    .collect::<Vec<_>>();
                if r13_integer(section, &path, &value) {
                    // Test the exact raw JSON spellings, including unsafe integers.
                    wrong.extend(
                        ["-1", "18446744073709551615", "18446744073709551616", "1.5"]
                            .iter()
                            .map(|s| Json::Number((*s).into())),
                    );
                    wrong.push(Json::String("wrong".into()));
                }
                for next in wrong {
                    negatives += 1;
                    let mut mutated = v.clone();
                    r13_replace(&mut mutated, &path, next.clone());
                    let text = base.replace(
                        &format!("section {section}\n{}\n", v.canonical()),
                        &format!("section {section}\n{}\n", mutated.canonical()),
                    );
                    assert_ne!(base, text);
                    // Admission only: no call to identity/check_hashes or execution
                    // may be needed to notice a malformed field.
                    match Object::parse(&text) {
                        Ok(_) => failures.push(format!(
                            "{name} {section}.{path:?} accepts {}",
                            next.canonical()
                        )),
                        Err(e) if !e.to_string().contains("§9.1") => failures.push(format!(
                            "{name} {section}.{path:?} {}: {e}",
                            next.canonical()
                        )),
                        Err(_) => {}
                    }
                }
            }
        }
    }
    eprintln!("R13 schema: {positives} positive objects, {fields} field visits, {negatives} negative mutations");
    assert!(
        failures.is_empty(),
        "{} failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
