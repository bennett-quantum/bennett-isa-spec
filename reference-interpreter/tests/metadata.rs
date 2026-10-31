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
    Object::parse(&with("schedule", &s))
        .unwrap()
        .check_hashes()
        .unwrap();
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

#[test]
fn r15_translation_frame_cardinality_is_checked_at_admission() {
    use support::r15::*;
    // The exact reviewer negative remains rejected after moving the rule
    // into the grammar. A two-point control preserves every identity pin.
    for len in 0..=3 {
        let point = format!(r#"["{IH}","entry",0]"#);
        let frame = format!("[{}]", vec![point; len].join(","));
        let text = FRAME.replace(r#""frame":[]"#, &format!(r#""frame":{frame}"#));
        assert!(FRAME.contains(r#""frame":[]"#));
        if len == 2 {
            let o = Object::parse(&text).unwrap();
            o.check_hashes().unwrap();
            assert_eq!(o.identity().unwrap().object_hash, OH);
        } else {
            let e = Object::parse(&text).unwrap_err();
            assert!(e.to_string().contains("§12.1"), "{e}");
        }
    }
}

#[test]
fn r15_oracle_schedule_epsilon_uses_exact_decimal_value() {
    use support::r15::*;
    assert_eq!(schedule(json(r#""0.0""#)), EPSILON);
    assert_eq!(
        bennett_isa_ref::sha256_hex(EPSILON.as_bytes()),
        "1431a1dbbe4991d170e3848995026610ebbffce08e7279ee05a68880f06c82df"
    );
    for epsilon in zeros() {
        let o = Object::parse(&schedule(epsilon.clone()))
            .unwrap_or_else(|e| panic!("zero {}: {e}", epsilon.canonical()));
        o.check_hashes().unwrap();
        let id = o.identity().unwrap();
        assert_eq!(id.object_hash, OH);
        assert_eq!(id.interface_hashes["inc8"], IH);
        assert_eq!(id.closure_hashes["inc8"].as_deref(), Some(IH));
    }
    for epsilon in nonzeros() {
        let e = Object::parse(&schedule(epsilon)).unwrap_err();
        assert!(e.to_string().contains("§5.2"), "{e}");
    }
    // The string exponent form is outside §9.1's decimal-string grammar.
    let e = Object::parse(&schedule(json(r#""1e-300""#))).unwrap_err();
    assert!(e.to_string().contains("§9.1"), "{e}");
}

#[test]
fn r16_default_expansion_preserves_witness_and_identity() {
    use support::r16::*;
    assert_eq!(
        bennett_isa_ref::sha256_hex(DEFAULT.as_bytes()),
        "8344b91cdba546f8bf29db14c4c239011cd088660863e015727c76ea8df25bfa"
    );
    let o = Object::parse(DEFAULT).unwrap();
    o.check_hashes().unwrap();
    let id = o.identity().unwrap();
    assert_eq!(id.object_hash, OH);
    assert_eq!(id.interface_hashes["inc8"], IH);
    assert_eq!(id.closure_hashes["inc8"].as_deref(), Some(IH));
    let (_, line) = DEFAULT.split_once("section schedule\n").unwrap();
    assert_eq!(o.section("schedule").unwrap().canonical(), line.trim_end());
    assert_eq!(bennett_isa_ref::sha256_hex(line.trim_end().as_bytes()), SH);
    assert_eq!(o.text, DEFAULT);
}

#[test]
fn r16_present_expansion_still_requires_closed_descriptor() {
    for descriptor in support::r16::malformed() {
        let e = Object::parse(&support::r16::expansion(descriptor))
            .expect_err("malformed present expansion");
        assert!(
            e.to_string().contains("§9.1") && e.to_string().contains("§11"),
            "{e}"
        );
    }
}

#[test]
fn r16_schedule_consumer_resolves_default_and_preserves_explicit_binding() {
    use support::r16::*;
    // §17 item 7: absence is ref at the object's ISA version, not a made-up pin.
    let o = Object::parse(DEFAULT).unwrap();
    let binding = o.schedule_expansion().unwrap().unwrap();
    assert_eq!(
        (binding.name, binding.version, binding.hash),
        ("ref", "0.8", None)
    );
    // Resolution must leave the canonical schedule and its identity untouched.
    assert_eq!(
        bennett_isa_ref::sha256_hex(o.section("schedule").unwrap().canonical().as_bytes()),
        SH
    );
    for (name, version) in [("ref", "0.8"), ("vendor", "2026.10")] {
        let descriptor = json(&format!(
            r#"{{"name":"{name}","version":"{version}","hash":"{}"}}"#,
            "a".repeat(64)
        ));
        let text = expansion(descriptor);
        let o = Object::parse(&text).unwrap();
        let binding = o.schedule_expansion().unwrap().unwrap();
        assert_eq!((binding.name, binding.version), (name, version));
        assert_eq!(
            binding.hash,
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );
        o.check_hashes().unwrap();
    }
    assert!(Object::parse(INC)
        .unwrap()
        .schedule_expansion()
        .unwrap()
        .is_none());
}

#[test]
fn r17_default_share_and_shadowed_witness_preserve_identity() {
    use support::{
        r16::{IH, OH},
        r17::*,
    };
    for (source, file_hash, schedule_hash) in [
        (
            EPS,
            "8af833c21de5d94c6684b36cf115c46f5080f528fc898ece2bdda9091b9fb2a5",
            EPS_SH,
        ),
        (
            TIE,
            "9fc08dca3c5697d9fcad31bf82f4a4a566d0fda6959d10675859d748696c1aff",
            TIE_SH,
        ),
    ] {
        assert_eq!(bennett_isa_ref::sha256_hex(source.as_bytes()), file_hash);
        let o = Object::parse(source).unwrap();
        o.check_hashes().unwrap();
        let id = o.identity().unwrap();
        assert_eq!(id.object_hash, OH);
        assert_eq!(id.interface_hashes["inc8"], IH);
        assert_eq!(id.closure_hashes["inc8"].as_deref(), Some(IH));
        assert_eq!(o.text, source);
        let raw = source
            .split_once("section schedule\n")
            .unwrap()
            .1
            .trim_end();
        assert_eq!(o.section("schedule").unwrap().canonical(), raw);
        assert_eq!(bennett_isa_ref::sha256_hex(raw.as_bytes()), schedule_hash);
        assert_eq!(
            schedule(source)
                .get("bindings")
                .unwrap()
                .get("budget")
                .unwrap(),
            &json("[3,0,0,0]")
        );
    }
}
#[test]
fn r17_present_gate_share_and_missing_realisation_reject() {
    for gate in support::r17::malformed_gate() {
        let e = Object::parse(&support::r17::gate(gate)).unwrap_err();
        assert!(e.to_string().contains("§9.1: §11"), "{e}");
    }
}
#[test]
fn r17_shadowed_tie_all_orders_and_genuine_maximum_ties() {
    use support::r17::*;
    let mut s = schedule(TIE);
    let entries = s.get("entries").unwrap().array().unwrap().to_vec();
    let operation = json(&format!(
        r#"{{"path":[{{"symbol":"{HASH}","point":"entry:0","iteration":[]}}],"step":1,"phase":"fwd","generated":0}}"#
    ));
    for order in [
        [0, 1, 2],
        [0, 2, 1],
        [1, 0, 2],
        [1, 2, 0],
        [2, 0, 1],
        [2, 1, 0],
    ] {
        set(
            &mut s,
            "entries",
            Json::Array(order.map(|i| entries[i].clone()).to_vec()),
        );
        let o = Object::parse(&document(TIE, &s)).unwrap();
        o.check_hashes().unwrap();
        assert_eq!(
            o.resolve_schedule(&operation, "cleanup").unwrap(),
            Some(&entries[2])
        );
    }
    for indices in [vec![0, 1], vec![0, 1, 2, 2]] {
        set(
            &mut s,
            "entries",
            Json::Array(indices.iter().map(|i| entries[*i].clone()).collect()),
        );
        let e = Object::parse(&document(TIE, &s)).unwrap_err();
        assert!(e.to_string().contains("multiple maximal"), "{e}");
    }
    set(
        &mut s,
        "entries",
        Json::Array(vec![entries[0].clone(), entries[2].clone()]),
    );
    Object::parse(&document(TIE, &s))
        .unwrap()
        .check_hashes()
        .unwrap();
}

#[test]
fn r17_specificity_lattice_family_matches_brute_force() {
    // An independent oracle enumerates concrete operations, and computes the
    // literal §11 lexicographic maximum. No production overlap/domain helpers.
    fn score(e: &Json) -> (usize, usize, bool, bool, bool) {
        let path = e.get("path").unwrap().array().unwrap();
        let specific = |v: &Json| v != &json(r#""*""#);
        (
            path.len(),
            path.iter()
                .filter(|p| specific(p.get("iteration").unwrap()))
                .count(),
            specific(e.get("step").unwrap()),
            specific(e.get("phase").unwrap()),
            specific(e.get("generated").unwrap()),
        )
    }
    fn matches(e: &Json, op: &Json) -> bool {
        let ep = e.get("path").unwrap().array().unwrap();
        let opath = op.get("path").unwrap().array().unwrap();
        ep.len() <= opath.len()
            && ep.iter().zip(opath).all(|(a, b)| {
                ["symbol", "point"]
                    .iter()
                    .all(|k| a.get(k).unwrap() == b.get(k).unwrap())
                    && (a.get("iteration").unwrap() == &json(r#""*""#)
                        || a.get("iteration").unwrap() == b.get("iteration").unwrap())
            })
            && ["step", "phase", "generated"].iter().all(|k| {
                e.get(k).unwrap() == &json(r#""*""#) || e.get(k).unwrap() == op.get(k).unwrap()
            })
    }
    let mut seed = 0x1718_cafe_u64;
    let mut draw = |n: u64| {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        seed % n
    };
    let mut cases = 0;
    let mut admitted = 0;
    for count in 2..=4 {
        for _ in 0..400 {
            let mut entries = Vec::new();
            for _ in 0..count {
                let mut e = entry(
                    &format!("entry:{}", draw(3)),
                    if draw(2) == 0 { "[]" } else { r#""*""# },
                );
                if draw(4) == 0 {
                    set(&mut e, "path", json("[]"));
                }
                for key in ["step", "phase", "generated"] {
                    let value = match draw(3) {
                        0 => json(r#""*""#),
                        n if key == "phase" => json(if n == 1 { r#""fwd""# } else { r#""rev""# }),
                        n => Json::Number((if key == "step" { n } else { n - 1 }).to_string()),
                    };
                    set(&mut e, key, value);
                }
                entries.push(e);
            }
            // 2 specific generated ordinals + one residual; likewise all three
            // budget steps, both phases, and all three root program points.
            let mut concrete = Vec::new();
            for point in 0..3 {
                for step in 1..=3 {
                    for phase in ["fwd", "rev"] {
                        for generated in 0..=2 {
                            concrete.push(json(&format!(r#"{{"path":[{{"symbol":"{HASH}","point":"entry:{point}","iteration":[]}}],"step":{step},"phase":"{phase}","generated":{generated}}}"#)));
                        }
                    }
                }
            }
            let winners: Vec<_> = concrete
                .iter()
                .map(|op| {
                    let matching: Vec<_> = entries
                        .iter()
                        .enumerate()
                        .filter(|(_, e)| matches(e, op))
                        .collect();
                    let best = matching.iter().map(|(_, e)| score(e)).max();
                    matching
                        .into_iter()
                        .filter(|(_, e)| Some(score(e)) == best)
                        .map(|(i, _)| i)
                        .collect::<Vec<_>>()
                })
                .collect();
            let ambiguous = winners.iter().any(|w| w.len() > 1);
            let mut s = schedule();
            set(&mut s, "entries", Json::Array(entries.clone()));
            let parsed = Object::parse(&with("schedule", &s));
            assert_eq!(
                parsed.is_err(),
                ambiguous,
                "case {cases}: {}",
                s.canonical()
            );
            if let Ok(o) = parsed {
                admitted += 1;
                for (op, w) in concrete.iter().zip(&winners) {
                    assert_eq!(
                        o.resolve_schedule(op, "gate").unwrap(),
                        w.first().map(|i| &entries[*i])
                    );
                }
            }
            // Document order independence includes rejected cases.
            entries.reverse();
            set(&mut s, "entries", Json::Array(entries));
            assert_eq!(Object::parse(&with("schedule", &s)).is_err(), ambiguous);
            cases += 1;
        }
    }
    assert_eq!(cases, 1200);
    eprintln!("R17 specificity family: {cases} schedules + reversed orders, {admitted} admitted, {} rejected; 54 concrete operations each; oracle agreement 100%", cases - admitted);
}

#[test]
fn r17_union_coverage_partial_coverage_and_budget_domains() {
    let mut a = entry("entry:0", r#""*""#);
    set(&mut a, "phase", json(r#""*""#));
    let b = a.clone();
    let mut fwd = a.clone();
    set(&mut fwd, "phase", json(r#""fwd""#));
    let mut rev = a.clone();
    set(&mut rev, "phase", json(r#""rev""#));
    let mut s = schedule();
    for (entries, valid) in [
        (vec![a.clone(), b.clone(), fwd.clone(), rev], true),
        (vec![a.clone(), b.clone(), fwd], false),
    ] {
        set(&mut s, "entries", Json::Array(entries));
        assert_eq!(Object::parse(&with("schedule", &s)).is_ok(), valid);
    }
    // step 4 lies outside the oracle's 1..B domain: no matchable maximum tie.
    set(&mut a, "step", json("4"));
    set(&mut s, "entries", Json::Array(vec![a.clone(), a]));
    Object::parse(&with("schedule", &s)).unwrap();
}

#[test]
fn r17_equal_share_consumer_keeps_exact_and_explicit_assignments() {
    use bennett_isa_ref::ScheduleEpsilonShare;
    let op = json(&format!(
        r#"{{"path":[{{"symbol":"{HASH}","point":"entry:0","iteration":[]}}],"step":1,"phase":"fwd","generated":0}}"#
    ));
    let o = Object::parse(support::r17::EPS).unwrap();
    let epsilon = o
        .section("schedule")
        .unwrap()
        .get("bindings")
        .unwrap()
        .get("epsilon")
        .unwrap();
    for n in [1, 3, 1000] {
        assert_eq!(
            o.schedule_epsilon_share(&op, false, n).unwrap(),
            Some(ScheduleEpsilonShare::EqualShare {
                epsilon,
                operations: n
            })
        );
    }
    assert!(o.schedule_epsilon_share(&op, false, 0).is_err());
    assert_eq!(
        o.schedule_epsilon_share(&op, true, 0).unwrap(),
        Some(ScheduleEpsilonShare::Exact)
    );
    let o = Object::parse(&support::r17::gate(json(
        r#"{"realisation":"native","epsilon_share":"0.125"}"#,
    )))
    .unwrap();
    let supplied = json(r#""0.125""#);
    assert_eq!(
        o.schedule_epsilon_share(&op, false, 3).unwrap(),
        Some(ScheduleEpsilonShare::Assigned(&supplied))
    );
    assert_eq!(
        o.schedule_epsilon_share(&op, true, 3).unwrap(),
        Some(ScheduleEpsilonShare::Exact)
    );
    let o = Object::parse(INC).unwrap();
    assert_eq!(o.schedule_epsilon_share(&op, false, 3).unwrap(), None);
}

#[test]
fn r17_loop_iteration_union_and_empty_domains() {
    for (bound, valid) in [(0, true), (2, true), (3, false), (1u64 << 40, false)] {
        let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#.replace("\"bounds\":[]", &format!("\"bounds\":[{bound}]"));
        let body = format!(
            "perm_action @a (t:wire mut) () () () ()\nregs {{  }} bounds {{{bound}}}\n^entry():\nqloop {bound} {{\nx t\nx t\n}}"
        );
        let o = support::rehash(&support::object(&[("a", &sig, &body)], "{}", ""));
        let hash = &o.identity().unwrap().interface_hashes["a"];
        let mut a = entry("entry:1", r#""*""#);
        let Json::Object(fields) = &mut a else {
            unreachable!()
        };
        let Json::Array(path) = fields.get_mut("path").unwrap() else {
            unreachable!()
        };
        set(&mut path[0], "symbol", Json::String(hash.clone()));
        let mut c = a.clone();
        let mut d = a.clone();
        for (e, iteration) in [(&mut c, 0), (&mut d, 1)] {
            let Json::Object(fields) = e else {
                unreachable!()
            };
            let Json::Array(path) = fields.get_mut("path").unwrap() else {
                unreachable!()
            };
            set(&mut path[0], "iteration", json(&format!("[{iteration}]")));
        }
        let entries = if bound == 0 {
            vec![a.clone(), a]
        } else {
            vec![a.clone(), a, c, d]
        };
        let mut s = schedule();
        let Json::Object(fields) = &mut s else {
            unreachable!()
        };
        let bindings = fields.get_mut("bindings").unwrap();
        set(bindings, "region", json(r#""a""#));
        set(bindings, "closure_hash", Json::String(hash.clone()));
        set(&mut s, "entries", Json::Array(entries));
        let text = format!("{}\nsection schedule\n{}\n", o.text, s.canonical());
        assert_eq!(Object::parse(&text).is_ok(), valid, "loop {bound}");
    }
}
