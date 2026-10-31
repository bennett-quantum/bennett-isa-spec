use bennett_isa_conformance::{
    json::{self, Json},
    ref_amplitudes::{column, Complex},
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
fn wrapper(width: usize, class: &str, state: &str, body: &str) -> Json {
    json::parse(&format!(r#"{{"parameters":[],"body":[{{"declare":"D","width":{width},"class":{{"class":"{class}"}},"state":"{state}","body":{body}}}]}}"#)).unwrap()
}
#[test]
fn allocation_output_copy_and_load_full_domains() {
    let mut registry = Registry::new();
    let mut cases = Vec::new();
    for n in 1..=4 {
        for cls in ["any", "ancilla", "data"] {
            for kind in ["qcopy", "qload.in", "qload.mut"] {
                for a in 1..=if kind == "qcopy" { 1 } else { 4 } {
                    let cells = 1 << a;
                    let (body, bundles) = if kind == "qcopy" {
                        (
                            format!(
                                r#"[{{"invoke":"qcopy.bound","args":[{n},"s","D"]}},{{"gate":"cx","wires":["D[0]","out"]}},{{"invoke":"quncopy.bound","args":[{n},"s","D"]}}]"#
                            ),
                            vec![("s", n), ("out", 1), ("control", 1)],
                        )
                    } else {
                        let role = kind.split('.').nth(1).unwrap();
                        (
                            format!(
                                r#"[{{"invoke":"{kind}","args":[{a},{cells},{n},"a",{{"address":{a},"cell":{n},"role":"{role}","origin":"local:r","cells":"s"}},"D"]}},{{"gate":"cx","wires":["D[0]","out"]}},{{"invoke":"qunload.{role}","args":[{a},{cells},{n},"a",{{"address":{a},"cell":{n},"role":"{role}","origin":"local:r","cells":"s"}},"D"]}}]"#
                            ),
                            vec![("s", cells * n), ("a", a), ("out", 1), ("control", 1)],
                        )
                    };
                    let hash = format!("{:064x}", cases.len());
                    registry.insert(hash.clone(), wrapper(n, cls, "clean", &body));
                    cases.push((hash, n, a, kind, cls, bundles));
                }
            }
        }
    }
    let mut e = Expander::new(include_str!("../../spec/ref-0.9-draft.json"), registry).unwrap();
    let mut tuples = 0u128;
    for (hash, n, a, kind, cls, bundles) in &cases {
        let leaves = e.expand(hash, &[], bundles).unwrap();
        assert!(leaves
            .iter()
            .filter_map(|l| l.allocation.as_ref())
            .any(|v| v.2 == *cls));
        let m = Machine::compile(&permutation(&leaves).unwrap(), bundles).unwrap();
        let mut b = Bdd::new();
        let mut state = m.symbolic_state();
        for name in ["a", "s", "out", "control"] {
            if let Some(ids) = m.inputs.get(name) {
                for id in ids {
                    state[*id] = b.variable();
                }
            }
        }
        let old = state.clone();
        let mut want = old.clone();
        let mut value = 0;
        if *kind == "qcopy" {
            value = old[m.inputs["s"][0]];
        } else {
            let selector = m.inputs["a"].iter().map(|i| old[*i]).collect::<Vec<_>>();
            for cell in 0..1 << a {
                let eq = b.equal(&selector, &Bdd::constant(cell, *a));
                let term = b.and(eq, old[m.inputs["s"][cell * n]]);
                value = b.xor(value, term);
            }
        }
        let enabled = old[m.inputs["control"][0]];
        let delta = b.and(enabled, value);
        let id = m.inputs["out"][0];
        want[id] = b.xor(want[id], delta);
        m.run_symbolic(&mut b, &mut state, enabled, false, 1)
            .unwrap();
        assert_eq!(state, want, "{kind} n={n} A={a}");
        m.run_symbolic(&mut b, &mut state, enabled, true, 1)
            .unwrap();
        assert_eq!(state, old);
        tuples += 1u128 << bundles.iter().map(|(_, w)| w).sum::<usize>();
    }
    assert_eq!(cases.len(), 108);
    println!("allocation copy/load: 108 shapes, {tuples} full-domain tuples");
}
#[test]
fn borrowed_output_restores_all_basis_columns_with_external_reference() {
    let mut registry = Registry::new();
    let mut cases = Vec::new();
    for n in 1..=4 {
        for cls in ["any", "ancilla", "data"] {
            let hash = format!("{:064x}", cases.len());
            let body = r#"[{"invoke":"qborrow","args":["D"]},{"gate":"cx","wires":["D[0]","out"]},{"gate":"cx","wires":["D[0]","out"]},{"invoke":"qreturn","args":["D"]}]"#;
            registry.insert(hash.clone(), wrapper(n, cls, "borrowed", body));
            cases.push((hash, n));
        }
    }
    let mut e = Expander::new(include_str!("../../spec/ref-0.9-draft.json"), registry).unwrap();
    let mut columns = 0;
    for (hash, n) in cases {
        let mut leaves = e.expand(&hash, &[], &[("out", 1)]).unwrap();
        let identity = leaves[0].allocation.as_ref().unwrap().0.clone();
        assert_eq!(leaves[0].allocation.as_ref().unwrap().3, "borrowed");
        // Expose unknown borrowed coordinates as interface inputs. Remove only
        // boundary events, preserving every semantic gate and all basis columns.
        leaves.retain(|l| l.allocation.is_none());
        let mut inputs = (0..n).map(|i| (identity.clone(), i)).collect::<Vec<_>>();
        inputs.push(("out".into(), 0));
        for basis in 0..1usize << (n + 1) {
            let mut expected = vec![Complex::default(); 1 << (n + 1)];
            expected[basis] = Complex::one();
            assert_eq!(column(&leaves, &inputs, basis).unwrap(), expected);
            columns += 1;
        }
    }
    assert_eq!(columns, 180);
    println!("borrowed full-basis columns {columns}");
}
#[test]
fn clean_allocation_source_rows_preserve_class_without_controls() {
    let mut registry = Registry::new();
    for n in 1..=4 {
        registry.insert(
            format!("{n:064x}"),
            wrapper(
                n,
                "special",
                "clean",
                r#"[{"invoke":"qalloc","args":["D"]},{"invoke":"qfree","args":["D"]}]"#,
            ),
        );
    }
    let mut e = Expander::new(include_str!("../../spec/ref-0.9-draft.json"), registry).unwrap();
    for n in 1..=4 {
        let leaves = e.expand(&format!("{n:064x}"), &[], &[]).unwrap();
        assert_eq!(leaves.len(), 2);
        assert_eq!(leaves[0].allocation, leaves[1].allocation);
        assert_eq!(leaves[0].allocation.as_ref().unwrap().2, "special");
        assert!(leaves.iter().all(|l| l.controls.is_empty()));
    }
}
