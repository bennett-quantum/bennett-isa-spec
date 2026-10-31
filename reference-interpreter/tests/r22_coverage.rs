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
