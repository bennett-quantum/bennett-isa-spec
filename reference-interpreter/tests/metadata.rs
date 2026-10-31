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
#[test]
fn r14_y_endpoints_require_decimal_strings() {
    use support::r14::*;
    assert_eq!(
        bennett_isa_ref::sha256_hex(Y.as_bytes()),
        "578c913bb276d1446e2e038af0ab0a77017d9abe5ec351e3e20b69c307068e11"
    );
    let o = Object::parse(&y_control()).unwrap();
    o.check_hashes().unwrap();
    let pins = json(include_str!("support/r14_pins.json"));
    let pin = &pins.array().unwrap()[0];
    assert_eq!(
        o.identity().unwrap().envelopes["inc8"],
        pin.get("envelope").unwrap().string().unwrap()
    );
    assert_eq!(
        o.identity().unwrap().object_hash,
        "8093da48bad04d6564325faf01ea1304a84b5ac5a120e685eee9951f5daec5d9"
    );
    assert!(matches!(
        o.verify_claim(IH, 8).unwrap(),
        bennett_isa_ref::ClaimVerdict::Unverified(_)
    ));
    for bad in std::iter::once(Y.to_string()).chain(negatives_y()) {
        let e = Object::parse(&bad).expect_err("numeric or nondecimal Y admitted");
        assert!(e.to_string().contains("§12.1"), "{e}");
    }
}
#[test]
fn r14_table_rows_and_components_are_typed_at_admission() {
    use support::r14::*;
    let o = Object::parse(TABLE).unwrap();
    o.check_hashes().unwrap();
    let pins = json(include_str!("support/r14_pins.json"));
    let pin = pins
        .array()
        .unwrap()
        .iter()
        .find(|p| p.get("name").unwrap().string().unwrap() == "r14-table-positive")
        .unwrap();
    assert_eq!(
        o.identity().unwrap().envelopes["a"],
        pin.get("envelope").unwrap().string().unwrap()
    );
    assert_eq!(
        o.identity().unwrap().object_hash,
        "ddc5d6729a98484acc2d5e8feb585c155799293bd36f31fd96cbc510f77666f0"
    );
    assert_eq!(
        o.verify_claim(AH, 0).unwrap(),
        bennett_isa_ref::ClaimVerdict::Evidence { elements: 1 }
    );
    let large = replace(
        TABLE,
        &["regions", AH, "claims", "0", "expected", "table"],
        json(r#"[["340282366920938463463374607431768211456"]]"#),
    );
    Object::parse(&large).unwrap().check_hashes().unwrap();
    for bad in negatives_table().into_iter().chain(
        [
            include_str!("support/r14-witnesses/r14-table-bool.bisa"),
            include_str!("support/r14-witnesses/r14-table-row-bool.bisa"),
            include_str!("support/r14-witnesses/r14-table-cell-bool.bisa"),
            include_str!("support/r14-witnesses/r14-table-cell-object.bisa"),
        ]
        .map(str::to_string),
    ) {
        let e = Object::parse(&bad).expect_err("malformed table admitted");
        assert!(e.to_string().contains("§12"), "{e}");
    }
}
