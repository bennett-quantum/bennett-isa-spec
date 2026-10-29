//! Council round 1: codex B14, muse B6 (§5.6 has no 16-wire limit).
mod support;
use bennett_isa_ref::{Object, QuantumBindings, QuantumState};

fn perm_action(width: u64, body: &str, bounds: &[u64]) -> String {
    let values: Vec<String> = bounds.iter().map(u64::to_string).collect();
    let signature = format!(
        r#"{{"abi":"0.8","args":[],"bounds":[{}],"in":[1],"kind":"perm_action","mem":[],"mut":[{width}],"tier":"q"}}"#,
        values.join(",")
    );
    let regs = if bounds.is_empty() {
        String::new()
    } else {
        format!("regs {{  }} bounds {{{}}}\n", values.join(", "))
    };
    support::object(
        &[(
            "a",
            &signature,
            &format!("perm_action @a (t:wire[{width}] mut) (x:wire in) () () ()\n{regs}^entry():\nb = qborrow any 1\n{body}\nqreturn b"),
        )],
        "{}",
        "",
    )
}
/// Runs @a with t = `t`, x = `x`, borrowed b = `b`; returns (t, b).
fn run(o: &Object, width: usize, t: u128, x: u128, b: u128) -> (u128, u128) {
    let mut q = QuantumState::new();
    let tw = q.allocate(width).unwrap();
    let xw = q.allocate(1).unwrap();
    let bw = q.allocate(1).unwrap();
    for (wires, value) in [(&tw, t), (&xw, x), (&bw, b)] {
        for (i, w) in wires.iter().enumerate() {
            if value >> i & 1 == 1 {
                q.gate("x", &[*w], None, &[]).unwrap();
            }
        }
    }
    let mut bindings = QuantumBindings::default();
    bindings.bundles.insert("t".into(), tw.clone());
    bindings.bundles.insert("x".into(), xw.clone());
    bindings.borrowed.push(bw.clone());
    o.execute_action("a", &mut q, &bindings, false, &[])
        .unwrap();
    (q.read_basis(&tw).unwrap(), q.read_basis(&bw).unwrap())
}
fn swaps17() -> String {
    let forward: Vec<String> = (0..16).map(|i| format!("swap b t[{i}]")).collect();
    let back: Vec<String> = (0..16).rev().map(|i| format!("swap b t[{i}]")).collect();
    [forward, back].concat().join("\n")
}

#[test]
fn codex_b14_seventeen_wire_swap_identity_frame() {
    let o = Object::parse(&perm_action(16, &swaps17(), &[])).unwrap();
    // Sample the 2^17 domain; the ignored sweep below enumerates all of it.
    for k in 0..512u128 {
        let (t, b) = ((k * 2654435761) & 0xffff, k & 1);
        assert_eq!(run(&o, 16, t, 0, b), (t, b));
    }
}
#[test]
#[ignore = "exhaustive 2^17 execution of the B14 frame; run with --ignored"]
fn codex_b14_seventeen_wire_swap_identity_frame_exhaustive() {
    let o = Object::parse(&perm_action(16, &swaps17(), &[])).unwrap();
    for t in 0..1u128 << 16 {
        for b in 0..2 {
            assert_eq!(run(&o, 16, t, 0, b), (t, b));
        }
    }
}
#[test]
fn muse_b6_eighteen_wire_paired_cx_frame_and_its_dependent_variant() {
    let pairs = |skip_first: bool| {
        (0..17)
            .flat_map(|i| {
                let one = format!("cx b t[{i}]");
                if i == 0 && skip_first {
                    vec![one]
                } else {
                    vec![one.clone(), one]
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    };
    let o = Object::parse(&perm_action(17, &pairs(false), &[])).unwrap();
    for k in 0..256u128 {
        let (t, b) = ((k * 40503) & 0x1ffff, k >> 7 & 1);
        assert_eq!(run(&o, 17, t, 1, b), (t, b));
    }
    // cx b t[0] once: t[0] ^= b depends on the borrowed entry value.
    let e = Object::parse(&perm_action(17, &pairs(true), &[]))
        .unwrap_err()
        .to_string();
    assert!(e.contains("§5.6"), "{e}");
}
#[test]
fn borrow_frames_inside_bounded_loops_are_composed_not_unrolled() {
    // (cx b t[0])^(2^40) = I; the former verifier refused loops above 10^6.
    let o = Object::parse(&perm_action(
        1,
        "qloop 1099511627776 {\ncx b t\n}",
        &[1099511627776],
    ))
    .unwrap();
    assert_eq!(run(&o, 1, 1, 0, 1), (1, 1));
    let e = Object::parse(&perm_action(1, "qloop 3 {\ncx b t\n}", &[3]))
        .unwrap_err()
        .to_string();
    assert!(e.contains("§5.6"), "{e}");
    // The order-3 linear map (b,t) -> (t, b xor t), cubed, restores both wires.
    Object::parse(&perm_action(1, "qloop 3 {\ncx b t\ncx t b\n}", &[3])).unwrap();
    Object::parse(&perm_action(1, "qloop 2 {\ncx b t\ncx t b\n}", &[2])).unwrap_err();
}
