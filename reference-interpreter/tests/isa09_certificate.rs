mod support;
use bennett_isa_ref::{json::Json, Object};
use std::collections::BTreeMap;
const SOURCE: &str =
    include_str!("../../conformance/fixtures/controlled-action-09/controlled.bisa");
fn fields(j: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(m) = j else { panic!("object") };
    m
}
fn replace(text: &str, name: &str, value: Option<&Json>) -> String {
    let mut lines = text.lines();
    let mut out = Vec::new();
    while let Some(line) = lines.next() {
        if line == format!("section {name}") {
            lines.next().unwrap();
            if let Some(value) = value {
                out.push(line.to_string());
                out.push(value.canonical());
            }
        } else {
            out.push(line.to_string());
        }
    }
    {
        let mut text = out.join("\n");
        while text.contains("\n\n\n") {
            text = text.replace("\n\n\n", "\n\n");
        }
        text.trim_end_matches('\n').to_string() + "\n"
    }
}
fn certificate() -> Json {
    Object::parse(SOURCE)
        .unwrap()
        .section("certificate")
        .unwrap()
        .clone()
}
fn claim(c: &mut Json) -> &mut Json {
    let root = c
        .get("bindings")
        .unwrap()
        .get("closure_hash")
        .unwrap()
        .string()
        .unwrap()
        .to_string();
    let r = fields(c).get_mut("regions").unwrap();
    let region = fields(r).get_mut(&root).unwrap();
    let Json::Array(claims) = fields(region).get_mut("claims").unwrap() else {
        panic!("claims")
    };
    &mut claims[0]
}
#[test]
fn standalone_claims_require_invocation_even_in_scheduled_objects() {
    for structural in [false, true] {
        let mut c = certificate();
        let cl = claim(&mut c);
        if structural {
            *cl = support::json(
                r#"{"kind":"structural","rule":"adjoint_table","subclaims":[],"witness":{}}"#,
            );
        } else {
            fields(fields(cl).get_mut("subject").unwrap()).remove("schedule_hash");
            fields(cl).remove("invocation");
        }
        let result = Object::parse(&replace(SOURCE, "certificate", Some(&c)));
        assert!(
            result.is_err(),
            "standalone structural={structural} admitted without invocation"
        );
        assert!(result.unwrap_err().to_string().contains("K13"));
    }
}
#[test]
fn expected_oracle_requires_its_own_realised_schedule_binding() {
    let mut c = certificate();
    let cl = claim(&mut c);
    fields(cl).insert("expected".into(), Json::String("oracle".into()));
    fields(fields(cl).get_mut("subject").unwrap()).remove("schedule_hash");
    fields(cl).insert(
        "invocation".into(),
        support::json(r#"{"args":[],"budget":["0","0","0","0"]}"#),
    );
    let source = SOURCE.to_string();
    let result = Object::parse(&replace(&source, "certificate", Some(&c)));
    assert!(
        result.is_err(),
        "expected:oracle admitted without subject.schedule_hash"
    );
    assert!(result.unwrap_err().to_string().contains("schedule"));
}
#[test]
fn a_data_root_certificate_is_a_diagnostic_and_never_a_panic() {
    let mut c = certificate();
    let mut base = replace(SOURCE, "certificate", None);
    let mut symbols = Object::parse(&base)
        .unwrap()
        .section("symbols")
        .unwrap()
        .clone();
    let data_index = fields(&mut symbols).len().to_string();
    fields(&mut symbols).insert(data_index,support::json(r#"{"name":"img","kind":"data","signature":{"abi":"0.9","kind":"data","tier":"-","size":1},"interface_hash":"0000000000000000000000000000000000000000000000000000000000000000"}"#));
    base = replace(&base, "symbols", Some(&symbols));
    base = base.replace(
        "\nsection schedule\n",
        "\nsection data\nimg 00\n\nsection schedule\n",
    );
    let base = support::rehash(&base);
    let id = base.identity().unwrap();
    let b = fields(fields(&mut c).get_mut("bindings").unwrap());
    b.insert("object_hash".into(), Json::String(id.object_hash));
    b.insert(
        "closure_hash".into(),
        Json::String(id.closure_hashes["img"].clone().unwrap()),
    );
    let source = format!("{}\nsection certificate\n{}\n", base.text, c.canonical());
    let result = std::panic::catch_unwind(|| Object::parse(&source));
    assert!(result.is_ok(), "untrusted certificate data root panicked");
    assert!(result.unwrap().is_err());
}

// K13: a schedule subject denotes the realised operation for every relation.
#[test]
fn scheduled_table_and_program_claims_never_prove_reference_only_replay() {
    use bennett_isa_ref::ClaimVerdict;
    let source = include_str!("../../conformance/fixtures/accepted-k91-09/pair-distinct.bisa");
    let object = Object::parse(source).unwrap();
    let root = object.identity().unwrap().closure_hashes["pair"]
        .clone()
        .unwrap();
    for relation in ["table", "program", "oracle"] {
        let mut c = object.section("certificate").unwrap().clone();
        let schedule = c
            .get("bindings")
            .unwrap()
            .get("schedule_hash")
            .unwrap()
            .clone();
        let cl = claim(&mut c);
        fields(fields(cl).get_mut("subject").unwrap()).insert("schedule_hash".into(), schedule);
        match relation {
            "program" => {
                fields(cl).insert(
                    "expected".into(),
                    support::json(&format!(r#"{{"program":"{root}"}}"#)),
                );
            }
            "oracle" => {
                fields(cl).insert("expected".into(), Json::String("oracle".into()));
            }
            _ => {}
        }
        let o = Object::parse(&replace(source, "certificate", Some(&c))).unwrap();
        assert!(
            matches!(
                o.verify_claim(&root, 0).unwrap(),
                ClaimVerdict::Unverified(_)
            ),
            "{relation}"
        );
        let reports = o.verify_certificate().unwrap().claims;
        assert!(
            matches!(reports[0].verdict, ClaimVerdict::Unverified(_)),
            "{relation}"
        );
        assert!(reports[0].coverage.is_none());

        // A syntactically valid hash must resolve to this object's schedule.
        fields(fields(claim(&mut c)).get_mut("subject").unwrap())
            .insert("schedule_hash".into(), Json::String("ab".repeat(32)));
        let error = Object::parse(&replace(source, "certificate", Some(&c))).unwrap_err();
        assert!(error.to_string().contains("schedule"), "{error}");
    }
}

#[test]
fn claim_schedule_hash_requires_a_present_matching_schedule() {
    let source = include_str!("../../conformance/fixtures/accepted-k91-09/pair-distinct.bisa");
    let object = Object::parse(source).unwrap();
    let mut c = object.section("certificate").unwrap().clone();
    let hash = c
        .get("bindings")
        .unwrap()
        .get("schedule_hash")
        .unwrap()
        .clone();
    fields(fields(claim(&mut c)).get_mut("subject").unwrap()).insert("schedule_hash".into(), hash);
    let missing = replace(&replace(source, "schedule", None), "certificate", Some(&c));
    let error = Object::parse(&missing).unwrap_err();
    assert!(
        error.to_string().contains("claim schedule is absent"),
        "{error}"
    );
}

#[test]
fn scheduled_and_reference_relations_reject_conflicting_subject_closures() {
    use bennett_isa_ref::ClaimVerdict;
    let source = include_str!("../../conformance/fixtures/accepted-k91-09/pair-distinct.bisa");
    let object = Object::parse(source).unwrap();
    let root = object.identity().unwrap().closure_hashes["pair"]
        .clone()
        .unwrap();
    for scheduled in [true, false] {
        for program in [false, true] {
            let mut c = object.section("certificate").unwrap().clone();
            let schedule = c
                .get("bindings")
                .unwrap()
                .get("schedule_hash")
                .unwrap()
                .clone();
            if scheduled {
                fields(fields(claim(&mut c)).get_mut("subject").unwrap())
                    .insert("schedule_hash".into(), schedule);
            }
            if program {
                fields(claim(&mut c)).insert(
                    "expected".into(),
                    support::json(&format!(r#"{{"program":"{root}"}}"#)),
                );
            }
            if scheduled {
                let valid = Object::parse(&replace(source, "certificate", Some(&c))).unwrap();
                assert!(matches!(
                    valid.verify_claim(&root, 0).unwrap(),
                    ClaimVerdict::Unverified(_)
                ));
                let report = valid.verify_certificate().unwrap();
                assert!(matches!(
                    report.claims[0].verdict,
                    ClaimVerdict::Unverified(_)
                ));
                assert!(report.claims[0].coverage.is_none());
            }
            fields(fields(claim(&mut c)).get_mut("subject").unwrap())
                .insert("closure_hash".into(), Json::String("ab".repeat(32)));
            match Object::parse(&replace(source, "certificate", Some(&c))) {
                Err(error) => assert!(error.to_string().contains("subject"), "{error}"),
                Ok(invalid) => {
                    assert!(
                        invalid.verify_claim(&root, 0).is_err(),
                        "scheduled={scheduled}, program={program}"
                    );
                    assert!(
                        invalid.verify_certificate().is_err(),
                        "scheduled={scheduled}, program={program}"
                    );
                }
            }
        }
    }
}
