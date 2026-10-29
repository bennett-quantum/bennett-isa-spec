mod support;
use bennett_isa_ref::{
    validate_translation_lemma, Instruction, Object, QuantumBindings, QuantumState,
};
use std::collections::BTreeSet;
use support::object;
fn gate(op: &str, operands: &[&str]) -> Instruction {
    Instruction {
        lhs: Vec::new(),
        op: op.into(),
        operands: operands.iter().map(|s| (*s).into()).collect(),
        bodies: Vec::new(),
        line: 1,
    }
}
#[test]
fn translation_lemma_checks_all_four_required_negative_patterns() {
    let b = BTreeSet::from(["b".into(), "b2".into()]);
    let x = BTreeSet::from(["x".into()]);
    let v = [gate("cx", &["x", "b"])];
    let w = [gate("cx", &["b", "t"])];
    validate_translation_lemma(&v, &w, &x, &b).unwrap();
    // A V;W;V^-1 prefix lacks the required final W and is checked as a full frame below.
    assert!(validate_translation_lemma(
        &[gate("cx", &["b2", "b"])],
        &w,
        &BTreeSet::from(["b2".into()]),
        &b
    )
    .is_err());
    assert!(validate_translation_lemma(
        &[gate("cx", &["b", "x"])],
        &w,
        &BTreeSet::from(["b".into()]),
        &b
    )
    .is_err());
    assert!(validate_translation_lemma(
        &v,
        &[gate("swap", &["y", "z"]), gate("cx", &["b", "y"])],
        &x,
        &b
    )
    .is_err());
    assert!(validate_translation_lemma(
        &v,
        &[gate("cx", &["b", "y"]), gate("cx", &["y", "z"])],
        &x,
        &b
    )
    .is_err());
}
fn borrowed(body: &str) -> String {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[1,1,1],"tier":"q"}"#;
    object(&[("a",signature,&format!("perm_action @a (t:wire mut, y:wire mut, z:wire mut) (x:wire in) () () ()\n^entry():\nb = qborrow any 1\n{body}\nqreturn b"))],"{}","")
}
#[test]
fn borrow_frame_exhaustion_rejects_hidden_dependence_on_borrowed_bits() {
    let raw = borrowed("cx b y\ncx y z\ncx b y");
    let error = Object::parse(&raw).expect_err("borrowed-dependent output accepted");
    assert!(error.to_string().contains("§5.6"));
}
#[test]
fn translation_pattern_returns_an_entangled_borrowed_system() {
    let o = Object::parse(&borrowed("cx x b\ncx b t\ncx x b\ncx b t")).unwrap();
    let mut q = QuantumState::new();
    let t = q.allocate(1).unwrap();
    let y = q.allocate(1).unwrap();
    let z = q.allocate(1).unwrap();
    let x = q.allocate(1).unwrap();
    let b = q.allocate(1).unwrap();
    let reference = q.allocate(1).unwrap();
    q.gate("h", &[b[0]], None, &[]).unwrap();
    q.gate("cx", &[b[0], reference[0]], None, &[]).unwrap();
    q.gate("x", &x, None, &[]).unwrap();
    let mut bindings = QuantumBindings::default();
    for (n, w) in [("t", t.clone()), ("y", y), ("z", z), ("x", x)] {
        bindings.bundles.insert(n.into(), w);
    }
    bindings.borrowed.push(b.clone());
    o.execute_action("a", &mut q, &bindings, false, &[])
        .unwrap();
    // The output is t=1 tensor the original Bell pair on borrowed b/reference.
    q.gate("x", &t, None, &[]).unwrap();
    q.gate("cx", &[b[0], reference[0]], None, &[]).unwrap();
    q.gate("h", &b, None, &[]).unwrap();
    q.release(&b).unwrap();
    q.release(&reference).unwrap();
}

#[test]
fn classical_region_calls_supply_only_designated_idle_registers() {
    use bennett_isa_ref::{ClassicalMachine, ClassicalValue, QuantumRegisters};
    let action = support::rehash(&borrowed("cx x b\ncx b t\ncx x b\ncx b t"));
    let sig = r#"{"abi":"0.8","kind":"cc","params":["qreg[1]","qreg[1]","qreg[1]","qreg[1]"],"results":[],"tier":"cc"}"#;
    let imports = format!(
        "{{\"a\":{{\"signature\":{}}}}}",
        action.symbols().unwrap()["0"]
            .get("signature")
            .unwrap()
            .canonical()
    );
    let caller=support::rehash(&object(&[("invoke",sig,"fn cc @invoke (t:qreg[1], y:qreg[1], z:qreg[1], x:qreg[1]) () () () ()\n^entry():\naction_call @a (t, y, z) (x) () ()\nret")],&imports,""));
    let program = Object::link(&[caller, action]).unwrap();
    let mut registers = QuantumRegisters::new();
    let mut ids = Vec::new();
    for _ in 0..5 {
        ids.push(registers.allocate(1).unwrap());
    }
    registers.prepare(ids[3], &[true]).unwrap();
    registers.prepare(ids[4], &[true]).unwrap();
    let args = ids[..4]
        .iter()
        .map(|id| ClassicalValue::Qreg(*id))
        .collect::<Vec<_>>();
    let mut m = ClassicalMachine::with_quantum_registers(&program, "invoke", &args, &[], registers)
        .unwrap();
    m.designate_idle_registers(&[ids[4]]).unwrap();
    let before = m.snapshot();
    assert_eq!(m.run(100).unwrap().fault, None);
    let r = m.quantum_registers();
    for id in [ids[0], ids[4]] {
        assert_eq!(
            r.quantum_state().read_basis(r.bundle(id).unwrap()).unwrap(),
            1
        );
    }
    m.reverse().unwrap();
    assert_eq!(m.snapshot(), before);
}

#[test]
fn lifted_permutation_actions_can_borrow_workspace_and_reverse() {
    use bennett_isa_ref::{Budget, OracleInput, OracleMachine};
    let action = support::rehash(&borrowed("cx x b\ncx b t\ncx x b\ncx b t"));
    let signature =
        r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let imports = format!(
        "{{\"a\":{{\"signature\":{}}}}}",
        action.symbols().unwrap()["0"]
            .get("signature")
            .unwrap()
            .canonical()
    );
    let caller=support::rehash(&object(&[("lifted",signature,"oracle @lifted (input:wire in) (target:wire) (fault:wire[4]) () ()\nregs { t:reg[1], y:reg[1], z:reg[1], x:reg[1] }\n^entry():\nx = lift.mov input\nt = lift.const.i1 0\ny = lift.const.i1 0\nz = lift.const.i1 0\nlift.perm_call @a (t, y, z) (x) () ()\nlift.retval t")],&imports,""));
    let o = Object::link(&[caller, action]).unwrap();
    for x in 0..2 {
        for steps in 0..8 {
            let input = OracleInput::bits(&[x]);
            let budget = Budget::steps(steps);
            let result = o.run_oracle("lifted", &input, budget).unwrap();
            assert_eq!(
                (result.payload, result.code),
                if steps >= 6 { (x, 0) } else { (0, 1) }
            );
            let mut m = OracleMachine::new(&o, "lifted", &input, budget).unwrap();
            let before = m.snapshot();
            for _ in 0..steps {
                m.forward_step().unwrap();
            }
            assert_eq!(m.outcome(), result);
            for _ in 0..steps {
                m.inverse_step().unwrap();
            }
            assert_eq!(m.snapshot(), before);
        }
    }
}

#[test]
fn exhaustive_borrow_identity_need_not_be_adjacent_gate_cancellation(){
    // The linear map (b,t) -> (t,b XOR t) has order three over GF(2).
    let text=borrowed("cx b t\ncx t b\ncx b t\ncx t b\ncx b t\ncx t b");
    let o=Object::parse(&text).unwrap();
    for b in 0..2{for t in 0..2{
        let mut q=QuantumState::new();let wires=q.allocate(5).unwrap();for (wire,value)in [(wires[0],t),(wires[4],b)]{if value==1{q.gate("x",&[wire],None,&[]).unwrap();}}
        let mut bindings=QuantumBindings::default();for (name,wire)in ["t","y","z","x"].iter().zip(&wires){bindings.bundles.insert((*name).into(),vec![*wire]);}bindings.borrowed.push(vec![wires[4]]);let before=q.clone();o.execute_action("a",&mut q,&bindings,false,&[]).unwrap();assert_eq!(q,before);
    }}
}
