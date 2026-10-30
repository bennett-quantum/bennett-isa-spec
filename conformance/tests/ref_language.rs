use bennett_isa_conformance::{
    json,
    ref_language::{Expander, Registry},
};

fn doc(recipes: &str) -> String {
    let mut recipes = json::parse(recipes).unwrap();
    if let json::Json::Arr(rs) = &mut recipes {
        for r in rs {
            if let json::Json::Obj(m) = r {
                m.entry("scratch".into()).or_insert(json::Json::Arr(vec![]));
                m.insert("removable".into(), json::Json::Arr(vec![]));
                m.insert(
                    "source".into(),
                    json::Json::Str("§11.1 independent binding witness".into()),
                );
            }
        }
    }
    format!(r#"{{"isa":"0.9","name":"ref","version":"0.9","recipes":{recipes}}}"#)
}
#[test]
fn typed_binding_capture_paths_angles_and_lists() {
    let d = doc(
        r#"[{"kind":"child","parameters":[{"name":"N","type":"integer"},{"name":"w","type":"wire","width":"N"},{"name":"theta","type":"angle"},{"name":"B","type":"body"}],"body":[{"gate":"rx","wires":["w[0]"],"params":["theta"]},{"splice":"B"}]},{"kind":"root","parameters":[{"name":"a","type":"wire","width":1}],"body":[{"invoke":"child","args":[1,{"range":"a","from":0,"to":1},"0x8000000000000000",[{"alloc":"tmp","width":1,"class":"any","state":"clean"},{"gate":"x","wires":["tmp[0]"]},{"gate":"x","wires":["tmp[0]"]},{"release":"tmp"}]]},{"inverse":true,"body":[{"gate":"ry","wires":["a[0]"],"params":["0x3ff0000000000000"]}]}]}]"#,
    );
    let mut e = Expander::new(&d, Registry::default()).unwrap();
    let leaves = e
        .expand("root", &[json::parse("\"a\"").unwrap()], &[("a", 1)])
        .unwrap();
    assert_eq!(leaves.len(), 6);
    assert_eq!(leaves[0].angles, vec![0x8000000000000000]);
    assert_eq!(leaves[5].angles, vec![0xbff0000000000000]);
    let bad = d.replace("\"to\":1", "\"to\":0");
    assert!(Expander::new(&bad, Registry::default())
        .unwrap()
        .expand("root", &[json::parse("\"a\"").unwrap()], &[("a", 1)])
        .is_err());
}
#[test]
fn variadic_controls_and_rejections() {
    let d = doc(
        r#"[{"kind":"mcx","parameters":[{"name":"n","type":"integer"},{"name":"w","type":"wire","width":"n"},{"name":"t","type":"wire","width":1}],"body":[{"gate":"mcx","wires":["w","t"]}]}]"#,
    );
    let args =
        json::parse(r#"[2,{"range":"a","from":0,"to":2},{"range":"a","from":2,"to":3}]"#).unwrap();
    let mut e = Expander::new(&d, Registry::default()).unwrap();
    assert_eq!(
        e.expand("mcx", args.as_arr().unwrap(), &[("a", 3)])
            .unwrap()[0]
            .wires
            .len(),
        3
    );
    assert!(e.expand("mcx", &[], &[]).is_err());
}

#[test]
fn bodies_are_lexical_and_calls_keep_ordinal_spaces() {
    let d = doc(
        r#"[{"kind":"repeat","parameters":[{"name":"B","type":"body"},{"name":"S","type":"symbol"}],"body":[{"alloc":"tmp","width":1,"class":"any","state":"clean"},{"splice":"B"},{"splice":"B"},{"release":"tmp"},{"call":"S","args":[]},{"call":"S","args":[]}]}]"#,
    );
    let hash = "a".repeat(64);
    let mut reg = Registry::new();
    reg.insert(hash.clone(),json::parse(r#"{"parameters":[],"body":[{"alloc":"z","width":1,"class":"any","state":"clean"},{"gate":"x","wires":["z[0]"]},{"gate":"x","wires":["z[0]"]},{"release":"z"}]}"#).unwrap());
    let args=json::parse(&format!(r#"[[{{"alloc":"tmp","width":1,"class":"any","state":"clean"}},{{"release":"tmp"}}],"{hash}"]"#)).unwrap();
    let out = Expander::new(&d, reg)
        .unwrap()
        .expand("repeat", args.as_arr().unwrap(), &[])
        .unwrap();
    let allocations: Vec<_> = out
        .iter()
        .filter(|l| l.gate == "alloc")
        .map(|l| &l.allocation.as_ref().unwrap().0)
        .collect();
    assert_eq!(
        allocations
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        5
    );
    assert_ne!(out[6].invocation, out[10].invocation);
    assert_eq!((out[6].ordinal, out[10].ordinal), (0, 0));
}
#[test]
fn each_restores_bindings_and_invalid_documents_fail_loudly() {
    let d = doc(
        r#"[{"kind":"each","parameters":[{"name":"w","type":"wire","width":2}],"body":[{"each":"bit","in":"w","body":[{"gate":"x","wires":["bit"]}]}]}]"#,
    );
    let args = json::parse(r#"["a"]"#).unwrap();
    let out = Expander::new(&d, Registry::new())
        .unwrap()
        .expand("each", args.as_arr().unwrap(), &[("a", 2)])
        .unwrap();
    assert_eq!(out.iter().map(|l| l.wires[0].1).collect::<Vec<_>>(), [0, 1]);
    for bad in [
        d.replace("\"gate\":\"x\"", "\"gate\":\"private\""),
        d.replace("\"wires\":[\"bit\"]", "\"wires\":[\"unknown[0]\"]"),
    ] {
        assert!(Expander::new(&bad, Registry::new())
            .unwrap()
            .expand("each", args.as_arr().unwrap(), &[("a", 2)])
            .is_err());
    }
}

#[test]
fn full_basis_h_and_phase_witnesses_are_exact() {
    use bennett_isa_conformance::ref_amplitudes::{column, Complex, Real};
    for gate in ["h", "s", "sdg", "t", "tdg"] {
        let d = doc(&format!(
            r#"[{{"kind":"g","parameters":[{{"name":"a","type":"wire","width":1}}],"body":[{{"gate":"{gate}","wires":["a"]}}]}}]"#
        ));
        let args = json::parse(r#"["a"]"#).unwrap();
        let leaves = Expander::new(&d, Registry::new())
            .unwrap()
            .expand("g", args.as_arr().unwrap(), &[("a", 1)])
            .unwrap();
        let inputs = vec![("a".into(), 0)];
        let q = Real::new(0, 1, 1);
        let z = Complex::default();
        let r = Complex {
            re: q,
            im: Real::integer(0),
        };
        let phase = match gate {
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
            _ => z,
        };
        for b in 0..2 {
            let expected = if gate == "h" {
                vec![r, if b == 0 { r } else { r.negated() }]
            } else if b == 0 {
                vec![Complex::one(), z]
            } else {
                vec![z, phase]
            };
            assert_eq!(column(&leaves, &inputs, b).unwrap(), expected);
        }
    }
}

#[test]
fn bound_classical_catalog_preserves_every_event_operand_byte() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let d = json::parse(bytes).unwrap();
    let mut count = 0;
    let mut expander = Expander::new(bytes, Registry::new()).unwrap();
    for r in d.get("recipes").unwrap().as_arr().unwrap() {
        let kind = r.get("kind").unwrap().as_str().unwrap();
        let Some(op) = kind.strip_prefix("cc.") else {
            continue;
        };
        for n in 1..=4 {
            for a in 0..1 << n {
                for b in 0..1 << n {
                    let opcode = if op == "measure.iN" {
                        format!("measure.i{n}")
                    } else {
                        op.to_string()
                    };
                    let event = json::parse(&format!(
                        r#"{{"instruction":"{opcode} {a} {b}","point":"block.0.row.{n}"}}"#
                    ))
                    .unwrap();
                    let leaves = expander
                        .expand(kind, std::slice::from_ref(&event), &[])
                        .unwrap();
                    assert_eq!(leaves.len(), 1);
                    assert_eq!(leaves[0].event.as_ref(), Some(&event));
                    assert!(leaves[0].controls.is_empty());
                    count += 1;
                }
            }
        }
    }
    assert_eq!(count, 48 * (4 + 16 + 64 + 256));
    println!("classical bound event byte-preservation: {count} configurations (not classical execution witnesses)");
}

#[test]
fn cyclic_typed_invokes_and_nonfinite_angles_reject() {
    let d = doc(r#"[{"kind":"root","parameters":[],"body":[{"invoke":"root","args":[]}]}]"#);
    assert!(Expander::new(&d, Registry::new()).is_err());
    let d = doc(
        r#"[{"kind":"qloop","parameters":[{"name":"B","type":"body"}],"body":[{"splice":"B"}]},{"kind":"root","parameters":[],"body":[{"invoke":"qloop","args":[[{"invoke":"root","args":[]}]]}]}]"#,
    );
    assert!(Expander::new(&d, Registry::new()).is_err());
    for angle in [
        "0x7ff0000000000000",
        "0x7ff8000000000001",
        "0xBFF0000000000000",
    ] {
        let d = doc(&format!(
            r#"[{{"kind":"r","parameters":[{{"name":"a","type":"wire","width":1}}],"body":[{{"gate":"rx","wires":["a"],"params":["{angle}"]}}]}}]"#
        ));
        let args = json::parse(r#"["a"]"#).unwrap();
        assert!(Expander::new(&d, Registry::new())
            .unwrap()
            .expand("r", args.as_arr().unwrap(), &[("a", 1)])
            .is_err());
    }
}

#[test]
fn effective_controls_are_disjoint_from_diagonal_and_swap_targets() {
    for (g, ws) in [("s", r#"["a"]"#), ("swap", r#"["a","b"]"#)] {
        let d = doc(&format!(
            r#"[{{"kind":"g","parameters":[{{"name":"a","type":"wire","width":1}},{{"name":"b","type":"wire","width":1}}],"body":[{{"under":"a","body":[{{"gate":"{g}","wires":{ws}}}]}}]}}]"#
        ));
        let args = json::parse(r#"["a","b"]"#).unwrap();
        assert!(Expander::new(&d, Registry::new())
            .unwrap()
            .expand("g", args.as_arr().unwrap(), &[("a", 1), ("b", 1)])
            .is_err());
    }
}

#[test]
fn library_scratch_requires_reverse_release_and_balanced_lifetimes() {
    for body in [
        r#"[{"alloc":"a","width":1,"class":"any","state":"clean"}]"#,
        r#"[{"alloc":"a","width":1,"class":"any","state":"clean"},{"alloc":"b","width":1,"class":"any","state":"clean"},{"release":"a"},{"release":"b"}]"#,
    ] {
        let d = doc(&format!(
            r#"[{{"kind":"bad","parameters":[],"scratch":["a","b"],"body":{body}}}]"#
        ));
        assert!(Expander::new(&d, Registry::new())
            .unwrap()
            .expand("bad", &[], &[])
            .is_err());
    }
}

#[test]
fn unused_recipe_constructor_and_formal_shapes_are_still_closed() {
    for body in [
        r#"[{"private":"x"}]"#,
        r#"[{"gate":"x","wires":[],"secret":true}]"#,
        r#"[{"for":"i","from":0,"to":0,"body":[{"private":"x"}]}]"#,
    ] {
        let d = doc(&format!(
            r#"[{{"kind":"unused","parameters":[],"body":{body}}}]"#
        ));
        assert!(Expander::new(&d, Registry::new()).is_err());
    }
    let d = doc(
        r#"[{"kind":"unused","parameters":[{"name":"w","type":"wire","width":1,"secret":true}],"body":[]}]"#,
    );
    assert!(Expander::new(&d, Registry::new()).is_err());
}

#[test]
fn canonical_integer_literal_has_only_lsb_ordered_x_leaves() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let args = json::parse(r#"[8,13,"w"]"#).unwrap();
    let leaves = e
        .expand("value.literal", args.as_arr().unwrap(), &[("w", 8)])
        .unwrap();
    assert_eq!(leaves.len(), 3);
    assert!(leaves.iter().all(|l| l.gate == "x"));
    assert_eq!(
        leaves.iter().map(|l| l.wires[0].1).collect::<Vec<_>>(),
        [0, 2, 3]
    );
}
