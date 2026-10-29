//! Council round 1: codex B15 (clean release decided exactly, no 1e-24 threshold).
mod support;
use bennett_isa_ref::{ClassicalMachine, Object, QuantumState};

#[test]
fn b15_tiny_rotation_is_not_clean_and_release_changes_nothing() {
    // ry(1e-13)|0> = cos(5e-14)|0> + sin(5e-14)|1>; sin(5e-14) != 0.
    let mut q = QuantumState::new();
    let w = q.allocate(1).unwrap();
    q.gate("ry", &w, Some(1e-13), &[]).unwrap();
    let before = q.clone();
    let e = q.release(&w).unwrap_err().to_string();
    assert!(e.contains("E_QREG_NOT_CLEAN"), "{e}");
    assert_eq!(q, before);
    assert_eq!(q.wire_count(), 1);
    // Even 1e-300: the exact shadow is not a tolerance.
    let mut q = QuantumState::new();
    let w = q.allocate(1).unwrap();
    q.gate("rx", &w, Some(1e-300), &[]).unwrap();
    assert!(q.release(&w).is_err());
}

#[test]
fn b15_exact_adjoints_release_cleanly() {
    for theta in [0.7, 1e-13, -2.5, std::f64::consts::TAU, 1e-300] {
        for gate in ["rx", "ry", "rz"] {
            let mut q = QuantumState::new();
            let w = q.allocate(2).unwrap();
            q.gate("h", &w[..1], None, &[]).unwrap();
            q.gate(gate, &w[1..], Some(theta), &[w[0]]).unwrap();
            q.gate(gate, &w[1..], Some(-theta), &[w[0]]).unwrap();
            q.gate("h", &w[..1], None, &[]).unwrap();
            q.release(&w)
                .unwrap_or_else(|e| panic!("{gate}({theta}): {e}"));
        }
    }
    // h t h ; h tdg h is the identity exactly (sqrt 2 and e^{i pi/4} arithmetic).
    let mut q = QuantumState::new();
    let w = q.allocate(1).unwrap();
    for g in ["h", "t", "h", "h", "tdg", "h"] {
        q.gate(g, &w, None, &[]).unwrap();
    }
    q.release(&w).unwrap();
    // s s s s = I; h s h has the exactly nonzero |1> amplitude (1 - i)/2.
    let mut q = QuantumState::new();
    let w = q.allocate(1).unwrap();
    for g in ["h", "s", "h"] {
        q.gate(g, &w, None, &[]).unwrap();
    }
    assert!(q.release(&w).is_err());
}

#[test]
fn b15_cc_free_of_an_almost_clean_register_traps() {
    let rot_sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let cc_sig = r#"{"abi":"0.8","kind":"cc","params":[],"results":[],"tier":"cc"}"#;
    let rot = "action @rot (t:wire mut) () () () ()\n^entry():\nry 1e-13 t";
    let main = |calls: &str| {
        format!(
            "fn cc @main () () () () ()\n^entry():\nq = qreg.new any 1\n{calls}qreg.free q\nret"
        )
    };
    let unclean = support::rehash(&support::object(
        &[
            ("rot", rot_sig, rot),
            ("main", cc_sig, &main("action_call @rot (q) () () ()\n")),
        ],
        "{}",
        "",
    ));
    let mut m = ClassicalMachine::new(&unclean, "main", &[], &[]).unwrap();
    let r = m.run(10).unwrap();
    // §3.3: an instruction whose precondition fails is a CC trap (FAULT_UB_TRAP).
    assert_eq!(r.fault, Some(4));
    assert!(r.trapped);
    // The handle is still live: the failed free consumed nothing.
    assert_eq!(m.quantum_registers().quantum_state().wire_count(), 1);
    let clean = support::rehash(&support::object(
        &[
            ("rot", rot_sig, rot),
            (
                "main",
                cc_sig,
                &main("action_call @rot (q) () () ()\nadj action_call @rot (q) () () ()\n"),
            ),
        ],
        "{}",
        "",
    ));
    let mut m = ClassicalMachine::new(&clean, "main", &[], &[]).unwrap();
    let entry = m.snapshot();
    let r = m.run(10).unwrap();
    assert_eq!(r.fault, None);
    m.reverse().unwrap();
    assert_eq!(m.snapshot(), entry);
    let _ = Object::parse;
}

#[test]
fn superpositions_have_no_wire_limit() {
    // The former simulator refused more than 24 live wires in a superposition.
    // A 200-wire GHZ state: h on wire 0, then a cx chain; two components.
    let mut q = QuantumState::new();
    let w = q.allocate(200).unwrap();
    q.gate("h", &w[..1], None, &[]).unwrap();
    for i in 1..200 {
        q.gate("cx", &[w[i - 1], w[i]], None, &[]).unwrap();
    }
    assert_eq!(q.exact_support().len(), 2);
    let branches = q.measurement_branches(&w[..1]).unwrap();
    assert_eq!(branches.len(), 2);
    for b in &branches {
        assert!((b.probability - 0.5).abs() < 1e-12);
        // The other 199 wires equal the measured bit on its branch.
        let rest = b.state.read_basis(&w[1..129]).unwrap();
        assert_eq!(rest, if b.bits[0] { u128::MAX } else { 0 });
    }
    for i in (1..200).rev() {
        q.gate("cx", &[w[i - 1], w[i]], None, &[]).unwrap();
    }
    q.gate("h", &w[..1], None, &[]).unwrap();
    q.release(&w).unwrap();
}
