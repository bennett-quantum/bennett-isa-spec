mod support;
use bennett_isa_ref::{ClaimVerdict, Object};
#[test]
fn sol_r22_b01_false_proof_and_controls() {
    for (text, full) in [
        (include_str!("fixtures/r22/false-cnot-proof.bisa"), false),
        (
            include_str!("fixtures/r22/false-cnot-renamed-control.bisa"),
            false,
        ),
        (
            include_str!("fixtures/r22/false-cnot-full-control.bisa"),
            true,
        ),
    ] {
        let o = Object::parse(text).unwrap();
        o.check_hashes().unwrap();
        let hash = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
        if full {
            let e = o.verify_claim(&hash, 0).unwrap_err().to_string();
            assert!(e.contains("§12.1") && e.contains("element 4"), "{e}");
        } else {
            assert_eq!(
                o.verify_claim(&hash, 0).unwrap(),
                ClaimVerdict::Evidence { elements: 4 }
            );
            assert_eq!(
                o.verify_certificate().unwrap()[0].verdict,
                ClaimVerdict::Evidence { elements: 4 }
            );
        }
    }
}

#[test]
fn sol_r22_b01_public_role_collision_family() {
    use bennett_isa_ref::json::Json;
    let text = include_str!("fixtures/r22/false-cnot-proof.bisa");
    let (base, cert) = text.split_once("\nsection certificate\n").unwrap();
    let o = Object::parse(text).unwrap();
    let hash = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
    let original_expected = o.identity().unwrap().closure_hashes["identity"]
        .clone()
        .unwrap();
    let mut labels = std::collections::BTreeMap::new();
    let mut cases = 0;
    for borrowed in ["t", "u", "b", "r"] {
        for subset in 0..4 {
            for full in [false, true] {
                let bundles = ["t", "u"]
                    .iter()
                    .enumerate()
                    .filter(|(i, _)| subset & (1 << i) != 0)
                    .map(|(_, name)| {
                        format!(
                            r#"{{"name":"{name}","width":1,"lo":0,"hi":{}}}"#,
                            u8::from(full)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let new_domain = Json::parse(&format!(r#"{{"bundles":[{bundles}],"regions":[],"borrowed":[{{"name":"{borrowed}","width":1,"lo":0,"hi":1}}]}}"#)).unwrap();
                let mut cert = Json::parse(cert).unwrap();
                fn replace_domain(j: &mut Json, domain: &Json) {
                    match j {
                        Json::Object(o) => {
                            for (k, v) in o {
                                if k == "domain" && matches!(v, Json::Object(_)) {
                                    *v = domain.clone();
                                } else {
                                    replace_domain(v, domain);
                                }
                            }
                        }
                        Json::Array(a) => {
                            for j in a {
                                replace_domain(j, domain);
                            }
                        }
                        _ => (),
                    }
                }
                replace_domain(&mut cert, &new_domain);
                // Compare each replay with itself, so verdict tests coverage,
                // independently of the witness's expected-program mismatch.
                let cert = cert.canonical().replace(
                    &format!("\"program\":\"{original_expected}\""),
                    &format!("\"program\":\"{hash}\""),
                );
                let o = Object::parse(&format!("{base}\nsection certificate\n{cert}\n")).unwrap();
                o.check_hashes().unwrap();
                let verdict = o.verify_claim(&hash, 0).unwrap();
                if let Some(previous) = labels.insert((subset, full), verdict.clone()) {
                    assert_eq!(
                        verdict, previous,
                        "borrowed renaming must preserve the exact label/count"
                    );
                }
                assert_eq!(
                    verdict == ClaimVerdict::Proof,
                    subset == 3 && full,
                    "borrowed={borrowed} subset={subset} full={full}: {verdict:?}"
                );
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 32);
}

#[test]
fn sol_r22_b01_table_spoof_and_borrowed_renaming() {
    let text = include_str!("fixtures/r22/domain-role-spoof.bisa");
    let o = Object::parse(text).unwrap();
    o.check_hashes().unwrap();
    let hash = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
    for name in ["t", "b", "unrelated"] {
        let renamed = text.replace(
            "\"name\":\"t\",\"width\":1",
            &format!("\"name\":\"{name}\",\"width\":1"),
        );
        let o = Object::parse(&renamed).unwrap();
        o.check_hashes().unwrap();
        assert_eq!(
            o.verify_claim(&hash, 0).unwrap(),
            ClaimVerdict::Evidence { elements: 2 }
        );
    }
}

#[test]
fn r23_public_oracle_roles_and_explicit_region_cells() {
    use bennett_isa_ref::json::Json;
    use std::collections::BTreeMap;
    fn set(value: &mut Json, key: &str, next: Json) {
        let Json::Object(fields) = value else {
            panic!("object expected")
        };
        fields.insert(key.into(), next);
    }
    fn replay(o: &Object, domain: Json, interface: Json) -> ClaimVerdict {
        let (_, template) = include_str!("fixtures/r22/false-cnot-proof.bisa")
            .split_once("\nsection certificate\n")
            .unwrap();
        let mut cert = Json::parse(template).unwrap();
        let id = o.identity().unwrap();
        let hash = id.closure_hashes["a"].clone().unwrap();
        let mut bindings = cert.get("bindings").unwrap().clone();
        for (key, value) in [
            ("object_hash", id.object_hash),
            ("interface_hash", id.interface_hashes["a"].clone()),
            ("closure_hash", hash.clone()),
            ("reference_program_hash", hash.clone()),
        ] {
            set(&mut bindings, key, Json::String(value));
        }
        set(&mut cert, "bindings", bindings);
        let mut region = cert
            .get("regions")
            .unwrap()
            .object()
            .unwrap()
            .values()
            .next()
            .unwrap()
            .clone();
        set(
            &mut region,
            "kind",
            Json::String(o.functions["a"].kind.clone()),
        );
        set(&mut region, "signature", o.functions["a"].signature());
        let mut frame = region.get("frame").unwrap().clone();
        set(&mut frame, "interface", interface);
        set(&mut region, "frame", frame);
        let mut claim = Json::Object(BTreeMap::from([
            ("kind".into(), Json::String("exhaustive".into())),
            ("domain".into(), domain),
            (
                "expected".into(),
                Json::Object(BTreeMap::from([(
                    "program".into(),
                    Json::String(hash.clone()),
                )])),
            ),
        ]));
        let schedule = (o.functions["a"].kind == "oracle").then(|| Json::parse(&format!(r#"{{"bindings":{{"budget":[1,0,0,0],"closure_hash":"{hash}","epsilon":0,"inherited_controls":0,"params_hash":"{}","region":"a","scheduler":{{"name":"test","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[]}}"#, "2".repeat(64), "3".repeat(64))).unwrap());
        if let Some(schedule) = &schedule {
            set(&mut claim,"subject",Json::parse(&format!(r#"{{"closure_hash":"{hash}","variant":"forward","inherited_controls":0,"schedule_hash":"{}"}}"#, bennett_isa_ref::sha256_hex(schedule.canonical().as_bytes()))).unwrap());
        }
        set(&mut region, "claims", Json::Array(vec![claim]));
        set(
            &mut cert,
            "regions",
            Json::Object(BTreeMap::from([(hash.clone(), region)])),
        );
        let schedule = schedule
            .map(|s| format!("\nsection schedule\n{}\n", s.canonical()))
            .unwrap_or_default();
        let p = Object::parse(&format!(
            "{}{schedule}\nsection certificate\n{}\n",
            o.text,
            cert.canonical()
        ))
        .unwrap();
        p.check_hashes().unwrap();
        let verdict = p.verify_claim(&hash, 0).unwrap();
        assert_eq!(p.verify_certificate().unwrap()[0].verdict, verdict);
        verdict
    }
    let sig =
        r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let body = "oracle @a (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x";
    let oracle = support::rehash(&support::object(&[("a", sig, body)], "{}", ""));
    let mut count = 0;
    let mut previous = BTreeMap::new();
    for borrowed in ["x", "t", "f", "other"] {
        for subset in 0..8 {
            let bundles = [("x", 1, 1), ("t", 1, 1), ("f", 4, 15)]
                .iter()
                .enumerate()
                .filter(|(i, _)| subset & (1 << i) != 0)
                .map(|(_, (name, width, hi))| {
                    format!(r#"{{"name":"{name}","width":{width},"lo":0,"hi":{hi}}}"#)
                })
                .collect::<Vec<_>>()
                .join(",");
            let domain=Json::parse(&format!(r#"{{"bundles":[{bundles}],"regions":[],"borrowed":[{{"name":"{borrowed}","width":1,"lo":0,"hi":1}}]}}"#)).unwrap();
            let verdict=replay(&oracle,domain,Json::parse(r#"[{"role":"in","width":1,"class":"any"},{"role":"tgt","width":1,"class":"any"},{"role":"fault","width":4,"class":"any"}]"#).unwrap());
            assert_eq!(
                verdict == ClaimVerdict::Proof,
                subset == 7,
                "{borrowed}/{subset}: {verdict:?}"
            );
            if let Some(old) = previous.insert(subset, verdict.clone()) {
                assert_eq!(old, verdict);
            }
            count += 1;
        }
    }
    let region = Object::parse(
        include_str!("fixtures/r21/sol-claim-region-all-small.bisa")
            .split_once("\nsection certificate\n")
            .unwrap()
            .0,
    )
    .unwrap();
    // This region has A=1, hence two cells: exercise explicit lists publicly.
    let mut previous = BTreeMap::new();
    for borrowed in ["t", "r", "other"] {
        for subset in 0..4 {
            for bundle in [false, true] {
                let cells = (0..2)
                    .filter(|i| subset & (1 << i) != 0)
                    .map(|i| format!(r#"{{"index":{i},"lo":0,"hi":1}}"#))
                    .collect::<Vec<_>>()
                    .join(",");
                let bundles = if bundle {
                    r#"{"name":"t","width":1,"lo":0,"hi":1}"#
                } else {
                    ""
                };
                let domain=Json::parse(&format!(r#"{{"bundles":[{bundles}],"regions":[{{"name":"r","cells":[{cells}]}}],"borrowed":[{{"name":"{borrowed}","width":1,"lo":0,"hi":1}}]}}"#)).unwrap();
                let verdict=replay(&region,domain,Json::parse(r#"[{"role":"mut","width":1,"class":"any"},{"role":"in","width":2,"class":"any"}]"#).unwrap());
                assert_eq!(verdict == ClaimVerdict::Proof, subset == 3 && bundle);
                if let Some(old) = previous.insert((subset, bundle), verdict.clone()) {
                    assert_eq!(old, verdict);
                }
                count += 1;
            }
        }
    }
    assert_eq!(count, 56);
}
