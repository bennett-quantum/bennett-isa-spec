use bennett_isa_conformance::{
    json,
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
#[test]
fn structured_bounds_and_full_domains() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    for width in 1..=4 {
        for bound in [0, 1, 2, 3, 17] {
            for kind in ["qloop", "qloop.while"] {
                let body = format!(
                    r#"[{{"for":"bit","from":0,"to":{width},"body":[{{"gate":"x","wires":["v[bit]"]}}]}}]"#
                );
                let args = json::parse(&if kind == "qloop" {
                    format!(r#"[{bound},{body}]"#)
                } else {
                    format!(r#"[{bound},"cond",{body}]"#)
                })
                .unwrap();
                let bundles = [("cond", bound), ("v", width)];
                let leaves = e.expand(kind, args.as_arr().unwrap(), &bundles).unwrap();
                assert_eq!(
                    leaves
                        .iter()
                        .filter(|l| l.gate == "x" && l.wires.iter().any(|w| w.0 == "v"))
                        .count(),
                    bound * width
                );
                let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
                let mut b = Bdd::new();
                let mut s = m.symbolic_state();
                for name in ["cond", "v"] {
                    for i in &m.inputs[name] {
                        s[*i] = b.variable();
                    }
                }
                let enabled = b.variable();
                let old = s.clone();
                let mut expected = old.clone();
                let mut prefix = enabled;
                let mut parity = 0;
                for i in 0..bound {
                    if kind == "qloop.while" {
                        prefix = b.and(prefix, old[m.inputs["cond"][i]]);
                    }
                    parity = b.xor(parity, prefix);
                }
                for i in &m.inputs["v"] {
                    expected[*i] = b.xor(expected[*i], parity);
                }
                m.run_symbolic(&mut b, &mut s, enabled, false, 1).unwrap();
                assert_eq!(s, expected, "{kind} N={bound} W={width}");
                m.run_symbolic(&mut b, &mut s, enabled, true, 1).unwrap();
                assert_eq!(s, old);
                println!(
                    "{kind} N={bound} W={width}: full 2^{} tuples",
                    bound + width + 1
                );
            }
        }
    }
}
#[test]
fn qif_and_nested_bodies_are_lexical() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let mut count = 0;
    for width in 1..=4 {
        let then = format!(
            r#"[{{"for":"bit","from":0,"to":{width},"body":[{{"gate":"x","wires":["v[bit]"]}}]}}]"#
        );
        let args = json::parse(&format!(r#"["p",{then},[]]"#)).unwrap();
        let bundles = [("p", 1), ("v", width)];
        let m = Machine::compile(
            &permutation(&e.expand("qif", args.as_arr().unwrap(), &bundles).unwrap()).unwrap(),
            &bundles,
        )
        .unwrap();
        let mut b = Bdd::new();
        let mut s = m.symbolic_state();
        for name in ["p", "v"] {
            for i in &m.inputs[name] {
                s[*i] = b.variable();
            }
        }
        let enabled = b.variable();
        let old = s.clone();
        let active = b.and(enabled, old[m.inputs["p"][0]]);
        let mut expected = old.clone();
        for i in &m.inputs["v"] {
            expected[*i] = b.xor(expected[*i], active);
        }
        m.run_symbolic(&mut b, &mut s, enabled, false, 1).unwrap();
        assert_eq!(s, expected);
        m.run_symbolic(&mut b, &mut s, enabled, true, 1).unwrap();
        assert_eq!(s, old);
        count += 1usize << (width + 2);
        // Expanding to a fixpoint resolves invoke inside captured bodies, and
        // a loop variable named bit in the callee never captures the actual.
        let nested =
            json::parse(&format!(r#"[3,[{{"invoke":"qloop","args":[2,{then}]}}]]"#)).unwrap();
        let leaves = e
            .expand("qloop", nested.as_arr().unwrap(), &bundles)
            .unwrap();
        assert_eq!(leaves.len(), 6 * width);
        let machine = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
        let mut b = Bdd::new();
        let mut s = machine.symbolic_state();
        for name in ["p", "v"] {
            for id in &machine.inputs[name] {
                s[*id] = b.variable();
            }
        }
        let enabled = b.variable();
        let old = s.clone();
        machine
            .run_symbolic(&mut b, &mut s, enabled, false, 1)
            .unwrap();
        assert_eq!(s, old);
        machine
            .run_symbolic(&mut b, &mut s, enabled, true, 1)
            .unwrap();
        assert_eq!(s, old);
    }
    assert_eq!(count, 120);
    println!("qif: {count} full-domain tuples plus nested fixpoint checks");
}

#[test]
fn coherent_scopes_qif_and_adjoint_full_matrix_columns() {
    use bennett_isa_conformance::ref_amplitudes::{column, Complex, Real};
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let q = Complex {
        re: Real::new(0, 1, 1),
        im: Real::integer(0),
    };
    let i = Complex {
        re: Real::integer(0),
        im: Real::integer(1),
    };
    let mut count = 0;
    for width in 1..=4 {
        let args=json::parse(r#"["c",[{"invoke":"qif","args":["p",[{"gate":"h","wires":["r[0]"]}],[{"gate":"s","wires":["r[0]"]}]]}]]"#).unwrap();
        let bundles = [("r", width), ("p", 1), ("c", 1)];
        let leaves = e.expand("ctl", args.as_arr().unwrap(), &bundles).unwrap();
        let inputs = (0..width)
            .map(|b| ("r".into(), b))
            .chain([("p".into(), 0), ("c".into(), 0)])
            .collect::<Vec<_>>();
        for basis in 0..1 << (width + 2) {
            let mut expected = vec![Complex::default(); 1 << (width + 2)];
            if basis >> (width + 1) & 1 == 0 {
                expected[basis] = Complex::one();
            } else if basis >> width & 1 == 1 {
                expected[basis & !1] = q;
                expected[basis | 1] = if basis & 1 == 0 { q } else { q.negated() };
            } else {
                expected[basis] = if basis & 1 == 1 { i } else { Complex::one() };
            }
            assert_eq!(column(&leaves, &inputs, basis).unwrap(), expected);
            count += 1;
        }
        for kind in ["qpred.scope", "ctl"] {
            let args = json::parse(r#"["p",[{"gate":"h","wires":["r[0]"]}]]"#).unwrap();
            let leaves = e.expand(kind, args.as_arr().unwrap(), &bundles).unwrap();
            let inputs = (0..width)
                .map(|b| ("r".into(), b))
                .chain([("p".into(), 0)])
                .collect::<Vec<_>>();
            for basis in 0..1 << (width + 1) {
                let mut expected = vec![Complex::default(); 1 << (width + 1)];
                if basis >> width & 1 == 0 {
                    expected[basis] = Complex::one();
                } else {
                    expected[basis & !1] = q;
                    expected[basis | 1] = if basis & 1 == 0 { q } else { q.negated() };
                }
                assert_eq!(column(&leaves, &inputs, basis).unwrap(), expected);
                count += 1;
            }
        }
        let forward=json::parse(r#"[{"gate":"h","wires":["r[0]"]},{"gate":"t","wires":["r[0]"]},{"gate":"s","wires":["r[0]"]}]"#).unwrap();
        // The source adjoint table produces SDG; TDG; H before binding adj.
        let source_adj = json::parse(r#"[{"gate":"sdg","wires":["r[0]"]},{"gate":"tdg","wires":["r[0]"]},{"gate":"h","wires":["r[0]"]}]"#).unwrap();
        let args = json::Json::Arr(vec![source_adj]);
        let mut leaves = e
            .expand("qloop", &[json::Json::Num(1.0), forward], &bundles)
            .unwrap();
        leaves.extend(e.expand("adj", args.as_arr().unwrap(), &bundles).unwrap());
        let inputs = (0..width).map(|b| ("r".into(), b)).collect::<Vec<_>>();
        for basis in 0..1 << width {
            let mut expected = vec![Complex::default(); 1 << width];
            expected[basis] = Complex::one();
            assert_eq!(column(&leaves, &inputs, basis).unwrap(), expected);
            count += 1;
        }
    }
    assert_eq!(count, 270);
    println!("coherent scoped/qif and H-T-S adjoint: {count} exact full-basis matrix columns");
}

#[test]
fn hadamard_loop_matrix_columns_at_static_bounds() {
    use bennett_isa_conformance::ref_amplitudes::{column, Complex, Real};
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    let q = Complex {
        re: Real::new(0, 1, 1),
        im: Real::integer(0),
    };
    let mut count = 0;
    for width in 1..=4 {
        for kind in ["qloop", "qloop.while"] {
            for bound in if kind == "qloop" {
                vec![0, 1, 2, 3, 17]
            } else {
                vec![0, 1, 2, 3]
            } {
                let args = json::parse(&if kind == "qloop" {
                    format!(r#"[{bound},[{{"gate":"h","wires":["r[0]"]}}]]"#)
                } else {
                    format!(r#"[{bound},"cond",[{{"gate":"h","wires":["r[0]"]}}]]"#)
                })
                .unwrap();
                let bundles = [("r", width), ("cond", bound)];
                let leaves = e.expand(kind, args.as_arr().unwrap(), &bundles).unwrap();
                let inputs = (0..width)
                    .map(|b| ("r".into(), b))
                    .chain((0..if kind == "qloop" { 0 } else { bound }).map(|b| ("cond".into(), b)))
                    .collect::<Vec<_>>();
                for basis in 0..1 << inputs.len() {
                    let prefix = if kind == "qloop" {
                        bound
                    } else {
                        (0..bound)
                            .take_while(|i| basis >> (width + i) & 1 == 1)
                            .count()
                    };
                    let mut expected = vec![Complex::default(); 1 << inputs.len()];
                    if prefix % 2 == 0 {
                        expected[basis] = Complex::one();
                    } else {
                        expected[basis & !1] = q;
                        expected[basis | 1] = if basis & 1 == 0 { q } else { q.negated() };
                    }
                    assert_eq!(
                        column(&leaves, &inputs, basis).unwrap(),
                        expected,
                        "{kind} N={bound}"
                    );
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 600);
    println!("Hadamard loop witnesses: {count} exact matrix columns");
}
