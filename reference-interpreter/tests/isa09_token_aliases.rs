mod support;
use bennett_isa_ref::{json::Json, ClaimVerdict, Object};
use std::collections::BTreeMap;
const BASE: &str = include_str!("../../conformance/fixtures/accepted-k84-09/baseline.bisa");
fn fields(value: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(fields) = value else {
        panic!("object")
    };
    fields
}
fn certificate_object(aliases: Option<&str>, second: u64) -> String {
    let (source, certificate) = BASE.split_once("section certificate\n").unwrap();
    let source = source.trim_end().to_string() + "\n";
    let mut symbols = Object::parse(&source)
        .unwrap()
        .section("symbols")
        .unwrap()
        .clone();
    let symbol = fields(&mut symbols).get_mut("0").unwrap();
    fields(fields(symbol).get_mut("signature").unwrap())
        .insert("args".into(), support::json(r#"["token","token"]"#));
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let section = lines.iter().position(|s| s == "section symbols").unwrap();
    lines[section + 1] = symbols.canonical();
    let source = (lines.join("\n") + "\n").replace(
        "(t:wire mut) () () () ()",
        "(t:wire mut) () () (a:token, b:token) ()",
    );
    let object = support::rehash(&source);
    let identity = object.identity().unwrap();
    let closure = identity.closure_hashes["flip"].as_ref().unwrap();
    let mut c = Json::parse(certificate.trim()).unwrap();
    let bindings = fields(fields(&mut c).get_mut("bindings").unwrap());
    for (key, value) in [
        ("object_hash", &identity.object_hash),
        ("interface_hash", &identity.interface_hashes["flip"]),
        ("closure_hash", closure),
        ("reference_program_hash", closure),
    ] {
        bindings.insert(key.into(), Json::String(value.clone()));
    }
    let mut invocation = support::json(&format!(
        r#"{{"args":[["1","0","0","0"],["{second}","0","0","0"]],"budget":["0","0","0","0"]}}"#
    ));
    if let Some(aliases) = aliases {
        fields(&mut invocation).insert("token_aliases".into(), support::json(aliases));
    }
    bindings.insert("invocation".into(), invocation.clone());
    let regions = fields(fields(&mut c).get_mut("regions").unwrap());
    let mut region = regions.values().next().unwrap().clone();
    fields(&mut region).insert(
        "signature".into(),
        object.functions["flip"].signature_for(object.isa_version),
    );
    let Json::Array(claims) = fields(&mut region).get_mut("claims").unwrap() else {
        panic!("claims")
    };
    fields(&mut claims[0]).insert("invocation".into(), invocation);
    fields(fields(&mut claims[0]).get_mut("subject").unwrap())
        .insert("closure_hash".into(), Json::String(closure.clone()));
    regions.clear();
    regions.insert(closure.clone(), region);
    fields(fields(&mut c).get_mut("composition").unwrap()).insert(
        "multiplicities".into(),
        support::json(&format!(r#"{{"{closure}:forward:0":1}}"#)),
    );
    format!("{}\nsection certificate\n{}\n", object.text, c.canonical())
}

#[test]
fn certificate_invocations_admit_both_equal_tuple_identity_partitions() {
    let mut hashes = Vec::new();
    for labels in ["[0,0]", "[0,1]"] {
        let object = Object::parse(&certificate_object(Some(labels), 1)).unwrap();
        object.check_hashes().unwrap();
        hashes.push(object.identity().unwrap().object_hash);
        let certificate = object.section("certificate").unwrap();
        let closure = certificate
            .get("bindings")
            .unwrap()
            .get("closure_hash")
            .unwrap()
            .string()
            .unwrap();
        assert_eq!(
            object.verify_claim(closure, 0).unwrap(),
            ClaimVerdict::Proof
        );
    }
    assert_eq!(
        hashes[0], hashes[1],
        "partition metadata does not change object identity"
    );
}

#[test]
fn missing_malformed_and_tuple_inconsistent_certificate_partitions_reject() {
    for (labels, second) in [
        (None, 1),
        (Some("[1,1]"), 1),
        (Some("[0,2]"), 1),
        (Some("[0]"), 1),
        (Some("[0,0]"), 2),
        (Some("[-1,0]"), 1),
    ] {
        let result = Object::parse(&certificate_object(labels, second));
        assert!(result.is_err(), "admitted {labels:?}/{second}");
        assert!(result.unwrap_err().to_string().contains("K91"));
    }
}

#[test]
fn uniform_token_claims_quantify_over_every_consistent_partition() {
    let original = certificate_object(Some("[0,1]"), 1);
    let (source, c) = original.split_once("section certificate\n").unwrap();
    for aliases in [false, true] {
        let mut certificate = Json::parse(c.trim()).unwrap();
        let regions = fields(fields(&mut certificate).get_mut("regions").unwrap());
        let region = regions.values_mut().next().unwrap();
        let mut claim = support::json(
            r#"{"kind":"structural","rule":"adjoint_table","subclaims":[],"witness":{},"invocation":{"args":"all","budget":"all"}}"#,
        );
        if aliases {
            fields(fields(&mut claim).get_mut("invocation").unwrap())
                .insert("token_aliases".into(), Json::String("all".into()));
        }
        fields(region).insert("claims".into(), Json::Array(vec![claim]));
        let result = Object::parse(&format!(
            "{source}section certificate\n{}\n",
            certificate.canonical()
        ));
        assert_eq!(result.is_ok(), aliases);
        if !aliases {
            assert!(result.unwrap_err().to_string().contains("K91"));
        }
    }
}

fn oracle_source(token: bool) -> Object {
    let signature = format!(
        r#"{{"abi":"0.8","args":{},"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}}"#,
        if token { r#"["token"]"# } else { "[]" }
    );
    let body = format!(
        "oracle @id (x:wire in) (t:wire) (f:wire[4]) () {}\n^entry():\nlift.retval x",
        if token { "(a:token)" } else { "()" }
    );
    support::rehash(
        &support::object(&[("id", &signature, &body)], "{}", "")
            .replace("=0.8", "=0.9")
            .replace("\"0.8\"", "\"0.9\""),
    )
}
fn scheduled(source: &Object, aliases: Option<&str>) -> String {
    let name = &source.function_order[0];
    let closure = source.identity().unwrap().closure_hashes[name]
        .clone()
        .unwrap();
    let mut schedule = support::json(&format!(
        r#"{{"bindings":{{"budget":[1,0,0,0],"closure_hash":"{closure}","epsilon":0,"inherited_controls":0,"params_hash":"{}","region":"{name}","scheduler":{{"name":"k91","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[]}}"#,
        bennett_isa_ref::sha256_hex(b"[]"),
        "0".repeat(64)
    ));
    if let Some(aliases) = aliases {
        fields(fields(&mut schedule).get_mut("bindings").unwrap())
            .insert("token_aliases".into(), support::json(aliases));
    }
    format!(
        "{}\nsection schedule\n{}\n",
        source.text,
        schedule.canonical()
    )
}

#[test]
fn schedule_alias_slots_prepend_primary_only_for_oracles() {
    let action = Object::parse(
        &(BASE
            .split_once("section certificate\n")
            .unwrap()
            .0
            .trim_end()
            .to_string()
            + "\n"),
    )
    .unwrap();
    for (source, valid, invalid) in [(&action, "[]", "[0]"), (&oracle_source(false), "[0]", "[]")] {
        Object::parse(&scheduled(source, Some(valid)))
            .unwrap()
            .check_hashes()
            .unwrap();
        Object::parse(&scheduled(source, None)).unwrap(); // legacy token-free binding
        assert!(Object::parse(&scheduled(source, Some(invalid)))
            .unwrap_err()
            .to_string()
            .contains("K91"));
    }
    let source = oracle_source(true);
    for aliases in ["[0,0]", "[0,1]"] {
        Object::parse(&scheduled(&source, Some(aliases))).unwrap();
    }
    let error = Object::parse(&scheduled(&source, None)).unwrap_err();
    assert!(error.to_string().contains("K91"));
}
