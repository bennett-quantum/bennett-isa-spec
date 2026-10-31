use bennett_isa_conformance::{
    json,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};

#[test]
fn k28_preserves_instruction_release_sequence_and_row_ordinals() {
    let draft = include_str!("../../spec/Bennett-ISA-Spec-0.9-draft.md");
    assert!(draft.contains("| K28 | §11.1 | **REF-INSTRUCTION-RELEASE**"));
    assert!(draft.contains("The reverse-allocation release rule governs recipe scratch; an instruction whose own semantics fix a release sequence (qinplace per §4.5: `qfree t; qfree f`) keeps that sequence, and its recipe MUST reproduce it exactly. [deps: ISA-4.5-P5, ISA-11.1-P21]"));
    let d = r#"{"isa":"0.9","name":"ref","version":"0.9","recipes":[{"kind":"witness","parameters":[{"name":"G","type":"symbol"},{"name":"x","type":"wire","width":1}],"body":[{"declare":"t","width":1,"class":{"class":"any"},"state":"clean","body":[{"declare":"f","width":4,"class":{"class":"any"},"state":"clean","body":[{"acquire":"t"},{"acquire":"f"},{"call":"G","args":["x","t","f"]},{"gate":"swap","wires":["x","t"]},{"call":"G","args":["x","t","f"]},{"relinquish":"t"},{"relinquish":"f"}]}]}],"scratch":[],"removable":[],"source":"§4.5 qinplace; §11.1 K28"}]}"#;
    // The two calls have their own ordinal spaces; the instruction has five
    // leaves. G is the one-bit identity oracle, also its own inverse.
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
