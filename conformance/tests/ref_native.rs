use bennett_isa_conformance::{
    json,
    ref_amplitudes::{column, rotation, Complex, Real},
    ref_language::{Expander, Registry},
};
#[test]
fn native_phase_and_hadamard_full_basis_matrices() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let q = Real::new(0, 1, 1);
    let mut count = 0;
    for width in 1..=4 {
        for g in ["h", "s", "sdg", "t", "tdg", "cz"] {
            if g == "cz" && width < 2 {
                continue;
            }
            let args = json::parse(&if g == "cz" {
                r#"["r[0]","r[1]","c"]"#.to_string()
            } else {
                r#"["r[0]","c"]"#.to_string()
            })
            .unwrap();
            let leaves = e
                .expand(
                    &format!("gate.{g}"),
                    args.as_arr().unwrap(),
                    &[("r", width), ("c", 1)],
                )
                .unwrap();
            let inputs = (0..width)
                .map(|i| ("r".into(), i))
                .chain([("c".into(), 0)])
                .collect::<Vec<_>>();
            for basis in 0..1 << (width + 1) {
                let mut expected = vec![Complex::default(); 1 << (width + 1)];
                if basis >> width & 1 == 0 {
                    expected[basis] = Complex::one();
                } else if g == "h" {
                    let a = Complex {
                        re: q,
                        im: Real::integer(0),
                    };
                    expected[basis & !1] = a;
                    expected[basis | 1] = if basis & 1 == 0 { a } else { a.negated() };
                } else {
                    let phase = if basis & 1 == 0 {
                        Complex::one()
                    } else {
                        match g {
                            "s" => Complex {
                                re: Real::integer(0),
                                im: Real::integer(1),
                            },
                            "sdg" => Complex {
                                re: Real::integer(0),
                                im: Real::integer(-1),
                            },
                            "t" => Complex { re: q, im: q },
                            "tdg" => Complex {
                                re: q,
                                im: Real::new(0, -1, 1),
                            },
                            "cz" => {
                                if basis & 2 != 0 {
                                    Complex::one().negated()
                                } else {
                                    Complex::one()
                                }
                            }
                            _ => unreachable!(),
                        }
                    };
                    expected[basis] = phase;
                }
                assert_eq!(column(&leaves, &inputs, basis).unwrap(), expected);
                count += 1;
            }
        }
    }
    assert_eq!(count, 356);
    println!("native Clifford+T: {count} exact matrix columns");
}
#[test]
fn variadic_mcx_full_domain_including_zero_and_large_arity() {
    use bennett_isa_conformance::{ref_bdd::Bdd, ref_catalog::Machine, ref_language::permutation};
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let mut count = 0;
    for k in [0, 1, 2, 3, 4, 17] {
        let args = json::parse(&format!(r#"[{k},"controls","target",[]]"#)).unwrap();
        let bundles = [("controls", k), ("target", 1)];
        let leaves = e
            .expand("gate.mcx", args.as_arr().unwrap(), &bundles)
            .unwrap();
        assert_eq!(leaves[0].wires.len(), k + 1);
        let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
        let mut b = Bdd::new();
        let mut s = m.symbolic_state();
        for name in ["controls", "target"] {
            for i in &m.inputs[name] {
                s[*i] = b.variable();
            }
        }
        let enabled = b.variable();
        let old = s.clone();
        let mut active = enabled;
        for i in &m.inputs["controls"] {
            active = b.and(active, old[*i]);
        }
        let mut expected = old.clone();
        let t = m.inputs["target"][0];
        expected[t] = b.xor(expected[t], active);
        m.run_symbolic(&mut b, &mut s, enabled, false, 1).unwrap();
        assert_eq!(s, expected);
        m.run_symbolic(&mut b, &mut s, enabled, true, 1).unwrap();
        assert_eq!(s, old);
        count += 1usize << (k + 2);
    }
    assert_eq!(count, 524412);
    println!("variadic mcx: {count} basis tuples");
}
#[test]
fn binary64_angles_round_trip_and_numerical_adjoint_matrices() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let mut count = 0;
    // Each representative bit pattern round-trips through the required encoding; these eight
    // representative finite angles include both zeros, subnormal and extrema.
    for bits in [
        0u64,
        1,
        0x8000000000000000,
        0x3ff0000000000000,
        0xbff0000000000000,
        0x400921fb54442d18,
        0x7fefffffffffffff,
        0xffefffffffffffff,
    ] {
        for g in ["rx", "ry", "rz"] {
            let args = json::parse(&format!(r#"["0x{bits:016x}","r[0]","c"]"#)).unwrap();
            let leaves = e
                .expand(
                    &format!("gate.{g}"),
                    args.as_arr().unwrap(),
                    &[("r", 1), ("c", 1)],
                )
                .unwrap();
            assert_eq!(leaves[0].gate, g);
            assert_eq!(leaves[0].angles, [bits]);
            assert_eq!(f64::from_bits(leaves[0].angles[0]).to_bits(), bits);
            // §5.3 negates the source angle before the adj adapter is bound.
            let adj_bits = bits ^ (1 << 63);
            let body = json::parse(&format!(
                r#"[[{{"gate":"{g}","wires":["r[0]"],"params":["0x{adj_bits:016x}"]}}]]"#
            ))
            .unwrap();
            let adj = e
                .expand("adj", body.as_arr().unwrap(), &[("r", 1)])
                .unwrap();
            assert_eq!(adj[0].angles, [bits ^ (1 << 63)]);
            let matrix = rotation(g, bits).unwrap();
            let inverse = rotation(g, bits ^ (1 << 63)).unwrap();
            for (col, _) in matrix[0].iter().enumerate() {
                for (row, inverse_row) in inverse.iter().enumerate() {
                    let mut value = (0.0, 0.0);
                    for k in 0..2 {
                        let (ar, ai) = inverse_row[k];
                        let (br, bi) = matrix[k][col];
                        value.0 += ar * br - ai * bi;
                        value.1 += ar * bi + ai * br;
                    }
                    let expected = if row == col { 1.0 } else { 0.0 };
                    assert!((value.0 - expected).abs() <= 8.0 * f64::EPSILON);
                    assert!(value.1.abs() <= 8.0 * f64::EPSILON);
                }
            }
            // All computational basis columns with widths 1..4, including
            // spectator and effective-control bits: embed the prescribed matrix.
            for width in 1..=4 {
                for basis in 0..1 << (width + 1) {
                    let actual =
                        bennett_isa_conformance::ref_amplitudes::numerical_rotation_column(
                            &leaves,
                            &(0..width)
                                .map(|i| ("r".into(), i))
                                .chain([("c".into(), 0)])
                                .collect::<Vec<_>>(),
                            basis,
                        )
                        .unwrap();
                    // Independent spectral/Pauli form of exp(-i theta P/2),
                    // using separate sin/cos calls rather than the backend's sin_cos.
                    let half = f64::from_bits(bits) / 2.0;
                    let c = half.cos();
                    let s = half.sin();
                    let prescribed = match g {
                        "rx" => [[(c, 0.0), (0.0, -s)], [(0.0, -s), (c, 0.0)]],
                        "ry" => [[(c, 0.0), (-s, 0.0)], [(s, 0.0), (c, 0.0)]],
                        "rz" => [[(c, -s), (0.0, 0.0)], [(0.0, 0.0), (c, s)]],
                        _ => unreachable!(),
                    };
                    let controlled = basis >> width & 1 == 1;
                    let col = basis & 1;
                    let entries = if controlled {
                        [prescribed[0][col], prescribed[1][col]]
                    } else {
                        [
                            if col == 0 { (1.0, 0.0) } else { (0.0, 0.0) },
                            if col == 1 { (1.0, 0.0) } else { (0.0, 0.0) },
                        ]
                    };
                    let mut expected = vec![(0.0, 0.0); 1 << (width + 1)];
                    expected[basis & !1] = entries[0];
                    expected[basis | 1] = entries[1];
                    for (actual, expected) in actual.iter().zip(&expected) {
                        assert!((actual.0 - expected.0).abs() <= 8.0 * f64::EPSILON);
                        assert!((actual.1 - expected.1).abs() <= 8.0 * f64::EPSILON);
                    }
                    let norm = entries.iter().map(|(r, i)| r * r + i * i).sum::<f64>();
                    assert!((norm - 1.0).abs() <= 8.0 * f64::EPSILON);
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 1440);
    println!("rotations: {count} numerical basis columns; entry bits exact, U†U absolute residual <=8*binary64 epsilon (evidence only)");
}

#[test]
fn typed_native_permutation_ports_and_environments() {
    use bennett_isa_conformance::{ref_bdd::Bdd, ref_catalog::Machine, ref_language::permutation};
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut count = 0;
    for (g, ports) in [
        ("x", vec!["a"]),
        ("cx", vec!["a", "b"]),
        ("ccx", vec!["a", "b", "t"]),
        ("swap", vec!["a", "b"]),
    ] {
        let args = json::Json::Arr(
            ports
                .iter()
                .chain(["c"].iter())
                .map(|s| json::Json::Str(s.to_string()))
                .collect(),
        );
        let bundles = ports
            .iter()
            .chain(["c"].iter())
            .map(|s| (*s, 1))
            .collect::<Vec<_>>();
        let leaves = e
            .expand(&format!("bind.gate.{g}"), args.as_arr().unwrap(), &bundles)
            .unwrap();
        let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
        let mut b = Bdd::new();
        let mut s = m.symbolic_state();
        for (name, _) in &bundles {
            for id in &m.inputs[*name] {
                s[*id] = b.variable();
            }
        }
        let old = s.clone();
        let mut expected = old.clone();
        let control = old[m.inputs["c"][0]];
        if g == "swap" {
            let a = m.inputs["a"][0];
            let t = m.inputs["b"][0];
            let d = b.xor(old[a], old[t]);
            let flip = b.and(control, d);
            expected[a] = b.xor(expected[a], flip);
            expected[t] = b.xor(expected[t], flip);
        } else {
            let mut active = control;
            for port in &ports[..ports.len() - 1] {
                active = b.and(active, old[m.inputs[*port][0]]);
            }
            let t = m.inputs[*ports.last().unwrap()][0];
            expected[t] = b.xor(expected[t], active);
        }
        m.run_symbolic(&mut b, &mut s, 1, false, 1).unwrap();
        assert_eq!(s, expected);
        m.run_symbolic(&mut b, &mut s, 1, true, 1).unwrap();
        assert_eq!(s, old);
        count += 1 << (ports.len() + 1);
    }
    assert_eq!(count, 36);
    println!("typed native permutation bindings: {count} tuples");
}
