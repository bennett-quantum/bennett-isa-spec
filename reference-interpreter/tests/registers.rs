mod support;
use bennett_isa_ref::{ClassicalMachine, ClassicalValue, Object, QuantumRegisters};
use support::{object, rehash};
fn program() -> Object {
    Object::parse(include_str!("fixtures/registers.bisa")).unwrap()
}

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

#[test]
fn r3_typed_cc_controlled_calls_execute_and_restore_history() {
    let ac = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let cc = r#"{"abi":"0.8","kind":"cc","params":["qreg[1]","qreg[1]"],"results":[],"tier":"cc"}"#;
    for forward_reference in [false, true] {
        let body = if forward_reference {
            "fn cc @p (input:qreg[1], q:qreg[1]) () () () ()\n^entry():\nbr ^def()\n^use():\nctl c action_call @a (q) () () ()\nret\n^def():\nyes = const.i1 1\nc = select yes input input\nbr ^use()"
        } else {
            "fn cc @p (c:qreg[1], q:qreg[1]) () () () ()\n^entry():\nctl c action_call @a (q) () () ()\nret"
        };
        let o = rehash(&object(
            &[
                (
                    "a",
                    ac,
                    "action @a (t:wire mut) () () () ()\n^entry():\nx t",
                ),
                ("p", cc, body),
            ],
            "{}",
            "",
        ));
        for c in 0..2 {
            for t in 0..2 {
                let mut regs = QuantumRegisters::new();
                let control = regs.allocate(1).unwrap();
                let target = regs.allocate(1).unwrap();
                regs.prepare(control, &[c == 1]).unwrap();
                regs.prepare(target, &[t == 1]).unwrap();
                let mut m = ClassicalMachine::with_quantum_registers(
                    &o,
                    "p",
                    &[ClassicalValue::Qreg(control), ClassicalValue::Qreg(target)],
                    &[],
                    regs,
                )
                .unwrap();
                let entry = m.snapshot();
                let outcome = m.run(10).unwrap();
                assert_eq!(
                    (outcome.fault, outcome.steps),
                    (None, if forward_reference { 6 } else { 2 })
                );
                let regs = m.quantum_registers();
                assert_eq!(
                    regs.quantum_state()
                        .read_basis(regs.bundle(target).unwrap())
                        .unwrap(),
                    t ^ c
                );
                assert_eq!(
                    regs.quantum_state()
                        .read_basis(regs.bundle(control).unwrap())
                        .unwrap(),
                    c
                );
                m.reverse().unwrap();
                assert_eq!(m.snapshot(), entry);
            }
        }
    }
}

fn dynamic_region_alias_case(kind: &str) {
    use bennett_isa_ref::Budget;
    for tier in ["cc", "r"] {
        let o = support::dynamic_region_alias_object(kind, tier);
        for c in 0..2 {
            let mut regs = QuantumRegisters::new();
            let q = regs.allocate(1).unwrap();
            let r = regs.allocate(1).unwrap();
            regs.prepare(r, &[true]).unwrap();
            let mut args = vec![
                ClassicalValue::int(1, c).unwrap(),
                ClassicalValue::Qreg(q),
                ClassicalValue::Qreg(r),
            ];
            if kind == "oracle" {
                let f = regs.allocate(4).unwrap();
                args.extend([
                    ClassicalValue::Qreg(f),
                    ClassicalValue::Token(Budget::steps(1)),
                ]);
            }
            let before_regs = regs.clone();
            let mut m =
                ClassicalMachine::with_quantum_registers(&o, "p", &args, &[], regs).unwrap();
            let entry = m.snapshot();
            let result = m.run(10).unwrap();
            // select is row 1; the failed call is terminal row 2. The
            // disjoint branch executes CX/X/XOR(1) then ret: 3 rows, q=1.
            assert_eq!(
                (result.fault, result.steps),
                if c == 1 { (Some(4), 2) } else { (None, 3) },
                "{tier}/{kind}"
            );
            assert!(result.values.is_empty());
            let regs = m.quantum_registers();
            if c == 1 {
                assert_eq!(
                    regs, &before_regs,
                    "a failed call must not mutate state or handles"
                );
            } else {
                assert_eq!(
                    regs.quantum_state()
                        .read_basis(regs.bundle(q).unwrap())
                        .unwrap(),
                    1
                );
                assert_eq!(
                    regs.quantum_state()
                        .read_basis(regs.bundle(r).unwrap())
                        .unwrap(),
                    1
                );
            }
            m.reverse().unwrap();
            assert_eq!(m.snapshot(), entry, "{tier}/{kind}/{c}");
        }
    }
}

#[test]
fn r4_dynamic_interface_alias_is_an_atomic_reversible_cc_and_r_fault() {
    dynamic_region_alias_case("interface");
}
#[test]
fn r4_dynamic_control_target_alias_is_an_atomic_reversible_cc_and_r_fault() {
    dynamic_region_alias_case("control");
}
#[test]
fn r4_dynamic_control_can_overlap_a_readonly_input() {
    dynamic_region_alias_case("readonly-control");
}
#[test]
fn r4_dynamic_oracle_input_output_alias_is_an_atomic_reversible_cc_and_r_fault() {
    dynamic_region_alias_case("oracle");
}

#[test]
fn r4_dynamic_alias_checks_cover_regions_block_parameters_and_child_targets() {
    // Mut bundle q[2] and region m[1,1] must be disjoint even if the child
    // is an identity. c=1 aliases both slots; c=0 binds two distinct handles.
    let ac = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[{"A":1,"W":1,"role":"mut"}],"mut":[2],"tier":"q"}"#;
    let ps =
        r#"{"abi":"0.8","kind":"cc","params":["i1","qreg[2]","qreg[2]"],"results":[],"tier":"cc"}"#;
    let o = rehash(&object(&[("a",ac,"action @a (t:wire[2] mut) () (m:region[1,1] mut) () ()\n^entry():\nx t[0]\nx t[0]"),("p",ps,"fn cc @p (c:i1, q:qreg[2], r:qreg[2]) () () () ()\n^entry():\ns = select c q r\naction_call @a (q) () (s) ()\nret")],"{}", "\nsection layout\n{\"A_tab\":0,\"H\":0,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n"));
    for c in 0..2 {
        let mut regs = QuantumRegisters::new();
        let q = regs.allocate(2).unwrap();
        let r = regs.allocate(2).unwrap();
        let before = regs.clone();
        let mut m = ClassicalMachine::with_quantum_registers(
            &o,
            "p",
            &[
                ClassicalValue::int(1, c).unwrap(),
                ClassicalValue::Qreg(q),
                ClassicalValue::Qreg(r),
            ],
            &[],
            regs,
        )
        .unwrap();
        let entry = m.snapshot();
        let outcome = m.run(10).unwrap();
        assert_eq!(
            (outcome.fault, outcome.steps),
            if c == 1 { (Some(4), 2) } else { (None, 3) }
        );
        assert_eq!(m.quantum_registers(), &before);
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), entry);
    }
    // A block parameter is another dynamic alias, and inherited controls
    // must be checked against targets reached through the child action.
    let base = support::dynamic_region_alias_object("control", "r");
    let ac = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let ps =
        r#"{"abi":"0.8","kind":"r","params":["i1","qreg[1]","qreg[1]"],"results":[],"tier":"r"}"#;
    let o = rehash(&object(&[("leaf",ac,"action @leaf (t:wire mut) () () () ()\n^entry():\nx t"),("a",ac,"action @a (t:wire mut) () () () ()\n^entry():\naction_call @leaf (t) () () ()"),("p",ps,"fn r @p (c:i1, q:qreg[1], r:qreg[1]) () () () ()\n^entry():\ns = select c q r\nbr ^invoke(s)\n^invoke(v:qreg[1]):\nctl v action_call @a (q) () () ()\nret")],"{}",""));
    // Also bind q and r directly to the SAME register at the R entry.
    for entry_alias in [false, true] {
        for o in [&base, &o] {
            let mut regs = QuantumRegisters::new();
            let q = regs.allocate(1).unwrap();
            let r = if entry_alias {
                q
            } else {
                regs.allocate(1).unwrap()
            };
            let before = regs.clone();
            let mut m = ClassicalMachine::with_quantum_registers(
                o,
                "p",
                &[
                    ClassicalValue::int(1, 1).unwrap(),
                    ClassicalValue::Qreg(q),
                    ClassicalValue::Qreg(r),
                ],
                &[],
                regs,
            )
            .unwrap();
            let entry = m.snapshot();
            let outcome = m.run(10).unwrap();
            assert_eq!(outcome.fault, Some(4));
            assert_eq!(
                outcome.steps,
                if o.functions.contains_key("leaf") {
                    3
                } else {
                    2
                }
            );
            assert_eq!(m.quantum_registers(), &before);
            m.reverse().unwrap();
            assert_eq!(m.snapshot(), entry);
        }
    }
}
