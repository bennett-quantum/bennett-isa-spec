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

#[test]
fn r3_nonpermutation_calls_touch_all_effective_controls() {
    let ac = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let build = |sig: &str, child: &str, body: &str| {
        support::object(&[("child",sig,child),("a",ac,&format!("action @a (t:wire mut) () () () ()\n^entry():\nd = qborrow any 1\n{body}\nqreturn d"))],"{}", "")
    };
    let child = "action @child (t:wire mut) () () () ()\n^entry():\nh t";
    for (tag, body) in [
        ("ctl", "ctl d action_call @child (t) () () ()"),
        (
            "inherited",
            "qpred d\naction_call @child (t) () () ()\nqunpred d",
        ),
        ("direct", "qpred d\nh t\nqunpred d"),
    ] {
        support::reject_cli(
            &build(ac, child, body),
            "§5.6",
            &format!("borrow-control-{tag}"),
        );
    }
    let pc = ac.replace("action", "perm_action");
    let child = "perm_action @child (t:wire mut) () () () ()\n^entry():\nx t";
    let o = Object::parse(&build(
        &pc,
        child,
        "ctl d action_call @child (t) () () ()\nctl d action_call @child (t) () () ()",
    ))
    .unwrap();
    // Controlled X twice is identity, including arbitrary borrowed states.
    for input in 0..5u32 {
        let mut q = QuantumState::new();
        let w = q.allocate(2).unwrap();
        for (bit, wire) in w.iter().enumerate() {
            if input == 4 {
                q.gate("h", &[*wire], None, &[]).unwrap();
            } else if input >> bit & 1 == 1 {
                q.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        let before = q.clone();
        let mut b = QuantumBindings::default();
        b.bundles.insert("t".into(), vec![w[0]]);
        b.borrowed.push(vec![w[1]]);
        o.execute_action("a", &mut q, &b, false, &[]).unwrap();
        assert_eq!(q, before);
        o.execute_action("a", &mut q, &b, true, &[]).unwrap();
        assert_eq!(q, before);
    }
}

/// c is identity (X twice), so CX(b,t); c[†]; CX(b,t) is identity on
/// every t,b and on any reference entangled with b. Local images/lifetimes
/// are unconditional even if c is called under a borrowed control.
fn check_local_region_lifetime(declaration: &str, bounds: &[u64], image: bool) {
    let ps = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let cs = ps.replace("\"bounds\":[]", &format!("\"bounds\":{bounds:?}"));
    let regs = if bounds.is_empty() {
        String::new()
    } else {
        "regs {  } bounds {0}\n".into()
    };
    let child = format!(
        "perm_action @c (t:wire mut) () () () ()\n{regs}^entry():\n{declaration}\nx t\nx t"
    );
    for direction in ["", "adj "] {
        for controlled in [false, true] {
            let call = format!("{direction}action_call @c (t) () () ()");
            let call = if controlled {
                format!("qpred b\n{call}\nqunpred b")
            } else {
                call
            };
            let parent = format!("perm_action @a (t:wire mut) () () () ()\n^entry():\nb = qborrow any 1\ncx b t\n{call}\ncx b t\nqreturn b");
            let layout = "\nsection layout\n{\"A_tab\":0,\"H\":0,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";
            let text = support::object_with_data(
                &[("c", &cs, &child), ("a", ps, &parent)],
                if image { &[("image", "ff")] } else { &[] },
                layout,
            );
            let o = support::rehash(&text);
            o.check_hashes().unwrap();
            // Four t,b basis entries, a superposition, and a Bell pair b/ref.
            for input in 0..6u32 {
                let mut q = QuantumState::new();
                let w = q.allocate(3).unwrap();
                if input == 5 {
                    q.gate("h", &[w[1]], None, &[]).unwrap();
                    q.gate("cx", &[w[1], w[2]], None, &[]).unwrap();
                    q.gate("h", &[w[0]], None, &[]).unwrap();
                } else {
                    for (bit, wire) in w[..2].iter().enumerate() {
                        if input == 4 {
                            q.gate("h", &[*wire], None, &[]).unwrap();
                        } else if input >> bit & 1 == 1 {
                            q.gate("x", &[*wire], None, &[]).unwrap();
                        }
                    }
                }
                let before = q.clone();
                let mut bindings = QuantumBindings::default();
                bindings.bundles.insert("t".into(), vec![w[0]]);
                bindings.borrowed.push(vec![w[1]]);
                for adjoint in [false, true] {
                    o.execute_action("a", &mut q, &bindings, adjoint, &[])
                        .unwrap();
                    assert_eq!(q, before, "{declaration}, {direction}, controlled={controlled}, input={input}, adjoint={adjoint}");
                }
            }
            // Pin the reported CLI acceptance and t=0/1 exact reversal too.
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join(".tmp")
                .join(format!(
                    "codex-local-{image}-{}-{}-{controlled}.bisa",
                    bounds.len(),
                    direction.len()
                ));
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, &o.text).unwrap();
            for t in ["0", "1"] {
                for command in ["run-action", "reverse"] {
                    let output = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                        .arg(command)
                        .arg(&path)
                        .args(["a", "--mut", t])
                        .output()
                        .unwrap();
                    assert!(
                        output.status.success(),
                        "{}",
                        String::from_utf8_lossy(&output.stderr)
                    );
                    let value = support::json(std::str::from_utf8(&output.stdout).unwrap());
                    let expected = support::json(&format!(r#"{{"{t}":[1,0]}}"#));
                    let field = if command == "reverse" {
                        "forward_state"
                    } else {
                        "state"
                    };
                    assert_eq!(value.get(field).unwrap(), &expected);
                    if command == "reverse" {
                        assert_eq!(
                            value.get("equals_entry").unwrap(),
                            &bennett_isa_ref::json::Json::Bool(true)
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn codex_r3_b03_adjoint_child_local_region_exact_witness() {
    check_local_region_lifetime("qregion r 1 1", &[], false);
}

#[test]
fn codex_r3_b03_adjoint_child_nonzero_image() {
    check_local_region_lifetime("qregion r 1 4 @image", &[], true);
}

#[test]
fn codex_r3_b03_adjoint_child_zero_trip_region_declaration() {
    // The load precedes the lexical declaration: even a zero-trip declaration
    // creates the region at function entry. The loaded image is restored.
    check_local_region_lifetime("z = qalloc any 1\nv = qload r z\nqunload r z v\nqfree z\nqloop 0 {\nqregion r 1 4 @image\n}", &[0], true);
}

#[test]
fn r4_loop_local_returns_cannot_be_restored_by_later_borrowings() {
    // At EACH return b_out = beta XOR 1. A second borrowing cannot complete
    // the first frame, even if the executor happens to reuse the same pool bit.
    for (tag, body, bounds) in [
        (
            "loop",
            "qloop 2 {\nb = qborrow any 1\nx b\nqreturn b\n}",
            vec![2],
        ),
        (
            "unrolled",
            "b = qborrow any 1\nx b\nqreturn b\nb = qborrow any 1\nx b\nqreturn b",
            vec![],
        ),
        (
            "nested",
            "qloop 2 {\nqloop 2 {\nb = qborrow any 1\nx b\nqreturn b\n}\n}",
            vec![2, 2],
        ),
        ("single", "b = qborrow any 1\nx b\nqreturn b", vec![]),
        // b is restored but the output depends on beta at each return.
        (
            "dependent",
            "qloop 2 {\nb = qborrow any 1\ncx b t\nqreturn b\n}",
            vec![2],
        ),
    ] {
        support::reject_cli(
            &support::borrow_lifetime_object(body, &bounds),
            "§5.6",
            &format!("r4-borrow-{tag}"),
        );
    }
}

#[test]
fn r4_borrow_lifetimes_inside_and_across_loops_preserve_full_state() {
    for (body, bounds) in [
        (
            "d = qalloc any 1\nb = qborrow any 1\nx b\nx b\nqfree d\nqreturn b",
            vec![],
        ),
        ("b = qborrow any 1\nqloop 2 {\nx b\n}\nqreturn b", vec![2]),
        (
            "qloop 2 {\nb = qborrow any 1\nx b\nx b\nqreturn b\n}",
            vec![2],
        ),
        (
            "qloop 2 {\nqloop 2 {\nb = qborrow any 1\nx b\nx b\nqreturn b\n}\n}",
            vec![2, 2],
        ),
        (
            "qloop 2 {\nb = qborrow any 1\nqloop 2 {\nx b\n}\nqreturn b\n}",
            vec![2, 2],
        ),
    ] {
        let o = support::rehash(&support::borrow_lifetime_object(body, &bounds));
        o.check_hashes().unwrap();
        support::borrow_identity(&o);
    }
}

#[test]
fn muse_r4_b01_each_iteration_has_an_independent_borrowed_entry() {
    // Each return must factor on its OWN arbitrary beta_i. For d->t,
    // t' = t XOR beta_i; for t->d, d' = beta_i XOR t. Both fail for
    // one iteration regardless of the parity of the surrounding loop count.
    for gate in ["cx d t", "cx t d"] {
        let lifetime = format!("d = qborrow any 1\n{gate}\nqreturn d");
        for (tag, body, bounds) in [
            ("single", lifetime.clone(), vec![]),
            ("two", format!("qloop 2 {{\n{lifetime}\n}}"), vec![2]),
            ("four", format!("qloop 4 {{\n{lifetime}\n}}"), vec![4]),
            (
                "nested",
                format!("qloop 2 {{\nqloop 2 {{\n{lifetime}\n}}\n}}"),
                vec![2, 2],
            ),
            (
                "unrolled",
                format!("{lifetime}\n{}", lifetime.replace('d', "e")),
                vec![],
            ),
        ] {
            support::reject_cli(
                &support::borrow_lifetime_object(&body, &bounds),
                "§5.6",
                &format!("muse-r4-{tag}-{}", gate.replace(' ', "-")),
            );
        }
    }
}

#[test]
fn muse_r4_b01_carried_and_local_borrow_cx_identities_have_no_trip_cap() {
    for gate in ["cx d t", "cx t d"] {
        let spanning = format!("d = qborrow any 1\nqloop 2 {{\n{gate}\n}}\nqreturn d");
        let o = support::rehash(&support::borrow_lifetime_object(&spanning, &[2]));
        support::borrow_identity(&o);
        for trips in [2, 1_099_511_627_776u64] {
            let body =
                format!("qloop {trips} {{\nd = qborrow any 1\n{gate}\n{gate}\nqreturn d\n}}");
            let o = support::rehash(&support::borrow_lifetime_object(&body, &[trips]));
            o.check_hashes().unwrap();
            support::borrow_identity(&o);
        }
    }
}

#[test]
fn sol_r5_b02_clean_surrounding_loop_controls_are_reachable_invariants() {
    for n in [1, 2, 3, 1 << 40] {
        for nested in [false, true] {
            let inner = format!("qloop {n} {{\nb = qborrow any 1\ncx d b\nqreturn b\n}}");
            let (loops, bounds) = if nested {
                (format!("qloop 3 {{\n{inner}\n}}"), vec![3, n])
            } else {
                (inner, vec![n])
            };
            // d=0 is unchanged: EACH CX is I on arbitrary/entangled beta_i.
            let body = format!("d = qalloc any 1\n{loops}\nqfree d");
            let o = support::rehash(&support::borrow_lifetime_object(&body, &bounds));
            support::borrow_identity(&o);
            let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
            let path = dir.join(format!("r5-clean-{n}-{nested}.bisa"));
            std::fs::write(&path, &o.text).unwrap();
            for cmd in ["hash", "check"] {
                let out = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                    .arg(cmd)
                    .arg(&path)
                    .output()
                    .unwrap();
                assert!(out.status.success(), "{out:?}");
            }
            // d=1 makes each return invalid, including at even trip counts.
            let bad = format!("d = qalloc any 1\nx d\n{loops}\nx d\nqfree d");
            support::reject_cli(
                &support::borrow_lifetime_object(&bad, &bounds),
                "§5.6",
                &format!("r5-dirty-{n}-{nested}"),
            );
            if n <= 3 {
                let unrolled = (0..n)
                    .map(|i| format!("b{i} = qborrow any 1\ncx d b{i}\nqreturn b{i}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                support::borrow_identity(&support::rehash(&support::borrow_lifetime_object(
                    &format!("d = qalloc any 1\n{unrolled}\nqfree d"),
                    &[],
                )));
            }
        }
    }
    // Reachability must include later iterations: d=0 on the first entry,
    // then toggles to 1. A first-iteration-only check would admit this.
    support::reject_cli(
        &support::borrow_lifetime_object(
            "d = qalloc any 1\nqloop 2 {\nb = qborrow any 1\ncx d b\nqreturn b\nx d\n}\nqfree d",
            &[2],
        ),
        "§5.6",
        "r5-later-entry",
    );
}

#[test]
fn codex_r5_b03_one_iteration_release_matches_unrolled_clean_entry() {
    for (i, (body, bounds, hash)) in [
        (
            "c = qalloc any 1\nb = qborrow any 1\nqloop 1 {\nqfree c\n}\nqreturn b",
            vec![1],
            "a42079194137127e703185faaeb6b5bc85c6941ccc03988eb67c19c14481e8df",
        ),
        (
            "c = qalloc any 1\nb = qborrow any 1\nqfree c\nqreturn b",
            vec![],
            "77ec83b71a86ed5ecff50d17e3eafe79fc371da7ff553ab53eb2de7a19abd218",
        ),
        (
            "c = qalloc any 1\nqloop 1 {\nb = qborrow any 1\nqfree c\nqreturn b\n}",
            vec![1],
            "1f7845d8ea763f50cbd05294dacce1f1c8de52f18a506a240ffee4b909ce7712",
        ),
    ]
    .iter()
    .enumerate()
    {
        // c is exactly zero, so freeing it once is legal; b never changes.
        let o = support::rehash(&support::borrow_lifetime_object(body, bounds));
        assert_eq!(o.identity().unwrap().interface_hashes["a"], *hash);
        o.check_hashes().unwrap();
        support::borrow_identity(&o);
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!(".tmp/codex-r5-b03-{i}.bisa"));
        std::fs::write(&path, &o.text).unwrap();
        for cmd in ["hash", "check", "run-action", "reverse"] {
            for t in ["0", "1"] {
                let mut command = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"));
                command.arg(cmd).arg(&path);
                if matches!(cmd, "run-action" | "reverse") {
                    command.args(["a", "--mut", t]);
                }
                let out = command.output().unwrap();
                assert!(out.status.success(), "{cmd}, {out:?}");
                if cmd == "run-action" {
                    let result = support::json(std::str::from_utf8(&out.stdout).unwrap());
                    assert_eq!(
                        result.get("state").unwrap().canonical(),
                        format!("{{\"{t}\":[1,0]}}")
                    );
                } else if cmd == "reverse" {
                    assert!(String::from_utf8_lossy(&out.stdout).contains("\"equals_entry\":true"));
                }
            }
        }
        // The same release with c=1 is unclean, irrespective of the wrapper.
        let dirty = body.replace("c = qalloc any 1", "c = qalloc any 1\nx c");
        assert!(Object::parse(&support::borrow_lifetime_object(&dirty, bounds)).is_err());
    }
}

#[test]
fn r6_sol_boundary_exact() {
    let (text, h, oh) = support::r6_witness("sol-boundary");
    let o = Object::parse(&text).unwrap();
    let id = o.identity().unwrap();
    assert_eq!(id.interface_hashes["a"], h);
    assert_eq!(id.closure_hashes["a"].as_deref(), Some(h));
    assert_eq!(id.object_hash, oh);
    o.check_hashes().unwrap();
    support::r6_map(&o, false);
}

#[test]
fn r6_codex_boundary_exact() {
    let (text, h, oh) = support::r6_witness("codex-boundary");
    let o = Object::parse(&text).unwrap();
    let id = o.identity().unwrap();
    assert_eq!(id.interface_hashes["a"], h);
    assert_eq!(id.closure_hashes["a"].as_deref(), Some(h));
    assert_eq!(id.object_hash, oh);
    o.check_hashes().unwrap();
    support::r6_map(&o, true);
}

#[test]
fn r6_boundary_variants_retain_clean_controls_and_reject_dirty_frames() {
    for (body, bounds, h) in [
        ("c = qalloc any 1\nb = qborrow any 1\nh t\ncx c b\nqreturn b\nh t\nqfree c", vec![], false),
        ("c = qalloc any 1\nh t\nb = qborrow any 1\nx b\nx b\nqreturn b\nqfree c", vec![], true),
        ("c = qalloc any 1\nqloop 3 {\nh t\nb = qborrow any 1\ncx c b\nqreturn b\nh t\n}\nqfree c", vec![3], false),
        ("c = qalloc any 1\nh t\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c\nh t", vec![], false),
        // Facts about c are invalidated by H c: this valid cleanup uses H c
        // again, but borrowed restoration is independent of that unknown c.
        ("c = qalloc any 1\nh c\nb = qborrow any 1\nx b\nx b\nqreturn b\nh c\nqfree c", vec![], false),
    ] {
        let text = support::borrow_lifetime_object(body, &bounds).replace("perm_action", "action");
        let o = support::rehash(&text);
        o.check_hashes().unwrap();
        support::r6_map(&o, h);
    }
    for body in [
        "c = qalloc any 1\nx c\nh t\nb = qborrow any 1\ncx c b\nqreturn b\nh t\nx c\nqfree c",
        "c = qalloc any 1\nh t\nb = qborrow any 1\nx b\nqreturn b\nh t\nqfree c",
        "c = qalloc any 1\nh t\nb = qborrow any 1\nqreturn b\nx c\nqfree c",
        "c = qalloc any 1\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nh c\nqfree c",
    ] {
        for certified in [false, true] {
            let text = if certified {
                support::certified_borrow_identity_kind(body, &[], "action")
            } else {
                support::borrow_lifetime_object(body, &[]).replace("perm_action", "action")
            };
            let e = Object::parse(&text).unwrap_err().to_string();
            assert!(e.contains("§5.6") || e.contains("§6.2"), "{e}");
        }
    }
}

#[test]
fn r7_restored_zero_before_borrow() {
    let (text, h, oh) = support::r7_witness("restored");
    let o = Object::parse(&text).unwrap();
    let id = o.identity().unwrap();
    assert_eq!(id.interface_hashes["a"], h);
    assert_eq!(id.closure_hashes["a"].as_deref(), Some(h));
    if !oh.is_empty() {
        assert_eq!(id.object_hash, oh);
    }
    o.check_hashes().unwrap();
    support::r6_map(&o, false);
}

#[test]
fn r7_nonpermutation_loop_positive() {
    let (text, h, oh) = support::r7_witness("loop-good");
    let o = Object::parse(&text).unwrap();
    let id = o.identity().unwrap();
    assert_eq!(id.interface_hashes["a"], h);
    assert_eq!(id.closure_hashes["a"].as_deref(), Some(h));
    if !oh.is_empty() {
        assert_eq!(id.object_hash, oh);
    }
    o.check_hashes().unwrap();
    support::r6_map(&o, false);
}

#[test]
fn r7_diagonal_basis() {
    let (text, h, oh) = support::r7_witness("diagonal");
    let o = Object::parse(&text).unwrap();
    let id = o.identity().unwrap();
    assert_eq!(id.interface_hashes["a"], h);
    assert_eq!(id.closure_hashes["a"].as_deref(), Some(h));
    if !oh.is_empty() {
        assert_eq!(id.object_hash, oh);
    }
    o.check_hashes().unwrap();
    support::r6_map(&o, false);
}

#[test]
fn r7_readonly_control() {
    let (text, h, oh) = support::r7_witness("control");
    let o = Object::parse(&text).unwrap();
    let id = o.identity().unwrap();
    assert_eq!(id.interface_hashes["a"], h);
    assert_eq!(id.closure_hashes["a"].as_deref(), Some(h));
    if !oh.is_empty() {
        assert_eq!(id.object_hash, oh);
    }
    o.check_hashes().unwrap();
    support::r6_map(&o, false);
}

#[test]
fn r7_reject_loop_bad() {
    let (text, _, _) = support::r7_witness("loop-bad");
    let e = Object::parse(&text).unwrap_err().to_string();
    assert!(e.contains("§5.6"), "{e}");
}

#[test]
fn r7_reject_rotate_4() {
    let (text, _, _) = support::r7_witness("rotate-4");
    let e = Object::parse(&text).unwrap_err().to_string();
    assert!(e.contains("§5.6"), "{e}");
}

#[test]
fn r7_reject_rotate_8() {
    let (text, _, _) = support::r7_witness("rotate-8");
    let e = Object::parse(&text).unwrap_err().to_string();
    assert!(e.contains("§5.6"), "{e}");
}

#[test]
fn r7_zero_trip_is_not_an_intervening_inverse_operand_touch() {
    // No instruction in the zero-trip body executes; H/H therefore restores c.
    let body = "c = qalloc any 1\nh c\nqloop 0 {\nx c\n}\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c";
    let text = support::borrow_lifetime_object(body, &[0]).replace("perm_action", "action");
    let o = Object::parse(&text).unwrap();
    support::r6_map(&o, false);
}

#[test]
fn r8_compound_adjoint_restores_borrow_control() {
    r8_identity("nested-x");
}
#[test]
fn r8_diagonal_nested_adjoint_restores_borrow_control() {
    r8_identity("nested-s");
}
fn r8_identity(case: &str) {
    let (text, h, oh) = support::r8_witness(case);
    let o = Object::parse(&text).unwrap();
    let id = o.identity().unwrap();
    assert_eq!(id.interface_hashes["a"], h);
    assert_eq!(id.closure_hashes["a"].as_deref(), Some(h));
    assert_eq!(id.object_hash, oh);
    o.check_hashes().unwrap();
    support::r6_map(&o, false);
}
