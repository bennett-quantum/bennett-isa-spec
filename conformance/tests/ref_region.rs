use bennett_isa_conformance::{
    json,
    ref_amplitudes::{sparse_column, Complex},
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
#[test]
fn arbitrary_image_scopes_are_unconditional_and_restore_clean_allocation() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut tuples = 0;
    let mut columns = 0;
    for (width, cells) in (1..=4).flat_map(|w| [(w, 1), (w, 8)]) {
        let mut images = if cells == 1 {
            vec![String::new()]
        } else if width == 1 {
            (0..256).map(|b| format!("{b:02x}")).collect::<Vec<_>>()
        } else {
            vec![
                "00".repeat(width),
                "ff".repeat(width),
                "aa".repeat(width),
                format!("81{}", "02".repeat(width - 1)),
            ]
        };
        images.sort();
        images.dedup();
        for image in images {
            let first = if image.is_empty() {
                0
            } else {
                u8::from_str_radix(&image[..2], 16).unwrap() & 1
            };
            let args=json::parse(&format!(r#"[{cells},{width},"{image}",[{{"parameters":[{{"name":"cells","type":"wire","width":{bits}}}],"body":[{{"gate":"cx","wires":["cells[0]","out"]}}]}}],"control"]"#,bits=cells*width)).unwrap();
            let bundles = [("out", 1), ("control", 1)];
            let leaves = e
                .expand("qregion", args.as_arr().unwrap(), &bundles)
                .unwrap();
            let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
            let mut b = Bdd::new();
            let mut old = m.symbolic_state();
            for name in ["out", "control"] {
                old[m.inputs[name][0]] = b.variable();
            }
            let mut expected = old.clone();
            let out = m.inputs["out"][0];
            let control = old[m.inputs["control"][0]];
            if first == 1 {
                expected[out] = b.xor(old[out], control);
            }
            let mut state = old.clone();
            m.run_symbolic(&mut b, &mut state, 1, false, 1).unwrap();
            assert_eq!(state, expected);
            m.run_symbolic(&mut b, &mut state, 1, true, 1).unwrap();
            assert_eq!(state, old);
            tuples += 4;
            // Coherent full-basis scope witness: H CX(imagebit,out) H = Z^imagebit.
            let matrix_args=json::parse(&args.to_string().replace(r#""body":[{"gate":"cx","wires":["cells[0]","out"]}]"#,r#""body":[{"gate":"h","wires":["out"]},{"gate":"cx","wires":["cells[0]","out"]},{"gate":"h","wires":["out"]}]"#)).unwrap();
            let leaves = e
                .expand("qregion", matrix_args.as_arr().unwrap(), &bundles)
                .unwrap();
            let inputs = vec![("out".into(), 0), ("control".into(), 0)];
            for basis in 0..4 {
                let mut expected = vec![Complex::default(); 4];
                expected[basis] = if first == 1 && basis == 3 {
                    Complex::one().negated()
                } else {
                    Complex::one()
                };
                assert_eq!(sparse_column(&leaves, &inputs, basis).unwrap(), expected);
                columns += 1;
            }
        }
    }
    assert_eq!(tuples, 1088);
    assert_eq!(columns, 1088);
    println!("qregion: W1..4, C1/8; 272 image configurations; {tuples} full-domain tuples and {columns} exact coherent matrix columns");
}

#[test]
fn sparse_and_dense_exact_columns_agree_on_entangled_clifford_t_states() {
    use bennett_isa_conformance::ref_amplitudes::column;
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let args=json::parse(r#"[[{"gate":"h","wires":["a[0]"]},{"gate":"cx","wires":["a[0]","a[1]"]},{"gate":"t","wires":["a[0]"]},{"gate":"sdg","wires":["a[1]"]},{"gate":"h","wires":["a[1]"]},{"gate":"cz","wires":["a[0]","a[1]"]},{"gate":"swap","wires":["a[0]","a[1]"]}]]"#).unwrap();
    let leaves = e
        .expand("adj", args.as_arr().unwrap(), &[("a", 2)])
        .unwrap();
    let inputs = vec![("a".into(), 0), ("a".into(), 1)];
    for basis in 0..4 {
        assert_eq!(
            column(&leaves, &inputs, basis).unwrap(),
            sparse_column(&leaves, &inputs, basis).unwrap()
        );
    }
}
