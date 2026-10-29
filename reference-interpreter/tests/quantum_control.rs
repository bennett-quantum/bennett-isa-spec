use bennett_isa_ref::{Object, QuantumBindings, QuantumRegion, QuantumState};

fn fixture() -> Object {
    let path = format!(
        "{}/tests/fixtures/control_flow.bisa",
        env!("CARGO_MANIFEST_DIR")
    );
    Object::parse(&std::fs::read_to_string(path).expect("canonical control-flow fixture")).unwrap()
}
fn prepare(q: &mut QuantumState, w: &[u64], value: u128) {
    for (bit, wire) in w.iter().enumerate() {
        if value & (1 << bit) != 0 {
            q.gate("x", &[*wire], None, &[]).unwrap();
        }
    }
}
#[test]
fn coherent_while_is_prefix_and_padding_with_an_exact_adjoint() {
    let o = fixture();
    for p in 0..8 {
        for t in 0..2 {
            let mut q = QuantumState::new();
            let target = q.allocate(1).unwrap();
            let selectors = q.allocate(3).unwrap();
            prepare(&mut q, &target, t);
            prepare(&mut q, &selectors, p);
            let mut b = QuantumBindings::default();
            b.bundles.insert("t".into(), target.clone());
            b.bundles.insert("p".into(), selectors.clone());
            let before = q.clone();
            o.execute_action("while3", &mut q, &b, false, &[]).unwrap();
            let prefix_ones = (0..3).take_while(|bit| p & (1 << bit) != 0).count();
            assert_eq!(
                q.read_basis(&target).unwrap(),
                t ^ (prefix_ones as u128 & 1)
            );
            assert_eq!(q.read_basis(&selectors).unwrap(), p);
            assert_eq!(q.wire_count(), 4);
            o.execute_action("while3", &mut q, &b, true, &[]).unwrap();
            assert_eq!(q, before);
        }
    }
    // Superpose all selectors and both target values, retaining coherent branches.
    let mut q = QuantumState::new();
    let wires = q.allocate(4).unwrap();
    for wire in &wires {
        q.gate("h", &[*wire], None, &[]).unwrap();
    }
    let mut b = QuantumBindings::default();
    b.bundles.insert("t".into(), wires[..1].to_vec());
    b.bundles.insert("p".into(), wires[1..].to_vec());
    let before = q.clone();
    o.execute_action("while3", &mut q, &b, false, &[]).unwrap();
    o.execute_action("while3", &mut q, &b, true, &[]).unwrap();
    assert_eq!(q, before);
}

#[test]
fn qif_and_nested_control_preserve_interference() {
    let o = fixture();
    let mut q = QuantumState::new();
    let t = q.allocate(1).unwrap();
    let p = q.allocate(1).unwrap();
    let mut b = QuantumBindings::default();
    b.bundles.insert("t".into(), t.clone());
    b.bundles.insert("p".into(), p.clone());
    q.gate("h", &p, None, &[]).unwrap();
    o.execute_action("conditional", &mut q, &b, false, &[])
        .unwrap();
    let a = q.amplitudes().unwrap();
    let half = 0.5;
    let h = std::f64::consts::FRAC_1_SQRT_2;
    for (actual, expected) in a.iter().zip([0.0, h, half, half]) {
        assert!((actual.re - expected).abs() < 1e-12);
        assert!(actual.im.abs() < 1e-12);
    }
    o.execute_action("conditional", &mut q, &b, true, &[])
        .unwrap();
    q.gate("h", &p, None, &[]).unwrap();
    q.release(&p).unwrap();
    q.release(&t).unwrap();
    let mut q = QuantumState::new();
    let t = q.allocate(1).unwrap();
    let p = q.allocate(1).unwrap();
    let mut b = QuantumBindings::default();
    b.bundles.insert("t".into(), t);
    b.bundles.insert("p".into(), p.clone());
    q.gate("h", &p, None, &[]).unwrap();
    o.execute_action("controlled_phase", &mut q, &b, false, &[])
        .unwrap();
    q.gate("h", &p, None, &[]).unwrap();
    assert!((q.amplitudes().unwrap()[2].re - 1.0).abs() < 1e-12);
}

#[test]
fn interface_exchange_and_little_endian_local_image() {
    let o = fixture();
    for address in 0..2 {
        for target in 0..4 {
            for c0 in 0..4 {
                for c1 in 0..4 {
                    let mut q = QuantumState::new();
                    let t = q.allocate(2).unwrap();
                    let a = q.allocate(1).unwrap();
                    let m0 = q.allocate(2).unwrap();
                    let m1 = q.allocate(2).unwrap();
                    for (w, v) in [(&t, target), (&a, address), (&m0, c0), (&m1, c1)] {
                        prepare(&mut q, w, v);
                    }
                    let mut b = QuantumBindings::default();
                    b.bundles.insert("t".into(), t.clone());
                    b.bundles.insert("a".into(), a);
                    b.regions.insert(
                        "m".into(),
                        QuantumRegion {
                            address_width: 1,
                            cell_width: 2,
                            cells: vec![m0.clone(), m1.clone()],
                        },
                    );
                    let before = q.clone();
                    o.execute_action("exchange", &mut q, &b, false, &[])
                        .unwrap();
                    assert_eq!(
                        q.read_basis(&t).unwrap(),
                        if address == 0 { c0 } else { c1 }
                    );
                    assert_eq!(
                        q.read_basis(&m0).unwrap(),
                        if address == 0 { target } else { c0 }
                    );
                    assert_eq!(
                        q.read_basis(&m1).unwrap(),
                        if address == 1 { target } else { c1 }
                    );
                    o.execute_action("exchange", &mut q, &b, true, &[]).unwrap();
                    assert_eq!(q, before);
                }
            }
        }
    }
    for address in 0..2 {
        let mut q = QuantumState::new();
        let t = q.allocate(8).unwrap();
        let p = q.allocate(1).unwrap();
        prepare(&mut q, &t, 0x35);
        prepare(&mut q, &p, address);
        let mut b = QuantumBindings::default();
        b.bundles.insert("t".into(), t.clone());
        b.bundles.insert("p".into(), p);
        o.execute_action("image", &mut q, &b, false, &[]).unwrap();
        assert_eq!(
            q.read_basis(&t).unwrap(),
            0x35 ^ if address == 0 { 0xde } else { 0xad }
        );
        assert_eq!(q.wire_count(), 9);
        o.execute_action("image", &mut q, &b, true, &[]).unwrap();
        assert_eq!(q.read_basis(&t).unwrap(), 0x35);
    }
}
