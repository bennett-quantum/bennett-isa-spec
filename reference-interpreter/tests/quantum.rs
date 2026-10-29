use bennett_isa_ref::{Complex, Object, QuantumBindings, QuantumState};

fn close(a: Complex, b: Complex) {
    assert!(
        (a.re - b.re).abs() < 1e-12 && (a.im - b.im).abs() < 1e-12,
        "{a:?} != {b:?}"
    );
}
#[test]
fn bell_state_born_branches_and_adjoint() {
    let mut q = QuantumState::new();
    let wires = q.allocate(2).unwrap();
    q.gate("h", &[wires[0]], None, &[]).unwrap();
    q.gate("cx", &wires, None, &[]).unwrap();
    let a = q.amplitudes().unwrap();
    let h = std::f64::consts::FRAC_1_SQRT_2;
    close(a[0], Complex::new(h, 0.0));
    close(a[3], Complex::new(h, 0.0));
    close(a[1], Complex::ZERO);
    close(a[2], Complex::ZERO);
    let branches = q.measurement_branches(&wires).unwrap();
    assert_eq!(branches.len(), 2);
    for b in branches {
        assert!((b.probability - 0.5).abs() < 1e-12);
        assert_eq!(b.bits[0], b.bits[1]);
        assert_eq!(b.state.wire_count(), 0);
    }
    q.gate("cx", &wires, None, &[]).unwrap();
    q.gate("h", &[wires[0]], None, &[]).unwrap();
    q.release(&wires).unwrap();
    assert_eq!(q.wire_count(), 0);
}
#[test]
fn every_gate_and_controlled_global_phase() {
    for (op, n, angle, inverse) in [
        ("x", 1, None, "x"),
        ("h", 1, None, "h"),
        ("s", 1, None, "sdg"),
        ("t", 1, None, "tdg"),
        ("rx", 1, Some(0.7), "rx"),
        ("ry", 1, Some(-0.2), "ry"),
        ("rz", 1, Some(1.1), "rz"),
        ("cx", 2, None, "cx"),
        ("ccx", 3, None, "ccx"),
        ("swap", 2, None, "swap"),
        ("cz", 2, None, "cz"),
    ] {
        let mut q = QuantumState::new();
        let w = q.allocate(n + 1).unwrap();
        for wire in &w {
            q.gate("h", &[*wire], None, &[]).unwrap();
        }
        let before = q.amplitudes().unwrap();
        q.gate(op, &w[..n], angle, &[w[n]]).unwrap();
        q.gate(inverse, &w[..n], angle.map(|x| -x), &[w[n]])
            .unwrap();
        for (a, b) in q.amplitudes().unwrap().into_iter().zip(before) {
            close(a, b);
        }
    }
    // Controlled Rz(2*pi) is -I on its enabled subspace; global phase matters.
    let mut q = QuantumState::new();
    let w = q.allocate(2).unwrap();
    q.gate("h", &[w[0]], None, &[]).unwrap();
    q.gate("rz", &[w[1]], Some(std::f64::consts::TAU), &[w[0]])
        .unwrap();
    q.gate("h", &[w[0]], None, &[]).unwrap();
    close(q.amplitudes().unwrap()[1], Complex::new(1.0, 0.0));
}
#[test]
fn permutation_fast_path_and_clean_release() {
    let mut q = QuantumState::new();
    let w = q.allocate(100).unwrap();
    q.gate("x", &[w[99]], None, &[]).unwrap();
    q.gate("mcx", &[w[99], w[3]], None, &[]).unwrap();
    assert!(q.is_basis());
    assert_eq!(q.read_basis(&[w[3]]).unwrap(), 1);
    assert!(q.release(&w).is_err());
    q.gate("mcx", &[w[99], w[3]], None, &[]).unwrap();
    q.gate("x", &[w[99]], None, &[]).unwrap();
    q.release(&w).unwrap();
}
#[test]
fn actions_copy_cleanup_quantum_memory_loops_and_adjoint() {
    let o = Object::parse(include_str!("fixtures/actions.bisa")).unwrap();
    for name in ["flip", "memory", "loop", "bell"] {
        let mut q = QuantumState::new();
        let t = q.allocate(2).unwrap();
        let p = q.allocate(2).unwrap();
        q.gate("x", &[p[0]], None, &[]).unwrap();
        let mut b = QuantumBindings::default();
        b.bundles.insert("t".into(), t.clone());
        b.bundles.insert("p".into(), p.clone());
        let before = q.amplitudes().unwrap();
        o.execute_action(name, &mut q, &b, false, &[]).unwrap();
        o.execute_action(name, &mut q, &b, true, &[]).unwrap();
        for (a, b) in q.amplitudes().unwrap().into_iter().zip(before) {
            close(a, b);
        }
    }
}

#[test]
fn lifted_permutation_calls_use_the_action_and_its_adjoint() {
    use bennett_isa_ref::{Budget, OracleInput, OracleMachine};
    let o = Object::parse(include_str!("fixtures/actions.bisa")).unwrap();
    for x in 0..4 {
        let input = OracleInput::bits(&[x]);
        let budget = Budget::steps(5);
        let result = o.run_oracle("lifted", &input, budget).unwrap();
        assert_eq!((result.payload, result.code), (x ^ 1, 0));
        let mut m = OracleMachine::new(&o, "lifted", &input, budget).unwrap();
        let before = m.snapshot();
        for _ in 0..5 {
            m.forward_step().unwrap();
        }
        assert_eq!(m.outcome().unwrap(), result);
        for _ in 0..5 {
            m.inverse_step().unwrap();
        }
        assert_eq!(m.snapshot(), before);
    }
}

#[test]
fn wide_structured_bundles_are_not_limited_to_integer_widths() {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[],"in":[256],"kind":"perm_action","mem":[],"mut":[256],"tier":"q"}"#;
    let text=format!("bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{{\"0\":{{\"interface_hash\":\"{}\",\"kind\":\"perm_action\",\"name\":\"wide\",\"signature\":{signature}}}}}\n\nsection functions\nperm_action @wide (t:wire[256] mut) (p:wire[256] in) () () ()\n^entry():\nd = qcopy p\nqxor d t\nquncopy p d\n","0".repeat(64));
    let object = Object::parse(&text).unwrap();
    let mut q = QuantumState::new();
    let t = q.allocate(256).unwrap();
    let p = q.allocate(256).unwrap();
    q.gate("x", &[p[255]], None, &[]).unwrap();
    let mut b = QuantumBindings::default();
    b.bundles.insert("t".into(), t.clone());
    b.bundles.insert("p".into(), p);
    object
        .execute_action("wide", &mut q, &b, false, &[])
        .unwrap();
    assert_eq!(q.read_basis(&[t[0], t[128], t[255]]).unwrap(), 4);
    assert_eq!(q.wire_count(), 512);
    object
        .execute_action("wide", &mut q, &b, true, &[])
        .unwrap();
    assert_eq!(q.read_basis(&[t[255]]).unwrap(), 0);
}

#[test]
fn sol_r4_b01_tiny_projection_preserves_entangled_residual_and_second_born_weights() {
    // The rare branch of Rx(theta)|1> is correlated with a residual Bell pair.
    // Its conditional state has two equal finite amplitudes, and measuring
    // one Bell wire subsequently has probabilities 1/2,1/2, despite the
    // unnormalised exact shadow having a norm below binary64 range.
    for angle in [1e-162, 1e-200, f64::from_bits(1)] {
        let mut q = QuantumState::new();
        let w = q.allocate(3).unwrap();
        q.gate("x", &[w[0]], None, &[]).unwrap();
        q.gate("rx", &[w[0]], Some(angle), &[]).unwrap();
        q.gate("h", &[w[1]], None, &[]).unwrap();
        q.gate("cx", &[w[1], w[2]], None, &[]).unwrap();
        // Entangle the measured system with the residual pair (odd parity on
        // the common branch, even parity on the rare branch).
        q.gate("cx", &[w[0], w[1]], None, &[]).unwrap();
        let first = q.measurement_branches(&[w[0]]).unwrap();
        assert_eq!(first.len(), 2);
        assert_eq!(first[0].bits, [false]);
        let residual = &first[0].state;
        let amps = residual.amplitudes().unwrap();
        assert!(amps.iter().all(|a| a.re.is_finite() && a.im.is_finite()));
        assert!((amps.iter().map(|a| a.re * a.re + a.im * a.im).sum::<f64>() - 1.0).abs() < 1e-12);
        let second = residual.measurement_branches(&[w[1]]).unwrap();
        assert_eq!(second.len(), 2);
        for branch in second {
            assert!((branch.probability - 0.5).abs() < 1e-12);
            assert_eq!(
                branch.state.read_basis(&[w[2]]).unwrap(),
                u128::from(branch.bits[0])
            );
            let a = branch.state.amplitudes().unwrap();
            assert!((a.iter().map(|z| z.re * z.re + z.im * z.im).sum::<f64>() - 1.0).abs() < 1e-12);
        }
    }
}

#[test]
fn sol_r4_b01_projection_recovers_amplitudes_below_binary64_range() {
    // Two rare independent rotations give amplitude about 2.5e-401 on 00.
    // It still projects to a normalized residual |+>, and applying H cleans it.
    let mut q = QuantumState::new();
    let w = q.allocate(3).unwrap();
    for wire in &w[..2] {
        q.gate("x", &[*wire], None, &[]).unwrap();
        q.gate("rx", &[*wire], Some(1e-200), &[]).unwrap();
    }
    q.gate("h", &[w[2]], None, &[]).unwrap();
    let branches = q.measurement_branches(&w[..2]).unwrap();
    assert_eq!(branches.len(), 4);
    let mut rare = branches[0].state.clone();
    for a in rare.amplitudes().unwrap() {
        assert!((a.re.abs() - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12);
        assert!(a.im.abs() < 1e-12);
    }
    rare.gate("h", &[w[2]], None, &[]).unwrap();
    assert_eq!(rare.read_basis(&[w[2]]).unwrap(), 0);
    rare.release(&[w[2]]).unwrap();
}
