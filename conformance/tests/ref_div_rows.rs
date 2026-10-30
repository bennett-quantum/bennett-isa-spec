use bennett_isa_conformance::{
    json,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
#[test]
fn division_rows_preserve_arbitrary_destinations_and_fault_entries() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let mut count = 0;
    for n in 1..=4 {
        let size = 1u64 << n;
        let signed = |v: u64| ((v << (64 - n)) as i64) >> (64 - n);
        for op in ["udiv", "urem", "sdiv", "srem"] {
            let bundles = [
                ("a", n),
                ("b", n),
                ("slot", n),
                ("w", n),
                ("flag", 1),
                ("faulted", 1),
                ("code", 4),
            ];
            let args = json::parse(&format!(
                r#"[{n},"a","b","slot","w","flag","faulted","code"]"#
            ))
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
            let mut packed = m.packed_state();
            let mut expected = Vec::new();
            let mut enables = 0;
            for a in 0..size {
                for b in 0..size {
                    for w in 0..size {
                        for faulted in 0..2 {
                            for code in 0..16 {
                                for enabled in [false, true] {
                                    let fault = if b == 0 {
                                        5
                                    } else if op.starts_with('s') && a == size / 2 && b == size - 1
                                    {
                                        6
                                    } else {
                                        0
                                    };
                                    let v = if fault != 0 {
                                        0
                                    } else {
                                        match op {
                                            "udiv" => a / b,
                                            "urem" => a % b,
                                            "sdiv" => (signed(a) / signed(b)) as u64 & (size - 1),
                                            "srem" => (signed(a) % signed(b)) as u64 & (size - 1),
                                            _ => unreachable!(),
                                        }
                                    };
                                    let lane = expected.len();
                                    for (name, value) in [
                                        ("a", a),
                                        ("b", b),
                                        ("w", w),
                                        ("faulted", faulted),
                                        ("code", code),
                                    ] {
                                        m.set_lane(&mut packed, name, value, lane)
                                    }
                                    if enabled {
                                        enables |= 1 << lane;
                                    }
                                    expected.push((
                                        if enabled && fault == 0 { v } else { w },
                                        if enabled && fault == 0 { w } else { 0 },
                                        u64::from(enabled && fault != 0),
                                        faulted ^ u64::from(enabled && fault != 0),
                                        code ^ if enabled { fault } else { 0 },
                                    ));
                                    count += 1;
                                    if expected.len() == 64 {
                                        let old = packed.clone();
                                        m.run_packed(&mut packed, enables, false).unwrap();
                                        for name in ["a", "b"] {
                                            for bit in &m.inputs[name] {
                                                assert_eq!(packed[*bit], old[*bit]);
                                            }
                                        }
                                        for (lane, (w, slot, flag, faulted, code)) in
                                            expected.iter().enumerate()
                                        {
                                            for (name, value) in [
                                                ("w", w),
                                                ("slot", slot),
                                                ("flag", flag),
                                                ("faulted", faulted),
                                                ("code", code),
                                            ] {
                                                assert_eq!(
                                                    m.get_lane(&packed, name, lane),
                                                    *value,
                                                    "{op} n={n} {name}"
                                                )
                                            }
                                        }
                                        m.run_packed(&mut packed, enables, true).unwrap();
                                        assert_eq!(packed, old);
                                        packed.fill(0);
                                        expected.clear();
                                        enables = 0;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            assert!(expected.is_empty());
        }
    }
    assert_eq!(count, 4 * 64 * (8 + 64 + 512 + 4096));
    println!("division assignment rows: {count} tuples");
}
