mod support;
use bennett_isa_ref::Object;

fn pair() -> Object {
    let src=support::object(&[
        ("empty",r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#,"perm_action @empty (t:wire mut) () () () ()\n^entry():\nqpred t\nqunpred t"),
        ("pair",r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#,"perm_action @pair (t:wire mut) (c:wire in) () () ()\n^entry():\naction_call @empty (t) () () ()\nqpred c\naction_call @empty (t) () () ()\nqunpred c")
    ],"{}","").replace("=0.8","=0.9").replace("\"0.8\"","\"0.9\"");
    support::rehash(&src)
}

#[test]
fn complete_source_context_inventory_includes_empty_and_predicated_calls() {
    let object = pair();
    let contexts = object
        .source_action_invocations(
            "pair",
            &[
                bennett_isa_conformance::json::Json::Str("t".into()),
                bennett_isa_conformance::json::Json::Str("c".into()),
            ],
            &[("t", 1), ("c", 1)],
            false,
            None,
        )
        .unwrap();
    assert_eq!(contexts.len(), 3);
    let root = contexts
        .iter()
        .find(|c| c.context_key.array().unwrap().is_empty())
        .unwrap();
    assert_eq!(root.binding.inherited_controls, 0);
    let mut calls = contexts
        .iter()
        .filter(|c| !c.context_key.array().unwrap().is_empty())
        .collect::<Vec<_>>();
    calls.sort_by_key(|c| c.context_key.canonical());
    for (call, point, controls) in [(calls[0], "entry:0", 0), (calls[1], "entry:2", 1)] {
        let edge = &call.context_key.array().unwrap()[0];
        assert_eq!(edge.get("point").unwrap().string().unwrap(), point);
        assert_eq!(edge.get("call_index").unwrap().uint().unwrap(), 0);
        assert!(edge.get("instance").unwrap().array().unwrap().is_empty());
        assert_eq!(call.binding.inherited_controls, controls);
    }
    assert!(object
        .expand_action_source(
            "pair",
            &[
                bennett_isa_conformance::json::Json::Str("t".into()),
                bennett_isa_conformance::json::Json::Str("c".into())
            ],
            &[("t", 1), ("c", 1)],
            false
        )
        .unwrap()
        .is_empty());
}

#[test]
fn published_pair_selects_all_frames_and_rejects_swapped_or_missing_contexts() {
    use bennett_isa_conformance::json::Json as Value;
    use bennett_isa_ref::json::Json;
    let source = include_str!("../../conformance/fixtures/accepted-k90-09/pair.bisa");
    let object = Object::parse(source).unwrap();
    let args = [Value::Str("t".into()), Value::Str("u".into())];
    let check = |o: &Object| {
        o.selected_action_frames(
            "pair",
            &args,
            &[("t", 1), ("u", 1)],
            false,
            None,
            &[vec!["A".into()], vec!["B".into()]],
        )
    };
    let selected = check(&object).unwrap();
    assert_eq!(selected.len(), 3);
    for n in 0..4 {
        let mut state = bennett_isa_ref::QuantumState::new();
        let wires = state.allocate(2).unwrap();
        for (i, w) in wires.iter().enumerate() {
            if (n >> i) & 1 == 1 {
                state.gate("x", &[*w], None, &[]).unwrap();
            }
        }
        let bindings = bennett_isa_ref::QuantumBindings {
            bundles: std::collections::BTreeMap::from([
                ("t".into(), vec![wires[0]]),
                ("u".into(), vec![wires[1]]),
            ]),
            ..Default::default()
        };
        object
            .execute_action("pair", &mut state, &bindings, false, &[])
            .unwrap();
        assert_eq!(state.read_basis(&wires).unwrap(), n ^ 3);
        object
            .execute_action("pair", &mut state, &bindings, true, &[])
            .unwrap();
        assert_eq!(state.read_basis(&wires).unwrap(), n);
    }
    let hp = object.identity().unwrap().closure_hashes["pair"]
        .clone()
        .unwrap();
    assert_eq!(
        hp,
        "b98f9111e8533f00a8e5ac195e1b335678d14efcabb653d754d9999e21297c16"
    );
    for (point, class) in [("entry:0", "A"), ("entry:1", "B")] {
        let frame = selected
            .iter()
            .find(|r| {
                r.context_key
                    .array()
                    .unwrap()
                    .first()
                    .is_some_and(|e| e.get("point").and_then(Json::string) == Ok(point))
            })
            .unwrap();
        assert_eq!(
            frame.frame.get("slots").unwrap().array().unwrap()[0]
                .get("class")
                .unwrap()
                .string()
                .unwrap(),
            class
        );
    }
    let mut cert = object.section("certificate").unwrap().clone();
    let hf = object.identity().unwrap().closure_hashes["flip"]
        .clone()
        .unwrap();
    let Json::Object(fields) = &mut cert else {
        unreachable!()
    };
    let Json::Object(regions) = fields.get_mut("regions").unwrap() else {
        unreachable!()
    };
    let Json::Object(child) = regions.get_mut(&hf).unwrap() else {
        unreachable!()
    };
    let Json::Array(records) = child.get_mut("frame_contexts").unwrap() else {
        unreachable!()
    };
    records.pop();
    let text = source
        .split("\nsection certificate\n")
        .next()
        .unwrap()
        .to_owned()
        + "\nsection certificate\n"
        + &cert.canonical()
        + "\n";
    let incomplete = Object::parse(&text).unwrap();
    assert!(check(&incomplete).is_err());
    assert!(object
        .selected_action_frames(
            "pair",
            &args,
            &[("t", 1), ("u", 1)],
            false,
            None,
            &[vec!["B".into()], vec!["A".into()]]
        )
        .is_err());
}

#[test]
fn runtime_checks_nested_context_classes_before_any_gate_and_reverses_fault() {
    use bennett_isa_ref::{json::Json, ClassicalMachine, ClassicalValue, QuantumRegisters};
    let original = Object::parse(include_str!(
        "../../conformance/fixtures/accepted-k90-09/pair.bisa"
    ))
    .unwrap();
    let mut symbols = original.section("symbols").unwrap().clone();
    let Json::Object(fields) = &mut symbols else {
        unreachable!()
    };
    fields.insert("2".into(),support::json(r#"{"name":"launch","kind":"r","signature":{"abi":"0.9","kind":"r","params":["qreg[1]","qreg[1]"],"results":[],"tier":"r"},"interface_hash":"0000000000000000000000000000000000000000000000000000000000000000"}"#));
    let raw=original.text.split("\nsection certificate\n").next().unwrap().replace(&original.section("symbols").unwrap().canonical(),&symbols.canonical())+"fn r @launch (a:qreg[1], b:qreg[1]) () () () ()\n^entry():\naction_call @pair (a, b) () () ()\nret\n";
    let base = support::rehash(&raw);
    for wrong in [false, true] {
        let mut cert = original.section("certificate").unwrap().clone();
        let Json::Object(c) = &mut cert else {
            unreachable!()
        };
        let Json::Object(b) = c.get_mut("bindings").unwrap() else {
            unreachable!()
        };
        b.insert(
            "object_hash".into(),
            Json::String(base.identity().unwrap().object_hash),
        );
        if wrong {
            let Json::Object(regions) = c.get_mut("regions").unwrap() else {
                unreachable!()
            };
            let region = regions
                .values_mut()
                .find(|r| r.object().unwrap().contains_key("frame_contexts"))
                .unwrap();
            let Json::Object(r) = region else {
                unreachable!()
            };
            let Json::Array(records) = r.get_mut("frame_contexts").unwrap() else {
                unreachable!()
            };
            let Json::Object(record) = &mut records[0] else {
                unreachable!()
            };
            let Json::Object(frame) = record.get_mut("frame").unwrap() else {
                unreachable!()
            };
            let Json::Array(slots) = frame.get_mut("slots").unwrap() else {
                unreachable!()
            };
            let Json::Object(slot) = &mut slots[0] else {
                unreachable!()
            };
            slot.insert("class".into(), Json::String("B".into()));
        }
        let object = Object::parse(&format!(
            "{}\nsection certificate\n{}\n",
            base.text,
            cert.canonical()
        ))
        .unwrap();
        let mut registers = QuantumRegisters::new();
        let a = registers.allocate_in_class(1, "A").unwrap();
        let b = registers.allocate_in_class(1, "B").unwrap();
        let mut machine = ClassicalMachine::with_quantum_registers(
            &object,
            "launch",
            &[ClassicalValue::Qreg(a), ClassicalValue::Qreg(b)],
            &[],
            registers,
        )
        .unwrap();
        let before = machine.snapshot();
        assert_eq!(
            machine.run(10).unwrap().fault,
            if wrong { Some(4) } else { None }
        );
        let ledger = machine.quantum_registers();
        assert_eq!(
            ledger
                .quantum_state()
                .read_basis(ledger.bundle(a).unwrap())
                .unwrap(),
            if wrong { 0 } else { 1 }
        );
        assert_eq!(
            ledger
                .quantum_state()
                .read_basis(ledger.bundle(b).unwrap())
                .unwrap(),
            if wrong { 0 } else { 1 }
        );
        machine.reverse().unwrap();
        assert_eq!(machine.snapshot(), before);
    }
}
