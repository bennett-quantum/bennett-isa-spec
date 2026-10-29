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

#[test]
fn r2_opaque_borrow_calls_bind_classical_args_and_tokens_through_inlining() {
    use bennett_isa_ref::{Argument, Budget};
    let os = r#"{"abi":"0.8","args":["i1"],"fault":4,"in":[],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let ps = r#"{"abi":"0.8","args":["token","i1","i1"],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[4],"tier":"q"}"#;
    let cs = r#"{"abi":"0.8","args":["token","i1"],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1,4],"tier":"q"}"#;
    let oracle = "oracle @o () (t:wire) (f:wire[4]) () (n:i1)\n^entry():\nlift.retval n";
    let child = "perm_action @child (d:wire mut, f:wire[4] mut) () () (token:token, value:i1) ()\n^entry():\noracle_call @o token () d f () (value)";
    for inline in [false, true] {
        let call = |v| {
            if inline {
                format!("action_call @child (d, f) () () (tk, {v})")
            } else {
                format!("oracle_call @o tk () d f () ({v})")
            }
        };
        let build = |second| {
            let body = format!("perm_action @a (f:wire[4] mut) () () (tk:token, x:i1, y:i1) ()\n^entry():\nd = qborrow any 1\n{}\n{}\nqreturn d", call("x"), call(second));
            support::object(
                &[("o", os, oracle), ("child", cs, child), ("a", ps, &body)],
                "{}",
                "",
            )
        };
        let e = Object::parse(&build("y")).expect_err("x and y are not the same classical binding");
        assert!(e.to_string().contains("§5.6"), "{e}");
        let o = Object::parse(&build("x")).unwrap();
        // d_out=d_in XOR x XOR x=d_in; fault XOR 0 twice, for every f,d.
        for x in 0..2 {
            for f in 0..16 {
                for d in 0..2 {
                    let mut q = QuantumState::new();
                    let fw = q.allocate(4).unwrap();
                    let dw = q.allocate(1).unwrap();
                    for (j, w) in fw.iter().chain(&dw).enumerate() {
                        if (f | (d << 4)) >> j & 1 == 1 {
                            q.gate("x", &[*w], None, &[]).unwrap();
                        }
                    }
                    let before = q.clone();
                    let mut b = QuantumBindings::default();
                    b.bundles.insert("f".into(), fw);
                    b.borrowed.push(dw);
                    b.args
                        .insert("tk".into(), Argument::Token(Budget::steps(1)));
                    b.args.insert("x".into(), Argument::Bits(x));
                    b.args.insert("y".into(), Argument::Bits(1 - x));
                    o.execute_action("a", &mut q, &b, false, &[]).unwrap();
                    assert_eq!(q, before);
                    o.execute_action("a", &mut q, &b, true, &[]).unwrap();
                    assert_eq!(q, before);
                }
            }
        }
    }
}

#[test]
fn r3_opaque_loop_composition_substitutes_inside_applications() {
    use bennett_isa_ref::{Argument, Budget};
    let os =
        r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let ps = r#"{"abi":"0.8","args":["token"],"bounds":[2],"in":[],"kind":"perm_action","mem":[],"mut":[1,4],"tier":"q"}"#;
    let call = "oracle_call @o tk (t) d f () ()";
    let build = |flip: bool, looped: bool| {
        let pair = format!("{call}{}", if flip { "\nx t" } else { "" });
        let body = if looped {
            format!("qloop 2 {{\n{pair}\n}}")
        } else {
            format!("{pair}\n{pair}")
        };
        let sig = if looped {
            ps.to_string()
        } else {
            ps.replace("[2]", "[]")
        };
        let regs = if looped { "regs {  } bounds {2}\n" } else { "" };
        support::object(&[("o",os,"oracle @o (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x"),("a",&sig,&format!("perm_action @a (t:wire mut, f:wire[4] mut) () () (tk:token) ()\n{regs}^entry():\nd = qborrow any 1\n{body}\nqreturn d"))],"{}", "")
    };
    for looped in [true, false] {
        // d' = d XOR t XOR (t XOR 1) = d XOR 1: never a borrow identity.
        support::reject_cli(
            &build(true, looped),
            "§5.6",
            &format!("opaque-loop-{looped}"),
        );
        let o = Object::parse(&build(false, looped)).unwrap();
        // Invariant inputs: d' = d XOR t XOR t = d; all 64 t,f,d values.
        for input in 0..65u32 {
            let mut q = QuantumState::new();
            let w = q.allocate(6).unwrap();
            for (bit, wire) in w.iter().enumerate() {
                if input == 64 {
                    q.gate("h", &[*wire], None, &[]).unwrap();
                } else if input >> bit & 1 == 1 {
                    q.gate("x", &[*wire], None, &[]).unwrap();
                }
            }
            let before = q.clone();
            let mut b = QuantumBindings::default();
            b.bundles.insert("t".into(), vec![w[0]]);
            b.bundles.insert("f".into(), w[1..5].to_vec());
            b.borrowed.push(vec![w[5]]);
            b.args
                .insert("tk".into(), Argument::Token(Budget::steps(1)));
            o.execute_action("a", &mut q, &b, false, &[]).unwrap();
            assert_eq!(q, before);
            o.execute_action("a", &mut q, &b, true, &[]).unwrap();
            assert_eq!(q, before);
        }
    }
}
