mod support;
use bennett_isa_ref::{json::Json, Object};
fn object() -> Object {
    let root = r#"{"abi":"0.9","args":["token","token"],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let worker = r#"{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[1],"result":1,"tier":"q","tokens":["token"]}"#;
    support::rehash(&support::object(&[("o",root,"oracle @o (x:wire in) (t:wire) (f:wire[4]) () (left:token, right:token)\nregs { r:reg[1] }\n^entry():\nqbranch x ^left ^right\n^left():\nr = lift.call @w (x) (left) () ()\nqjump ^done\n^right():\nr = lift.call @w (x) (right) () ()\nqjump ^done\n^done():\nlift.retval r"),("w",worker,"lproc @w (x:reg[1]) (r:reg[1]) (tk:token) () ()\n^entry():\nlift.ret x")],"{}","\nsection layout\n{\"A_tab\":1,\"H\":2,\"S_cells\":3,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9"))
}
fn entry(o: &Object, function: &str, point: &str, instance: &str) -> Json {
    let symbol = &o.identity().unwrap().interface_hashes[function];
    support::json(&format!(
        r#"{{"generated":"*","kind":"gate","path":[{{"instance":{instance},"iteration":[],"point":"{point}","symbol":"{symbol}"}}],"phase":"*","step":"*","gate":{{"epsilon_share":0,"realisation":"native"}}}}"#
    ))
}
fn scheduled(o: &Object, entries: &[Json], steps: u64) -> Result<Object, bennett_isa_ref::Error> {
    scheduled_args(
        o,
        entries,
        steps,
        r#"[["1","0","0","0"],["1","0","0","0"]]"#,
    )
}
fn scheduled_args(
    o: &Object,
    entries: &[Json],
    steps: u64,
    args: &str,
) -> Result<Object, bennett_isa_ref::Error> {
    let hash = o.identity().unwrap().closure_hashes["o"].clone().unwrap();
    let params = bennett_isa_ref::sha256_hex(args.as_bytes());
    let entries = Json::Array(entries.to_vec()).canonical();
    let schedule = support::json(&format!(
        r#"{{"bindings":{{"budget":[{steps},0,5,1],"closure_hash":"{hash}","epsilon":0,"inherited_controls":0,"params_hash":"{params}","region":"o","scheduler":{{"name":"source-coordinate","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":{entries}}}"#,
        "0".repeat(64)
    ));
    Object::parse(&format!(
        "{}\nsection schedule\n{}\n",
        o.text,
        schedule.canonical()
    ))
}
#[test]
fn support_points_and_lproc_rows_are_in_the_containing_oracle_invocation() {
    let o = object();
    for point in ["$construct", "$step", "$commit", "$release"] {
        scheduled(&o, &[entry(&o, "o", point, "[]")], 0).unwrap();
    }
    for instance in ["[0]", "[1]", "\"*\""] {
        scheduled(&o, &[entry(&o, "w", "entry:0", instance)], 5).unwrap();
    }
    for instance in ["[]", "[2]", "[0,1]"] {
        assert!(scheduled(&o, &[entry(&o, "w", "entry:0", instance)], 5).is_err());
    }
    assert!(scheduled(&o, &[entry(&o, "o", "$construct", "[0]")], 0).is_err());
    assert!(scheduled(&o, &[entry(&o, "w", "$construct", "[0]")], 0).is_err());
}
#[test]
fn disjoint_instance_selectors_do_not_tie_and_resolve_independently() {
    let o = object();
    let a = entry(&o, "w", "entry:0", "[0]");
    let b = entry(&o, "w", "entry:0", "[1]");
    let bound = scheduled(&o, &[a.clone(), b.clone()], 5).unwrap();
    assert_eq!(bound.resolve_schedule(&a, "gate").unwrap(), Some(&a));
    assert_eq!(bound.resolve_schedule(&b, "gate").unwrap(), Some(&b));
    assert!(scheduled(&o, &[a.clone(), a], 5).is_err());
}
#[test]
fn support_ties_exist_outside_steps_even_for_zero_budget() {
    let o = object();
    let construct = entry(&o, "o", "$construct", "[]");
    assert!(scheduled(&o, &[construct.clone(), construct], 0)
        .unwrap_err()
        .to_string()
        .contains("multiple maximal"));
    let step = entry(&o, "o", "$step", "[]");
    scheduled(&o, &[step.clone(), step], 0).unwrap();
}

#[test]
fn covering_both_phases_of_one_instance_does_not_hide_another_instances_tie() {
    let o = object();
    let broad = entry(&o, "w", "entry:0", "\"*\"");
    let mut fwd = entry(&o, "w", "entry:0", "[0]");
    let mut rev = fwd.clone();
    if let Json::Object(fields) = &mut fwd {
        fields.insert("phase".into(), Json::String("fwd".into()));
    }
    if let Json::Object(fields) = &mut rev {
        fields.insert("phase".into(), Json::String("rev".into()));
    }
    let error = scheduled(&o, &[broad.clone(), broad, fwd, rev], 5)
        .expect_err("instance[1] still has two maximal broad entries");
    assert!(error.to_string().contains("multiple maximal"));
}

fn nested_object() -> Object {
    let signature = r#"{"abi":"0.9","args":["token","token"],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let child =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let layout = object().section("layout").unwrap().canonical();
    support::rehash(&support::object(&[("o",signature,"oracle @o (x:wire in) (t:wire) (f:wire[4]) () (left:token, right:token)\nregs { r:reg[1], fr:reg[4] }\n^entry():\nlift.oracle_call @inner left (x) r fr () ()\nlift.retval r"),("inner",child,"oracle @inner (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x")],"{}",&format!("\nsection layout\n{layout}\n")).replace("=0.8","=0.9"))
}
#[test]
fn source_leaf_matching_reads_the_oracle_level_selected_by_each_path() {
    use bennett_isa_conformance::ref_language::{
        Leaf, SourceEdge, SourceInvocation, SourceLocation, SourcePoint, SourceStep,
    };
    use std::sync::Arc;
    let o = nested_object();
    let id = o.identity().unwrap();
    let root = &id.interface_hashes["o"];
    let inner = &id.interface_hashes["inner"];
    let call = SourcePoint {
        symbol: root.clone(),
        point: "entry:0".into(),
        iteration: vec![],
        instance: vec![],
    };
    let point = SourcePoint {
        symbol: inner.clone(),
        point: "entry:0".into(),
        iteration: vec![],
        instance: vec![],
    };
    let mut leaf = Leaf {
        invocation: vec![0, 1],
        ordinal: 0,
        gate: "x".into(),
        wires: vec![("t".into(), 0)],
        controls: vec![],
        angles: vec![],
        allocation: None,
        event: None,
        scratch: false,
        source: Some(Arc::new(SourceLocation {
            path: vec![SourceEdge {
                point: call,
                call_index: 0,
            }],
            terminal: point,
            steps: vec![
                SourceStep {
                    invocation: vec![0],
                    symbol: root.clone(),
                    step: 1,
                    reverse: false,
                },
                SourceStep {
                    invocation: vec![0, 1],
                    symbol: inner.clone(),
                    step: 3,
                    reverse: false,
                },
            ],
            invocations: vec![
                SourceInvocation {
                    invocation: vec![0],
                    symbol: root.clone(),
                    oracle: true,
                },
                SourceInvocation {
                    invocation: vec![0, 1],
                    symbol: inner.clone(),
                    oracle: true,
                },
            ],
        })),
    };
    // Finite matching adapter, independently of the source-construction gates:
    // a root-prefix selector reads step1 inside nested step3 or nested support.
    let mut broad = entry(&o, "o", "entry:0", "[]");
    if let Json::Object(fields) = &mut broad {
        fields.insert("path".into(), Json::Array(vec![]));
        fields.insert("step".into(), Json::Number("1".into()));
        fields.insert("phase".into(), Json::String("fwd".into()));
    }
    let bound = scheduled_args(
        &o,
        &[broad.clone()],
        5,
        r#"[["3","0","0","0"],["1","0","0","0"]]"#,
    )
    .unwrap();
    assert_eq!(
        bound.resolve_source_leaf(&leaf, "gate").unwrap(),
        Some(&broad)
    );
    let mut nested = entry(&o, "o", "entry:0", "[]");
    if let Json::Object(fields) = &mut nested {
        fields.insert("step".into(), Json::Number("3".into()));
        fields.insert("phase".into(), Json::String("fwd".into()));
        if let Json::Array(path) = fields.get_mut("path").unwrap() {
            if let Json::Object(edge) = &mut path[0] {
                edge.insert("call_index".into(), Json::Number("0".into()));
            }
        }
    }
    let bound = scheduled_args(
        &o,
        &[broad.clone(), nested.clone()],
        5,
        r#"[["3","0","0","0"],["1","0","0","0"]]"#,
    )
    .unwrap();
    assert_eq!(
        bound.resolve_source_leaf(&leaf, "gate").unwrap(),
        Some(&nested)
    );
    let mut support = (*leaf.source.as_ref().unwrap().as_ref()).clone();
    support.steps.pop();
    support.terminal.point = "$construct".into();
    leaf.source = Some(Arc::new(support));
    assert_eq!(
        bound.resolve_source_leaf(&leaf, "gate").unwrap(),
        Some(&broad)
    );
}

#[test]
fn real_source_instances_share_the_oracle_level_and_match_distinct_selectors() {
    use bennett_isa_conformance::{json, ref_catalog::Machine, ref_language::permutation};
    use bennett_isa_ref::{ref_library::OracleInvocation, Budget};
    let o = object();
    let a = entry(&o, "w", "entry:0", "[0]");
    let b = entry(&o, "w", "entry:0", "[1]");
    let bound = scheduled(&o, &[a.clone(), b.clone()], 1).unwrap();
    let arguments = json::parse(r#"["x","t","f",["1","0","0","0"],["1","0","0","0"]]"#).unwrap();
    let context = json::parse(r#"{"token":["1","0","5","1"]}"#).unwrap();
    let capture = OracleInvocation::capture(
        &bound,
        "o",
        Some(&context),
        Budget {
            steps: 1,
            heap: 0,
            stack: 5,
            depth: 1,
        },
        arguments.as_arr().unwrap(),
    )
    .unwrap();
    let leaves = capture
        .expand_source(
            &bound.bound_ref_library().unwrap(),
            &[("x", 1), ("t", 1), ("f", 4)],
        )
        .unwrap();
    let worker = &bound.identity().unwrap().interface_hashes["w"];
    let mut seen = std::collections::BTreeSet::new();
    for leaf in &leaves {
        let source = leaf.source.as_ref().unwrap();
        if &source.terminal.symbol == worker {
            assert!(source.path.is_empty());
            assert_eq!(source.invocations.len(), 1);
            assert_eq!(source.steps.len(), 1);
            seen.insert(source.terminal.instance.clone());
            if leaf.allocation.is_none() {
                assert_eq!(
                    bound.resolve_source_leaf(leaf, "gate").unwrap(),
                    Some(if source.terminal.instance == [0] {
                        &a
                    } else {
                        &b
                    })
                );
            }
        }
    }
    assert_eq!(seen, std::collections::BTreeSet::from([vec![0], vec![1]]));
    let machine = Machine::compile(
        &permutation(&leaves).unwrap(),
        &[("x", 1), ("t", 1), ("f", 4)],
    )
    .unwrap();
    for x in 0..2 {
        let mut state = machine.state();
        machine.set(&mut state, "x", x);
        machine.set(&mut state, "t", 1);
        machine.set(&mut state, "f", 13);
        let entry = state.clone();
        machine.run(&mut state, true, false).unwrap();
        assert_eq!(machine.get(&state, "t"), 1);
        assert_eq!(machine.get(&state, "f"), 12);
        machine.run(&mut state, true, true).unwrap();
        assert_eq!(state, entry);
    }
}
