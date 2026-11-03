mod support;
use bennett_isa_ref::{json::Json, ClaimVerdict, Object};
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
    Object::parse(&format!(
        "{base}section certificate\n{}\n",
        certificate.canonical()
    ))
    .unwrap()
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
#[test]
fn published_full_tuple_tables_verify_as_proof() {
    for source in [BASE, CONTROLLED] {
        assert_eq!(
            verify_root(&Object::parse(source).unwrap()).unwrap(),
            ClaimVerdict::Proof
        );
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
