//! The published ctl fixture is derived through the existing catalog expander.
use bennett_isa_conformance::{
    json,
    ref_amplitudes::{column, Complex},
    ref_hash::envelope_hash,
    ref_language::{Expander, Registry},
};

#[test]
fn published_ctl_leaf_and_every_exact_matrix_column() {
    let catalog = include_str!("../../spec/ref-0.9.json");
    assert_eq!(catalog, include_str!("../../spec/ref-0.9.json"));
    assert_eq!(
        envelope_hash(catalog),
        "5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197"
    );
    let fixture = concat!(env!("CARGO_MANIFEST_DIR"), "/fixtures/controlled-action-09");
    let source = std::fs::read_to_string(format!("{fixture}/controlled.bisa")).unwrap();
    assert!(source.contains("ctl c action_call @flip (t) () () ()"));
    let symbols = json::parse(
        source
            .split("section symbols\n")
            .nth(1)
            .unwrap()
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    let flip = symbols
        .at_path("1.interface_hash")
        .unwrap()
        .as_str()
        .unwrap();
    let mut registry = Registry::new();
    registry.insert(flip.into(), json::parse(r#"{"parameters":[{"name":"t","type":"wire","width":1}],"body":[{"gate":"x","wires":["t"]}]}"#).unwrap());
    let args = json::parse(&format!(
        r#"["c",[{{"invoke":"action_call","args":["{flip}",["t"]]}}]]"#
    ))
    .unwrap();
    let leaves = Expander::new(catalog, registry)
        .unwrap()
        .expand("ctl", args.as_arr().unwrap(), &[("t", 1), ("c", 1)])
        .unwrap();
    assert_eq!(leaves.len(), 1);
    let leaf = &leaves[0];
    assert_eq!(leaf.gate, "x");
    assert_eq!(leaf.ordinal, 0);
    assert_eq!(leaf.wires, [("t".into(), 0)]);
    assert_eq!(leaf.controls, [("c".into(), 0)]);
    assert!(leaf.allocation.is_none() && leaf.event.is_none() && !leaf.scratch);
    let printed =
        json::parse(&std::fs::read_to_string(format!("{fixture}/leaves.json")).unwrap()).unwrap();
    assert_eq!(
        printed.at_path("0.gate").unwrap().as_str(),
        Some(leaf.gate.as_str())
    );
    assert_eq!(
        printed.at_path("0.generated"),
        Some(&json::Json::Num(leaf.ordinal as f64))
    );
    // Root order (t,c), then callee order (inherited c,t). All amplitudes
    // are exact 0/1; V†U-I is identically zero, including global phase.
    for (inputs, control_first) in [
        (vec![("t".into(), 0), ("c".into(), 0)], false),
        (vec![("c".into(), 0), ("t".into(), 0)], true),
    ] {
        for basis in 0..4 {
            let expected_index = if control_first {
                basis ^ ((basis & 1) << 1)
            } else {
                basis ^ (basis >> 1)
            };
            let mut expected = vec![Complex::default(); 4];
            expected[expected_index] = Complex::one();
            assert_eq!(column(&leaves, &inputs, basis).unwrap(), expected);
        }
    }
}
