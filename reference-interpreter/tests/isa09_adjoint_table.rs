mod support;
use bennett_isa_ref::{json::Json, ClaimVerdict, Object};
use std::collections::BTreeMap;
const SOURCE: &str = include_str!("support/adjoint-table/arith.bij.add.w1.bisa");
const CLAIM: &str = r#"{"kind":"structural","rule":"adjoint_table","subclaims":[],"witness":{},"invocation":{"args":"all","budget":"all"},"label":"proof"}"#;

fn fields(j: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(m) = j else { panic!("object") };
    m
}
fn claims<'a>(c: &'a mut Json, root: &str) -> &'a mut Vec<Json> {
    let record = fields(fields(c).get_mut("regions").unwrap())
        .get_mut(root)
        .unwrap();
    let Json::Array(a) = fields(record).get_mut("claims").unwrap() else {
        panic!("claims")
    };
    a
}
fn attach(base: &Object, list: Vec<Json>) -> (String, String) {
    let id = base.identity().unwrap();
    let (name, f) = base.functions.first_key_value().unwrap();
    let root = id.closure_hashes[name].clone().unwrap();
    let mut c = Object::parse(SOURCE)
        .unwrap()
        .section("certificate")
        .unwrap()
        .clone();
    let bindings = fields(fields(&mut c).get_mut("bindings").unwrap());
    for (key, value) in [
        ("object_hash", &id.object_hash),
        ("interface_hash", &id.interface_hashes[name]),
        ("closure_hash", &root),
        ("reference_program_hash", &root),
    ] {
        bindings.insert(key.into(), Json::String(value.clone()));
    }
    bindings.remove("invocation");
    let regions = fields(fields(&mut c).get_mut("regions").unwrap());
    let mut rec = regions.values().next().unwrap().clone();
    fields(&mut rec).insert("kind".into(), Json::String(f.kind.clone()));
    fields(&mut rec).insert("signature".into(), f.signature_for(base.isa_version));
    fields(&mut rec).insert("bounds".into(), support::json(&format!("{:?}", f.bounds)));
    let slots = if matches!(f.kind.as_str(), "action" | "perm_action") {
        r#"[{"id":"mut:0","role":"mut","class":"any","width":1}]"#
    } else {
        "[]"
    };
    fields(&mut rec).insert(
        "frame".into(),
        support::json(&format!(
            r#"{{"slots":{slots},"scratch":[],"history":[],"borrow":[],"ancillae":[]}}"#
        )),
    );
    fields(&mut rec).insert("claims".into(), Json::Array(list));
    regions.clear();
    regions.insert(root.clone(), rec);
    (
        format!("{}\nsection certificate\n{}\n", base.text, c.canonical()),
        root,
    )
}
fn action(kind: &str, body: &str) -> Object {
    let signature = format!(
        r#"{{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"{kind}","mem":[],"mut":[1],"tier":"q"}}"#
    );
    let text = support::object(
        &[(
            "a",
            &signature,
            &format!("{kind} @a (x:wire mut) () () () ()\n^entry():\n{body}"),
        )],
        "{}",
        "",
    )
    .replace("0.8", "0.9");
    support::rehash(&text)
}
fn verify(text: &str, root: &str, index: usize) -> bennett_isa_ref::Result<ClaimVerdict> {
    Object::parse(text)?.verify_claim(root, index)
}
#[test]
fn published_arithmetic_adjoint_claims_establish() {
    let o = Object::parse(SOURCE).unwrap();
    o.check_hashes().unwrap();
    let id = o.identity().unwrap();
    let sub = id.closure_hashes["arith.bij.sub.w1"].as_ref().unwrap();
    assert_eq!(o.verify_claim(sub, 1).unwrap(), ClaimVerdict::Proof);
    let add = id.closure_hashes["arith.bij.add.w1"].as_ref().unwrap();
    let mut c = o.section("certificate").unwrap().clone();
    claims(&mut c, add).push(support::json(CLAIM));
    let text = format!(
        "{}section certificate\n{}\n",
        SOURCE.split_once("section certificate\n").unwrap().0,
        c.canonical()
    );
    let o = Object::parse(&text).unwrap();
    o.check_hashes().unwrap();
    assert_eq!(o.verify_claim(add, 1).unwrap(), ClaimVerdict::Proof);
    assert!(o
        .verify_certificate()
        .unwrap()
        .claims
        .iter()
        .all(|r| r.verdict == ClaimVerdict::Proof));
}
#[test]
fn minimal_non_involution_and_quantum_action_establish() {
    for (kind, body) in [("perm_action", "x x[0]"), ("action", "h x[0]\ns x[0]")] {
        let (text, root) = attach(&action(kind, body), vec![support::json(CLAIM)]);
        assert_eq!(verify(&text, &root, 0).unwrap(), ClaimVerdict::Proof);
    }
}
#[test]
fn non_action_subject_rejects() {
    let base = support::rehash(
        &support::object(
            &[(
                "a",
                r#"{"abi":"0.9","kind":"cc","tier":"cc","params":[],"results":[]}"#,
                "fn cc @a () () () () ()\n^entry():\nret",
            )],
            "{}",
            "",
        )
        .replace("0.8", "0.9"),
    );
    let (text, root) = attach(&base, vec![support::json(CLAIM)]);
    assert!(
        verify(&text, &root, 0).is_err(),
        "non-action adjoint admitted"
    );
}
#[test]
fn malformed_witness_and_indices_reject() {
    for bad in [
        CLAIM.replace(r#""witness":{}"#, r#""witness":{"other":0}"#),
        CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0]"),
        CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[8]"),
        CLAIM.replace(
            r#""args":"all","budget":"all""#,
            r#""args":[],"budget":"all""#,
        ),
    ] {
        let (text, root) = attach(&action("perm_action", "x x[0]"), vec![support::json(&bad)]);
        assert!(
            verify(&text, &root, 0).is_err(),
            "malformed claim admitted: {bad}"
        );
    }
}
fn replay(root: &str, kind: &str, hi: u32) -> Json {
    let sampler = if kind == "sampled" {
        r#", "sampler":{"algorithm":"xoshiro256**","seed":["1","0","0","0"],"count":1}"#
    } else {
        ""
    };
    support::json(&format!(
        r#"{{"kind":"{kind}","domain":{{"bundles":[{{"name":"x","width":1,"lo":0,"hi":{hi}}}],"regions":[],"borrowed":[]}},"expected":{{"program":"{root}"}},"subject":{{"closure_hash":"{root}","variant":"open_reference_name","inherited_controls":0}},"invocation":{{"args":[],"budget":["0","0","0","0"]}}{sampler}}}"#
    ))
}
#[test]
fn subclaims_are_checked_without_promoting_evidence_or_partial_domains() {
    let base = action("perm_action", "x x[0]");
    let root = base.identity().unwrap().closure_hashes["a"]
        .clone()
        .unwrap();
    for (kind, hi, proof) in [
        ("exhaustive", 1, true),
        ("exhaustive", 0, false),
        ("sampled", 1, false),
    ] {
        let structural = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0]"));
        let (text, _) = attach(&base, vec![replay(&root, kind, hi), structural]);
        let verdict = verify(&text, &root, 1).unwrap();
        if proof {
            assert_eq!(verdict, ClaimVerdict::Proof);
        } else {
            assert!(
                matches!(verdict, ClaimVerdict::Unverified(ref s) if s.contains("adjoint_table subclaim 0")),
                "{verdict:?}"
            );
        }
    }
}
#[test]
fn a_refuted_later_premise_is_not_hidden_by_an_unverified_one() {
    let base = action("perm_action", "x x[0]");
    let root = base.identity().unwrap().closure_hashes["a"]
        .clone()
        .unwrap();
    let mut false_claim = replay(&root, "exhaustive", 1);
    fields(&mut false_claim).insert("expected".into(), support::json(r#"{"table":[{"before":{"bundles":["0"],"borrowed":[],"regions":[]},"after":{"bundles":["0"],"borrowed":[],"regions":[]}},{"before":{"bundles":["1"],"borrowed":[],"regions":[]},"after":{"bundles":["1"],"borrowed":[],"regions":[]}}]}"#));
    let claim = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0,1]"));
    let (text, _) = attach(&base, vec![replay(&root, "sampled", 1), false_claim, claim]);
    assert!(verify(&text, &root, 2).is_err(), "refuted premise hidden");
}

#[test]
fn unavailable_linked_body_and_unverified_subclaim_stay_unverified() {
    let sig = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let imports = format!(
        r#"{{"b":{{"signature":{sig},"pinned_closure_hash":"{}"}}}}"#,
        "ab".repeat(32)
    );
    let base = support::rehash(
        &support::object(
            &[(
                "a",
                sig,
                "perm_action @a (x:wire mut) () () () ()\n^entry():\naction_call @b (x) () () ()",
            )],
            &imports,
            "",
        )
        .replace("0.8", "0.9"),
    );
    let dependent = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0]"));
    let (text, root) = attach(&base, vec![support::json(CLAIM), dependent]);
    let o = Object::parse(&text).unwrap();
    assert!(
        matches!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Unverified(ref s) if s.contains("linked definition unavailable: @b"))
    );
    assert!(
        matches!(o.verify_claim(&root, 1).unwrap(), ClaimVerdict::Unverified(ref s) if s.contains("adjoint_table subclaim 0"))
    );
}

#[test]
fn checked_source_discipline_is_required_even_through_public_fields() {
    let (text, root) = attach(&action("perm_action", "x x[0]"), vec![support::json(CLAIM)]);
    for op in ["unmapped_instruction", "h"] {
        let mut o = Object::parse(&text).unwrap();
        o.functions.get_mut("a").unwrap().blocks[0].instructions[0].op = op.into();
        assert!(
            o.verify_claim(&root, 0).is_err(),
            "unchecked source {op} proved"
        );
    }
}

#[test]
fn repeated_unverified_premises_do_not_become_a_cycle() {
    let base = action("perm_action", "x x[0]");
    let root = base.identity().unwrap().closure_hashes["a"]
        .clone()
        .unwrap();
    let dependency = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0]"));
    let claim = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[1,1]"));
    let (text, _) = attach(&base, vec![replay(&root, "sampled", 1), dependency, claim]);
    assert!(
        matches!(verify(&text, &root, 2).unwrap(), ClaimVerdict::Unverified(ref s) if s.contains("adjoint_table subclaim 1"))
    );
}

#[test]
fn binding_and_source_obligations_reject_instead_of_establishing() {
    let base = action("perm_action", "x x[0]");
    for bad in [
        CLAIM.replace(r#""budget":"all""#, r#""budget":["0","0","0","0"]"#),
        CLAIM.replace(r#""witness":{}"#, r#""witness":{},"subject":{"closure_hash":"INVALID","variant":"adj","inherited_controls":0}"#),
        CLAIM.replace(r#""witness":{}"#, r#""witness":{},"domain":{"bundles":[],"borrowed":[],"regions":[]}"#),
    ] {
        let (text, root) = attach(&base, vec![support::json(&bad)]);
        assert!(verify(&text, &root, 0).is_err(), "invalid binding/field admitted: {bad}");
    }
    let (text, root) = attach(&base, vec![support::json(CLAIM)]);
    let mut o = Object::parse(&text).unwrap();
    o.functions.get_mut("a").unwrap().bounds.push(1);
    assert!(
        o.verify_claim(&root, 0).is_err(),
        "wrong source bounds established"
    );
}
