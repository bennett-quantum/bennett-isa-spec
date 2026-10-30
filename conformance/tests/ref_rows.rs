use bennett_isa_conformance::{
    json,
    ref_catalog::Machine,
    ref_language::{Expander, Registry},
};
#[test]
fn assignment_rows_full_domain() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut expander = Expander::new(bytes, Registry::new()).unwrap();
    let mut count = 0;
    for n in 1..=4 {
        for op in [
            "add", "sub", "mul", "and", "or", "xor", "shl", "lshr", "ashr",
        ] {
            let args = json::parse(&format!(r#"[{n},"a","b","slot","w"]"#)).unwrap();
            let leaves = expander
                .expand(
                    &format!("lift.{op}"),
                    args.as_arr().unwrap(),
                    &[("a", n), ("b", n), ("slot", n), ("w", n)],
                )
                .unwrap();
            let m = Machine::compile(
                &bennett_isa_conformance::ref_language::permutation(&leaves).unwrap(),
                &[("a", n), ("b", n), ("slot", n), ("w", n)],
            )
            .unwrap();
            let size = 1u64 << n;
            let mask = size - 1;
            for a in 0..size {
                for b in 0..size {
                    for w in 0..size {
                        for enabled in [false, true] {
                            let v = match op {
                                "add" => a.wrapping_add(b) & mask,
                                "sub" => a.wrapping_sub(b) & mask,
                                "mul" => (a * b) & mask,
                                "and" => a & b,
                                "or" => a | b,
                                "xor" => a ^ b,
                                "shl" => {
                                    if b < n as u64 {
                                        a << b & mask
                                    } else {
                                        0
                                    }
                                }
                                "lshr" => {
                                    if b < n as u64 {
                                        a >> b
                                    } else {
                                        0
                                    }
                                }
                                "ashr" => {
                                    if b < n as u64 {
                                        let signed = ((a << (64 - n)) as i64) >> (64 - n);
                                        (signed >> b) as u64 & mask
                                    } else {
                                        0
                                    }
                                }
                                _ => unreachable!(),
                            };
                            let mut state = m.state();
                            for (name, value) in [("a", a), ("b", b), ("w", w)] {
                                m.set(&mut state, name, value)
                            }
                            let before = state.clone();
                            m.run(&mut state, enabled, false).unwrap();
                            assert_eq!(m.get(&state, "w"), if enabled { v } else { w });
                            assert_eq!(m.get(&state, "slot"), if enabled { w } else { 0 });
                            assert_eq!(m.get(&state, "a"), a);
                            assert_eq!(m.get(&state, "b"), b);
                            m.run(&mut state, enabled, true).unwrap();
                            assert_eq!(state, before);
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(count, 9 * 2 * (8 + 64 + 512 + 4096));
    println!("assignment rows widths 1..4: {count} tuples");
}

#[test]
fn all_assignment_comparators_and_constants() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let mut count = 0;
    for n in 1..=4 {
        let size = 1u64 << n;
        for op in ["eq", "ne", "ult", "ule", "slt", "sle"] {
            let args = json::parse(&format!(r#"[{n},"a","b","slot","w"]"#)).unwrap();
            let bundles = [("a", n), ("b", n), ("slot", 1), ("w", 1)];
            let m = Machine::compile(
                &bennett_isa_conformance::ref_language::permutation(
                    &e.expand(&format!("lift.icmp.{op}"), args.as_arr().unwrap(), &bundles)
                        .unwrap(),
                )
                .unwrap(),
                &bundles,
            )
            .unwrap();
            for a in 0..size {
                for b in 0..size {
                    for w in 0..2 {
                        for enabled in [false, true] {
                            let signed = |v: u64| ((v << (64 - n)) as i64) >> (64 - n);
                            let v = u64::from(match op {
                                "eq" => a == b,
                                "ne" => a != b,
                                "ult" => a < b,
                                "ule" => a <= b,
                                "slt" => signed(a) < signed(b),
                                "sle" => signed(a) <= signed(b),
                                _ => unreachable!(),
                            });
                            let mut state = m.state();
                            m.set(&mut state, "a", a);
                            m.set(&mut state, "b", b);
                            m.set(&mut state, "w", w);
                            let old = state.clone();
                            m.run(&mut state, enabled, false).unwrap();
                            assert_eq!(m.get(&state, "w"), if enabled { v } else { w });
                            assert_eq!(m.get(&state, "slot"), if enabled { w } else { 0 });
                            m.run(&mut state, enabled, true).unwrap();
                            assert_eq!(state, old);
                            count += 1;
                        }
                    }
                }
            }
        }
        for c in 0..size {
            for op in ["const", "label", "addr"] {
                let args = json::parse(&format!(r#"[{n},{c},"slot","w"]"#)).unwrap();
                let bundles = [("slot", n), ("w", n)];
                let m = Machine::compile(
                    &bennett_isa_conformance::ref_language::permutation(
                        &e.expand(&format!("lift.{op}"), args.as_arr().unwrap(), &bundles)
                            .unwrap(),
                    )
                    .unwrap(),
                    &bundles,
                )
                .unwrap();
                for w in 0..size {
                    for enabled in [false, true] {
                        let mut state = m.state();
                        m.set(&mut state, "w", w);
                        let old = state.clone();
                        m.run(&mut state, enabled, false).unwrap();
                        assert_eq!(m.get(&state, "w"), if enabled { c } else { w });
                        assert_eq!(m.get(&state, "slot"), if enabled { w } else { 0 });
                        m.run(&mut state, enabled, true).unwrap();
                        assert_eq!(state, old);
                        count += 1;
                    }
                }
            }
        }
    }
    assert_eq!(count, 10200);
    println!("comparison/literal rows: {count} tuples");
}

#[test]
fn cast_select_mov_and_parallel_alias_rows() {
    use bennett_isa_conformance::ref_language::permutation;
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let mut count = 0;
    for n in 1..=4 {
        let size = 1u64 << n;
        for dest in 1..=4 {
            for op in if dest > n {
                vec!["zext", "sext"]
            } else if dest < n {
                vec!["trunc"]
            } else {
                vec![]
            } {
                let bundles = [("a", n), ("slot", dest), ("w", dest)];
                let args = json::parse(&format!(r#"[{n},{dest},"a","slot","w"]"#)).unwrap();
                let m = Machine::compile(
                    &permutation(
                        &e.expand(&format!("lift.{op}"), args.as_arr().unwrap(), &bundles)
                            .unwrap(),
                    )
                    .unwrap(),
                    &bundles,
                )
                .unwrap();
                for a in 0..size {
                    for w in 0..1u64 << dest {
                        for en in [false, true] {
                            let v = if op == "sext" {
                                ((((a << (64 - n)) as i64) >> (64 - n)) as u64) & ((1 << dest) - 1)
                            } else {
                                a & ((1 << dest) - 1)
                            };
                            let mut s = m.state();
                            m.set(&mut s, "a", a);
                            m.set(&mut s, "w", w);
                            let old = s.clone();
                            m.run(&mut s, en, false).unwrap();
                            assert_eq!(m.get(&s, "w"), if en { v } else { w });
                            assert_eq!(m.get(&s, "slot"), if en { w } else { 0 });
                            m.run(&mut s, en, true).unwrap();
                            assert_eq!(s, old);
                            count += 1;
                        }
                    }
                }
            }
        }
        let bundles = [("a", n), ("b", n), ("p", 1), ("slot", n), ("w", n)];
        let args = json::parse(&format!(r#"[{n},"a","b","p","slot","w"]"#)).unwrap();
        let m = Machine::compile(
            &permutation(
                &e.expand("lift.select", args.as_arr().unwrap(), &bundles)
                    .unwrap(),
            )
            .unwrap(),
            &bundles,
        )
        .unwrap();
        for a in 0..size {
            for b in 0..size {
                for p in 0..2 {
                    for w in 0..size {
                        for en in [false, true] {
                            let mut s = m.state();
                            for (k, v) in [("a", a), ("b", b), ("p", p), ("w", w)] {
                                m.set(&mut s, k, v)
                            }
                            let old = s.clone();
                            m.run(&mut s, en, false).unwrap();
                            assert_eq!(
                                m.get(&s, "w"),
                                if en {
                                    if p == 1 {
                                        a
                                    } else {
                                        b
                                    }
                                } else {
                                    w
                                }
                            );
                            assert_eq!(m.get(&s, "slot"), if en { w } else { 0 });
                            m.run(&mut s, en, true).unwrap();
                            assert_eq!(s, old);
                            count += 1;
                        }
                    }
                }
            }
        }
        // Both heterogeneous sources may alias destinations: reverse two n-bit
        // locations in one flattened parmov. No exchange precedes any capture.
        let bundles = [("w", 2 * n), ("slot", 2 * n)];
        let args=json::parse(&format!(r#"[{},[{{"range":"w","from":{n},"to":{}}},{{"range":"w","from":0,"to":{n}}}],"slot","w"]"#,2*n,2*n)).unwrap();
        let m = Machine::compile(
            &permutation(
                &e.expand("lift.parmov", args.as_arr().unwrap(), &bundles)
                    .unwrap(),
            )
            .unwrap(),
            &bundles,
        )
        .unwrap();
        for w in 0..1u64 << (2 * n) {
            for en in [false, true] {
                let v = (w & (size - 1)) << n | w >> n;
                let mut s = m.state();
                m.set(&mut s, "w", w);
                let old = s.clone();
                m.run(&mut s, en, false).unwrap();
                assert_eq!(m.get(&s, "w"), if en { v } else { w });
                assert_eq!(m.get(&s, "slot"), if en { w } else { 0 });
                m.run(&mut s, en, true).unwrap();
                assert_eq!(s, old);
                count += 1;
            }
        }
        for op in ["mov", "retval"] {
            let bundles = [("a", n), ("slot", n), ("w", n)];
            let args = json::parse(&if op == "mov" {
                format!(r#"[{n},"a","slot","w"]"#)
            } else {
                format!(r#"[{n},"a","w"]"#)
            })
            .unwrap();
            let m = Machine::compile(
                &permutation(
                    &e.expand(&format!("lift.{op}"), args.as_arr().unwrap(), &bundles)
                        .unwrap(),
                )
                .unwrap(),
                &bundles,
            )
            .unwrap();
            for a in 0..size {
                for w in 0..size {
                    for en in [false, true] {
                        let mut s = m.state();
                        m.set(&mut s, "a", a);
                        m.set(&mut s, "w", w);
                        let old = s.clone();
                        m.run(&mut s, en, false).unwrap();
                        assert_eq!(
                            m.get(&s, "w"),
                            if en {
                                if op == "mov" {
                                    a
                                } else {
                                    w ^ a
                                }
                            } else {
                                w
                            }
                        );
                        m.run(&mut s, en, true).unwrap();
                        assert_eq!(s, old);
                        count += 1;
                    }
                }
            }
        }
    }
    assert_eq!(count, 22440);
    println!("cast/select/mov/retval/parmov rows: {count} tuples");
}

#[test]
fn fault_rows_full_domain() {
    use bennett_isa_conformance::ref_language::permutation;
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let mut count = 0;
    for op in ["fault", "lift.faultif"] {
        for c in 0..16 {
            let bundles = [("p", 1), ("flag", 1), ("faulted", 1), ("code", 4)];
            let args = if op == "fault" {
                format!(r#"[4,{c},"faulted","code"]"#)
            } else {
                format!(r#"[4,{c},"p","flag","faulted","code"]"#)
            };
            let args = json::parse(&args).unwrap();
            let m = Machine::compile(
                &permutation(&e.expand(op, args.as_arr().unwrap(), &bundles).unwrap()).unwrap(),
                &bundles,
            )
            .unwrap();
            for p in 0..if op == "fault" { 1 } else { 2 } {
                for f in 0..2 {
                    for code in 0..16 {
                        for en in [false, true] {
                            let active = en && (op == "fault" || p == 1);
                            let mut s = m.state();
                            m.set(&mut s, "p", p);
                            m.set(&mut s, "faulted", f);
                            m.set(&mut s, "code", code);
                            let old = s.clone();
                            m.run(&mut s, en, false).unwrap();
                            assert_eq!(m.get(&s, "faulted"), f ^ u64::from(active));
                            assert_eq!(
                                m.get(&s, "code"),
                                code ^ if active {
                                    if c == 0 {
                                        4
                                    } else {
                                        c
                                    }
                                } else {
                                    0
                                }
                            );
                            assert_eq!(m.get(&s, "flag"), u64::from(op != "fault" && active));
                            m.run(&mut s, en, true).unwrap();
                            assert_eq!(s, old);
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(count, 3072);
    println!("fault rows: {count} tuples");
}

#[test]
fn repeated_location_sources_and_destination_aliases() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut count = 0;
    for n in 1..=4 {
        let size = 1u64 << n;
        for op in [
            "add", "sub", "mul", "and", "or", "xor", "shl", "lshr", "ashr",
        ] {
            let bundles = [("w", n), ("slot", n)];
            let args = json::parse(&format!(r#"[{n},"w","w","slot","w"]"#)).unwrap();
            let m = Machine::compile(
                &bennett_isa_conformance::ref_language::permutation(
                    &e.expand(&format!("lift.{op}"), args.as_arr().unwrap(), &bundles)
                        .unwrap(),
                )
                .unwrap(),
                &bundles,
            )
            .unwrap();
            for w in 0..size {
                for en in [false, true] {
                    let v = match op {
                        "add" => (w + w) & (size - 1),
                        "sub" | "xor" => 0,
                        "mul" => (w * w) & (size - 1),
                        "and" | "or" => w,
                        "shl" => {
                            if w < n as u64 {
                                w << w & (size - 1)
                            } else {
                                0
                            }
                        }
                        "lshr" => {
                            if w < n as u64 {
                                w >> w
                            } else {
                                0
                            }
                        }
                        "ashr" => {
                            if w < n as u64 {
                                ((((w << (64 - n)) as i64) >> (64 - n)) >> w) as u64 & (size - 1)
                            } else {
                                0
                            }
                        }
                        _ => unreachable!(),
                    };
                    let mut s = m.state();
                    m.set(&mut s, "w", w);
                    let old = s.clone();
                    m.run(&mut s, en, false).unwrap();
                    assert_eq!(m.get(&s, "w"), if en { v } else { w });
                    assert_eq!(m.get(&s, "slot"), if en { w } else { 0 });
                    m.run(&mut s, en, true).unwrap();
                    assert_eq!(s, old);
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 540);
    println!("repeated/assigned location aliases: {count} tuples");
}

#[test]
fn positional_copy_exchange_bindings_full_domains() {
    use bennett_isa_conformance::ref_bdd::Bdd;
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut count = 0;
    for n in 1..=4 {
        for kind in ["bind.qxor", "bind.qexch"] {
            let args = json::parse(&format!(r#"[{n},"left","right"]"#)).unwrap();
            let bundles = [("left", n), ("right", n)];
            let m = Machine::compile(
                &bennett_isa_conformance::ref_language::permutation(
                    &e.expand(kind, args.as_arr().unwrap(), &bundles).unwrap(),
                )
                .unwrap(),
                &bundles,
            )
            .unwrap();
            let mut b = Bdd::new();
            let mut s = m.symbolic_state();
            for name in ["left", "right"] {
                for id in &m.inputs[name] {
                    s[*id] = b.variable();
                }
            }
            let enabled = b.variable();
            let old = s.clone();
            let mut expected = old.clone();
            for bit in 0..n {
                let a = m.inputs["left"][bit];
                let t = m.inputs["right"][bit];
                let flip = if kind == "bind.qxor" {
                    b.and(enabled, old[a])
                } else {
                    let d = b.xor(old[a], old[t]);
                    b.and(enabled, d)
                };
                expected[t] = b.xor(expected[t], flip);
                if kind == "bind.qexch" {
                    expected[a] = b.xor(expected[a], flip);
                }
            }
            m.run_symbolic(&mut b, &mut s, enabled, false, 1).unwrap();
            assert_eq!(s, expected);
            m.run_symbolic(&mut b, &mut s, enabled, true, 1).unwrap();
            assert_eq!(s, old);
            count += 1 << (2 * n + 1);
        }
    }
    assert_eq!(count, 1360);
    println!("positional copy/exchange adapters: {count} full-domain tuples");
}
