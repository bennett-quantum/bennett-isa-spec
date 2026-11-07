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
    let object = Object::parse(&(published_source().to_string() + "\n")).unwrap();
    inverse_certificate_for(&object)
}
fn inverse_certificate_for(object: &Object) -> String {
    use bennett_isa_ref::json::Json;
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
    fields(&mut record).insert(
        "bounds".into(),
        Json::Array(
            object.functions["addya"]
                .bounds
                .iter()
                .map(|n| Json::Number(n.to_string()))
                .collect(),
        ),
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

    let reports = object.verify_certificate().unwrap().claims;
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].verdict, bennett_isa_ref::ClaimVerdict::Proof);
    assert!(reports[0].coverage.is_none());
}

#[test]
fn inverse_pair_certificate_rejects_a_noninverse_and_an_unbound_claim() {
    let object = Object::parse(&inverse_certificate()).unwrap();
    let identity = object.identity().unwrap();
    let gi = identity.closure_hashes["ginv"].as_ref().unwrap();
    let g = identity.closure_hashes["g"].as_ref().unwrap();
    let original = inverse_certificate();
    let wrong = original.replace(&format!(r#""ginv":"{gi}""#), &format!(r#""ginv":"{g}""#));
    assert_ne!(wrong, original);
    let wrong = Object::parse(&wrong).unwrap();
    let owner = identity.closure_hashes["addya"].as_ref().unwrap();
    let error = wrong.verify_claim(owner, 0).unwrap_err();
    assert!(error.to_string().contains("differs from x"), "{error}");
    let error = wrong.verify_certificate().unwrap_err();
    assert!(error.to_string().contains("missing owner-local"), "{error}");
    let no_entry = original.replace(
        &format!(r#"[{{"budget":[3,0,0,0],"claim":0,"g":"{g}","ginv":"{gi}"}}]"#),
        "[]",
    );
    assert_ne!(no_entry, original);
    let error = Object::parse(&no_entry).unwrap_err();
    assert!(
        error.to_string().contains("owner-local entry binding"),
        "{error}"
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

fn quantum_input(
    x: u128,
    y: u128,
) -> (
    bennett_isa_ref::QuantumState,
    bennett_isa_ref::QuantumBindings,
    Vec<u64>,
) {
    use bennett_isa_ref::{QuantumBindings, QuantumState};
    let mut q = QuantumState::new();
    let wires = q.allocate(4).unwrap();
    for (i, w) in wires.iter().enumerate() {
        if (x | (y << 2)) >> i & 1 != 0 {
            q.gate("x", &[*w], None, &[]).unwrap();
        }
    }
    let mut b = QuantumBindings::default();
    b.bundles.insert("x".into(), wires[..2].to_vec());
    b.bundles.insert("y".into(), wires[2..].to_vec());
    (q, b, wires)
}
#[test]
fn declared_parameterised_pair_executes_every_x_y_a_and_its_adjoint() {
    use bennett_isa_ref::{Argument, Budget};
    let object = Object::parse(&inverse_certificate()).unwrap();
    for steps in [3, 9] {
        for x in 0..4 {
            for y in 0..4 {
                for a in 0..4 {
                    let (mut q, mut b, wires) = quantum_input(x, y);
                    b.args
                        .insert("tk".into(), Argument::Token(Budget::steps(steps)));
                    b.args.insert("a".into(), Argument::Bits(a));
                    let before = q.clone();
                    object
                        .execute_action_with_token_aliases("addya", &mut q, &b, false, &[], &[0])
                        .unwrap();
                    assert_eq!(q.read_basis(&wires).unwrap(), ((x + y + a) & 3) | (y << 2));
                    object
                        .execute_action_with_token_aliases("addya", &mut q, &b, true, &[], &[0])
                        .unwrap();
                    assert_eq!(q, before);
                }
            }
        }
    }
}
#[test]
fn uncovered_pair_rejects_before_the_earliest_effect_of_the_region() {
    use bennett_isa_ref::{Argument, Budget};
    let text = published_source().replace(
        "qinplace @g @ginv tk x (y) () (a)",
        "x x[0]\nqinplace @g @ginv tk x (y) () (a)",
    ) + "\n";
    let source = support::rehash(&text);
    let packet = inverse_certificate_for(&source).replace(
        "\"hi\":3,\"lo\":0,\"name\":\"a\"",
        "\"hi\":2,\"lo\":0,\"name\":\"a\"",
    );
    for object in [source, Object::parse(&packet).unwrap()] {
        for (steps, a) in [(2, 1), (3, 3)] {
            let (mut q, mut b, _) = quantum_input(2, 1);
            b.args
                .insert("tk".into(), Argument::Token(Budget::steps(steps)));
            b.args.insert("a".into(), Argument::Bits(a));
            let before = q.clone();
            let result =
                object.execute_action_with_token_aliases("addya", &mut q, &b, false, &[], &[0]);
            assert!(result.is_err());
            assert_eq!(q, before, "uncovered tk={steps}/a={a} changed a prefix");
        }
    }
}

#[test]
fn source_qinplace_adjoints_preserve_instruction_allocation_and_release_order() {
    use bennett_isa_conformance::{json, ref_catalog::Machine, ref_language::permutation};
    let object = Object::parse(&inverse_certificate()).unwrap();
    let root = object.identity().unwrap().closure_hashes["addya"]
        .clone()
        .unwrap();
    let args = json::parse(r#"["x","y",["3","0","0","0"],"1"]"#).unwrap();
    for adjoint in [false, true] {
        let leaves = object
            .expand_action_source_with_token_aliases(
                "addya",
                args.as_arr().unwrap(),
                &[("x", 2), ("y", 2)],
                adjoint,
                &[0],
            )
            .unwrap();
        let own = leaves
            .iter()
            .filter(|l| l.source.as_ref().is_some_and(|s| s.terminal.symbol == root))
            .collect::<Vec<_>>();
        let allocation = own
            .iter()
            .filter(|l| l.gate == "alloc")
            .map(|l| l.allocation.as_ref().unwrap().1)
            .collect::<Vec<_>>();
        let release = own
            .iter()
            .filter(|l| l.gate == "release")
            .map(|l| l.allocation.as_ref().unwrap().1)
            .collect::<Vec<_>>();
        assert_eq!(allocation, [2, 4], "adjoint={adjoint}");
        assert_eq!(release, [2, 4], "adjoint={adjoint}");
        let machine =
            Machine::compile(&permutation(&leaves).unwrap(), &[("x", 2), ("y", 2)]).unwrap();
        for x in 0..4 {
            for y in 0..4 {
                let mut state = machine.state();
                machine.set(&mut state, "x", x);
                machine.set(&mut state, "y", y);
                let before = state.clone();
                machine.run(&mut state, true, false).unwrap();
                assert_eq!(
                    machine.get(&state, "x"),
                    if adjoint {
                        (x + 8 - y - 1) & 3
                    } else {
                        (x + y + 1) & 3
                    }
                );
                assert_eq!(machine.get(&state, "y"), y);
                machine.run(&mut state, true, true).unwrap();
                assert_eq!(state, before);
            }
        }
    }
}

fn change_certificate(text: &str, change: impl FnOnce(&mut bennett_isa_ref::json::Json)) -> String {
    let (source, packet) = text.split_once("section certificate\n").unwrap();
    let mut certificate = bennett_isa_ref::json::Json::parse(packet.trim()).unwrap();
    change(&mut certificate);
    format!("{source}section certificate\n{}\n", certificate.canonical())
}
fn root_invocation(steps: u64, a: u64) -> bennett_isa_ref::json::Json {
    support::json(&format!(
        r#"{{"args":[["{steps}","0","0","0"],"{a}"],"budget":["0","0","0","0"],"token_aliases":[0]}}"#
    ))
}
fn linked_inverse_certificate(wrong_pair: bool) -> String {
    let text = inverse_certificate();
    let object = Object::parse(&text).unwrap();
    let identity = object.identity().unwrap();
    let owner = identity.closure_hashes["addya"].as_ref().unwrap();
    let g = identity.closure_hashes["g"].as_ref().unwrap();
    change_certificate(&text, |certificate| {
        let regions = fields(fields(certificate).get_mut("regions").unwrap());
        let mut donor = regions[owner].clone();
        fields(&mut donor).insert("kind".into(), support::json("\"oracle\""));
        fields(&mut donor).insert("bounds".into(), support::json("[]"));
        fields(&mut donor).insert(
            "signature".into(),
            object.functions["g"].signature_for(object.isa_version),
        );
        fields(&mut donor).insert("frame".into(), support::json(r#"{"slots":[{"id":"in:0","role":"in","class":"any","width":2},{"id":"in:1","role":"in","class":"any","width":2},{"id":"tgt:0","role":"tgt","class":"any","width":2},{"id":"fault:0","role":"fault","class":"any","width":4}],"scratch":[],"history":[],"borrow":[],"ancillae":[]}"#));
        let record = fields(regions.get_mut(owner).unwrap());
        record.insert("claims".into(), support::json(&format!(r#"[{{"kind":"structural","rule":"inverse_pair_library","subclaims":[],"witness":{{"library_closure_hash":"{g}","claim_index":0}},"invocation":{}}}]"#, root_invocation(3,1).canonical())));
        if wrong_pair {
            let entries = fields(record.get_mut("verification").unwrap())
                .get_mut("inverse_pairs")
                .unwrap();
            let bennett_isa_ref::json::Json::Array(entries) = entries else {
                panic!("entries")
            };
            fields(&mut entries[0]).insert(
                "ginv".into(),
                bennett_isa_ref::json::Json::String(g.clone()),
            );
        }
        regions.insert(g.clone(), donor);
    })
}
#[test]
fn checker_rejects_missing_owner_local_qinplace_inventory() {
    let text = change_certificate(&inverse_certificate(), |c| {
        let record = fields(fields(c).get_mut("regions").unwrap())
            .values_mut()
            .next()
            .unwrap();
        fields(record).insert("claims".into(), support::json("[]"));
        fields(fields(record).get_mut("verification").unwrap())
            .insert("inverse_pairs".into(), support::json("[]"));
    });
    let error = Object::parse(&text)
        .unwrap()
        .verify_certificate()
        .unwrap_err();
    assert!(error.to_string().contains("K92"), "{error}");
}
#[test]
fn checker_rejects_bound_invocation_below_the_pair_minimum() {
    let text = change_certificate(&inverse_certificate(), |c| {
        fields(fields(c).get_mut("bindings").unwrap())
            .insert("invocation".into(), root_invocation(2, 1));
    });
    let error = Object::parse(&text)
        .unwrap()
        .verify_certificate()
        .unwrap_err();
    assert!(
        error.to_string().contains("no covering inverse-pair entry"),
        "{error}"
    );
}
#[test]
fn standalone_pair_proof_reports_unestablished_applicability() {
    let reports = Object::parse(&inverse_certificate())
        .unwrap()
        .verify_certificate()
        .unwrap()
        .claims;
    assert_eq!(reports[0].verdict, bennett_isa_ref::ClaimVerdict::Proof);
    assert!(
        matches!(
            reports[0].qinplace_applicability,
            Some(bennett_isa_ref::QinplaceApplicability::Unestablished(_))
        ),
        "{reports:?}"
    );
}
#[test]
fn library_proof_cannot_transfer_to_a_different_pair() {
    let object = Object::parse(&linked_inverse_certificate(true)).unwrap();
    let owner = object.identity().unwrap().closure_hashes["addya"]
        .clone()
        .unwrap();
    // g(x;y,a)=x+y+a mod 4: g(g(0;0,1);0,1)=2, so g/g is not inverse.
    let error = object.verify_claim(&owner, 0).unwrap_err();
    assert!(error.to_string().contains("same resolved pair"), "{error}");
    let error = object.verify_certificate().unwrap_err();
    assert!(error.to_string().contains("K92"), "{error}");
}
#[test]
fn linked_library_pair_executes_every_argument_and_adjoint() {
    use bennett_isa_ref::{Argument, Budget, ClaimVerdict};
    let object = Object::parse(&linked_inverse_certificate(false)).unwrap();
    let owner = object.identity().unwrap().closure_hashes["addya"]
        .clone()
        .unwrap();
    assert_eq!(object.verify_claim(&owner, 0).unwrap(), ClaimVerdict::Proof);
    for steps in [3, 9] {
        for x in 0..4 {
            for y in 0..4 {
                for a in 0..4 {
                    let (mut q, mut b, wires) = quantum_input(x, y);
                    b.args
                        .insert("tk".into(), Argument::Token(Budget::steps(steps)));
                    b.args.insert("a".into(), Argument::Bits(a));
                    let before = q.clone();
                    object
                        .execute_action_with_token_aliases("addya", &mut q, &b, false, &[], &[0])
                        .unwrap();
                    assert_eq!(q.read_basis(&wires).unwrap(), ((x + y + a) & 3) | (y << 2));
                    object
                        .execute_action_with_token_aliases("addya", &mut q, &b, true, &[], &[0])
                        .unwrap();
                    assert_eq!(q, before);
                }
            }
        }
    }
}

fn loop_certificate(iterations: usize) -> String {
    let text = published_source()
        .replace("\"bounds\":[]", &format!("\"bounds\":[{iterations}]"))
        .replace(
            "(tk:token, a:i2) ()\n^entry():",
            &format!("(tk:token, a:i2) ()\nregs {{  }} bounds {{{iterations}}}\n^entry():"),
        )
        .replace(
            "qinplace @g @ginv tk x (y) () (a)",
            &format!("qloop {iterations} {{\nqinplace @g @ginv tk x (y) () (a)\n}}"),
        )
        + "\n";
    inverse_certificate_for(&support::rehash(&text))
}
#[test]
fn zero_trip_qinplace_requires_inventory_but_has_no_applicability_visit() {
    let text = loop_certificate(0);
    let bound = change_certificate(&text, |c| {
        fields(fields(c).get_mut("bindings").unwrap())
            .insert("invocation".into(), root_invocation(2, 1));
    });
    let reports = Object::parse(&bound)
        .unwrap()
        .verify_certificate()
        .unwrap()
        .claims;
    assert_eq!(
        reports[0].qinplace_applicability,
        Some(bennett_isa_ref::QinplaceApplicability::Established)
    );
    let missing = change_certificate(&bound, |c| {
        let record = fields(fields(c).get_mut("regions").unwrap())
            .values_mut()
            .next()
            .unwrap();
        fields(record).insert("claims".into(), support::json("[]"));
        fields(fields(record).get_mut("verification").unwrap())
            .insert("inverse_pairs".into(), support::json("[]"));
    });
    let error = Object::parse(&missing)
        .unwrap()
        .verify_certificate()
        .unwrap_err();
    assert!(error.to_string().contains("K92"), "{error}");
}
#[test]
fn nested_qinplace_below_minimum_rejects_in_checker_and_before_execution() {
    use bennett_isa_ref::{Argument, Budget};
    let text = change_certificate(&loop_certificate(1), |c| {
        fields(fields(c).get_mut("bindings").unwrap())
            .insert("invocation".into(), root_invocation(2, 1));
    });
    let object = Object::parse(&text).unwrap();
    let error = object.verify_certificate().unwrap_err();
    assert!(
        error.to_string().contains("no covering inverse-pair entry"),
        "{error}"
    );
    let (mut q, mut b, _) = quantum_input(0, 0);
    b.args
        .insert("tk".into(), Argument::Token(Budget::steps(2)));
    b.args.insert("a".into(), Argument::Bits(1));
    let before = q.clone();
    let error = object
        .execute_action_with_token_aliases("addya", &mut q, &b, false, &[], &[0])
        .unwrap_err();
    assert!(
        error.to_string().contains("no covering inverse-pair entry"),
        "{error}"
    );
    assert_eq!(q, before);
}
#[test]
fn library_reuse_preserves_the_proved_minimum_and_argument_domain() {
    let text = linked_inverse_certificate(false);
    let object = Object::parse(&text).unwrap();
    let owner = object.identity().unwrap().closure_hashes["addya"]
        .clone()
        .unwrap();
    let too_small = change_certificate(&text, |c| {
        let record = fields(fields(c).get_mut("regions").unwrap())
            .get_mut(&owner)
            .unwrap();
        let entries = fields(fields(record).get_mut("verification").unwrap())
            .get_mut("inverse_pairs")
            .unwrap();
        let bennett_isa_ref::json::Json::Array(entries) = entries else {
            panic!("entries")
        };
        fields(&mut entries[0]).insert("budget".into(), support::json("[2,0,0,0]"));
    });
    let error = Object::parse(&too_small)
        .unwrap()
        .verify_claim(&owner, 0)
        .unwrap_err();
    assert!(error.to_string().contains("token context"), "{error}");
    let scoped = text.replace(
        "\"hi\":3,\"lo\":0,\"name\":\"a\"",
        "\"hi\":2,\"lo\":0,\"name\":\"a\"",
    );
    assert_ne!(scoped, text);
    let uncovered = change_certificate(&scoped, |c| {
        fields(fields(c).get_mut("bindings").unwrap())
            .insert("invocation".into(), root_invocation(3, 3));
    });
    let error = Object::parse(&uncovered)
        .unwrap()
        .verify_certificate()
        .unwrap_err();
    assert!(
        error.to_string().contains("no covering inverse-pair entry"),
        "{error}"
    );
}

fn transitive_certificate(include_inner: bool) -> String {
    use bennett_isa_ref::json::Json;
    let original = Object::parse(&(published_source().to_string() + "\n")).unwrap();
    let mut symbols = original.section("symbols").unwrap().clone();
    let mut inner = fields(&mut symbols)["2"].clone();
    fields(&mut inner).insert("name".into(), Json::String("inner".into()));
    fields(&mut symbols).insert("4".into(), inner);
    let mut lines = original
        .text
        .lines()
        .map(str::to_string)
        .collect::<Vec<_>>();
    let i = lines.iter().position(|s| s == "section symbols").unwrap() + 1;
    lines[i] = symbols.canonical();
    let source = (lines.join("\n")+"\n").replace("qinplace @g @ginv tk x (y) () (a)", "action_call @inner (x) (y) () (tk, a)")
        + "perm_action @inner (x:wire[2]) (y:wire[2] in) () (tk:token, a:i2) ()\n^entry():\nqinplace @g @ginv tk x (y) () (a)\n";
    let object = support::rehash(&source);
    let text = inverse_certificate_for(&object);
    change_certificate(&text, |c| {
        if include_inner {
            let hash = object.identity().unwrap().closure_hashes["inner"]
                .clone()
                .unwrap();
            let regions = fields(fields(c).get_mut("regions").unwrap());
            let record = regions.values().next().unwrap().clone();
            regions.insert(hash, record);
        }
    })
}
#[test]
fn bound_reference_tree_checks_transitive_owner_and_actual_arguments() {
    for (include_inner, steps) in [(false, 3), (true, 2), (true, 3), (true, 9)] {
        let text = change_certificate(&transitive_certificate(include_inner), |c| {
            fields(fields(c).get_mut("bindings").unwrap())
                .insert("invocation".into(), root_invocation(steps, 1));
        });
        let result = Object::parse(&text).unwrap().verify_certificate();
        if include_inner && steps >= 3 {
            let reports = result.unwrap().claims;
            let inner = reports.iter().find(|r| r.region == "inner").unwrap();
            assert_eq!(
                inner.qinplace_applicability,
                Some(bennett_isa_ref::QinplaceApplicability::Established)
            );
        } else {
            let error = result.unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("no covering inverse-pair entry in @inner"),
                "{error}"
            );
        }
    }
}
#[test]
fn disabled_coherent_predicate_keeps_the_qinplace_obligation() {
    let source = published_source().replace("qinplace @g @ginv tk x (y) () (a)", "c = qalloc any 1\nqif c {\nqinplace @g @ginv tk x (y) () (a)\n} {\nx x[0]\nx x[0]\n}\nqfree c") + "\n";
    let text = change_certificate(&inverse_certificate_for(&support::rehash(&source)), |c| {
        fields(fields(c).get_mut("bindings").unwrap())
            .insert("invocation".into(), root_invocation(2, 1));
    });
    let error = Object::parse(&text)
        .unwrap()
        .verify_certificate()
        .unwrap_err();
    assert!(
        error.to_string().contains("no covering inverse-pair entry"),
        "{error}"
    );
}

#[test]
fn missing_library_body_reports_unestablished_and_cannot_execute() {
    use bennett_isa_ref::{Argument, Budget, ClaimVerdict, QinplaceApplicability};
    let text = linked_inverse_certificate(false);
    let object = Object::parse(&text).unwrap();
    let g = object.identity().unwrap().closure_hashes["g"]
        .clone()
        .unwrap();
    let text = change_certificate(&text, |c| {
        fields(fields(c).get_mut("regions").unwrap()).remove(&g);
    });
    let object = Object::parse(&text).unwrap();
    let report = object.verify_certificate().unwrap().claims.remove(0);
    assert!(
        matches!(report.verdict, ClaimVerdict::Unverified(_)),
        "{report:?}"
    );
    assert!(
        matches!(
            report.qinplace_applicability,
            Some(QinplaceApplicability::Unestablished(_))
        ),
        "{report:?}"
    );
    let (mut q, mut b, _) = quantum_input(0, 0);
    b.args
        .insert("tk".into(), Argument::Token(Budget::steps(3)));
    b.args.insert("a".into(), Argument::Bits(1));
    let before = q.clone();
    let error = object
        .execute_action_with_token_aliases("addya", &mut q, &b, false, &[], &[0])
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("unestablished qinplace applicability"),
        "{error}"
    );
    assert_eq!(q, before);
}

enum CertificateContents {
    PairClaim,
    EmptyClaims,
    EmptyRecords,
}

fn unsupported_bound_root_certificate(contents: CertificateContents) -> String {
    use bennett_isa_ref::json::Json;
    let text = transitive_certificate(false);
    let identity = Object::parse(&text).unwrap().identity().unwrap();
    change_certificate(&text, |c| {
        let bindings = fields(fields(c).get_mut("bindings").unwrap());
        for (key, value) in [
            (
                "closure_hash",
                identity.closure_hashes["run"].as_ref().unwrap(),
            ),
            (
                "reference_program_hash",
                identity.closure_hashes["run"].as_ref().unwrap(),
            ),
            ("interface_hash", &identity.interface_hashes["run"]),
        ] {
            bindings.insert(key.into(), Json::String(value.clone()));
        }
        bindings.insert("invocation".into(), root_invocation(2, 1));
        let regions = fields(fields(c).get_mut("regions").unwrap());
        match contents {
            CertificateContents::PairClaim => {}
            CertificateContents::EmptyRecords => regions.clear(),
            CertificateContents::EmptyClaims => {
                for record in regions.values_mut() {
                    fields(record).insert("claims".into(), support::json("[]"));
                    fields(fields(record).get_mut("verification").unwrap())
                        .insert("inverse_pairs".into(), support::json("[]"));
                }
            }
        }
    })
}

#[test]
fn unsupported_bound_root_keeps_applicability_without_direct_owner() {
    use bennett_isa_ref::{ClaimVerdict, QinplaceApplicability};
    let object = Object::parse(&unsupported_bound_root_certificate(
        CertificateContents::PairClaim,
    ))
    .unwrap();
    let identity = object.identity().unwrap();
    let owner = identity.closure_hashes["addya"].as_ref().unwrap();
    let root = identity.closure_hashes["run"].as_ref().unwrap();
    let report = object.verify_certificate().unwrap();
    assert_eq!(report.claims.len(), 1);
    assert_eq!(report.claims[0].verdict, ClaimVerdict::Proof);
    assert_eq!(object.verify_claim(owner, 0).unwrap(), ClaimVerdict::Proof);
    assert_eq!(report.qinplace_applicability.len(), 1);
    assert!(
        matches!(report.qinplace_applicability.get(root), Some(QinplaceApplicability::Unestablished(reason)) if reason.contains("bound @run")),
        "{report:?}"
    );
}

#[test]
fn unsupported_bound_root_keeps_applicability_without_claims() {
    let object = Object::parse(&unsupported_bound_root_certificate(
        CertificateContents::EmptyClaims,
    ))
    .unwrap();
    let root = object.identity().unwrap().closure_hashes["run"]
        .clone()
        .unwrap();
    let report = object.verify_certificate().unwrap();
    assert!(report.claims.is_empty(), "{report:?}");
    assert!(
        matches!(report.qinplace_applicability.get(&root), Some(bennett_isa_ref::QinplaceApplicability::Unestablished(reason)) if reason.contains("bound @run")),
        "{report:?}"
    );
}

#[test]
fn unsupported_bound_root_keeps_applicability_without_records() {
    let object = Object::parse(&unsupported_bound_root_certificate(
        CertificateContents::EmptyRecords,
    ))
    .unwrap();
    let root = object.identity().unwrap().closure_hashes["run"]
        .clone()
        .unwrap();
    let report = object.verify_certificate().unwrap();
    assert!(report.claims.is_empty(), "{report:?}");
    assert!(
        matches!(report.qinplace_applicability.get(&root), Some(bennett_isa_ref::QinplaceApplicability::Unestablished(reason)) if reason.contains("bound @run")),
        "{report:?}"
    );
}

#[test]
fn uncheckable_uniform_transitive_claim_keeps_applicability() {
    // The outer caller has no local site; universal transitive coverage is
    // separate from the structural theorem.
    let source = transitive_certificate(false);
    let owner = Object::parse(&source)
        .unwrap()
        .identity()
        .unwrap()
        .closure_hashes["addya"]
        .clone()
        .unwrap();
    let text = change_certificate(&source, |c| {
        let record = fields(fields(c).get_mut("regions").unwrap())
            .get_mut(&owner)
            .unwrap();
        fields(record).insert("claims".into(), support::json(r#"[{"kind":"structural","rule":"adjoint_table","subclaims":[],"witness":{},"invocation":{"args":"all","budget":"all","token_aliases":"all"}}]"#));
        fields(fields(record).get_mut("verification").unwrap())
            .insert("inverse_pairs".into(), support::json("[]"));
    });
    let report = Object::parse(&text).unwrap().verify_certificate().unwrap();
    assert_eq!(report.claims.len(), 1);
    assert!(
        matches!(report.qinplace_applicability.get(&owner), Some(bennett_isa_ref::QinplaceApplicability::Unestablished(reason)) if reason.contains("parameter-uniform")),
        "{report:?}"
    );
}

#[test]
fn scheduled_root_keeps_unchecked_tree_obligation_without_owners_or_claims() {
    use bennett_isa_ref::{json::Json, sha256_hex, QinplaceApplicability};
    let text = transitive_certificate(false);
    let original = Object::parse(&text).unwrap();
    let mut symbols = original.section("symbols").unwrap().clone();
    fields(
        fields(fields(&mut symbols).get_mut("2").unwrap())
            .get_mut("signature")
            .unwrap(),
    )
    .insert("bounds".into(), support::json("[0]"));
    let mut lines = text.lines().map(str::to_string).collect::<Vec<_>>();
    let index = lines
        .iter()
        .position(|line| line == "section symbols")
        .unwrap()
        + 1;
    lines[index] = symbols.canonical();
    let text = lines.join("\n");
    let source = text.split_once("section certificate\n").unwrap().0.replace(
        "action_call @inner (x) (y) () (tk, a)",
        "qloop 0 {\naction_call @inner (x) (y) () (tk, a)\n}",
    ).replace(
        "perm_action @addya (x:wire[2]) (y:wire[2] in) () (tk:token, a:i2) ()\n^entry():",
        "perm_action @addya (x:wire[2]) (y:wire[2] in) () (tk:token, a:i2) ()\nregs {  } bounds {0}\n^entry():",
    );
    let object = support::rehash(&(source.trim_end().to_string() + "\n"));
    let owner = object.identity().unwrap().closure_hashes["addya"]
        .clone()
        .unwrap();
    let invocation = root_invocation(3, 1);
    let params = sha256_hex(invocation.get("args").unwrap().canonical().as_bytes());
    let schedule = support::json(&format!(
        r#"{{"bindings":{{"budget":[0,0,0,0],"closure_hash":"{owner}","epsilon":0,"inherited_controls":0,"params_hash":"{params}","region":"addya","scheduler":{{"name":"test","version":"1"}},"target_hash":"{}","token_aliases":[0],"variant":"forward"}},"entries":[]}}"#,
        "0".repeat(64)
    ));
    let text = change_certificate(&inverse_certificate_for(&object), |c| {
        fields(fields(c).get_mut("regions").unwrap()).clear();
        let bindings = fields(fields(c).get_mut("bindings").unwrap());
        bindings.insert("invocation".into(), invocation);
        bindings.insert("target_hash".into(), Json::String("0".repeat(64)));
        bindings.insert(
            "schedule_hash".into(),
            Json::String(sha256_hex(schedule.canonical().as_bytes())),
        );
    });
    let (source, certificate) = text.split_once("section certificate\n").unwrap();
    let text = format!(
        "{}\n\nsection schedule\n{}\n\nsection certificate\n{certificate}",
        source.trim_end(),
        schedule.canonical()
    );
    let report = Object::parse(&text).unwrap().verify_certificate().unwrap();
    assert!(report.claims.is_empty());
    assert!(
        matches!(report.qinplace_applicability.get(&owner), Some(QinplaceApplicability::Unestablished(reason)) if reason.contains("realised invocation tree")),
        "{report:?}"
    );
}

#[test]
fn checked_concrete_context_does_not_discharge_uniform_applicability() {
    let text = linked_inverse_certificate(false);
    let object = Object::parse(&text).unwrap();
    let owner = object.identity().unwrap().closure_hashes["addya"]
        .clone()
        .unwrap();
    let text = change_certificate(&text, |c| {
        fields(fields(c).get_mut("bindings").unwrap())
            .insert("invocation".into(), root_invocation(3, 1));
        let record = fields(fields(c).get_mut("regions").unwrap())
            .get_mut(&owner)
            .unwrap();
        let bennett_isa_ref::json::Json::Array(claims) = fields(record).get_mut("claims").unwrap()
        else {
            panic!("claims")
        };
        fields(&mut claims[0]).insert(
            "invocation".into(),
            support::json(r#"{"args":"all","budget":"all","token_aliases":"all"}"#),
        );
    });
    let report = Object::parse(&text).unwrap().verify_certificate().unwrap();
    assert_eq!(
        report
            .claims
            .iter()
            .find(|c| c.closure_hash == owner)
            .unwrap()
            .verdict,
        bennett_isa_ref::ClaimVerdict::Proof
    );
    assert!(
        matches!(report.qinplace_applicability.get(&owner), Some(bennett_isa_ref::QinplaceApplicability::Unestablished(reason)) if reason.contains("parameter-uniform")),
        "{report:?}"
    );
}
