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
