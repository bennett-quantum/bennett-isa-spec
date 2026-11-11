#[path = "support/native_operands.rs"]
mod native_operands;
mod support;
use bennett_isa_ref::{
    json::Json,
    target::{verify_native_model, ModelContext, ModelVerification, Rational},
    ClassicalMachine, ClassicalValue, Object, QuantumRegisters,
};
use std::collections::BTreeMap;
fn fields(v: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(m) = v else { panic!("object") };
    m
}
fn model() -> Json {
    let target = Json::parse(include_str!(
        "../../conformance/fixtures/accepted-k86-09/controlled-x.target.json"
    ))
    .unwrap();
    target.get("gates").unwrap().array().unwrap()[0]
        .get("realisation_model")
        .unwrap()
        .clone()
}
fn verify(
    m: &Json,
    own: usize,
    total: usize,
    eps: i128,
) -> bennett_isa_ref::Result<ModelVerification> {
    verify_native_model(
        Some(m),
        if own == 1 { "cx" } else { "x" },
        &ModelContext {
            operands: native_operands::operands(own, total, 1),
            own_controls: own,
            total_controls: total,
            requested_epsilon: Some(Rational::integer(3)),
            theta_bits: None,
        },
        &Rational::integer(eps),
        64,
    )
}
#[test]
fn legal_generated_map_is_not_rejected() {
    let mut m = model();
    fields(&mut m).insert(
        "operand_map".into(),
        support::json(r#"{"kind":"entry_to_allocation"}"#),
    );
    let r = verify(&m, 0, 1, 0);
    assert!(r.is_ok(), "{r:?}");
}
#[test]
fn legal_identity_matrix_constructor_is_not_rejected() {
    let mut m = model();
    fields(&mut m).insert(
        "data".into(),
        support::json(r#"{"matrix":{"kind":"identity","dimension":2}}"#),
    );
    let r = verify(&m, 0, 1, 2);
    assert!(r.is_ok(), "{r:?}");
}
#[test]
fn legal_size_expression_is_not_rejected() {
    let mut m = model();
    fields(&mut m).insert("arity".into(), support::json(r#"{"op":"+","args":[1,0]}"#));
    let r = verify(&m, 0, 1, 0);
    assert!(r.is_ok(), "{r:?}");
}
#[test]
fn legal_d_binding_is_not_rejected() {
    let mut m = model();
    fields(&mut m).insert("parameter_bindings".into(), support::json(r#"{"d":"d"}"#));
    let r = verify(&m, 0, 1, 0);
    assert!(r.is_ok(), "{r:?}");
}
#[test]
fn legal_optional_unitarity_certificate_is_not_rejected() {
    let mut m = model();
    let d = fields(&mut m).get_mut("data").unwrap();
    fields(d).insert("certificates".into(),support::json(r#"{"unitarity":{"version":"bennett.unitarity/1","gram":{"kind":"identity","dimension":2}}}"#));
    let r = verify(&m, 0, 1, 0);
    assert!(r.is_ok(), "{r:?}");
}
#[test]
fn legal_coinciding_own_and_inherited_control_is_not_rejected() {
    let mut m = model();
    fields(&mut m).insert("operand_map".into(), support::json("[0,1]"));
    let mut ctx = ModelContext {
        operands: native_operands::operands(1, 2, 1),
        own_controls: 1,
        total_controls: 2,
        requested_epsilon: Some(Rational::integer(3)),
        theta_bits: None,
    };
    ctx.operands.effective_controls = ctx.operands.own_controls.clone();
    ctx.operands.allocation_order = ctx
        .operands
        .own_controls
        .iter()
        .chain(&ctx.operands.targets)
        .cloned()
        .collect();
    let r = verify_native_model(Some(&m), "cx", &ctx, &Rational::integer(0), 64);
    assert!(matches!(r.unwrap(), ModelVerification::Checked(_)));
}
#[test]
fn canonical_control_order_can_place_inherited_before_own() {
    let mut m = model();
    fields(&mut m).insert("operand_map".into(), support::json("[0,1,2]"));
    let mut ctx = ModelContext {
        operands: native_operands::operands(1, 2, 1),
        own_controls: 1,
        total_controls: 2,
        requested_epsilon: Some(Rational::integer(3)),
        theta_bits: None,
    };
    ctx.operands.own_controls[0].0 = "z-own".into();
    ctx.operands.allocation_order = ctx
        .operands
        .effective_controls
        .iter()
        .chain(&ctx.operands.own_controls)
        .chain(&ctx.operands.targets)
        .cloned()
        .collect();
    let r = verify_native_model(Some(&m), "cx", &ctx, &Rational::integer(0), 64);
    assert!(matches!(r.unwrap(), ModelVerification::Checked(_)));
}
fn local_program() -> Object {
    local_program_class("A")
}
fn local_program_class(class: &str) -> Object {
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let raw=support::object(&[
 ("empty",sig,"perm_action @empty (b:wire mut) () () () ()\n^entry():\nqpred b\nqunpred b"),
 ("outer",sig,"perm_action @outer (t:wire mut) () () () ()\n^entry():\nb = qalloc A 1\naction_call @empty (b) () () ()\nadj action_call @empty (b) () () ()\nqfree b\nx t"),
 ("launch",r#"{"abi":"0.8","kind":"r","params":["qreg[1]"],"results":[],"tier":"r"}"#,"fn r @launch (a:qreg[1]) () () () ()\n^entry():\naction_call @outer (a) () () ()\nret")],"{}","").replace("=0.8","=0.9").replace("\"0.8\"","\"0.9\"");
    let raw = raw.replace("qalloc A 1", &format!("qalloc {class} 1"));
    let base = support::rehash(&raw);
    let template = Object::parse(include_str!(
        "../../conformance/fixtures/accepted-k90-09/pair.bisa"
    ))
    .unwrap();
    let mut cert = template.section("certificate").unwrap().clone();
    let donor = cert
        .get("regions")
        .unwrap()
        .object()
        .unwrap()
        .values()
        .next()
        .unwrap()
        .clone();
    let mut regions = BTreeMap::new();
    let identity = base.identity().unwrap();
    for name in ["outer", "empty"] {
        let mut region = donor.clone();
        fields(&mut region).remove("frame_contexts");
        fields(&mut region).insert(
            "signature".into(),
            base.functions[name].signature_for(base.isa_version),
        );
        let mut frame = support::json(
            r#"{"slots":[{"id":"mut:0","role":"mut","width":1,"class":"A"}],"scratch":[],"borrow":[],"history":[],"ancillae":[]}"#,
        );
        if name == "outer" {
            fields(&mut frame).insert(
                "scratch".into(),
                support::json(r#"[{"id":"b","class":"A","state":"clean","width":1}]"#),
            );
        }
        fields(&mut region).insert("frame".into(), frame);
        regions.insert(identity.closure_hashes[name].clone().unwrap(), region);
    }
    fields(&mut cert).insert("regions".into(), Json::Object(regions));
    let bindings = fields(fields(&mut cert).get_mut("bindings").unwrap());
    bindings.insert("object_hash".into(), Json::String(identity.object_hash));
    for key in ["interface_hash", "closure_hash", "reference_program_hash"] {
        bindings.insert(
            key.into(),
            Json::String(identity.closure_hashes["outer"].clone().unwrap()),
        );
    }
    let object = Object::parse(&format!(
        "{}\nsection certificate\n{}\n",
        base.text,
        cert.canonical()
    ))
    .unwrap();
    object.check_hashes().unwrap();
    object
}
#[test]
fn locally_allocated_class_has_a_valid_child_frame() {
    let object = local_program();
    let result = object.selected_action_frames(
        "outer",
        &[bennett_isa_conformance::json::Json::Str("t".into())],
        &[("t", 1)],
        false,
        None,
        &[vec!["A".into()]],
    );
    assert!(result.is_ok(), "{result:?}");
}
#[test]
fn valid_local_child_call_does_not_fault() {
    let object = local_program();
    let mut registers = QuantumRegisters::new();
    let a = registers.allocate_in_class(1, "A").unwrap();
    let mut machine = ClassicalMachine::with_quantum_registers(
        &object,
        "launch",
        &[ClassicalValue::Qreg(a)],
        &[],
        registers,
    )
    .unwrap();
    let before = machine.snapshot();
    let result = machine.run(10).unwrap();
    assert_eq!(result.fault, None);
    let ledger = machine.quantum_registers();
    assert_eq!(
        ledger
            .quantum_state()
            .read_basis(ledger.bundle(a).unwrap())
            .unwrap(),
        1
    );
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}

#[test]
fn local_child_reference_executes_without_frame_preflight() {
    let original = local_program();
    let text = original
        .text
        .split("\nsection certificate\n")
        .next()
        .unwrap();
    let object = Object::parse(text).unwrap();
    object.check_hashes().unwrap();
    let mut registers = QuantumRegisters::new();
    let a = registers.allocate_in_class(1, "A").unwrap();
    let mut machine = ClassicalMachine::with_quantum_registers(
        &object,
        "launch",
        &[ClassicalValue::Qreg(a)],
        &[],
        registers,
    )
    .unwrap();
    let before = machine.snapshot();
    assert_eq!(machine.run(10).unwrap().fault, None);
    let ledger = machine.quantum_registers();
    assert_eq!(
        ledger
            .quantum_state()
            .read_basis(ledger.bundle(a).unwrap())
            .unwrap(),
        1
    );
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}

#[test]
fn malformed_known_models_reject_before_unsupported_reporting() {
    for (key, value) in [
        ("control_models", r#"{"kind":"unpublished"}"#),
        ("data", r#"{}"#),
        (
            "data",
            r#"{"matrix":{"kind":"identity","dimension":2,"unknown":0}}"#,
        ),
        ("data", r#"{"matrix":{"kind":"tensor_power","count":1}}"#),
        ("data", r#"{"matrix":{"kind":"sum","matrices":[]}}"#),
        (
            "data",
            r#"{"matrix":{"kind":"product","matrices":[{"kind":"identity","dimension":2},{"kind":"identity","dimension":3}]}}"#,
        ),
        ("data", r#"{"matrix":{"kind":"zero","rows":0,"columns":2}}"#),
        (
            "data",
            r#"{"matrix":{"kind":"identity","dimension":2},"representation":{"kind":"kraus","operators":[]}}"#,
        ),
    ] {
        let mut m = model();
        fields(&mut m).insert(key.into(), support::json(value));
        assert!(verify(&m, 0, 1, 0).is_err(), "{value}");
    }
}
#[test]
fn published_matrix_and_representation_productions_are_not_rejected() {
    let original = model()
        .get("data")
        .unwrap()
        .get("matrix")
        .unwrap()
        .canonical();
    for matrix in [
        format!(r#"{{"kind":"tensor_power","count":1,"matrix":{original}}}"#),
        format!(r#"{{"kind":"sum","matrices":[{original}]}}"#),
    ] {
        let mut m = model();
        fields(&mut m).insert(
            "data".into(),
            support::json(&format!(r#"{{"matrix":{matrix}}}"#)),
        );
        assert!(verify(&m, 0, 1, 0).is_ok());
    }
    let mut m = model();
    let d = fields(&mut m).get_mut("data").unwrap();
    fields(d).insert(
        "representation".into(),
        support::json(&format!(r#"{{"kind":"kraus","operators":[{original}]}}"#)),
    );
    assert!(matches!(
        verify(&m, 0, 1, 0).unwrap(),
        ModelVerification::Unchecked(_)
    ));
}

#[test]
fn local_any_class_does_not_fault_or_claim_class_establishment() {
    let object = local_program_class("any");
    let selected = object
        .selected_action_frames(
            "outer",
            &[bennett_isa_conformance::json::Json::Str("t".into())],
            &[("t", 1)],
            false,
            None,
            &[vec!["A".into()]],
        )
        .unwrap();
    assert!(selected.iter().any(|f| !f.interface_classes_established));
    let mut registers = QuantumRegisters::new();
    let a = registers.allocate_in_class(1, "A").unwrap();
    let mut machine = ClassicalMachine::with_quantum_registers(
        &object,
        "launch",
        &[ClassicalValue::Qreg(a)],
        &[],
        registers,
    )
    .unwrap();
    let before = machine.snapshot();
    assert_eq!(machine.run(10).unwrap().fault, None);
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}
#[test]
fn resolved_local_class_mismatch_still_faults_before_effects() {
    let object = local_program_class("B");
    assert!(object
        .selected_action_frames(
            "outer",
            &[bennett_isa_conformance::json::Json::Str("t".into())],
            &[("t", 1)],
            false,
            None,
            &[vec!["A".into()]]
        )
        .is_err());
    let mut registers = QuantumRegisters::new();
    let a = registers.allocate_in_class(1, "A").unwrap();
    let mut machine = ClassicalMachine::with_quantum_registers(
        &object,
        "launch",
        &[ClassicalValue::Qreg(a)],
        &[],
        registers,
    )
    .unwrap();
    let before = machine.snapshot();
    assert_eq!(machine.run(10).unwrap().fault, Some(4));
    let ledger = machine.quantum_registers();
    assert_eq!(
        ledger
            .quantum_state()
            .read_basis(ledger.bundle(a).unwrap())
            .unwrap(),
        0
    );
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}
#[test]
fn explicit_distinct_control_context_is_valid_but_unchecked() {
    let mut m = model();
    fields(&mut m).insert(
        "control_models".into(),
        support::json(r#"{"kind":"explicit","controls":"c"}"#),
    );
    fields(&mut m).insert(
        "arity".into(),
        support::json(r#"{"op":"+","args":[{"var":"d"},1]}"#),
    );
    fields(&mut m).insert(
        "basis".into(),
        support::json(r#"{"controls":"c","targets":["t"]}"#),
    );
    fields(&mut m).insert("parameter_bindings".into(), support::json(r#"{"d":"d"}"#));
    let x = model()
        .get("data")
        .unwrap()
        .get("matrix")
        .unwrap()
        .canonical();
    fields(&mut m).insert("data".into(),support::json(&format!(r#"{{"contexts":[{{"controls":{{"min":2,"max":2}},"data":{{"matrix":{{"kind":"controlled","controls":{{"var":"d"}},"matrix":{x}}}}}}}]}}"#)));
    let mut operands = native_operands::operands(1, 2, 1);
    operands.effective_controls = operands.own_controls.clone();
    operands.allocation_order = operands
        .own_controls
        .iter()
        .chain(&operands.targets)
        .cloned()
        .collect();
    let ctx = ModelContext {
        operands,
        own_controls: 1,
        total_controls: 2,
        requested_epsilon: Some(Rational::leaf("1", "2").unwrap()),
        theta_bits: None,
    };
    assert!(matches!(
        verify_native_model(Some(&m), "cx", &ctx, &Rational::leaf("1", "4").unwrap(), 64).unwrap(),
        ModelVerification::Unchecked(_)
    ));
    let mut bad = m.clone();
    fields(&mut bad).insert("operand_map".into(), support::json("[0,0,1]"));
    assert!(verify_native_model(Some(&bad), "cx", &ctx, &Rational::integer(0), 64).is_err());
    fields(&mut bad).insert("operand_map".into(), support::json("[0,1]"));
    fields(&mut bad).insert("parameter_bindings".into(), support::json(r#"{"d":"c"}"#));
    assert!(verify_native_model(Some(&bad), "cx", &ctx, &Rational::integer(0), 64).is_err());
}
#[test]
fn distinct_overlap_uses_two_bits_at_positive_epsilon() {
    let m = model();
    let mut operands = native_operands::operands(1, 2, 1);
    operands.effective_controls = operands.own_controls.clone();
    operands.allocation_order = operands
        .own_controls
        .iter()
        .chain(&operands.targets)
        .cloned()
        .collect();
    let ctx = ModelContext {
        operands,
        own_controls: 1,
        total_controls: 2,
        requested_epsilon: Some(Rational::leaf("1", "2").unwrap()),
        theta_bits: None,
    };
    let ModelVerification::Checked(actual) =
        verify_native_model(Some(&m), "cx", &ctx, &Rational::leaf("1", "4").unwrap(), 64).unwrap()
    else {
        panic!("overlap must be checked")
    };
    assert_eq!(actual.rows(), 4);
    for col in 0..4 {
        for row in 0..4 {
            assert_eq!(
                actual.entry(row, col).unwrap().re.lower(),
                &Rational::integer(i128::from([0, 1, 3, 2][col] == row))
            );
        }
    }
}

#[test]
fn optional_unsupported_obligations_do_not_hide_refuted_base_or_malformed_instrument() {
    let mut m = model();
    let d = fields(&mut m).get_mut("data").unwrap();
    let matrix = d.get("matrix").unwrap().canonical();
    fields(d).insert("representation".into(),support::json(&format!(r#"{{"kind":"instrument","outcomes":[{{"label":"z","operators":[{matrix}]}},{{"label":"a","operators":[{matrix}]}}]}}"#)));
    assert!(verify(&m, 0, 1, 0).is_err(), "unsorted instrument labels");
}
#[test]
fn unsupported_certificates_do_not_hide_refuted_dense_base() {
    let mut m = model();
    let d = fields(&mut m).get_mut("data").unwrap();
    fields(d).insert(
        "certificates".into(),
        support::json(r#"{"unitarity":{"version":"bennett.unitarity/99"}}"#),
    );
    let invalid = d
        .get("matrix")
        .unwrap()
        .canonical()
        .replace("[\"1\",\"1\"]", "[\"2\",\"1\"]");
    fields(d).insert("matrix".into(), support::json(&invalid));
    assert!(
        verify(&m, 0, 1, 3).is_err(),
        "refuted dense base must reject even with unsupported certificates"
    );
}

#[test]
fn unselected_explicit_rows_keep_their_own_context_dimensions() {
    let mut m = model();
    fields(&mut m).insert(
        "control_models".into(),
        support::json(r#"{"kind":"explicit","controls":"c"}"#),
    );
    fields(&mut m).insert("arity".into(), support::json("2"));
    fields(&mut m).insert("basis".into(), support::json(r#"["c[0]","t"]"#));
    fields(&mut m).insert("data".into(),support::json(r#"{"contexts":[{"controls":{"min":0,"max":0},"data":{"matrix":{"kind":"identity","dimension":2}}},{"controls":{"min":1,"max":"unbounded"},"data":{"matrix":{"kind":"identity","dimension":{"op":"pow2","args":[{"op":"+","args":[{"var":"d"},1]}]}}}}]}"#));
    assert!(matches!(
        verify(&m, 0, 1, 2).unwrap(),
        ModelVerification::Unchecked(_)
    ));
}

#[test]
fn every_published_constructor_and_representation_is_admitted_honestly() {
    let zero = r#"[["0","1"],["0","1"]]"#;
    let one = r#"[["1","1"],["0","1"]]"#;
    let x = model()
        .get("data")
        .unwrap()
        .get("matrix")
        .unwrap()
        .canonical();
    let identity = r#"{"kind":"identity","dimension":2}"#;
    for matrix in [
        identity.to_owned(),
        r#"{"kind":"zero","rows":2,"columns":2}"#.into(),
        format!(r#"{{"kind":"diagonal","entries":[{one},{one}]}}"#),
        format!(r#"{{"kind":"adjoint","matrix":{x}}}"#),
        format!(r#"{{"kind":"scale","scalar":{one},"matrix":{x}}}"#),
        format!(r#"{{"kind":"sum","matrices":[{x}]}}"#),
        format!(r#"{{"kind":"product","matrices":[{x},{identity}]}}"#),
        format!(r#"{{"kind":"tensor","matrices":[{x}]}}"#),
        format!(r#"{{"kind":"block_diagonal","matrices":[{x}]}}"#),
        format!(r#"{{"kind":"controlled","controls":0,"matrix":{x}}}"#),
        format!(r#"{{"kind":"tensor_power","count":1,"matrix":{x}}}"#),
    ] {
        let mut m = model();
        fields(&mut m).insert(
            "data".into(),
            support::json(&format!(r#"{{"matrix":{matrix}}}"#)),
        );
        assert!(
            matches!(verify(&m,0,1,3).unwrap(),ModelVerification::Unchecked(reason) if reason.contains("unsupported")),
            "{matrix}"
        );
    }
    for representation in [
        format!(r#"{{"kind":"kraus","operators":[{x}]}}"#),
        format!(r#"{{"kind":"instrument","outcomes":[{{"label":"a","operators":[{x}]}}]}}"#),
        format!(
            r#"{{"kind":"environment","arity":1,"state":[[{one},{zero}],[{zero},{zero}]],"matrix":{{"kind":"tensor","matrices":[{x},{identity}]}}}}"#
        ),
    ] {
        let mut m = model();
        fields(fields(&mut m).get_mut("data").unwrap())
            .insert("representation".into(), support::json(&representation));
        assert!(
            matches!(
                verify(&m, 0, 1, 0).unwrap(),
                ModelVerification::Unchecked(_)
            ),
            "{representation}"
        );
    }
}
