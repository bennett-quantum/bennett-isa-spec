mod support;
use bennett_isa_ref::{Argument, Budget, Object, OracleInput};

#[test]
fn seed_inc8_full_input_domain_and_budget_boundaries() {
    let o = Object::parse(include_str!("fixtures/inc8.bisa")).unwrap();
    for x in 0..256 {
        for b in 0..=4 {
            let r = o
                .run_oracle("inc8", &OracleInput::bits(&[x]), Budget::steps(b))
                .unwrap();
            assert_eq!(
                (r.payload, r.code),
                if b >= 3 { ((x + 1) & 255, 0) } else { (0, 1) },
                "x={x},B={b}"
            );
            assert_eq!(r.steps, b.min(3));
        }
    }
}

#[test]
fn seed_recursive_memory_interface_and_stack_boundaries() {
    let o = Object::parse(include_str!("fixtures/sum_db.bisa")).unwrap();
    // Hand derivation: db[i]=i+1, so n bytes sum to n(n+1)/2.
    for n in 0..24 {
        let mut input = OracleInput::bits(&[n]);
        input.regions.insert("db".into(), (1..=16).collect());
        let budget = Budget {
            steps: 1000,
            heap: 4080,
            stack: 512,
            depth: 64,
        };
        let r = o.run_oracle("sum_db", &input, budget).unwrap();
        assert_eq!(
            (r.payload, r.code),
            if n <= 16 {
                (n * (n + 1) / 2, 0)
            } else {
                (0, 7)
            },
            "n={n}"
        );
        if n <= 16 {
            assert_eq!(r.steps, 14 * n as u64 + 10);
            for stack in [16 * (n + 1), 16 * (n + 1) - 1] {
                let r = o
                    .run_oracle("sum_db", &input, Budget { stack, ..budget })
                    .unwrap();
                assert_eq!(r.code, if stack % 16 == 0 { 0 } else { 3 });
            }
            let r = o
                .run_oracle("sum_db", &input, Budget { depth: n, ..budget })
                .unwrap();
            assert_eq!(r.code, 3);
            let r = o
                .run_oracle(
                    "sum_db",
                    &input,
                    Budget {
                        depth: n + 1,
                        ..budget
                    },
                )
                .unwrap();
            assert_eq!(r.code, 0);
            let r = o
                .run_oracle(
                    "sum_db",
                    &input,
                    Budget {
                        steps: r.steps - 1,
                        ..budget
                    },
                )
                .unwrap();
            assert_eq!(r.code, 1);
        } else {
            assert_eq!(r.steps, 13);
        }
    }
}

#[test]
fn all_arithmetic_rows_and_faults() {
    let o = Object::parse(include_str!("fixtures/arithmetic.bisa")).unwrap();
    for (name, input, expected) in [
        ("add", vec![255, 2], (1, 0)),
        ("sub", vec![0, 1], (255, 0)),
        ("mul", vec![128, 3], (128, 0)),
        ("and", vec![170, 240], (160, 0)),
        ("or", vec![170, 240], (250, 0)),
        ("xor", vec![170, 240], (90, 0)),
        ("shl", vec![1, 7], (128, 0)),
        ("shl", vec![1, 8], (0, 0)),
        ("lshr", vec![128, 7], (1, 0)),
        ("ashr", vec![128, 7], (255, 0)),
        ("ashr", vec![128, 8], (0, 0)),
        ("udiv", vec![255, 2], (127, 0)),
        ("urem", vec![255, 2], (1, 0)),
        ("sdiv", vec![249, 2], (253, 0)),
        ("srem", vec![249, 2], (255, 0)),
        ("udiv", vec![3, 0], (0, 5)),
        ("sdiv", vec![128, 255], (0, 6)),
        ("srem", vec![128, 255], (0, 6)),
        ("eq", vec![2, 2], (1, 0)),
        ("ne", vec![2, 2], (0, 0)),
        ("ult", vec![255, 0], (0, 0)),
        ("ule", vec![2, 2], (1, 0)),
        ("slt", vec![255, 0], (1, 0)),
        ("sle", vec![128, 128], (1, 0)),
        ("select", vec![2, 9], (2, 0)),
        ("casts", vec![128, 0], (255, 0)),
        ("parmov", vec![3, 9], (9, 0)),
        ("jump", vec![0, 0], (42, 0)),
        ("jump", vec![1, 9], (9, 0)),
        ("badjump", vec![0, 0], (0, 4)),
    ] {
        let r = o
            .run_oracle(name, &OracleInput::bits(&input), Budget::steps(100))
            .unwrap();
        assert_eq!((r.payload, r.code), expected, "{name} {input:?}");
    }
    for code in 0..=10 {
        let r = o
            .run_oracle(
                &format!("fault{code}"),
                &OracleInput::bits(&[0, 0]),
                Budget::steps(1),
            )
            .unwrap();
        assert_eq!((r.payload, r.code), (0, code));
    }
}

#[test]
fn heap_allocation_free_and_inherent_checks_are_total() {
    let o = Object::parse(include_str!("fixtures/heap.bisa")).unwrap();
    for (name, expected) in [
        ("heap", (0x1234, 0)),
        ("zeroalloc", (1, 0)),
        ("afterfree", (0, 7)),
        ("pinned", (0, 7)),
        ("doublefree", (0, 7)),
        ("badstore", (0, 7)),
        ("outoftable", (0, 2)),
        ("faultif", (0, 9)),
    ] {
        let r = o
            .run_oracle(
                name,
                &OracleInput::bits(&[]),
                Budget {
                    steps: 100,
                    heap: 32,
                    stack: 0,
                    depth: 0,
                },
            )
            .unwrap();
        assert_eq!((r.payload, r.code), expected, "{name}");
    }
    let r = o
        .run_oracle(
            "heap",
            &OracleInput::bits(&[]),
            Budget {
                steps: 100,
                heap: 1,
                stack: 0,
                depth: 0,
            },
        )
        .unwrap();
    assert_eq!(r.code, 2);
    assert!(o
        .run_oracle(
            "heap",
            &OracleInput::bits(&[]),
            Budget {
                steps: 100,
                heap: 33,
                stack: 0,
                depth: 0
            }
        )
        .is_err());
}

#[test]
fn nested_oracles_xor_targets_and_faults_with_explicit_propagation() {
    let o = Object::parse(include_str!("fixtures/nested.bisa")).unwrap();
    for b in [0, 2, 3, 4] {
        let mut input = OracleInput::bits(&[255]);
        input.args.push(Argument::Token(Budget::steps(b)));
        for (name, expected) in [
            ("nested", if b >= 3 { (0, 0) } else { (0, 9) }),
            ("cancel", (0, 0)),
            ("ignore", (0, 0)),
        ] {
            let r = o.run_oracle(name, &input, Budget::steps(20)).unwrap();
            assert_eq!((r.payload, r.code), expected, "{name},B={b}");
        }
        input.ins[0] = 8;
        let r = o.run_oracle("nested", &input, Budget::steps(20)).unwrap();
        assert_eq!((r.payload, r.code), if b >= 3 { (9, 0) } else { (0, 9) });
    }
}

#[test]
#[ignore = "full 8-bit arithmetic sweep"]
fn exhaustive_unsigned_and_signed_division_u8() {
    let o = Object::parse(include_str!("fixtures/arithmetic.bisa")).unwrap();
    for a in 0..=255u128 {
        for b in 0..=255u128 {
            for name in ["udiv", "urem", "sdiv", "srem"] {
                let r = o
                    .run_oracle(name, &OracleInput::bits(&[a, b]), Budget::steps(2))
                    .unwrap();
                let overflow = name.starts_with('s') && a == 128 && b == 255;
                if b == 0 || overflow {
                    assert_eq!((r.payload, r.code), (0, if b == 0 { 5 } else { 6 }));
                } else {
                    let sa = a as u8 as i8 as i16;
                    let sb = b as u8 as i8 as i16;
                    let expected = match name {
                        "udiv" => a / b,
                        "urem" => a % b,
                        "sdiv" => (sa / sb) as u8 as u128,
                        _ => (sa % sb) as u8 as u128,
                    };
                    assert_eq!((r.payload, r.code), (expected, 0), "{name}({a},{b})");
                }
            }
        }
    }
}
#[test]
fn const_literals_outside_the_destination_range_are_rejected_not_reduced() {
    // SPEC-QUESTIONS 12 (revised): -2^(N-1) <= c < 2^N; reducing 2^128+5 to 5
    // would be the silent truncation §3.3 forbids. Hand check for N = 8:
    // -128 and 255 are the extremes; -129 and 256 are outside.
    for literal in [
        "340282366920938463463374607431768211461",
        "-340282366920938463463374607431768211461",
        "256",
        "-129",
    ] {
        let text = include_str!("fixtures/inc8.bisa")
            .replace("lift.const.i8 1", &format!("lift.const.i8 {literal}"));
        let e = Object::parse(&text).unwrap_err().to_string();
        assert!(e.contains("§3.3"), "{literal}: {e}");
    }
    for (literal, addend) in [("255", 255u128), ("-128", 128), ("-1", 255)] {
        let text = include_str!("fixtures/inc8.bisa")
            .replace("lift.const.i8 1", &format!("lift.const.i8 {literal}"));
        let o = support::rehash(&text);
        for x in [0u128, 1, 127, 128, 255] {
            let r = o
                .run_oracle("inc8", &OracleInput::bits(&[x]), Budget::steps(3))
                .unwrap();
            assert_eq!(
                (r.payload, r.code),
                ((x + addend) & 255, 0),
                "{literal} {x}"
            );
        }
    }
}
