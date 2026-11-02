mod support;
use bennett_isa_conformance::{json, ref_catalog::Machine, ref_language::permutation};
use bennett_isa_ref::{Object, QuantumBindings, QuantumState};
fn source() -> Object {
    let signature = r#"{"abi":"0.9","args":[],"bounds":[0],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    support::rehash(&support::object(&[("gap",signature,"action @gap (t:wire mut) () () () ()\nregs {  } bounds {0}\n^entry():\nqloop 0 {\nqregion local 0 1\n}\nx t")],"{}","").replace("=0.8","=0.9"))
}
#[test]
fn hoisted_region_in_zero_trip_loop_remains_a_valid_reversible_source_program() {
    let object = source();
    object.check_hashes().unwrap();
    let actuals = json::parse(r#"["t"]"#).unwrap();
    let leaves = object
        .expand_action_source("gap", actuals.as_arr().unwrap(), &[("t", 1)], false)
        .unwrap();
    assert_eq!(leaves.iter().filter(|leaf| leaf.gate == "alloc").count(), 1);
    assert_eq!(
        leaves.iter().filter(|leaf| leaf.gate == "release").count(),
        1
    );
    let machine = Machine::compile(&permutation(&leaves).unwrap(), &[("t", 1)]).unwrap();
    for t in 0..2 {
        let mut bits = machine.state();
        machine.set(&mut bits, "t", t);
        let entry = bits.clone();
        machine.run(&mut bits, true, false).unwrap();
        assert_eq!(machine.get(&bits, "t"), t ^ 1);
        machine.run(&mut bits, true, true).unwrap();
        assert_eq!(bits, entry);
        let mut q = QuantumState::new();
        let wire = q.allocate(1).unwrap();
        if t == 1 {
            q.gate("x", &wire, None, &[]).unwrap();
        }
        let mut bindings = QuantumBindings::default();
        bindings.bundles.insert("t".into(), wire.clone());
        object
            .execute_action("gap", &mut q, &bindings, false, &[])
            .unwrap();
        assert_eq!(q.read_basis(&wire).unwrap(), u128::from(t ^ 1));
        object
            .execute_action("gap", &mut q, &bindings, true, &[])
            .unwrap();
        assert_eq!(q.read_basis(&wire).unwrap(), u128::from(t));
    }
}

#[test]
fn k83_zero_trip_hoisted_region_support_has_reserved_addresses() {
    let object = source();
    let actuals = json::parse(r#"["t"]"#).unwrap();
    let leaves = object
        .expand_action_source("gap", actuals.as_arr().unwrap(), &[("t", 1)], false)
        .unwrap();
    assert_eq!(leaves.len(), 3);
    let trace = leaves
        .iter()
        .map(|leaf| {
            let source = leaf
                .source
                .as_ref()
                .expect("K83 defines every hoisted support address");
            assert!(source.terminal.iteration.is_empty());
            assert!(source.terminal.instance.is_empty());
            assert!(source.steps.is_empty());
            (
                leaf.gate.as_str(),
                source.terminal.point.as_str(),
                leaf.ordinal,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        trace,
        vec![
            ("alloc", "$construct", 0),
            ("x", "entry:2", 0),
            ("release", "$release", 0)
        ]
    );
}

#[test]
fn linked_source_addresses_match_the_program_point_interface_identity() {
    let signature = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let object=support::rehash(&support::object(&[("flip",signature,"perm_action @flip (t:wire) () () () ()\n^entry():\naction_call @leaf (t) () () ()\nx t"),("leaf",signature,"perm_action @leaf (t:wire) () () () ()\n^entry():\nx t")],"{}","").replace("=0.8","=0.9"));
    let identity = object.identity().unwrap();
    let interface = &identity.interface_hashes["flip"];
    let closure = identity.closure_hashes["flip"].as_ref().unwrap();
    // All imports have been linked before source construction.
    assert_eq!(interface, closure);
    let actuals = json::parse(r#"["t"]"#).unwrap();
    let leaves = object
        .expand_action_source("flip", actuals.as_arr().unwrap(), &[("t", 1)], false)
        .unwrap();
    assert_eq!(leaves.len(), 2);
    let own = leaves
        .iter()
        .find(|l| l.source.as_ref().unwrap().path.is_empty())
        .unwrap();
    assert_eq!(&own.source.as_ref().unwrap().terminal.symbol, interface);
    assert_eq!(own.source.as_ref().unwrap().terminal.point, "entry:1");
    let callee = leaves
        .iter()
        .find(|l| !l.source.as_ref().unwrap().path.is_empty())
        .unwrap();
    assert_eq!(
        &callee.source.as_ref().unwrap().path[0].point.symbol,
        interface
    );
}
