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
    let bytes = include_str!("../../spec/ref-0.9.json");
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
    let mut e = Expander::new(include_str!("../../spec/ref-0.9.json"), Registry::new()).unwrap();
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

#[test]
fn nested_body_captures_share_lexical_snapshots_and_preserve_free_wires() {
    // Every level captures two bodies. Deep copying both captured contexts
    // recursively doubles storage even though the expanded result is two gates.
    let mut recipes = Vec::new();
    for level in 0..=32 {
        let body = if level == 32 {
            r#"[{"splice":"B"},{"splice":"C"}]"#.to_string()
        } else {
            format!(
                r#"[{{"invoke":"r{:02}","args":[[{{"splice":"B"}}],[{{"splice":"C"}}]]}}]"#,
                level + 1
            )
        };
        recipes.push(format!(r#"{{"kind":"r{level:02}","parameters":[{{"name":"B","type":"body"}},{{"name":"C","type":"body"}}],"body":{body}}}"#));
    }
    let catalog = doc(&format!("[{}]", recipes.join(",")));
    let args =
        json::parse(r#"[[{"gate":"x","wires":["a"]}],[{"gate":"x","wires":["b"]}]]"#).unwrap();
    let leaves = Expander::new(&catalog, Registry::new())
        .unwrap()
        .expand("r00", args.as_arr().unwrap(), &[("a", 1), ("b", 1)])
        .unwrap();
    assert_eq!(
        leaves
            .iter()
            .map(|leaf| leaf.wires.clone())
            .collect::<Vec<_>>(),
        vec![vec![("a".into(), 0)], vec![("b".into(), 0)]]
    );
}

#[test]
fn nested_allocation_scopes_use_the_default_stack_and_keep_lifetime_order() {
    use json::Json;
    use std::collections::BTreeMap;
    const DEPTH: usize = 192;
    let mut body = Vec::new();
    for i in 0..DEPTH {
        body.push(Json::Obj(BTreeMap::from([(
            "acquire".into(),
            Json::Str(format!("q{i}")),
        )])));
    }
    for _ in 0..2 {
        body.push(Json::Obj(BTreeMap::from([
            ("gate".into(), Json::Str("x".into())),
            ("wires".into(), Json::Arr(vec![Json::Str("q0".into())])),
        ])));
    }
    for i in (0..DEPTH).rev() {
        body.push(Json::Obj(BTreeMap::from([(
            "relinquish".into(),
            Json::Str(format!("q{i}")),
        )])));
    }
    for i in (0..DEPTH).rev() {
        body = vec![Json::Obj(BTreeMap::from([
            ("declare".into(), Json::Str(format!("q{i}"))),
            ("width".into(), Json::Num(1.0)),
            ("class".into(), json::parse(r#"{"class":"any"}"#).unwrap()),
            ("state".into(), Json::Str("clean".into())),
            ("body".into(), Json::Arr(body)),
        ]))];
    }
    let hash = "a".repeat(64);
    let registry = Registry::from([(
        hash.clone(),
        Json::Obj(BTreeMap::from([
            ("parameters".into(), Json::Arr(vec![])),
            ("body".into(), Json::Arr(body)),
        ])),
    )]);
    let leaves = Expander::new(include_str!("../../spec/ref-0.9.json"), registry)
        .unwrap()
        .expand(&hash, &[], &[])
        .unwrap();
    assert_eq!(leaves.len(), 2 * DEPTH + 2);
    assert_eq!(leaves[DEPTH].gate, "x");
    assert_eq!(leaves[DEPTH + 1].gate, "x");
    for i in 0..DEPTH {
        assert_eq!(leaves[i].gate, "alloc");
        let release = &leaves[leaves.len() - 1 - i];
        assert_eq!(release.gate, "release");
        assert_eq!(leaves[i].allocation, release.allocation);
        assert_eq!(leaves[i].allocation.as_ref().unwrap().1, 1);
    }
}

#[test]
fn source_leaf_budget_fails_explicitly_resets_and_leaves_plain_catalogs_unchanged() {
    use bennett_isa_conformance::ref_language::{
        SourceBody, SourceFactory, HOST_RESOURCE_LIMIT_PREFIX,
    };
    use std::{collections::BTreeMap, sync::Arc};
    #[derive(Debug)]
    struct Factory(json::Json);
    impl SourceFactory for Factory {
        fn construct(
            &self,
            hash: &str,
            _: &BTreeMap<String, json::Json>,
            _: &BTreeMap<String, json::Json>,
            _: Option<&json::Json>,
            _: Option<&bennett_isa_conformance::ref_language::SourceTokenContext>,
            _: bool,
        ) -> Result<SourceBody, String> {
            Ok(SourceBody {
                body: self.0.clone(),
                symbols: vec![],
                symbol: hash.into(),
                oracle: false,
                points: BTreeMap::new(),
                rows: vec![],
            })
        }
    }
    let body = json::parse(r#"[{"for":"i","from":0,"to":9,"body":[{"gate":"x","wires":["w"]}]}]"#)
        .unwrap();
    let hash = "c".repeat(64);
    let recipe = json::parse(&format!(
        r#"{{"parameters":[{{"name":"w","type":"wire","width":1}}],"body":{body}}}"#
    ))
    .unwrap();
    let mut expander = Expander::new(&doc("[]"), Registry::from([(hash.clone(), recipe)]))
        .unwrap()
        .with_source_factory(Arc::new(Factory(body)))
        .with_source_leaf_limit(8);
    let args = [json::Json::Str("a".into())];
    let error = expander.expand(&hash, &args, &[("a", 1)]).unwrap_err();
    assert!(error.starts_with(HOST_RESOURCE_LIMIT_PREFIX));
    assert!(error.contains("leaf budget (8)"));
    let mut expander = expander.with_source_leaf_limit(9);
    for _ in 0..2 {
        assert_eq!(expander.expand(&hash, &args, &[("a", 1)]).unwrap().len(), 9);
    }
    let plain = doc(
        r#"[{"kind":"plain","parameters":[{"name":"w","type":"wire","width":1}],"body":[{"gate":"x","wires":["w"]}]}]"#,
    );
    assert_eq!(
        Expander::new(&plain, Registry::new())
            .unwrap()
            .with_source_leaf_limit(0)
            .expand("plain", &args, &[("a", 1)])
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn raw_inverse_retains_the_complete_forward_source_tree_and_leaf_identities() {
    use bennett_isa_conformance::ref_language::{
        SourceBody, SourceFactory, SourcePoint, SourceSymbol,
    };
    use std::{collections::BTreeMap, sync::Arc};
    #[derive(Debug)]
    struct Factory;
    impl SourceFactory for Factory {
        fn construct(
            &self,
            hash: &str,
            _: &BTreeMap<String, json::Json>,
            _: &BTreeMap<String, json::Json>,
            _: Option<&json::Json>,
            _: Option<&bennett_isa_conformance::ref_language::SourceTokenContext>,
            _: bool,
        ) -> Result<SourceBody, String> {
            let point = SourcePoint {
                symbol: hash.into(),
                point: "entry:0".into(),
                iteration: vec![],
                instance: vec![],
            };
            let child = "c".repeat(64);
            let nested = hash != child;
            Ok(SourceBody {
                symbol: hash.into(),
                oracle: true,
                points: BTreeMap::from([("$source-point/entry:0".into(), point)]),
                rows: vec![],
                symbols: if nested {
                    vec![SourceSymbol {
                        name: "C".into(),
                        hash: child,
                        oracle_invocation: None,
                        tokens: None,
                        adjoint: false,
                    }]
                } else {
                    vec![]
                },
                body: json::parse(if nested {
                    r#"[{"for":"$source-point/entry:0","from":0,"to":1,"body":[{"alloc":"tmp","width":1,"class":"any","state":"clean"},{"call":"C","args":["w"]},{"call":"C","args":["w"]},{"release":"tmp"}]}]"#
                } else {
                    r#"[{"for":"$source-point/entry:0","from":0,"to":1,"body":[{"alloc":"tmp","width":1,"class":"any","state":"clean"},{"gate":"rx","wires":["w"],"params":["0x3ff0000000000000"]},{"release":"tmp"}]}]"#
                })?,
            })
        }
    }
    let hash = "d".repeat(64);
    let source =
        json::parse(r#"{"parameters":[{"name":"w","type":"wire","width":1}],"body":[]}"#).unwrap();
    let forward = r#"[{"call":"S","args":"A"}]"#;
    let inverse = format!(r#"[{{"inverse":true,"body":{forward}}}]"#);
    let expand = |body: &str| {
        let catalog = doc(&format!(
            r#"[{{"kind":"root","parameters":[{{"name":"S","type":"symbol"}},{{"name":"A","type":"arguments","symbol":"S"}}],"body":{body}}}]"#
        ));
        let args = json::parse(&format!(r#"["{hash}",["a"]]"#)).unwrap();
        Expander::new(
            &catalog,
            Registry::from([
                (hash.clone(), source.clone()),
                ("c".repeat(64), source.clone()),
            ]),
        )
        .unwrap()
        .with_source_factory(Arc::new(Factory))
        .expand("root", args.as_arr().unwrap(), &[("a", 1)])
        .unwrap()
    };
    let fwd = expand(forward);
    let rev = expand(&inverse);
    assert_eq!(fwd.len(), rev.len());
    let occurrences = fwd
        .iter()
        .filter_map(|leaf| {
            leaf.source
                .as_ref()?
                .path
                .first()
                .map(|edge| edge.call_index)
        })
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(occurrences, std::collections::BTreeSet::from([0, 1]));
    for (a, b) in fwd.iter().zip(rev.iter().rev()) {
        assert_eq!(a.invocation, b.invocation);
        assert_eq!(a.ordinal, b.ordinal);
        assert_eq!(a.source, b.source);
        assert_eq!(a.wires, b.wires);
        assert_eq!(a.controls, b.controls);
        assert_eq!(a.allocation, b.allocation);
        if a.gate == "rx" {
            assert_eq!(a.angles[0] ^ (1u64 << 63), b.angles[0]);
        } else {
            assert!(matches!(
                (a.gate.as_str(), b.gate.as_str()),
                ("alloc", "release") | ("release", "alloc")
            ));
        }
    }
}

#[test]
fn direct_empty_registry_entry_uses_its_explicit_bundle_map_and_nested_calls_stay_closed() {
    let hash = "a".repeat(64);
    let free =
        json::parse(r#"{"parameters":[],"body":[{"gate":"x","wires":["external"]}]}"#).unwrap();
    let out = Expander::new(&doc("[]"), Registry::from([(hash.clone(), free.clone())]))
        .unwrap()
        .expand(&hash, &[], &[("external", 1)])
        .unwrap();
    assert_eq!(out[0].wires, vec![("external".into(), 0)]);
    let caller = doc(
        r#"[{"kind":"caller","parameters":[{"name":"S","type":"symbol"}],"body":[{"call":"S","args":[]}]}]"#,
    );
    assert!(
        Expander::new(&caller, Registry::from([(hash.clone(), free)]))
            .unwrap()
            .expand(
                "caller",
                &[json::Json::Str(hash.clone())],
                &[("external", 1)]
            )
            .is_err()
    );
    let typed=json::parse(r#"{"parameters":[{"name":"w","type":"wire","width":1}],"body":[{"gate":"x","wires":["w"]}]}"#).unwrap();
    let out = Expander::new(&doc("[]"), Registry::from([(hash.clone(), typed)]))
        .unwrap()
        .expand(&hash, &[json::Json::Str("a".into())], &[("a", 1), ("w", 2)])
        .unwrap();
    assert_eq!(out[0].wires, vec![("a".into(), 0)]);
}
