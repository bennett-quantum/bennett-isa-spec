mod support;
use bennett_isa_ref::{
    json::Json, target::frame_slot_classes, ClassicalMachine, ClassicalValue, Object,
};
use std::collections::BTreeMap;

fn fields(v: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(m) = v else { panic!("object") };
    m
}

#[test]
fn canonical_fragments_expand_one_slot_in_bit_order() {
    let slot = support::json(
        r#"{"id":"mut:0","role":"mut","width":2,"fragments":[{"offset":0,"width":1,"class":"A"},{"offset":1,"width":1,"class":"B"}]}"#,
    );
    assert_eq!(frame_slot_classes(&slot, 0).unwrap(), ["A", "B"]);
    let region = support::json(
        r#"{"id":"mem:0","role":"mem_in","width":4,"fragments":[{"offset":0,"width":1,"class":"A"},{"offset":1,"width":1,"class":"B"},{"offset":2,"width":1,"class":"A"},{"offset":3,"width":1,"class":"B"}]}"#,
    );
    assert_eq!(
        frame_slot_classes(&region, 0).unwrap(),
        ["A", "B", "A", "B"]
    );
    for bad in [
        r#"[{"offset":0,"width":1,"class":"A"},{"offset":2,"width":1,"class":"B"}]"#,
        r#"[{"offset":0,"width":2,"class":"A"},{"offset":1,"width":1,"class":"B"}]"#,
        r#"[{"offset":0,"width":0,"class":"A"},{"offset":0,"width":2,"class":"B"}]"#,
        r#"[{"offset":0,"width":1,"class":"A"},{"offset":1,"width":1,"class":"A"}]"#,
        r#"[{"offset":0,"width":2,"class":"A"}]"#,
        r#"[{"offset":1,"width":1,"class":"B"},{"offset":0,"width":1,"class":"A"}]"#,
    ] {
        let mut invalid = slot.clone();
        fields(&mut invalid).insert("fragments".into(), support::json(bad));
        assert!(frame_slot_classes(&invalid, 0).is_err(), "{bad}");
    }
    let mut both = slot.clone();
    fields(&mut both).insert("class".into(), Json::String("A".into()));
    assert!(frame_slot_classes(&both, 0).is_err());
    let legacy = support::json(r#"{"id":"mut:0","role":"mut","width":2,"class":"A"}"#);
    assert_eq!(frame_slot_classes(&legacy, 0).unwrap(), ["A", "A"]);
}

fn class_bound_program() -> Object {
    let source=support::object(&[
        ("same",r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#,"perm_action @same (t:wire mut) () () () ()\n^entry():\nx t\nx t"),
        ("run",r#"{"abi":"0.8","kind":"r","params":[],"results":[],"tier":"r"}"#,"fn r @run () () () () ()\n^entry():\na = qreg.new \"A\" 1\nb = qreg.new \"B\" 1\nt = qreg.concat a b\nv = qreg.slice t 1 1\naction_call @same (v) () () ()\nqreg.free t\nret")
    ],"{}","").replace("=0.8","=0.9").replace("\"0.8\"","\"0.9\"");
    let base = support::rehash(&source);
    let identities = base.identity().unwrap();
    let hash = identities.closure_hashes["same"].clone().unwrap();
    let template = Object::parse(include_str!(
        "../../conformance/fixtures/controlled-action-09/controlled.bisa"
    ))
    .unwrap();
    let mut cert = template.section("certificate").unwrap().clone();
    let region = cert
        .get("regions")
        .unwrap()
        .object()
        .unwrap()
        .values()
        .find(|v| {
            v.get("frame")
                .unwrap()
                .get("slots")
                .unwrap()
                .array()
                .unwrap()
                .len()
                == 1
        })
        .unwrap()
        .clone();
    let mut region = region;
    fields(&mut region).insert(
        "signature".into(),
        base.functions["same"].signature_for(base.isa_version),
    );
    fields(&mut region).insert("claims".into(), Json::Array(vec![]));
    fields(&mut region).insert("error".into(), support::json("{}"));
    fields(&mut region).insert("verification".into(),support::json(r#"{"oracle":null,"borrow_frames":[],"inverse_pairs":[],"unitary":[],"channel":[],"fault_rate":{"domain":{"bundles":[],"borrowed":[],"regions":[]},"faults_by_code":{},"undefined_excluded":0,"layout_dependent_excluded":0}}"#));
    fields(&mut region).insert("frame".into(),support::json(r#"{"slots":[{"id":"mut:0","role":"mut","width":1,"class":"A"}],"scratch":[],"history":[],"borrow":[],"ancillae":[]}"#));
    fields(&mut cert).insert(
        "regions".into(),
        Json::Object(BTreeMap::from([(hash.clone(), region)])),
    );
    let b = fields(fields(&mut cert).get_mut("bindings").unwrap());
    b.remove("schedule_hash");
    b.insert("object_hash".into(), Json::String(identities.object_hash));
    for key in ["interface_hash", "closure_hash", "reference_program_hash"] {
        b.insert(key.into(), Json::String(hash.clone()));
    }
    fields(fields(&mut cert).get_mut("composition").unwrap())
        .insert("multiplicities".into(), support::json("{}"));
    Object::parse(&format!(
        "{}\nsection certificate\n{}\n",
        base.text,
        cert.canonical()
    ))
    .unwrap()
}

#[test]
fn wrong_sliced_class_faults_before_view_consumption_and_reverses() {
    let object = class_bound_program();
    let mut machine = ClassicalMachine::new(&object, "run", &[], &[]).unwrap();
    let before = machine.snapshot();
    let result = machine.run(30).unwrap();
    assert_eq!(result.fault, Some(4));
    let delta = before.delta(&machine.snapshot()).unwrap();
    let ids = delta
        .registers
        .into_iter()
        .filter_map(|r| match r.post {
            Some(ClassicalValue::Qreg(id)) => Some((r.name, id)),
            _ => None,
        })
        .collect::<BTreeMap<_, _>>();
    assert!(machine.quantum_registers().bundle(ids["t"]).is_err());
    assert_eq!(
        machine.quantum_registers().classes(ids["v"]).unwrap(),
        ["B"]
    );
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}

#[test]
fn portable_interface_vectors_check_control_class_capacity_and_vertices() {
    use bennett_isa_ref::target::NativeTarget;
    let vectors = support::json(include_str!(
        "../../conformance/fixtures/accepted-k88-09/vectors.json"
    ));
    let target = NativeTarget::parse(include_str!(
        "../../conformance/fixtures/accepted-k88-09/target.json"
    ))
    .unwrap();
    let frame = vectors.get("frame").unwrap();
    assert_eq!(
        target
            .frame_interface_vertices(frame, &["C".into()], None)
            .unwrap(),
        vec![vec![2], vec![0, 1]]
    );
    assert_eq!(
        target
            .frame_interface_vertices(frame, &["C".into()], Some(&[vec![2], vec![0, 1]]))
            .unwrap(),
        vec![vec![2], vec![0, 1]]
    );
    for bad in [
        vec![vec![2], vec![1, 0]],
        vec![vec![2], vec![0, 0]],
        vec![vec![2], vec![0, 3]],
        vec![vec![2], vec![0]],
    ] {
        assert!(target
            .frame_interface_vertices(frame, &["C".into()], Some(&bad))
            .is_err());
    }
    assert!(target
        .frame_interface_vertices(frame, &["A".into()], None)
        .is_err());
    let source=support::object(&[("split",r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[2],"tier":"q"}"#,"perm_action @split (t:wire[2] mut) () () () ()\n^entry():\nx t[0]\nx t[1]")],"{}","").replace("=0.8","=0.9").replace("\"0.8\"","\"0.9\"");
    let object = support::rehash(&source);
    for n in 0..4 {
        let mut state = bennett_isa_ref::QuantumState::new();
        let wires = state.allocate(2).unwrap();
        for (bit, wire) in wires.iter().enumerate() {
            if (n >> bit) & 1 == 1 {
                state.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        let bindings = bennett_isa_ref::QuantumBindings {
            bundles: BTreeMap::from([("t".into(), wires.clone())]),
            ..Default::default()
        };
        object
            .execute_action("split", &mut state, &bindings, false, &[])
            .unwrap();
        assert_eq!(state.read_basis(&wires).unwrap(), n ^ 3);
        object
            .execute_action("split", &mut state, &bindings, true, &[])
            .unwrap();
        assert_eq!(state.read_basis(&wires).unwrap(), n);
    }
}
