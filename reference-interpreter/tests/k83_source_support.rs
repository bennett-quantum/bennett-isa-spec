mod support;
use bennett_isa_conformance::{
    json::{self, Json},
    ref_catalog::Machine,
    ref_language::{invert_trace, permutation, Leaf},
};
use bennett_isa_ref::{sha256_hex, Object, QuantumBindings, QuantumState};
use std::{collections::BTreeMap, path::PathBuf};
const VECTOR: &str = include_str!("../../conformance/fixtures/accepted-k83-09/k83.json");
const VECTOR_HASH: &str = "31471bd3c4a527ccb7dd7d31f10d2b4f8c98230e9418d9dedbb8bb50a3b69034";
fn count(value: &Json) -> usize {
    let Json::Num(number) = value else {
        panic!("integer")
    };
    assert_eq!(number.fract(), 0.0);
    assert!(*number >= 0.0);
    *number as usize
}
fn text(value: &Json) -> &str {
    value.as_str().unwrap()
}
fn array(values: impl IntoIterator<Item = Json>) -> Json {
    Json::Arr(values.into_iter().collect())
}
fn number(value: usize) -> Json {
    Json::Num(value as f64)
}
fn normalized(leaves: &[Leaf], complete: bool, widths: &Json) -> Json {
    let mut allocations = BTreeMap::new();
    let mut events = Vec::new();
    for leaf in leaves {
        let source = leaf
            .source
            .as_ref()
            .expect("every source leaf has an address");
        if !complete && !matches!(source.terminal.point.as_str(), "$construct" | "$release") {
            continue;
        }
        let owner = if let Some((name, width, class, state)) = &leaf.allocation {
            assert_eq!((class.as_str(), state.as_str()), ("any", "clean"));
            if leaf.gate == "alloc" {
                let alias = format!("a{}", allocations.len());
                assert!(
                    allocations.insert(name.clone(), alias).is_none(),
                    "separate calls must not alias allocations"
                );
            }
            let alias = allocations.get(name).unwrap();
            assert_eq!(*width, count(widths.get(alias).unwrap()));
            alias.clone()
        } else {
            let name = &leaf.wires[0].0;
            allocations.get(name).unwrap_or(name).clone()
        };
        let support = matches!(source.terminal.point.as_str(), "$construct" | "$release");
        if support {
            assert!(
                leaf.controls.is_empty(),
                "region preparation/restoration is unconditional"
            );
            assert!(source.terminal.iteration.is_empty());
            assert!(source.terminal.instance.is_empty());
            assert!(source.steps.is_empty());
        }
        for edge in &source.path {
            assert_eq!(edge.call_index, 0);
        }
        events.push(array([
            Json::Str(leaf.gate.clone()),
            Json::Str(source.terminal.point.clone()),
            number(leaf.ordinal),
            Json::Str(owner),
            if leaf.allocation.is_some() {
                Json::Null
            } else {
                number(leaf.wires[0].1)
            },
            array(source.terminal.iteration.iter().copied().map(number)),
            array(source.terminal.instance.iter().copied().map(number)),
            source.path.last().map_or(Json::Null, |edge| {
                array(edge.point.iteration.iter().copied().map(number))
            }),
        ]));
    }
    array(events)
}
fn expect_interface(name: &str, bundle: &str, value: u64) -> u64 {
    if matches!(name, "zero-trip" | "qif-arms") && bundle == "t" {
        value ^ 1
    } else {
        value
    }
}
#[test]
fn accepted_k83_vectors_match_actual_source_coordinates_and_reversible_execution() {
    assert_eq!(sha256_hex(VECTOR.as_bytes()), VECTOR_HASH);
    let vectors = json::parse(VECTOR).unwrap();
    for case in vectors.get("cases").unwrap().as_arr().unwrap() {
        let name = text(case.get("name").unwrap());
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../conformance/fixtures/accepted-k83-09")
            .join(text(case.get("object").unwrap()));
        let bytes = std::fs::read_to_string(path).unwrap();
        assert_eq!(
            sha256_hex(bytes.as_bytes()),
            text(case.get("sha256").unwrap())
        );
        let object = Object::parse(&bytes).unwrap();
        object.check_hashes().unwrap();
        let bundles = case
            .get("bundles")
            .unwrap()
            .as_arr()
            .unwrap()
            .iter()
            .map(|bundle| {
                let fields = bundle.as_arr().unwrap();
                (text(&fields[0]), count(&fields[1]))
            })
            .collect::<Vec<_>>();
        let entry = text(case.get("entry").unwrap());
        let actuals = case.get("actuals").unwrap().as_arr().unwrap();
        let leaves = object
            .expand_action_source(entry, actuals, &bundles, false)
            .unwrap();
        assert_eq!(
            normalized(
                &leaves,
                case.get("complete_trace").unwrap().as_bool().unwrap(),
                case.get("allocation_widths").unwrap()
            ),
            *case.get("trace").unwrap(),
            "{name}"
        );
        let mut raw_inverse = leaves.clone();
        invert_trace(&mut raw_inverse).unwrap();
        for (forward, inverse) in leaves.iter().zip(raw_inverse.iter().rev()) {
            assert_eq!(forward.source, inverse.source);
            assert_eq!(forward.ordinal, inverse.ordinal);
            assert_eq!(forward.invocation, inverse.invocation);
            assert_eq!(forward.allocation, inverse.allocation);
            assert_eq!(forward.wires, inverse.wires);
            assert_eq!(forward.controls, inverse.controls);
        }
        for trace in [&leaves, &raw_inverse] {
            let machine = Machine::compile(&permutation(trace).unwrap(), &bundles).unwrap();
            for basis in 0..1u64 << bundles.iter().map(|(_, width)| width).sum::<usize>() {
                let mut state = machine.state();
                let mut offset = 0;
                for (bundle, width) in &bundles {
                    machine.set(&mut state, bundle, (basis >> offset) & ((1 << width) - 1));
                    offset += width;
                }
                let before = state.clone();
                machine.run(&mut state, true, false).unwrap();
                offset = 0;
                for (bundle, width) in &bundles {
                    let value = (basis >> offset) & ((1 << width) - 1);
                    assert_eq!(
                        machine.get(&state, bundle),
                        expect_interface(name, bundle, value),
                        "{name}"
                    );
                    offset += width;
                }
                machine.run(&mut state, true, true).unwrap();
                assert_eq!(state, before, "{name}");
            }
        }
        for basis in 0..1u64 << bundles.iter().map(|(_, width)| width).sum::<usize>() {
            let mut quantum = QuantumState::new();
            let mut bindings = QuantumBindings::default();
            let mut offset = 0;
            for (bundle, width) in &bundles {
                let wires = quantum.allocate(*width).unwrap();
                for (bit, wire) in wires.iter().enumerate() {
                    if (basis >> (offset + bit)) & 1 != 0 {
                        quantum.gate("x", &[*wire], None, &[]).unwrap();
                    }
                }
                offset += width;
                bindings.bundles.insert((*bundle).into(), wires);
            }
            let before = quantum.amplitudes().unwrap();
            let width = quantum.wire_count();
            object
                .execute_action(entry, &mut quantum, &bindings, false, &[])
                .unwrap();
            offset = 0;
            for (bundle, width) in &bundles {
                let value = (basis >> offset) & ((1 << width) - 1);
                assert_eq!(
                    quantum.read_basis(&bindings.bundles[*bundle]).unwrap(),
                    u128::from(expect_interface(name, bundle, value))
                );
                offset += width;
            }
            object
                .execute_action(entry, &mut quantum, &bindings, true, &[])
                .unwrap();
            assert_eq!(quantum.wire_count(), width);
            assert_eq!(quantum.amplitudes().unwrap(), before);
        }
    }
}

#[test]
fn accepted_k83_schedule_vectors_gate_real_support_and_empty_domains() {
    assert_eq!(sha256_hex(VECTOR.as_bytes()), VECTOR_HASH);
    let vectors = json::parse(VECTOR).unwrap();
    for case in vectors.get("schedule_cases").unwrap().as_arr().unwrap() {
        let name = text(case.get("name").unwrap());
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../conformance/fixtures/accepted-k83-09")
            .join(text(case.get("object").unwrap()));
        let bytes = std::fs::read_to_string(path).unwrap();
        assert_eq!(
            sha256_hex(bytes.as_bytes()),
            text(case.get("sha256").unwrap())
        );
        let result = Object::parse(&bytes);
        if case.get("accepted").unwrap().as_bool().unwrap() {
            result
                .unwrap_or_else(|error| panic!("{name}: {error}"))
                .check_hashes()
                .unwrap();
        } else {
            let error = result.expect_err(name).to_string();
            assert!(error.contains("§11"), "{name}: {error}");
            if name.contains("duplicate") {
                assert!(
                    error.contains("multiple maximal matching schedule entries"),
                    "{name}: {error}"
                );
            }
        }
    }
}

#[test]
fn action_support_has_no_own_step_and_retains_an_enclosing_oracle_selection() {
    use bennett_isa_ref::{ref_library::OracleInvocation, Budget};
    let oracle =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let image = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[],"tier":"q"}"#;
    let object=support::rehash(&support::object_with_data(&[("o",oracle,"oracle @o () (t:wire) (f:wire[4]) () ()\nregs { zero:reg[1] }\n^entry():\nlift.perm_call @image () () () ()\nlift.retval zero"),("image",image,"perm_action @image () () () () ()\n^entry():\nqregion r 0 8 @blob")],&[("blob","81")],"").replace("=0.8","=0.9").replace("\"abi\":\"0.8\"","\"abi\":\"0.9\""));
    let identity = object.identity().unwrap();
    let hash = identity.closure_hashes["o"].as_ref().unwrap();
    let caller = &identity.interface_hashes["o"];
    let callee = &identity.interface_hashes["image"];
    let gate = r#""gate":{"epsilon_share":0,"realisation":"native"}"#;
    let root_rule =
        format!(r#"{{"generated":"*","kind":"gate","path":[],"phase":"fwd","step":1,{gate}}}"#);
    let edge = format!(
        r#"{{"call_index":0,"instance":[],"iteration":[],"point":"entry:0","symbol":"{caller}"}}"#
    );
    let child_rule = format!(
        r#"{{"generated":"*","kind":"gate","path":[{edge}],"phase":"fwd","step":1,{gate}}}"#
    );
    let bind = |entries: &str| {
        let schedule = support::json(&format!(
            r#"{{"bindings":{{"budget":[1,0,0,0],"closure_hash":"{hash}","epsilon":0,"inherited_controls":0,"params_hash":"{}","region":"o","scheduler":{{"name":"k83-level","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[{entries}]}}"#,
            sha256_hex(b"[]"),
            "0".repeat(64)
        ));
        Object::parse(&format!(
            "{}\nsection schedule\n{}\n",
            object.text,
            schedule.canonical()
        ))
        .unwrap()
    };
    let bound = bind(&format!("{root_rule},{child_rule}"));
    let args = json::parse(r#"["t","f"]"#).unwrap();
    let context = json::parse(r#"{"token":["1","0","0","0"]}"#).unwrap();
    let invocation = OracleInvocation::capture(
        &bound,
        "o",
        Some(&context),
        Budget::steps(1),
        args.as_arr().unwrap(),
    )
    .unwrap();
    let leaves = invocation
        .expand_source(&bound.bound_ref_library().unwrap(), &[("t", 1), ("f", 4)])
        .unwrap();
    let expected = support::json(&root_rule);
    let mut phases = std::collections::BTreeSet::new();
    for leaf in &leaves {
        let source = leaf.source.as_ref().unwrap();
        if &source.terminal.symbol != callee || leaf.gate != "x" {
            continue;
        }
        assert!(matches!(
            source.terminal.point.as_str(),
            "$construct" | "$release"
        ));
        assert_eq!(source.invocations.len(), 2);
        assert!(!source.invocations[1].oracle);
        assert_eq!(source.steps.len(), 1);
        let reverse = source.steps[0].reverse;
        phases.insert(reverse);
        assert_eq!(
            bound.resolve_source_leaf(leaf, "gate").unwrap(),
            if reverse { None } else { Some(&expected) }
        );
    }
    assert_eq!(phases, std::collections::BTreeSet::from([false, true]));
}
