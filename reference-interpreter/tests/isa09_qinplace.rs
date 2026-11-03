mod support;
use bennett_isa_ref::Object;
fn published_source() -> &'static str {
    include_str!("../../spec/Bennett-ISA-Spec-0.9.md")
        .split_once("**Example 7**")
        .unwrap()
        .1
        .split_once("```\n")
        .unwrap()
        .1
        .split_once("\n```")
        .unwrap()
        .0
}
#[test]
fn published_parameterised_qinplace_source_is_admitted_without_edits() {
    let object = Object::parse(&(published_source().to_string() + "\n")).unwrap();
    object.check_hashes().unwrap();
    assert_eq!(
        object.functions["addya"]
            .instructions()
            .next()
            .unwrap()
            .operands
            .len(),
        7
    );
}
#[test]
fn parameterised_qinplace_checks_both_signatures_and_full_operand_lists() {
    for (from, to) in [
        ("tk x (y) () (a)", "tk x (x) () (a)"),
        ("tk x (y) () (a)", "tk x () () (a)"),
        ("tk x (y) () (a)", "tk x (y) () ()"),
        ("tk x (y) () (a)", "a x (y) () (a)"),
        ("tk x (y) () (a)", "tk x"),
        (
            "oracle @ginv (x:wire[2] in, y:wire[2] in)",
            "oracle @ginv (x:wire[2] in, y:wire in)",
        ),
    ] {
        let bad = published_source().replace(from, to) + "\n";
        assert!(
            Object::parse(&bad).is_err(),
            "admitted mutation {from} -> {to}"
        );
    }
}

fn fields(
    j: &mut bennett_isa_ref::json::Json,
) -> &mut std::collections::BTreeMap<String, bennett_isa_ref::json::Json> {
    let bennett_isa_ref::json::Json::Object(m) = j else {
        panic!("object")
    };
    m
}
fn copy_history(change_input: bool) -> String {
    let object = Object::parse(&(published_source().to_string() + "\n")).unwrap();
    let mut symbols = object.section("symbols").unwrap().clone();
    let signature = fields(fields(&mut symbols).get_mut("2").unwrap())
        .get_mut("signature")
        .unwrap();
    fields(signature).insert("mut".into(), support::json("[2,2]"));
    fields(signature).insert("in".into(), support::json("[]"));
    let mut lines = object.text.lines().map(str::to_string).collect::<Vec<_>>();
    let index = lines.iter().position(|s| s == "section symbols").unwrap() + 1;
    lines[index] = symbols.canonical();
    (lines.join("\n")+"\n")
        .replace("(x:wire[2]) (y:wire[2] in)","(x:wire[2] mut, y:wire[2] mut) ()")
        .replace("action_call @addya (x) (y)","action_call @addya (x, y) ()")
        .replace("qinplace @g @ginv tk x (y) () (a)",&format!("d = qcopy x\nqinplace @g @ginv tk d (y) () (a)\n{}qinplace @ginv @g tk d (y) () (a)\nquncopy x d",if change_input {"x y[0]\n"}else{""}))
}
#[test]
fn qinplace_transfer_keeps_the_read_input_versions_in_its_inverse_key() {
    Object::parse(&copy_history(false)).unwrap();
    let error = Object::parse(&copy_history(true)).unwrap_err();
    assert!(error.to_string().contains("§6.2"), "{error}");
}
fn inverse_certificate() -> String {
    use bennett_isa_ref::json::Json;
    let object = Object::parse(&(published_source().to_string() + "\n")).unwrap();
    let identity = object.identity().unwrap();
    let owner = identity.closure_hashes["addya"].as_ref().unwrap();
    let g = identity.closure_hashes["g"].as_ref().unwrap();
    let gi = identity.closure_hashes["ginv"].as_ref().unwrap();
    let base = include_str!("../../conformance/fixtures/accepted-k84-09/baseline.bisa");
    let mut certificate =
        Json::parse(base.split_once("section certificate\n").unwrap().1.trim()).unwrap();
    let bindings = fields(fields(&mut certificate).get_mut("bindings").unwrap());
    for (key, value) in [
        ("object_hash", &identity.object_hash),
        ("interface_hash", &identity.interface_hashes["addya"]),
        ("closure_hash", owner),
        ("reference_program_hash", owner),
    ] {
        bindings.insert(key.into(), Json::String(value.clone()));
    }
    bindings.remove("invocation");
    let regions = fields(fields(&mut certificate).get_mut("regions").unwrap());
    let mut record = regions.values().next().unwrap().clone();
    fields(&mut record).insert(
        "signature".into(),
        object.functions["addya"].signature_for(object.isa_version),
    );
    fields(&mut record).insert("frame".into(),support::json(r#"{"slots":[{"id":"mut:0","role":"mut","class":"any","width":2},{"id":"in:0","role":"in","class":"any","width":2}],"scratch":[],"history":[],"borrow":[],"ancillae":[]}"#));
    fields(&mut record).insert("resources".into(), support::json("{}"));
    fields(&mut record).insert("gates_used".into(), support::json("{}"));
    let claim = support::json(
        r#"{"kind":"exhaustive","expected":"inverse_pair","domain":{"args":[{"name":"a","width":2,"lo":0,"hi":3}],"borrowed":[],"bundles":[{"name":"x","width":2,"lo":0,"hi":3},{"name":"y","width":2,"lo":0,"hi":3}],"regions":[]}}"#,
    );
    fields(&mut record).insert("claims".into(), Json::Array(vec![claim]));
    let verification = fields(fields(&mut record).get_mut("verification").unwrap());
    verification.insert("unitary".into(), support::json("[]"));
    verification.insert(
        "inverse_pairs".into(),
        support::json(&format!(
            r#"[{{"g":"{g}","ginv":"{gi}","claim":0,"budget":[3,0,0,0]}}]"#
        )),
    );
    regions.clear();
    regions.insert(owner.clone(), record);
    fields(fields(&mut certificate).get_mut("composition").unwrap())
        .insert("multiplicities".into(), support::json("{}"));
    format!(
        "{}\nsection certificate\n{}\n",
        object.text,
        certificate.canonical()
    )
}
#[test]
fn k4_inverse_pair_claim_binds_its_entry_and_quantified_arguments() {
    let object = Object::parse(&inverse_certificate()).unwrap();
    let closure = object.identity().unwrap().closure_hashes["addya"]
        .clone()
        .unwrap();
    assert_eq!(
        object.verify_claim(&closure, 0).unwrap(),
        bennett_isa_ref::ClaimVerdict::Proof
    );
}

#[test]
fn inverse_pair_proof_checks_padded_success_and_rejects_a_faulting_minimum() {
    for steps in [2, 3, 9] {
        let text = inverse_certificate().replace(
            "\"budget\":[3,0,0,0]",
            &format!("\"budget\":[{steps},0,0,0]"),
        );
        let object = Object::parse(&text).unwrap();
        let owner = object.identity().unwrap().closure_hashes["addya"]
            .clone()
            .unwrap();
        let result = object.verify_claim(&owner, 0);
        if steps == 2 {
            assert!(result
                .unwrap_err()
                .to_string()
                .contains("inverse-pair oracle faults"));
        } else {
            assert_eq!(
                result.unwrap(),
                bennett_isa_ref::ClaimVerdict::Proof,
                "B={steps}"
            );
        }
    }
}
#[test]
fn inverse_pair_quantum_domains_must_be_full_but_argument_ranges_are_scoped() {
    for (from, to) in [
        (
            "\"hi\":3,\"lo\":0,\"name\":\"x\"",
            "\"hi\":2,\"lo\":0,\"name\":\"x\"",
        ),
        (
            "\"hi\":3,\"lo\":0,\"name\":\"a\",\"width\":2",
            "\"hi\":3,\"lo\":0,\"name\":\"a\",\"width\":3",
        ),
    ] {
        let original = inverse_certificate();
        assert!(original.contains(from));
        let result = Object::parse(&original.replace(from, to));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("K4"));
    }
    let text = inverse_certificate().replace(
        "\"hi\":3,\"lo\":0,\"name\":\"a\"",
        "\"hi\":2,\"lo\":0,\"name\":\"a\"",
    );
    let object = Object::parse(&text).unwrap();
    let owner = object.identity().unwrap().closure_hashes["addya"]
        .clone()
        .unwrap();
    assert_eq!(
        object.verify_claim(&owner, 0).unwrap(),
        bennett_isa_ref::ClaimVerdict::Proof
    );
}
