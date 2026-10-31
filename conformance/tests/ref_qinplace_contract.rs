use bennett_isa_conformance::{
    json,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};

#[test]
fn k28_preserves_instruction_release_sequence_and_row_ordinals() {
    let draft = include_str!("../../spec/Bennett-ISA-Spec-0.9-draft.md");
    assert!(draft.contains("| K28 | §11.1 | **REF-INSTRUCTION-RELEASE**"));
    assert!(draft.contains("The reverse-allocation release rule governs recipe scratch; an instruction whose own semantics fix a release sequence (qinplace per §4.5: `qfree t; qfree f`) keeps that sequence, and its recipe MUST reproduce it exactly. [deps: ISA-4.5-P5, ISA-11.1-P29.S5]"));
    let d = r#"{"isa":"0.9","name":"ref","version":"0.9","recipes":[{"kind":"witness","parameters":[{"name":"G","type":"symbol"},{"name":"x","type":"wire","width":1}],"body":[{"declare":"t","width":1,"class":{"class":"any"},"state":"clean","body":[{"declare":"f","width":4,"class":{"class":"any"},"state":"clean","body":[{"acquire":"t"},{"acquire":"f"},{"call":"G","args":["x","t","f"]},{"gate":"swap","wires":["x","t"]},{"call":"G","args":["x","t","f"]},{"relinquish":"t"},{"relinquish":"f"}]}]}],"scratch":[],"removable":[],"source":"§4.5 qinplace; §11.1 K28"}]}"#;
    // The two calls have their own ordinal spaces; the instruction has five
    // leaves. G is a synthetic one-CX interface fixture, also its own inverse.
    let hash = "a".repeat(64);
    let mut registry = Registry::new();
    registry.insert(hash, json::parse(r#"{"parameters":[{"name":"x","type":"wire","width":1},{"name":"t","type":"wire","width":1},{"name":"f","type":"wire","width":4}],"body":[{"gate":"cx","wires":["x","t"]}]}"#).unwrap());
    let leaves = Expander::new(d, registry)
        .unwrap()
        .expand(
            "witness",
            &[json::Json::Str("a".repeat(64)), json::Json::Str("x".into())],
            &[("x", 1)],
        )
        .unwrap();
    let row = leaves
        .iter()
        .filter(|l| l.invocation.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        row.iter().map(|l| l.ordinal).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4]
    );
    assert_eq!(
        row.iter().map(|l| l.gate.as_str()).collect::<Vec<_>>(),
        ["alloc", "alloc", "swap", "release", "release"]
    );
    assert_eq!(row[0].allocation, row[3].allocation);
    assert_eq!(row[1].allocation, row[4].allocation);
    assert_eq!(
        row[1].allocation.as_ref().unwrap().1,
        4,
        "§5.4 legal fault width"
    );
    assert!(row.iter().all(|l| !l.scratch && l.controls.is_empty()));
    assert_ne!(leaves[2].invocation, leaves[4].invocation);
    assert_eq!((leaves[2].ordinal, leaves[4].ordinal), (0, 0));
    let machine = Machine::compile(&permutation(&leaves).unwrap(), &[("x", 1)]).unwrap();
    for x in 0..2 {
        for enabled in [false, true] {
            let mut state = machine.state();
            machine.set(&mut state, "x", x);
            let entry = state.clone();
            machine.run(&mut state, enabled, false).unwrap();
            assert_eq!(state, entry);
            machine.run(&mut state, enabled, true).unwrap();
            assert_eq!(state, entry);
        }
    }
}

#[test]
fn scheduled_action_adj_consumes_source_adjoint_with_fresh_ordinals() {
    // Actual bound qinplace and adj recipes, with synthetic identity-map
    // callees. The source replacement has exchanged the H/G adapters.
    let g = "a".repeat(64);
    let h = "b".repeat(64);
    let root = "c".repeat(64);
    let mut registry = Registry::new();
    let params = r#"[{"name":"u","type":"wire","width":1},{"name":"v","type":"wire","width":1},{"name":"f","type":"wire","width":4}]"#;
    registry.insert(g.clone(), json::parse(&format!(r#"{{"parameters":{params},"body":[{{"gate":"x","wires":["f[0]"]}},{{"gate":"x","wires":["f[0]"]}},{{"gate":"cx","wires":["u[0]","v[0]"]}}]}}"#)).unwrap());
    registry.insert(
        h.clone(),
        json::parse(&format!(
            r#"{{"parameters":{params},"body":[{{"gate":"cx","wires":["u[0]","v[0]"]}}]}}"#
        ))
        .unwrap(),
    );
    let adapter = |symbol: &str| {
        format!(
            r#"[{{"parameters":{params},"body":[{{"call":"{symbol}","args":["u","v","f"]}}]}}]"#
        )
    };
    registry.insert(root.clone(), json::parse(&format!(r#"{{"parameters":[{{"name":"G","type":"symbol"}},{{"name":"H","type":"symbol"}},{{"name":"x","type":"wire","width":1}}],"body":[{{"invoke":"adj","args":[[{{"invoke":"qinplace","args":[1,4,"x",{}, {},[]]}}]]}}]}}"#, adapter("H"), adapter("G"))).unwrap());
    let all = Expander::new(include_str!("../../spec/ref-0.9-draft.json"), registry)
        .unwrap()
        .expand(
            &root,
            &[
                json::Json::Str(g),
                json::Json::Str(h),
                json::Json::Str("x".into()),
            ],
            &[("x", 1)],
        )
        .unwrap();
    let calls = all
        .iter()
        .filter(|l| !l.invocation.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        calls.iter().map(|l| l.gate.as_str()).collect::<Vec<_>>(),
        ["cx", "x", "x", "cx"],
        "H then G"
    );
    assert_ne!(calls[0].invocation, calls[1].invocation);
    assert_eq!(
        calls.iter().map(|l| l.ordinal).collect::<Vec<_>>(),
        [0, 0, 1, 2]
    );
    let leaves = all
        .iter()
        .filter(|l| l.invocation.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        leaves.iter().map(|l| l.gate.as_str()).collect::<Vec<_>>(),
        ["alloc", "alloc", "swap", "release", "release"]
    );
    assert_eq!(
        leaves.iter().map(|l| l.ordinal).collect::<Vec<_>>(),
        [0, 1, 2, 3, 4]
    );
    assert_eq!(leaves[0].allocation.as_ref().unwrap().1, 1);
    assert_eq!(leaves[1].allocation.as_ref().unwrap().1, 4);
    assert_eq!(leaves[0].allocation, leaves[3].allocation);
    assert_eq!(leaves[1].allocation, leaves[4].allocation);
    let classes = leaves
        .iter()
        .filter(|l| l.gate == "alloc")
        .map(|l| if l.ordinal == 0 { "special" } else { "default" })
        .collect::<Vec<_>>();
    assert_eq!(classes, ["special", "default"]);
}
