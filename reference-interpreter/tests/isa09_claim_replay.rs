mod support;
use bennett_isa_ref::{json::Json, sha256_hex, ClaimVerdict, Object};
use std::collections::BTreeMap;

const BASE: &str = include_str!("../../conformance/fixtures/accepted-k84-09/baseline.bisa");
const CONTROLLED: &str =
    include_str!("../../conformance/fixtures/controlled-action-09/controlled.bisa");

fn fields(value: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(fields) = value else {
        panic!("object")
    };
    fields
}
fn changed(source: &str, mutate: impl FnOnce(&mut Json)) -> Object {
    Object::parse(&changed_text(source, mutate)).unwrap()
}
fn changed_text(source: &str, mutate: impl FnOnce(&mut Json)) -> String {
    let (base, certificate) = source.split_once("section certificate\n").unwrap();
    let mut certificate = Json::parse(certificate.trim()).unwrap();
    let root = certificate
        .get("bindings")
        .unwrap()
        .get("closure_hash")
        .unwrap()
        .string()
        .unwrap()
        .to_string();
    let region = fields(fields(&mut certificate).get_mut("regions").unwrap())
        .get_mut(&root)
        .unwrap();
    let Json::Array(claims) = fields(region).get_mut("claims").unwrap() else {
        panic!("claims")
    };
    mutate(&mut claims[0]);
    format!("{base}section certificate\n{}\n", certificate.canonical())
}
fn verify_root(object: &Object) -> bennett_isa_ref::Result<ClaimVerdict> {
    let closure = object
        .section("certificate")
        .unwrap()
        .get("bindings")
        .unwrap()
        .get("closure_hash")
        .unwrap()
        .string()
        .unwrap();
    object.verify_claim(closure, 0)
}

fn with_certificate(object: &Object, name: &str, mut claim: Json, slots: Json) -> Object {
    let mut certificate = Object::parse(BASE)
        .unwrap()
        .section("certificate")
        .unwrap()
        .clone();
    let identity = object.identity().unwrap();
    let closure = identity.closure_hashes[name].as_ref().unwrap();
    fields(fields(&mut claim).get_mut("subject").unwrap())
        .insert("closure_hash".into(), Json::String(closure.clone()));
    let bindings = fields(fields(&mut certificate).get_mut("bindings").unwrap());
    for (key, value) in [
        ("object_hash", &identity.object_hash),
        ("interface_hash", &identity.interface_hashes[name]),
        ("closure_hash", closure),
        ("reference_program_hash", closure),
    ] {
        bindings.insert(key.into(), Json::String(value.clone()));
    }
    bindings.insert(
        "invocation".into(),
        claim.get("invocation").unwrap().clone(),
    );
    let regions = fields(fields(&mut certificate).get_mut("regions").unwrap());
    let mut region = regions.values().next().unwrap().clone();
    fields(&mut region).insert(
        "signature".into(),
        object.functions[name].signature_for(object.isa_version),
    );
    fields(&mut region).insert(
        "kind".into(),
        Json::String(object.functions[name].kind.clone()),
    );
    fields(&mut region).insert("claims".into(), Json::Array(vec![claim]));
    fields(fields(&mut region).get_mut("frame").unwrap()).insert("slots".into(), slots);
    regions.clear();
    regions.insert(closure.clone(), region);
    fields(fields(&mut certificate).get_mut("composition").unwrap())
        .insert("multiplicities".into(), support::json("{}"));
    Object::parse(&format!(
        "{}\nsection certificate\n{}\n",
        object.text,
        certificate.canonical()
    ))
    .unwrap()
}

fn typed_oracle(argument: &str, steps: u64) -> Object {
    let source = support::object(&[("sum",
        r#"{"abi":"0.8","args":["i2"],"fault":4,"in":[2],"kind":"oracle","mem":[{"A":1,"W":8,"role":"in"}],"tgt":2,"tier":"q"}"#,
        "oracle @sum (x:wire[2] in) (t:wire[2]) (f:wire[4]) (db:region[1,8] in) (a:i2)\nregs { r:reg[2] }\n^entry():\nr = lift.add x a\nlift.retval r")], "{}",
        "\nsection layout\n{\"A_tab\":1,\"H\":2,\"S_cells\":0,\"globals\":{},\"heap_base\":3,\"heap_limit_max\":0,\"interface_regions\":{\"sum\":{\"db\":{\"base\":1}}},\"null_size\":1,\"ptr_width\":2}\n")
        .replace("=0.8", "=0.9").replace("\"0.8\"", "\"0.9\"").replace("ptr_width=32", "ptr_width=2");
    let object = support::rehash(&source);
    let mut claim = support::json(
        r#"{
        "kind":"exhaustive","subject":{"closure_hash":"","variant":"forward","inherited_controls":0},
        "invocation":{"args":["2"],"budget":["2","0","0","0"]},
        "domain":{"bundles":[{"name":"x","width":2,"lo":1,"hi":1},{"name":"t","width":2,"lo":2,"hi":2},{"name":"f","width":4,"lo":3,"hi":3}],
          "borrowed":[{"name":"dirty","width":1,"lo":1,"hi":1}],
          "regions":[{"name":"db","cells":[{"index":0,"lo":2,"hi":2},{"index":1,"lo":1,"hi":1}]}]},
        "expected":{"table":[{"before":{"bundles":["1","2","3"],"borrowed":["1"],"regions":[["2","1"]]},
                               "after":{"bundles":["1","1","3"],"borrowed":["1"],"regions":[["2","1"]]}}]}}
    "#,
    );
    fields(fields(&mut claim).get_mut("invocation").unwrap()).insert(
        "args".into(),
        Json::Array(vec![Json::String(argument.into())]),
    );
    fields(fields(&mut claim).get_mut("invocation").unwrap()).insert(
        "budget".into(),
        support::json(&format!("[\"{steps}\",\"0\",\"0\",\"0\"]")),
    );
    with_certificate(
        &object,
        "sum",
        claim,
        support::json(
            r#"[
        {"id":"in:0","role":"in","width":2,"class":"fast"},
        {"id":"tgt:0","role":"tgt","width":2,"class":"fast"},
        {"id":"fault:0","role":"fault","width":4,"class":"fast"},
        {"id":"mem:0","role":"mem_in","width":16,"class":"fast"}]"#,
        ),
    )
}

#[test]
fn standalone_oracle_binds_its_own_argument_budget_and_complete_interface() {
    let object = typed_oracle("2", 2);
    assert_eq!(
        verify_root(&object).unwrap(),
        ClaimVerdict::Evidence { elements: 1 }
    );
    for object in [typed_oracle("1", 2), typed_oracle("2", 1)] {
        let error = verify_root(&object).unwrap_err().to_string();
        assert!(error.contains("full tuple mismatch"), "{error}");
    }
    for field in ["bundles", "borrowed", "regions"] {
        let object = changed(&typed_oracle("2", 2).text, |claim| {
            let table = fields(fields(claim).get_mut("expected").unwrap())
                .get_mut("table")
                .unwrap();
            let Json::Array(rows) = table else {
                panic!("rows")
            };
            let after = fields(&mut rows[0]).get_mut("after").unwrap();
            let Json::Array(values) = fields(after).get_mut(field).unwrap() else {
                panic!("values")
            };
            if field == "regions" {
                let Json::Array(cells) = &mut values[0] else {
                    panic!("cells")
                };
                cells[0] = Json::String("3".into());
            } else {
                values[0] = Json::String("0".into());
            }
        });
        assert!(verify_root(&object).is_err(), "ignored changed {field}");
    }
}
#[test]
fn published_full_tuple_tables_prove_only_the_bound_reference_relation() {
    for source in [BASE, CONTROLLED] {
        let object = Object::parse(source).unwrap();
        let certificate = object.section("certificate").unwrap();
        let root = certificate
            .get("bindings")
            .unwrap()
            .get("closure_hash")
            .unwrap()
            .string()
            .unwrap();
        let claim = &certificate
            .get("regions")
            .unwrap()
            .get(root)
            .unwrap()
            .get("claims")
            .unwrap()
            .array()
            .unwrap()[0];
        if claim
            .get("subject")
            .unwrap()
            .object()
            .unwrap()
            .contains_key("schedule_hash")
        {
            assert!(matches!(
                verify_root(&object).unwrap(),
                ClaimVerdict::Unverified(_)
            ));
            let root_report = |o: &Object| {
                o.verify_certificate()
                    .unwrap()
                    .into_iter()
                    .find(|r| r.closure_hash == root && r.index == 0)
                    .unwrap()
            };
            assert!(matches!(
                root_report(&object).verdict,
                ClaimVerdict::Unverified(_)
            ));
            // Keep the published table as a full reference-semantic oracle,
            // without misrepresenting it as execution of its realisation.
            let invocation = certificate
                .get("bindings")
                .unwrap()
                .get("invocation")
                .unwrap()
                .clone();
            let reference = changed(source, |claim| {
                fields(fields(claim).get_mut("subject").unwrap()).remove("schedule_hash");
                fields(claim).insert("invocation".into(), invocation);
            });
            assert_eq!(verify_root(&reference).unwrap(), ClaimVerdict::Proof);
            assert_eq!(root_report(&reference).verdict, ClaimVerdict::Proof);
            assert_eq!(
                reference.identity().unwrap().object_hash,
                object.identity().unwrap().object_hash
            );
        } else {
            assert_eq!(verify_root(&object).unwrap(), ClaimVerdict::Proof);
        }
    }
}
#[test]
fn a_wrong_before_or_after_tuple_rejects() {
    for side in ["before", "after"] {
        let object = changed(BASE, |claim| {
            let table = fields(fields(claim).get_mut("expected").unwrap())
                .get_mut("table")
                .unwrap();
            let Json::Array(rows) = table else {
                panic!("rows")
            };
            let tuple = fields(&mut rows[0]).get_mut(side).unwrap();
            fields(tuple).insert("bundles".into(), support::json("[\"1\"]"));
            if side == "after" {
                fields(tuple).insert("bundles".into(), support::json("[\"0\"]"));
            } else {
                let tuple = fields(&mut rows[1]).get_mut(side).unwrap();
                fields(tuple).insert("bundles".into(), support::json("[\"0\"]"));
            }
        });
        let error = verify_root(&object).unwrap_err().to_string();
        assert!(error.contains("K13"), "{error}");
    }
}
#[test]
fn singleton_sampling_counts_distinct_elements_and_uses_one_table_row() {
    let object = changed(BASE, |claim| {
        fields(claim).insert("kind".into(), Json::String("sampled".into()));
        fields(claim).insert(
            "sampler".into(),
            support::json(r#"{"algorithm":"xoshiro256**","seed":["1","0","0","0"],"count":4}"#),
        );
        let domain = fields(claim).get_mut("domain").unwrap();
        fields(domain).insert(
            "bundles".into(),
            support::json(r#"[{"name":"t","width":1,"lo":0,"hi":0}]"#),
        );
        let table = fields(fields(claim).get_mut("expected").unwrap())
            .get_mut("table")
            .unwrap();
        let Json::Array(rows) = table else {
            panic!("rows")
        };
        rows.truncate(1);
    });
    assert_eq!(
        verify_root(&object).unwrap(),
        ClaimVerdict::Evidence { elements: 1 }
    );
}

#[test]
fn inherited_control_positions_have_label_only_names_and_complete_tuples() {
    let bare = Object::parse(
        &(BASE
            .split_once("section certificate\n")
            .unwrap()
            .0
            .trim_end()
            .to_string()
            + "\n"),
    )
    .unwrap();
    let mut claim = Object::parse(BASE)
        .unwrap()
        .section("certificate")
        .unwrap()
        .get("regions")
        .unwrap()
        .object()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .get("claims")
        .unwrap()
        .array()
        .unwrap()[0]
        .clone();
    fields(fields(&mut claim).get_mut("subject").unwrap())
        .insert("inherited_controls".into(), Json::Number("1".into()));
    fields(fields(&mut claim).get_mut("domain").unwrap()).insert(
        "bundles".into(),
        support::json(
            r#"[{"name":"t","width":1,"lo":0,"hi":1},{"name":"t","width":1,"lo":0,"hi":1}]"#,
        ),
    );
    let rows = (0..2).flat_map(|c| (0..2).map(move |t| {
        support::json(&format!(r#"{{"before":{{"bundles":["{c}","{t}"],"borrowed":[],"regions":[]}},"after":{{"bundles":["{c}","{}"],"borrowed":[],"regions":[]}}}}"#,t^c))
    })).collect();
    fields(fields(&mut claim).get_mut("expected").unwrap())
        .insert("table".into(), Json::Array(rows));
    let object = with_certificate(
        &bare,
        "flip",
        claim,
        support::json(r#"[{"id":"mut:0","role":"mut","width":1,"class":"fast"}]"#),
    );
    assert_eq!(verify_root(&object).unwrap(), ClaimVerdict::Proof);
}

#[test]
fn exact_global_phase_is_part_of_a_permutation_table_assertion() {
    let source = support::object(&[("phase",
        r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#,
        "action @phase (t:wire mut) () () () ()\n^entry():\ns t\ns t")],"{}","")
        .replace("=0.8","=0.9").replace("\"0.8\"","\"0.9\"");
    let bare = support::rehash(&source);
    let claim = support::json(
        r#"{"kind":"exhaustive","subject":{"closure_hash":"","variant":"forward","inherited_controls":0},"invocation":{"args":[],"budget":["0","0","0","0"]},"domain":{"bundles":[{"name":"t","width":1,"lo":0,"hi":1}],"borrowed":[],"regions":[]},"expected":{"table":[{"before":{"bundles":["0"],"borrowed":[],"regions":[]},"after":{"bundles":["0"],"borrowed":[],"regions":[]}},{"before":{"bundles":["1"],"borrowed":[],"regions":[]},"after":{"bundles":["1"],"borrowed":[],"regions":[]}}]}}"#,
    );
    let object = with_certificate(
        &bare,
        "phase",
        claim,
        support::json(r#"[{"id":"mut:0","role":"mut","width":1,"class":"fast"}]"#),
    );
    let error = verify_root(&object).unwrap_err().to_string();
    assert!(error.contains("exact +1 basis phase"), "{error}");
}

fn fixed_image_oracle() -> Object {
    changed(&typed_oracle("2", 2).text, |claim| {
        let domain = fields(claim).get_mut("domain").unwrap();
        fields(domain).insert(
            "bundles".into(),
            support::json(
                r#"[
            {"name":"x","width":2,"lo":0,"hi":3},
            {"name":"t","width":2,"lo":0,"hi":3},
            {"name":"f","width":4,"lo":0,"hi":15}]"#,
            ),
        );
        fields(domain).insert(
            "borrowed".into(),
            support::json(r#"[{"name":"dirty","width":1,"lo":0,"hi":1}]"#),
        );
        fields(domain).insert("regions".into(), support::json("[]"));
        fields(domain).insert(
            "fixed_regions".into(),
            support::json(&format!(
                r#"[{{"name":"db","A":1,"W":8,"image":"0201","sha256":"{}"}}]"#,
                sha256_hex(&[2, 1])
            )),
        );
        let mut rows = Vec::new();
        for x in 0..4 {
            for t in 0..4 {
                for f in 0..16 {
                    for b in 0..2 {
                        rows.push(support::json(&format!(r#"{{
                "before":{{"bundles":["{x}","{t}","{f}"],"borrowed":["{b}"],"regions":[["2","1"]]}},
                "after":{{"bundles":["{x}","{}","{f}"],"borrowed":["{b}"],"regions":[["2","1"]]}}}}"#, t^((x+2)&3))));
                    }
                }
            }
        }
        fields(fields(claim).get_mut("expected").unwrap())
            .insert("table".into(), Json::Array(rows));
    })
}

#[test]
fn fixed_image_metadata_is_admitted_without_inspecting_quantum_state() {
    let object = fixed_image_oracle();
    object.check_hashes().unwrap();
    assert_eq!(
        verify_root(&object).unwrap(),
        ClaimVerdict::ProofRelativeToImage
    );
    let report = object.verify_certificate().unwrap().remove(0);
    let coverage = report.coverage.unwrap();
    assert_eq!(
        (
            coverage.elements,
            coverage.full_domain_elements.as_str(),
            coverage.covered_fraction.as_str()
        ),
        (512, "33554432", "1/65536")
    );
    assert_eq!(coverage.fixed_regions.len(), 1);
    assert_eq!(coverage.fixed_regions[0].sha256, sha256_hex(&[2, 1]));
    assert_eq!(report.context_key, support::json("[]"));
    let request = changed(&object.text, |claim| {
        fields(claim).insert("label".into(), Json::String("proof".into()));
    });
    assert!(verify_root(&request)
        .unwrap_err()
        .to_string()
        .contains("requested proof"));
}

#[test]
fn fixed_image_stale_hash_wrong_dimensions_and_duplicate_regions_reject() {
    let object = fixed_image_oracle();
    for mutation in ["hash", "width", "duplicate", "also-ranged", "uppercase"] {
        let text = changed_text(&object.text, |claim| {
            let domain = fields(claim).get_mut("domain").unwrap();
            if mutation == "also-ranged" {
                fields(domain).insert(
                    "regions".into(),
                    support::json(r#"[{"name":"db","cells":"all"}]"#),
                );
                return;
            }
            let Json::Array(images) = fields(domain).get_mut("fixed_regions").unwrap() else {
                panic!("images")
            };
            match mutation {
                "hash" => {
                    fields(&mut images[0]).insert("sha256".into(), Json::String("0".repeat(64)));
                }
                "width" => {
                    fields(&mut images[0]).insert("W".into(), Json::Number("7".into()));
                }
                "uppercase" => {
                    fields(&mut images[0]).insert("image".into(), Json::String("AB01".into()));
                }
                "duplicate" => images.push(images[0].clone()),
                _ => unreachable!(),
            }
        });
        let error = Object::parse(&text).unwrap_err().to_string();
        assert!(error.contains("K85"), "{mutation}: {error}");
    }
}

#[test]
fn sampled_singleton_report_has_exact_distinct_coverage() {
    let object = changed(&typed_oracle("2", 2).text, |claim| {
        fields(claim).insert("kind".into(), Json::String("sampled".into()));
        fields(claim).insert(
            "sampler".into(),
            support::json(r#"{"algorithm":"xoshiro256**","seed":["1","0","0","0"],"count":4}"#),
        );
    });
    let report = object.verify_certificate().unwrap().remove(0);
    assert_eq!(report.verdict, ClaimVerdict::Evidence { elements: 1 });
    let coverage = report.coverage.unwrap();
    assert_eq!(coverage.covered_fraction, "1/33554432");
}
