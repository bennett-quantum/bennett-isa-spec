mod support;
use bennett_isa_ref::{ClassicalMachine, ClassicalValue, Object, QuantumRegisters};
use support::{object, rehash};
fn program() -> Object { Object::parse(include_str!("fixtures/registers.bisa")).unwrap() }

#[test]
fn register_handles_enforce_views_consumption_and_cleanliness() {
    let mut r = QuantumRegisters::new();
    let q = r.allocate(2).unwrap();
    let s = r.slice(q, 0, 1).unwrap();
    assert!(r.bundle(q).is_err());
    assert!(r.free(s).is_err());
    assert!(r.slice(s, 0, 1).is_err());
    r.complete_call(&[s]).unwrap();
    assert!(r.bundle(s).is_err());
    assert_eq!(r.bundle(q).unwrap().len(), 2);
    let b = r.allocate(1).unwrap();
    let joined = r.concat(q, b).unwrap();
    assert!(r.bundle(q).is_err());
    assert!(r.bundle(b).is_err());
    assert_eq!(r.bundle(joined).unwrap().len(), 3);
    r.free(joined).unwrap();
    assert!(r.bundle(joined).is_err());
}
#[test]
fn allocation_views_and_region_calls_reverse_with_their_frames() {
    let p = program();
    for name in ["allocation", "views"] {
        let mut m = ClassicalMachine::new(&p, name, &[], &[]).unwrap();
        let before = m.snapshot();
        assert_eq!(m.run(100).unwrap().fault, None);
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), before, "{name}");
    }
    let mut m = ClassicalMachine::new(&p, "unclean", &[], &[]).unwrap();
    let before = m.snapshot();
    assert_eq!(m.run(100).unwrap().fault, Some(4));
    m.reverse().unwrap();
    assert_eq!(m.snapshot(), before);
}
#[test]
fn measurement_uses_born_branches_and_prevents_uncall_and_reversal() {
    let p = program();
    for (draw, expected) in [(0.1, 0), (0.9, 3)] {
        let mut m = ClassicalMachine::new(&p, "measurement", &[], &[]).unwrap();
        m.push_measurement_draw(draw).unwrap();
        let r = m.run(100).unwrap();
        assert_eq!(r.values, vec![ClassicalValue::int(2, expected).unwrap()]);
        assert!(m.reverse().is_err());
    }
    let mut m = ClassicalMachine::new(&p, "bad_uncall", &[], &[]).unwrap();
    m.push_measurement_draw(0.1).unwrap();
    assert_eq!(m.run(100).unwrap().fault, Some(4));
    assert!(m.reverse().is_err());
}

#[test]
fn viewed_parent_and_view_only_one_call_are_static_errors() {
    let p = program();
    for replacement in [
        "qreg.free q\naction_call @toggle (s) () () ()",
        "qreg.free s\naction_call @toggle (s) () () ()",
        "action_call @toggle (s) () () ()\naction_call @toggle (s) () () ()",
    ] {
        let bad = p
            .text
            .replacen("action_call @toggle (s) () () ()", replacement, 1);
        let error = Object::parse(&bad).expect_err("invalid qreg view lifetime accepted");
        assert!(error.to_string().contains("§3.3"), "{error}");
    }
}

#[test]
fn external_register_binding_oracle_xor_and_history_inverse() {
    use bennett_isa_ref::Budget;
    let inc = Object::parse(include_str!("fixtures/inc8.bisa")).unwrap();
    let sig = r#"{"abi":"0.8","kind":"cc","params":["qreg[8]","qreg[8]","qreg[4]","token"],"results":[],"tier":"cc"}"#;
    let imports = format!(
        "{{\"inc8\":{{\"signature\":{}}}}}",
        inc.symbols().unwrap()["0"]
            .get("signature")
            .unwrap()
            .canonical()
    );
    let caller=rehash(&object(&[("invoke",sig,"fn cc @invoke (x:qreg[8], t:qreg[8], f:qreg[4], tk:token) () () () ()\n^entry():\noracle_call @inc8 tk (x) t f () ()\nret")],&imports,""));
    let p = Object::link(&[caller, inc]).unwrap();
    for steps in [2, 3] {
        let mut regs = QuantumRegisters::new();
        let x = regs.allocate(8).unwrap();
        let t = regs.allocate(8).unwrap();
        let f = regs.allocate(4).unwrap();
        regs.prepare(x, &[true, true, true, false, false, false, false, false])
            .unwrap();
        regs.prepare(t, &[true, true, true, true, true, false, false, false])
            .unwrap();
        regs.prepare(f, &[true, true, false, false]).unwrap();
        assert!(regs.prepare(x, &[false; 8]).is_err());
        let args = [
            ClassicalValue::Qreg(x),
            ClassicalValue::Qreg(t),
            ClassicalValue::Qreg(f),
            ClassicalValue::Token(Budget::steps(steps)),
        ];
        let mut m =
            ClassicalMachine::with_quantum_registers(&p, "invoke", &args, &[], regs).unwrap();
        let before = m.snapshot();
        m.run(100).unwrap();
        let r = m.quantum_registers();
        assert_eq!(
            r.quantum_state().read_basis(r.bundle(t).unwrap()).unwrap(),
            if steps == 3 { 31 ^ 8 } else { 31 }
        );
        assert_eq!(
            r.quantum_state().read_basis(r.bundle(f).unwrap()).unwrap(),
            if steps == 3 { 3 } else { 2 }
        );
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), before);
    }
}
