//! Pass XII / K81 finite source adapter. Production admission remains 0.8;
//! neither these carrier coordinates nor pair formation come from Expander.
mod support;
use bennett_isa_conformance::{
    json::Json,
    ref_language::{Expander, Leaf, Registry},
};
use bennett_isa_ref::{Object, QuantumBindings, QuantumState};
use std::collections::BTreeMap;

fn source(primitive: bool, gap: bool, allocated: bool) -> Object {
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[2],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let child = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[2],"tier":"q"}"#;
    let start = if allocated {
        "d = qalloc any 2"
    } else {
        "d = qcopy x"
    };
    let end = if allocated { "qfree d" } else { "quncopy x d" };
    let forward = if primitive {
        "cx d[0] d[1]"
    } else {
        "action_call @mix (d) () () ()"
    };
    let inverse = if primitive {
        forward
    } else {
        "adj action_call @mix (d) () () ()"
    };
    let unrelated = if gap { "x t\n" } else { "" };
    support::rehash(&support::object(&[
        ("outer", sig, &format!("perm_action @outer (t:wire mut) (x:wire[2] in) () () ()\n^entry():\n{start}\n{forward}\n{unrelated}{inverse}\n{end}\n{unrelated}")),
        ("mix", child, "perm_action @mix (a:wire[2] mut) () () () ()\n^entry():\ncx a[0] a[1]"),
    ], "{}", ""))
}

// Deliberately finite adapter: source versions/matching are checked by Object.
// The inverse CX has no standalone K65 call carrier. qfree coalesces its
// terminal inverse; intrinsic quncopy leaves the adjoint call independent.
fn carriers(o: &Object) -> Vec<(usize, Vec<usize>)> {
    let rows = &o.functions["outer"].blocks[0].instructions;
    let inverse = rows
        .iter()
        .position(|i| i.op == "adj" || (i.op == "cx" && i.line > rows[1].line))
        .unwrap();
    let release = rows
        .iter()
        .position(|i| matches!(i.op.as_str(), "quncopy" | "qfree"))
        .unwrap();
    if rows[release].op == "qfree" {
        vec![(release, vec![inverse, release])]
    } else {
        let mut pairs = Vec::new();
        if rows[inverse].op == "adj" {
            pairs.push((inverse, vec![inverse]));
        }
        pairs.push((release, vec![release]));
        pairs
    }
}

#[test]
fn pass_xii_literal_call_and_primitive_pairs_restore_exact_versions() {
    for primitive in [false, true] {
        for gap in [false, true] {
            let o = source(primitive, gap, false);
            o.check_hashes().unwrap();
            for input in 0..32 {
                let mut q = QuantumState::new();
                let w = q.allocate(4).unwrap();
                for (bit, wire) in w.iter().enumerate() {
                    if input >> bit & 1 != 0 {
                        q.gate("x", &[*wire], None, &[]).unwrap();
                    }
                }
                if input & 16 != 0 {
                    q.gate("h", &[w[0]], None, &[]).unwrap();
                    q.gate("cx", &[w[0], w[3]], None, &[]).unwrap();
                    q.gate("h", &[w[2]], None, &[]).unwrap();
                }
                let b = QuantumBindings {
                    bundles: BTreeMap::from([
                        ("x".into(), w[..2].to_vec()),
                        ("t".into(), vec![w[2]]),
                    ]),
                    ..Default::default()
                };
                let before = q.clone();
                for adj in [false, true] {
                    o.execute_action("outer", &mut q, &b, adj, &[]).unwrap();
                    assert_eq!(
                        q, before,
                        "primitive={primitive} gap={gap} input={input} adj={adj}"
                    );
                }
            }
        }
    }
    // The bead's indexed two-slot list is schematic: §16.2 lists contain
    // NAMEs only. The admitted bundled callee above denotes the same CX.
    let literal = source(false, false, false)
        .text
        .replace("action_call @mix (d)", "action_call @mix (d[0], d[1])");
    assert!(Object::parse(&literal)
        .unwrap_err()
        .to_string()
        .contains("operand lists require names"));
    // An un-restored call target cannot be discharged by intrinsic quncopy.
    let text = source(false, false, false).text;
    assert!(
        Object::parse(&text.replace("adj action_call @mix (d) () () ()\n", ""))
            .unwrap_err()
            .to_string()
            .contains("quncopy outside its computing window")
    );
}

#[test]
fn pass_xii_intrinsic_boundary_preserves_independent_call_carrier_and_ties() {
    for gap in [false, true] {
        let mut o = source(false, gap, false);
        let inverse = if gap { 3 } else { 2 };
        let release = inverse + 1;
        assert_eq!(
            carriers(&o),
            [(inverse, vec![inverse]), (release, vec![release])]
        );
        assert_eq!(
            carriers(&source(true, gap, false)),
            [(release, vec![release])]
        );
        assert_eq!(
            carriers(&source(false, gap, true)),
            [(release, vec![inverse, release])]
        );
        let hash = o.identity().unwrap().interface_hashes["outer"].clone();
        let operation = |point| {
            support::json(&format!(
                r#"{{"path":[{{"symbol":"{hash}","point":"entry:{point}","iteration":[]}}],"step":"*","phase":"*","generated":0}}"#
            ))
        };
        let entry = |point, mode| {
            let mut op = operation(point);
            let bennett_isa_ref::json::Json::Object(fields) = &mut op else {
                unreachable!()
            };
            fields.insert("kind".into(), support::json("\"cleanup\""));
            fields.insert(
                "cleanup".into(),
                support::json(&format!(r#"{{"realisation":"{mode}"}}"#)),
            );
            op.canonical()
        };
        let call = entry(inverse, "recompute");
        let copy = entry(release, "reverse");
        o.metadata.insert(
            "schedule".into(),
            support::json(&format!(r#"{{"entries":[{call},{copy}]}}"#)),
        );
        for (point, _) in carriers(&o) {
            assert_eq!(
                o.schedule_cleanup(&operation(point)).unwrap(),
                if point == inverse {
                    "recompute"
                } else {
                    "reverse"
                }
            );
        }
        o.metadata.insert(
            "schedule".into(),
            support::json(&format!(r#"{{"entries":[{call},{call},{copy}]}}"#)),
        );
        assert!(o
            .schedule_cleanup(&operation(inverse))
            .unwrap_err()
            .to_string()
            .contains("multiple maximal"));
        assert_eq!(o.schedule_cleanup(&operation(release)).unwrap(), "reverse");
    }
}

fn j(s: &str) -> Json {
    bennett_isa_conformance::json::parse(s).unwrap()
}
// Construct the four independent reverse/recompute combinations. The call's
// fresh zero-based computation is delta(x)=mix(x) XOR x, not mix(0).
// qalloc/qfree/action_call are read from the frozen catalog unchanged.
fn expansion(call_recompute: bool, copy_recompute: bool) -> Vec<(usize, Leaf)> {
    let call = r#"{"invoke":"action_call","args":["S",["u[0]","u[1]"]]}"#;
    let xor = |a: &str, b: &str| format!(r#"{{"gate":"cx","wires":["{a}","{b}"]}}"#);
    let copy = |a: &str, b: &str| {
        vec![
            xor(&format!("{a}[0]"), &format!("{b}[0]")),
            xor(&format!("{a}[1]"), &format!("{b}[1]")),
        ]
    };
    let fresh = |gates: Vec<String>| {
        format!(
            r#"{{"declare":"u","width":2,"class":{{"class":"any"}},"state":"clean","body":[{{"invoke":"qalloc","args":["u"]}},{},{{"invoke":"qfree","args":["u"]}}]}}"#,
            gates.join(",")
        )
    };
    let mut call_body = copy("x", "u");
    call_body.push(call.into());
    call_body.extend(copy("x", "u"));
    call_body.extend(copy("u", "d"));
    call_body.extend(copy("x", "u"));
    call_body.push(call.into()); // mix is CX, so its source adjoint is itself.
    call_body.extend(copy("x", "u"));
    let call_body = if call_recompute {
        fresh(call_body)
    } else {
        r#"{"invoke":"action_call","args":["S",["d[0]","d[1]"]]}"#.into()
    };
    let mut copy_body = copy("x", "u");
    copy_body.extend(copy("u", "d"));
    copy_body.extend(copy("x", "u"));
    let copy_body = if copy_recompute {
        fresh(copy_body)
    } else {
        copy("x", "d").join(",")
    };
    let bodies = [
        format!(
            r#"{{"invoke":"qalloc","args":["d"]}},{}"#,
            copy("x", "d").join(",")
        ),
        r#"{"invoke":"action_call","args":["S",["d[0]","d[1]"]]}"#.into(),
        call_body,
        format!(r#"{copy_body},{{"invoke":"qfree","args":["d"]}}"#),
    ];
    let hash = "a".repeat(64);
    let mut doc = j(include_str!("../../spec/ref-0.9.json"));
    let Json::Obj(fields) = &mut doc else {
        unreachable!()
    };
    let Json::Arr(recipes) = fields.get_mut("recipes").unwrap() else {
        unreachable!()
    };
    recipes.retain(|r| {
        matches!(
            r.get("kind").unwrap().as_str().unwrap(),
            "qalloc" | "qfree" | "action_call"
        )
    });
    let body = bodies.join(",");
    recipes.push(j(&format!(r#"{{"kind":"zz.partition","parameters":[{{"name":"S","type":"symbol"}},{{"name":"x","type":"wire","width":2}}],"body":[{{"declare":"d","width":2,"class":{{"class":"any"}},"state":"clean","body":[{body}]}}],"scratch":[],"removable":[],"source":"K81 finite witness"}}"#)));
    let registry = Registry::from([(
        hash.clone(),
        j(
            r#"{"parameters":[{"name":"a","type":"wire","width":1},{"name":"b","type":"wire","width":1}],"body":[{"gate":"cx","wires":["a","b"]}]}"#,
        ),
    )]);
    let leaves = Expander::new(&doc.to_string(), registry)
        .unwrap()
        .expand("zz.partition", &[Json::Str(hash), j(r#""x""#)], &[("x", 2)])
        .unwrap();
    let call_end = 4 + if call_recompute { 14 } else { 1 };
    leaves
        .into_iter()
        .enumerate()
        .map(|(i, l)| {
            (
                if i < 3 {
                    0
                } else if i == 3 {
                    1
                } else if i < call_end {
                    2
                } else {
                    3
                },
                l,
            )
        })
        .collect()
}

#[test]
fn pass_xii_independent_recompute_preserves_nonzero_history_and_call_domains() {
    for call_recompute in [false, true] {
        for copy_recompute in [false, true] {
            let leaves = expansion(call_recompute, copy_recompute);
            // Explicit K63 serialization by the finite adapter; Expander's
            // traversal IDs are opaque and are not serialized call_index.
            let mut seen = BTreeMap::new();
            let calls: Vec<_> = leaves
                .iter()
                .filter(|(_, l)| !l.invocation.is_empty())
                .map(|(p, l)| {
                    let next = seen.keys().filter(|(point, _)| point == p).count();
                    let ci = *seen.entry((*p, l.invocation.clone())).or_insert(next);
                    (*p, ci, l.ordinal)
                })
                .collect();
            assert_eq!(
                calls,
                if call_recompute {
                    vec![(1, 0, 0), (2, 0, 0), (2, 1, 0)]
                } else {
                    vec![(1, 0, 0), (2, 0, 0)]
                }
            );
            let caller_counts = |point| {
                leaves
                    .iter()
                    .filter(|(p, l)| *p == point && l.invocation.is_empty())
                    .count()
            };
            assert_eq!(caller_counts(2), if call_recompute { 12 } else { 0 });
            assert_eq!(caller_counts(3), if copy_recompute { 9 } else { 3 });
            for input in 0..8 {
                let mut q = QuantumState::new();
                let x = q.allocate(3).unwrap();
                for (i, w) in x.iter().enumerate() {
                    if input >> i & 1 != 0 {
                        q.gate("x", &[*w], None, &[]).unwrap();
                    }
                }
                if input & 4 != 0 {
                    q.gate("h", &[x[0]], None, &[]).unwrap();
                    q.gate("cx", &[x[0], x[2]], None, &[]).unwrap();
                }
                let before = q.clone();
                for reverse in [false, true] {
                    let mut allocations = BTreeMap::<String, Vec<u64>>::new();
                    let stream: Vec<_> = if reverse {
                        leaves.iter().rev().collect()
                    } else {
                        leaves.iter().collect()
                    };
                    let mut checked_entry = false;
                    for (point, l) in stream {
                        if !reverse && *point == 3 && !checked_entry {
                            let mut expected = before.clone();
                            let d = expected.allocate(2).unwrap();
                            for i in 0..2 {
                                expected.gate("cx", &[x[i], d[i]], None, &[]).unwrap();
                            }
                            assert_eq!(
                                q.amplitudes().unwrap(),
                                expected.amplitudes().unwrap(),
                                "call cleanup must restore d=x before quncopy"
                            );
                            checked_entry = true;
                        }
                        if let Some((name, width, _, _)) = &l.allocation {
                            if (l.gate == "alloc") ^ reverse {
                                allocations.insert(name.clone(), q.allocate(*width).unwrap());
                            } else {
                                q.release(&allocations.remove(name).unwrap()).unwrap();
                            }
                        } else {
                            let wires: Vec<_> = l
                                .wires
                                .iter()
                                .map(|(n, i)| if n == "x" { x[*i] } else { allocations[n][*i] })
                                .collect();
                            q.gate(&l.gate, &wires, None, &[]).unwrap();
                        }
                    }
                    assert!(allocations.is_empty());
                    assert_eq!(q.wire_count(), before.wire_count());
                    assert_eq!(q.amplitudes().unwrap(), before.amplitudes().unwrap());
                }
            }
        }
    }
}
