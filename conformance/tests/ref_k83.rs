//! Portable K83 coordinates and trace inversion. This finite event adapter is
//! separate from the reference's source-construction and native execution gate.
use bennett_isa_conformance::{
    json::{self, Json},
    ref_catalog::Machine,
    ref_hash::sha256,
    ref_language::{
        invert_trace, permutation, Leaf, SourceEdge, SourceInvocation, SourceLocation, SourcePoint,
    },
};
use std::sync::Arc;
const VECTOR: &str = include_str!("../fixtures/accepted-k83-09/k83.json");
fn text(value: &Json) -> &str {
    value.as_str().unwrap()
}
fn n(value: &Json) -> usize {
    let Json::Num(value) = value else {
        panic!("number")
    };
    assert_eq!(value.fract(), 0.0);
    assert!(*value >= 0.0);
    *value as usize
}
fn key(value: &Json) -> Vec<usize> {
    value.as_arr().unwrap().iter().map(n).collect()
}
#[test]
fn k83_published_coordinates_reverse_without_renumbering_and_preserve_every_basis_state() {
    assert_eq!(
        sha256(VECTOR.as_bytes()),
        "31471bd3c4a527ccb7dd7d31f10d2b4f8c98230e9418d9dedbb8bb50a3b69034"
    );
    let vectors = json::parse(VECTOR).unwrap();
    for case in vectors.get("cases").unwrap().as_arr().unwrap() {
        let name = text(case.get("name").unwrap());
        let source = std::fs::read_to_string(format!(
            "{}/fixtures/accepted-k83-09/{}",
            env!("CARGO_MANIFEST_DIR"),
            text(case.get("object").unwrap())
        ))
        .unwrap();
        assert_eq!(sha256(source.as_bytes()), text(case.get("sha256").unwrap()));
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
        let hash = |name: &str| {
            let Json::Obj(symbols) = &symbols else {
                panic!("symbols")
            };
            symbols
                .values()
                .find(|symbol| symbol.get("name").and_then(Json::as_str) == Some(name))
                .unwrap()
                .get("interface_hash")
                .unwrap()
                .as_str()
                .unwrap()
                .to_string()
        };
        let root = text(case.get("entry").unwrap());
        let support = if name.contains("image") {
            "image"
        } else {
            root
        };
        let mut leaves = Vec::new();
        for row in case.get("trace").unwrap().as_arr().unwrap() {
            let row = row.as_arr().unwrap();
            let gate = text(&row[0]);
            let owner = text(&row[3]);
            let external = row[7] == Json::Null;
            let invocation = if external {
                vec![0]
            } else {
                vec![0, if key(&row[7]) == [1] { 2 } else { 1 }]
            };
            let point = SourcePoint {
                symbol: hash(support),
                point: text(&row[1]).into(),
                iteration: key(&row[5]),
                instance: key(&row[6]),
            };
            let edge = if external {
                vec![]
            } else {
                vec![SourceEdge {
                    point: SourcePoint {
                        symbol: hash(root),
                        point: if root == "repeat" {
                            "entry:1"
                        } else {
                            "entry:0"
                        }
                        .into(),
                        iteration: key(&row[7]),
                        instance: vec![],
                    },
                    call_index: 0,
                }]
            };
            let mut invocations = vec![SourceInvocation {
                tokens: None,
                invocation: vec![0],
                symbol: hash(root),
                oracle: false,
            }];
            if !external {
                invocations.push(SourceInvocation {
                    tokens: None,
                    invocation: invocation.clone(),
                    symbol: hash(support),
                    oracle: false,
                });
            }
            leaves.push(Leaf {
                invocation,
                ordinal: n(&row[2]),
                gate: gate.into(),
                wires: if row[4] == Json::Null {
                    vec![]
                } else {
                    vec![(owner.into(), n(&row[4]))]
                },
                controls: vec![],
                angles: vec![],
                allocation: if matches!(gate, "alloc" | "release") {
                    Some((
                        owner.into(),
                        n(case.get("allocation_widths").unwrap().get(owner).unwrap()),
                        "any".into(),
                        "clean".into(),
                    ))
                } else {
                    None
                },
                event: None,
                scratch: matches!(gate, "alloc" | "release"),
                source: Some(Arc::new(SourceLocation {
                    path: edge,
                    terminal: point,
                    steps: vec![],
                    invocations,
                })),
            });
        }
        let original = leaves.clone();
        invert_trace(&mut leaves).unwrap();
        for (a, b) in original.iter().zip(leaves.iter().rev()) {
            assert_eq!(a.ordinal, b.ordinal);
            assert_eq!(a.source, b.source);
            assert_eq!(a.invocation, b.invocation);
            assert_eq!(a.wires, b.wires);
            assert_eq!(a.allocation, b.allocation);
        }
        let bundles = case
            .get("bundles")
            .unwrap()
            .as_arr()
            .unwrap()
            .iter()
            .map(|pair| {
                let pair = pair.as_arr().unwrap();
                (text(&pair[0]), n(&pair[1]))
            })
            .collect::<Vec<_>>();
        for trace in [&original, &leaves] {
            let machine = Machine::compile(&permutation(trace).unwrap(), &bundles).unwrap();
            for bits in 0..1u64 << bundles.iter().map(|(_, width)| width).sum::<usize>() {
                let mut state = machine.state();
                for (index, (bundle, _)) in bundles.iter().enumerate() {
                    machine.set(&mut state, bundle, bits >> index & 1);
                }
                let entry = state.clone();
                machine.run(&mut state, true, false).unwrap();
                for (index, (bundle, _)) in bundles.iter().enumerate() {
                    let want = bits >> index & 1;
                    assert_eq!(
                        machine.get(&state, bundle),
                        if name == "zero-trip" && *bundle == "t" {
                            want ^ 1
                        } else {
                            want
                        }
                    );
                }
                machine.run(&mut state, true, true).unwrap();
                assert_eq!(state, entry);
            }
        }
        invert_trace(&mut leaves).unwrap();
        assert_eq!(leaves, original);
    }
}
#[test]
fn inverse_trace_rejects_irreversible_events_before_mutating_any_leaf() {
    let mut leaves = vec![Leaf {
        invocation: vec![],
        ordinal: 0,
        gate: "x".into(),
        wires: vec![("t".into(), 0)],
        controls: vec![],
        angles: vec![],
        allocation: None,
        event: None,
        scratch: false,
        source: None,
    }];
    let mut irreversible = leaves[0].clone();
    irreversible.gate = "classical".into();
    leaves.push(irreversible);
    let original = leaves.clone();
    assert!(invert_trace(&mut leaves).is_err());
    assert_eq!(leaves, original);
}
