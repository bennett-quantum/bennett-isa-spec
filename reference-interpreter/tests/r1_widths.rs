//! Council round 1: fable R1-B1, codex B01, muse B1 (§3.3 bounds only iN).
//! Every expected value below is derived by hand from §3.3, §3.6, §4.2, §5.2.
mod support;
use bennett_isa_ref::{
    BitVector, Budget, Object, OracleInput, OracleVectorInput, OracleVectorMachine,
};

fn oracle(name: &str, header: &str, regs: &str, body: &str, inputs: &[u64], tgt: u64) -> Object {
    let ins = inputs
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let signature = format!(
        r#"{{"abi":"0.8","args":[],"fault":4,"in":[{ins}],"kind":"oracle","mem":[],"tgt":{tgt},"tier":"q"}}"#
    );
    let text = format!("{header}\n{regs}^entry():\n{body}");
    support::rehash(&support::object(&[(name, &signature, &text)], "{}", ""))
}
fn pow2(width: usize, k: usize) -> BitVector {
    let mut v = BitVector::zero(width).unwrap();
    v.set(k, true).unwrap();
    v
}
/// F_B and T2 for one input: the sequential engine, the literal machine with
/// its outcome, and the restoration of the complete tuple after B inverse steps.
fn both(o: &Object, name: &str, x: &[BitVector], steps: u64) -> (BitVector, BitVector, u64) {
    let input = OracleVectorInput {
        ins: x.to_vec(),
        ..Default::default()
    };
    let budget = Budget::steps(steps);
    let r = o.run_oracle_vectors(name, &input, budget).unwrap();
    let mut m = OracleVectorMachine::new(o, name, &input, budget).unwrap();
    let entry = m.snapshot();
    for _ in 0..steps {
        m.forward_step().unwrap();
    }
    assert_eq!(m.outcome().unwrap(), r, "machine agrees with §14.1");
    for _ in 0..steps {
        m.inverse_step().unwrap();
    }
    assert_eq!(m.snapshot(), entry, "prefix reversal restores the tuple");
    assert!(m.frames_clean());
    (r.payload, r.code, r.steps)
}

#[test]
fn codex_b01_wire129_identity_at_b0_and_b1() {
    let o = oracle(
        "id129",
        "oracle @id129 (x:wire[129] in) (t:wire[129]) (f:wire[4]) () ()",
        "",
        "lift.retval x",
        &[129],
        129,
    );
    for x in [BitVector::zero(129).unwrap(), pow2(129, 128)] {
        // B = 0 commits FAULT_BUDGET_STEPS on every input (§3.6).
        let (p, c, s) = both(&o, "id129", std::slice::from_ref(&x), 0);
        assert!(p.is_zero());
        assert_eq!((c.to_u128().unwrap(), s), (1, 0));
        // B = 1: lift.retval x halts with success after one row.
        let (p, c, s) = both(&o, "id129", std::slice::from_ref(&x), 1);
        assert_eq!(p, x);
        assert_eq!((c.to_u128().unwrap(), s), (0, 1));
    }
}

#[test]
fn codex_b01_reg129_move_with_bit_128_set_reverses() {
    let o = oracle(
        "mov129",
        "oracle @mov129 (x:wire[129] in) (t:wire[129]) (f:wire[4]) () ()",
        "regs { r:reg[129] }\n",
        "r = lift.mov x\nlift.retval r",
        &[129],
        129,
    );
    let mut x = pow2(129, 128);
    x.set(0, true).unwrap();
    let (p, c, s) = both(&o, "mov129", std::slice::from_ref(&x), 2);
    assert_eq!(p, x);
    assert_eq!((c.to_u128().unwrap(), s), (0, 2));
    let (p, c, _) = both(&o, "mov129", &[x], 1);
    assert!(p.is_zero());
    assert_eq!(c.to_u128().unwrap(), 1);
}

#[test]
fn fable_r1_b1_wide_add_is_2x_mod_2_256_in_both_engines() {
    let o = oracle(
        "wide",
        "oracle @wide (x:wire[256] in) (t:wire[256]) (f:wire[4]) () ()",
        "regs { r:reg[256] }\n",
        "r = lift.add x x\nlift.retval r",
        &[256],
        256,
    );
    let all_ones = BitVector::zero(256)
        .unwrap()
        .binary("sub", &pow2(256, 0))
        .unwrap();
    let cases = [
        (BitVector::zero(256).unwrap(), BitVector::zero(256).unwrap()),
        (pow2(256, 0), pow2(256, 1)),
        (pow2(256, 200), pow2(256, 201)),
        // 2(2^256 - 1) mod 2^256 = 2^256 - 2.
        (
            all_ones.clone(),
            all_ones.binary("sub", &pow2(256, 0)).unwrap(),
        ),
    ];
    for (x, twice) in cases {
        for b in 0..=3u64 {
            let (p, c, s) = both(&o, "wide", std::slice::from_ref(&x), b);
            if b >= 2 {
                assert_eq!((p, c.to_u128().unwrap(), s), (twice.clone(), 0, 2));
            } else {
                assert!(p.is_zero());
                assert_eq!((c.to_u128().unwrap(), s), (1, b));
            }
        }
    }
}

#[test]
fn fable_r1_b1_unused_wide_input_through_the_small_integer_api() {
    // The witness that panicked with 1u128 << 256: a 256-bit input is legal.
    let o = oracle(
        "wide",
        "oracle @wide (x:wire[256] in) (t:wire[8]) (f:wire[4]) () ()",
        "regs { one:reg[8] }\n",
        "one = lift.const.i8 1\nlift.retval one",
        &[256],
        8,
    );
    let r = o
        .run_oracle("wide", &OracleInput::bits(&[0]), Budget::steps(2))
        .unwrap();
    assert_eq!((r.payload, r.code, r.steps), (1, 0, 2));
    let r = o
        .run_oracle("wide", &OracleInput::bits(&[7]), Budget::steps(1))
        .unwrap();
    assert_eq!((r.payload, r.code, r.steps), (0, 1, 1));
}

#[test]
fn muse_b1_mov_of_2_255() {
    let o = oracle(
        "w",
        "oracle @w (x:wire[256] in) (t:wire[256]) (f:wire[4]) () ()",
        "regs { r:reg[256] }\n",
        "r = lift.mov x\nlift.retval r",
        &[256],
        256,
    );
    let x = pow2(256, 255);
    let (p, c, s) = both(&o, "w", std::slice::from_ref(&x), 2);
    assert_eq!((p, c.to_u128().unwrap(), s), (x, 0, 2));
}

#[test]
fn wide_lifted_suffixes_constants_and_casts() {
    // SPEC-QUESTIONS 15: lift.const.iN / lift.zext.iM name reg widths, not iN.
    let o = oracle(
        "k",
        "oracle @k (x:wire[8] in) (t:wire[300]) (f:wire[4]) () ()",
        "regs { c:reg[300], z:reg[300], r:reg[300] }\n",
        // 2^299 = 1018517988167243043134222844204689080525734196832968125318070224677190649881668353091698688
        "c = lift.const.i300 1018517988167243043134222844204689080525734196832968125318070224677190649881668353091698688\nz = lift.zext.i300 x\nr = lift.or c z\nlift.retval r",
        &[8],
        300,
    );
    let x = BitVector::from_u128(8, 0xa5).unwrap();
    let (p, c, s) = both(&o, "k", &[x], 4);
    let mut expected = pow2(300, 299);
    for bit in [0, 2, 5, 7] {
        expected.set(bit, true).unwrap();
    }
    assert_eq!((p, c.to_u128().unwrap(), s), (expected, 0, 4));
}

#[test]
fn bundle_widths_above_u16_and_large_region_dimensions_are_well_formed() {
    // §3.3 states no bound for wire[N], reg[N] or region[A,W].
    let o = oracle(
        "big",
        "oracle @big (x:wire[70000] in) (t:wire[70000]) (f:wire[4]) () ()",
        "",
        "lift.retval x",
        &[70000],
        70000,
    );
    let mut x = pow2(70000, 69999);
    x.set(3, true).unwrap();
    let (p, c, s) = both(&o, "big", std::slice::from_ref(&x), 1);
    assert_eq!((p, c.to_u128().unwrap(), s), (x, 0, 1));
    assert_eq!(
        bennett_isa_ref::Type::parse("region[200,8]").unwrap(),
        bennett_isa_ref::Type::Region(200, 8)
    );
}

#[test]
fn ashr_by_n_or_more_is_zero_in_every_engine() {
    // §4.1 "wrap; shift >= N -> 0": 0x80 ashr 8 = 0 (not 0xff), 0x80 ashr 7 = 0xff.
    let o = oracle(
        "sh",
        "oracle @sh (x:wire[8] in, y:wire[8] in) (t:wire[8]) (f:wire[4]) () ()",
        "regs { r:reg[8] }\n",
        "r = lift.ashr x y\nlift.retval r",
        &[8, 8],
        8,
    );
    for (y, expected) in [(7u128, 0xffu128), (8, 0), (200, 0)] {
        let x = [
            BitVector::from_u128(8, 0x80).unwrap(),
            BitVector::from_u128(8, y).unwrap(),
        ];
        let (p, _, _) = both(&o, "sh", &x, 2);
        assert_eq!(p.to_u128().unwrap(), expected, "0x80 ashr {y}");
    }
}

#[test]
fn derived_counter_widths_follow_section_3_7() {
    // H = 127 with ptr_width = 128: Hc >= H + 2 = 129 (codex B01), Ac = A_tab + 1,
    // Sc = S_cells + 1, Dc = ceil(log2(depth_limit + 2)).
    let layout = r#"
section layout
{"A_tab":3,"H":127,"S_cells":5,"globals":{},"heap_base":16,"heap_limit_max":1024,"interface_regions":{},"null_size":16,"ptr_width":128}
"#;
    let signature =
        r#"{"abi":"0.8","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#;
    let text = support::object(
        &[(
            "h",
            signature,
            "oracle @h (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()\n^entry():\nlift.retval x",
        )],
        "{}",
        layout,
    )
    .replace("ptr_width=32", "ptr_width=128");
    let o = support::rehash(&text);
    let budget = Budget {
        steps: 1,
        heap: 1024,
        stack: 32,
        depth: 6,
    };
    let w = bennett_isa_ref::DerivedWidths::derive(&o, "h", budget).unwrap();
    // depth_limit 6: ceil(log2 8) = 3. One row: 2^P > 1 needs P = 1.
    assert_eq!((w.hc, w.ac, w.sc, w.dc, w.p), (129, 4, 6, 3, 1));
    // H = 128 = ptr_width is legal (§7.2 H <= ptr_width): Hc = 130.
    let o = support::rehash(&text.replace("\"H\":127", "\"H\":128"));
    let w = bennett_isa_ref::DerivedWidths::derive(&o, "h", budget).unwrap();
    assert_eq!(w.hc, 130);
}

#[test]
fn r13_derived_counter_width_overflow_reports_host_limit() {
    for key in ["S_cells", "A_tab"] {
        let o = support::r13_check(key, "inc8");
        let input = OracleVectorInput {
            ins: vec![BitVector::from_u128(8, 8).unwrap()],
            ..Default::default()
        };
        let b = Budget::steps(0);
        let r = o.run_oracle_vectors("inc8", &input, b).unwrap();
        assert_eq!(
            (
                r.payload.to_u128().unwrap(),
                r.code.to_u128().unwrap(),
                r.steps
            ),
            (0, 1, 0)
        );
        let derived =
            std::panic::catch_unwind(|| bennett_isa_ref::DerivedWidths::derive(&o, "inc8", b));
        let e = derived
            .expect("width derivation must not panic")
            .unwrap_err();
        assert!(
            e.to_string().contains("host limit") && e.to_string().contains("§3.7"),
            "{e}"
        );
        let machine = std::panic::catch_unwind(|| OracleVectorMachine::new(&o, "inc8", &input, b));
        let e = machine
            .expect("machine construction must not panic")
            .err()
            .expect("host limit");
        assert!(
            e.to_string().contains("host limit") && e.to_string().contains("§3.7"),
            "{e}"
        );
    }
}

#[test]
fn r13_generated_derived_width_extremes_family() {
    let mut cases = 0;
    let mut admitted = 0;
    let mut rejected = 0;
    let mut host_limits = 0;
    let values = [
        0u128,
        1,
        63,
        64,
        127,
        128,
        u64::MAX as u128 - 64,
        u64::MAX as u128 - 63,
        u64::MAX as u128 - 2,
        u64::MAX as u128 - 1,
        u64::MAX as u128,
        1u128 << 64,
    ];
    for (text, root) in [
        (
            support::r13_text("S_cells")
                .replace(r#""S_cells":"18446744073709551615""#, r#""S_cells":0"#),
            "inc8",
        ),
        (
            support::r13_text("S_cells")
                .replace(r#""S_cells":"18446744073709551615""#, r#""S_cells":0"#)
                .replace("ptr_width=32", "ptr_width=128")
                .replace(r#""ptr_width":32"#, r#""ptr_width":128"#)
                .replace(r#""H":6"#, r#""H":128"#),
            "inc8",
        ),
        (include_str!("fixtures/heap.bisa").into(), "heap"),
        (include_str!("fixtures/sum_db.bisa").into(), "sum_db"),
        (
            include_str!("fixtures/tiny_memory.bisa").into(),
            "tiny_load",
        ),
    ] {
        let o = Object::parse(&text).unwrap();
        let root = if o.functions.contains_key(root) {
            root
        } else {
            o.function_order[0].as_str()
        };
        let layout = o.section("layout").unwrap();
        // All scalars, including nested global base/size and interface base.
        fn paths(
            v: &bennett_isa_ref::json::Json,
            path: &mut Vec<String>,
            out: &mut Vec<Vec<String>>,
        ) {
            if let bennett_isa_ref::json::Json::Object(m) = v {
                for (k, v) in m {
                    path.push(k.clone());
                    paths(v, path, out);
                    path.pop();
                }
            } else if matches!(v, bennett_isa_ref::json::Json::Number(_)) {
                out.push(path.clone());
            }
        }
        fn put(v: &mut bennett_isa_ref::json::Json, path: &[String], n: u128) {
            if path.is_empty() {
                *v = if n <= 9_007_199_254_740_991 {
                    bennett_isa_ref::json::Json::Number(n.to_string())
                } else {
                    bennett_isa_ref::json::Json::String(n.to_string())
                };
            } else if let bennett_isa_ref::json::Json::Object(m) = v {
                put(m.get_mut(&path[0]).unwrap(), &path[1..], n);
            }
        }
        let mut all = Vec::new();
        paths(layout, &mut Vec::new(), &mut all);
        for path in all {
            for n in values {
                cases += 1;
                let mut l = layout.clone();
                put(&mut l, &path, n);
                let mutated = text.replace(&layout.canonical(), &l.canonical());
                let result = std::panic::catch_unwind(|| -> bennett_isa_ref::Result<bool> {
                    let object = match Object::parse(&mutated) {
                        Ok(o) => o,
                        Err(e) => {
                            assert!(e.to_string().contains('§'), "{e}");
                            return Ok(false);
                        }
                    };
                    let f = &object.functions[root];
                    let mut input = OracleVectorInput::default();
                    for p in &f.params[0] {
                        input
                            .ins
                            .push(BitVector::zero(p.ty.bits(object.ptr_width)?)?);
                    }
                    for p in &f.params[3] {
                        if let bennett_isa_ref::Type::Region(a, w) = p.ty {
                            input.regions.insert(
                                p.name.clone(),
                                vec![0; (1usize << a) * (w as usize).div_ceil(8)],
                            );
                        }
                    }
                    let b = Budget::steps(0);
                    // A legal layout always preserves the sequential B=0 result.
                    let r = object.run_oracle_vectors(root, &input, b)?;
                    assert_eq!(
                        (r.payload.to_u128()?, r.code.to_u128()?, r.steps),
                        (0, 1, 0)
                    );
                    let d = bennett_isa_ref::DerivedWidths::derive(&object, root, b)?;
                    assert_eq!(d.hc as u128, l.get("H")?.uint()? + 2);
                    assert_eq!(d.ac as u128, l.get("A_tab")?.uint()? + 1);
                    assert_eq!(d.sc as u128, l.get("S_cells")?.uint()? + 1);
                    assert_eq!(d.w_cell, d.p.max(d.sc).max(d.did).max(d.w_max));
                    let m = OracleVectorMachine::new(&object, root, &input, b)?;
                    assert!(m.frames_clean());
                    Ok(true)
                });
                match result.unwrap_or_else(|_| panic!("panic for {root} {path:?}={n}")) {
                    Ok(true) => admitted += 1,
                    Ok(false) => rejected += 1,
                    Err(e) => {
                        assert!(e.to_string().contains('§'), "{root} {path:?}={n}: {e}");
                        host_limits += 1;
                    }
                }
            }
        }
    }
    // Dc's logarithm and d+2 boundary are independent of the layout exponents.
    let text = support::r13_text("S_cells")
        .replace(r#""S_cells":"18446744073709551615""#, r#""S_cells":128"#);
    let o = Object::parse(&text).unwrap();
    let input = OracleVectorInput {
        ins: vec![BitVector::from_u128(8, 8).unwrap()],
        ..Default::default()
    };
    for depth in values
        .into_iter()
        .chain([u128::MAX - 2, u128::MAX - 1, u128::MAX])
    {
        cases += 1;
        let b = Budget {
            depth,
            ..Budget::steps(0)
        };
        let result = std::panic::catch_unwind(|| -> bennett_isa_ref::Result<()> {
            let d = bennett_isa_ref::DerivedWidths::derive(&o, "inc8", b)?;
            let expected = (128 - (depth + 1).leading_zeros() as usize).max(1);
            assert_eq!(d.dc, expected);
            OracleVectorMachine::new(&o, "inc8", &input, b)?;
            Ok(())
        })
        .expect("depth arithmetic must not panic");
        match result {
            Ok(()) => admitted += 1,
            Err(e) => {
                assert!(e.to_string().contains('§'), "{e}");
                host_limits += 1;
            }
        }
    }
    eprintln!("R13 widths: {cases} cases; {admitted} constructed, {rejected} layout rejections, {host_limits} resource diagnostics");
    assert_eq!(cases, 507);
    assert_eq!(cases, admitted + rejected + host_limits);
}
