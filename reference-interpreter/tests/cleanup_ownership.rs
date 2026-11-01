//! Pass X / K65: finite multirow source-binding witness, not a general 0.9
//! cleanup builder. The reference still admits 0.8. Address annotation below
//! is explicit adapter metadata; Expander supplies real leaves/callee spaces.
mod support;
use bennett_isa_conformance::{
    json::{self, Json},
    ref_language::{Expander, Leaf, Registry},
};
use bennett_isa_ref::{Object, QuantumBindings, QuantumState};
use std::collections::BTreeMap;

fn j(s: &str) -> Json {
    json::parse(s).unwrap()
}
const COMPUTE: &str = "d = qalloc any 1\ncx x d\ncx z d\n";
const REVERSE: &str = "cx z d\ncx x d\nqfree d";
fn source(cleanup: &str) -> Object {
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1,1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    support::rehash(&support::object(
        &[("multi", sig, &format!("perm_action @multi (t:wire mut) (x:wire in, z:wire in) () () ()\n^entry():\n{COMPUTE}{cleanup}"))], "{}", ""))
}

fn source_calls() -> Object {
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1,1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let child_sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    support::rehash(&support::object(&[
        ("multi",sig,"perm_action @multi (t:wire mut) (x:wire in, z:wire in) () () ()\n^entry():\nd = qalloc any 1\naction_call @flip (d) (x) () ()\naction_call @flip (d) (z) () ()\nadj action_call @flip (d) (z) () ()\nadj action_call @flip (d) (x) () ()\nqfree d"),
        ("flip",child_sig,"perm_action @flip (d:wire mut) (a:wire in) () () ()\n^entry():\ncx a d")],"{}",""))
}

#[test]
fn pass_x_literal_multirow_reverse_keeps_written_rows_and_restores_full_state() {
    let objects = [source(REVERSE), source_calls()];
    // Literal source rows remain individually addressable in the reference.
    let instructions = &objects[0].functions["multi"].blocks[0].instructions;
    assert_eq!(instructions.len(), 6);
    assert_eq!(instructions[3].operands, ["z", "d"]);
    assert_eq!(instructions[4].operands, ["x", "d"]);
    assert_eq!(instructions[5].op, "qfree");
    for (case, o) in objects.iter().enumerate() {
        o.check_hashes().unwrap();
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(format!("pass-x-multirow-{case}.bisa"));
        std::fs::write(&path, &o.text).unwrap();
        for command in ["hash", "check"] {
            let out = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                .arg(command)
                .arg(&path)
                .output()
                .unwrap();
            assert!(out.status.success(), "{out:?}");
        }
        // All x,z,target,control values, then coherent entangled inputs too.
        for input in 0..32 {
            let mut q = QuantumState::new();
            let w = q.allocate(4).unwrap();
            for (bit, wire) in w.iter().enumerate() {
                if input >> bit & 1 != 0 {
                    q.gate("x", &[*wire], None, &[]).unwrap();
                }
            }
            if input & 16 != 0 {
                q.gate("h", &[w[1]], None, &[]).unwrap();
                q.gate("cx", &[w[1], w[2]], None, &[]).unwrap();
                q.gate("h", &[w[3]], None, &[]).unwrap();
            }
            let mut b = QuantumBindings::default();
            for (name, wire) in ["t", "x", "z"].iter().zip(&w) {
                b.bundles.insert((*name).into(), vec![*wire]);
            }
            let before = q.clone();
            for adjoint in [false, true] {
                o.execute_action("multi", &mut q, &b, adjoint, &[w[3]])
                    .unwrap();
                assert_eq!(q, before, "case={case}, input={input}, adjoint={adjoint}");
            }
        }
    }
}

// Read the published catalog unchanged and select its three used recipes.
// The finite builder root and linked one-CX callee are synthetic inputs.
fn expansion(calls: bool) -> Vec<Leaf> {
    let hash = "a".repeat(64);
    let parameters = j(
        r#"[{"name":"x","type":"wire","width":1},{"name":"z","type":"wire","width":1},{"name":"p","type":"wire","width":1},{"name":"S","type":"symbol"}]"#,
    );
    let cx = |a: &str, b: &str| format!(r#"{{"gate":"cx","wires":["{a}","{b}"]}}"#);
    let call =
        |a: &str, b: &str| format!(r#"{{"invoke":"action_call","args":["S",["{a}","{b}"]]}}"#);
    let compute = if calls {
        vec![call("x", "u"), call("z", "u")]
    } else {
        vec![cx("x", "u"), cx("z", "u")]
    };
    // Materialise the adjoint sequence before numbering the NEW realisation.
    // These are separate call occurrences, not reverse traversal of the first
    // two already-numbered calls. CX's source adjoint is CX.
    let original = if calls {
        vec![call("x", "d"), call("z", "d")]
    } else {
        vec![cx("x", "d"), cx("z", "d")]
    };
    let body = format!(
        r#"[{{"declare":"d","width":1,"class":{{"class":"any"}},"state":"clean","body":[{{"invoke":"qalloc","args":["d"]}},{},{},{{"declare":"u","width":1,"class":{{"class":"any"}},"state":"clean","body":[{{"invoke":"qalloc","args":["u"]}},{},{},{},{},{},{{"invoke":"qfree","args":["u"]}}]}},{{"invoke":"qfree","args":["d"]}}]}}]"#,
        original[0],
        original[1],
        compute[0],
        compute[1],
        cx("u", "d"),
        compute[1],
        compute[0]
    );
    let body = format!(r#"[{{"under":"p","body":{body}}}]"#);
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
            "action_call" | "qalloc" | "qfree"
        )
    });
    recipes.push(j(&format!(r#"{{"kind":"zz.cleanup-witness","parameters":{parameters},"body":{body},"scratch":[],"removable":[],"source":"§11 K80 finite multirow source binding"}}"#)));
    let mut registry = Registry::new();
    registry.insert(hash.clone(), j(r#"{"parameters":[{"name":"a","type":"wire","width":1},{"name":"b","type":"wire","width":1}],"body":[{"gate":"cx","wires":["a","b"]}]}"#));
    Expander::new(&doc.to_string(), registry)
        .unwrap()
        .expand(
            "zz.cleanup-witness",
            &[j(r#""x""#), j(r#""z""#), j(r#""p""#), Json::Str(hash)],
            &[("x", 1), ("z", 1), ("p", 1)],
        )
        .unwrap()
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Address {
    point: usize,
    call_index: Option<usize>,
    ordinal: usize,
}
// The finite builder places this entire replacement at the selected carrier.
// Callee traversal IDs from Expander are opaque; serialize K63 call indices
// in first-entry order, independently of leaf ordinals. No production source
// adapter currently supplies source points or serialized call_index for us.
fn addresses(leaves: &[Leaf]) -> Vec<Address> {
    let mut calls = BTreeMap::new();
    let mut ordinals = BTreeMap::new();
    leaves
        .iter()
        .enumerate()
        .map(|(i, l)| {
            let point = if i < 3 { i } else { 5 };
            let call_index = if l.invocation.is_empty() {
                None
            } else {
                let next = calls.keys().filter(|(p, _)| *p == point).count();
                Some(*calls.entry((point, l.invocation.clone())).or_insert(next))
            };
            let ordinal = if call_index.is_some() {
                l.ordinal
            } else {
                let n = ordinals.entry(point).or_insert(0);
                let ordinal = *n;
                *n += 1;
                ordinal
            };
            Address {
                point,
                call_index,
                ordinal,
            }
        })
        .collect()
}

// Raw QuantumState allocation IDs are monotonic. Unlike execute_action, this
// low-level harness has no allocator checkpoint API, so compare the entire
// live state including phase and require every local allocation released.
fn same_live_state(a: &QuantumState, b: &QuantumState) {
    assert_eq!(a.wire_count(), b.wire_count());
    assert_eq!(a.amplitudes().unwrap(), b.amplitudes().unwrap());
}

fn apply(q: &mut QuantumState, leaves: &[Leaf], reverse: bool, w: &[u64]) {
    let mut allocations = BTreeMap::new();
    // Run both directions without rebuilding the expanded stream. Keep its
    // allocation identities, including u, rather than minting fresh addresses.
    let indices: Vec<_> = if reverse {
        (0..leaves.len()).rev().collect()
    } else {
        (0..leaves.len()).collect()
    };
    for i in indices {
        let l = &leaves[i];
        if let Some((name, width, _, _)) = &l.allocation {
            if (l.gate == "alloc") ^ reverse {
                allocations.insert(name.clone(), q.allocate(*width).unwrap());
            } else {
                q.release(&allocations.remove(name).unwrap()).unwrap();
            }
        } else {
            let operands = l
                .wires
                .iter()
                .map(|(name, bit)| match name.as_str() {
                    "x" => w[0],
                    "z" => w[1],
                    "p" => w[2],
                    _ => allocations[name][*bit],
                })
                .collect::<Vec<_>>();
            let controls = l
                .controls
                .iter()
                .map(|(name, bit)| match name.as_str() {
                    "p" => w[2],
                    _ => allocations[name][*bit],
                })
                .collect::<Vec<_>>();
            q.gate(&l.gate, &operands, None, &controls).unwrap();
        }
    }
    assert!(allocations.is_empty());
}

#[test]
fn pass_x_recompute_has_one_fresh_leaf_space_and_exact_cleanup_semantics() {
    let leaves = expansion(false);
    let actual = addresses(&leaves);
    assert_eq!(
        leaves.iter().map(|l| l.gate.as_str()).collect::<Vec<_>>(),
        ["alloc", "cx", "cx", "alloc", "cx", "cx", "cx", "cx", "cx", "release", "release"]
    );
    let mut expected = (0..3)
        .map(|point| Address {
            point,
            call_index: None,
            ordinal: 0,
        })
        .collect::<Vec<_>>();
    expected.extend((0..8).map(|ordinal| Address {
        point: 5,
        call_index: None,
        ordinal,
    }));
    assert_eq!(actual, expected);
    for input in 0..8 {
        let mut q = QuantumState::new();
        let w = q.allocate(3).unwrap();
        for (bit, wire) in w.iter().enumerate() {
            if input >> bit & 1 != 0 {
                q.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        let before = q.clone();
        apply(&mut q, &leaves, false, &w);
        same_live_state(&q, &before);
        apply(&mut q, &leaves, true, &w);
        same_live_state(&q, &before);
    }
    // The carrier is a different kind at the same numerical address as alloc.
    // Exercise the real reference selector engine, without claiming 0.9 parse.
    let mut o = source(REVERSE);
    let hash = o.identity().unwrap().interface_hashes["multi"].clone();
    let path = |point| format!(r#"[{{"symbol":"{hash}","point":"entry:{point}","iteration":[]}}]"#);
    let selector = |point, generated, kind: &str, payload: &str| {
        support::json(&format!(
            r#"{{"path":{},"step":"*","phase":"*","generated":{generated},"kind":"{kind}","{kind}":{payload}}}"#,
            path(point)
        ))
    };
    let entries = vec![
        selector(5, 0, "cleanup", r#"{"realisation":"recompute"}"#),
        selector(1, 0, "gate", r#"{"realisation":"original"}"#),
        selector(5, 0, "gate", r#"{"realisation":"wrong-kind"}"#),
        selector(5, 1, "gate", r#"{"realisation":"fresh"}"#),
    ];
    o.metadata.insert(
        "schedule".into(),
        support::json(&format!(
            r#"{{"entries":[{}]}}"#,
            entries
                .iter()
                .map(|e| e.canonical())
                .collect::<Vec<_>>()
                .join(",")
        )),
    );
    let operation = |point, generated| {
        support::json(&format!(
            r#"{{"path":{},"step":"*","phase":"*","generated":{generated}}}"#,
            path(point)
        ))
    };
    assert_eq!(o.schedule_cleanup(&operation(5, 0)).unwrap(), "recompute");
    assert_eq!(
        o.resolve_schedule(&operation(1, 0), "gate").unwrap(),
        Some(&entries[1])
    );
    // Only query gate selection for emitted gates; alloc at 0 is not a gate.
    for (l, a) in leaves.iter().zip(&actual).filter(|(l, _)| l.gate == "cx") {
        let selected = o
            .resolve_schedule(&operation(a.point, a.ordinal), "gate")
            .unwrap();
        let expected = match (a.point, a.ordinal) {
            (1, 0) => Some(&entries[1]),
            (5, 1) => Some(&entries[3]),
            _ => None,
        };
        assert_eq!(selected, expected, "{l:?}, {a:?}");
    }
    let schedule = o.metadata.get_mut("schedule").unwrap();
    let bennett_isa_ref::json::Json::Object(fields) = schedule else {
        unreachable!()
    };
    let bennett_isa_ref::json::Json::Array(selected) = fields.get_mut("entries").unwrap() else {
        unreachable!()
    };
    selected.push(entries[3].clone());
    assert!(o
        .resolve_schedule(&operation(5, 1), "gate")
        .unwrap_err()
        .to_string()
        .contains("multiple maximal"));
    assert_eq!(
        o.resolve_schedule(&operation(1, 0), "gate").unwrap(),
        Some(&entries[1])
    );
}

#[test]
fn pass_x_recompute_calls_are_fresh_edges_with_separate_callee_spaces() {
    let leaves = expansion(true);
    let a = addresses(&leaves);
    assert_eq!(
        a.iter()
            .filter(|a| a.point == 5 && a.call_index.is_none())
            .map(|a| a.ordinal)
            .collect::<Vec<_>>(),
        [0, 1, 2, 3]
    );
    let calls = a
        .iter()
        .filter(|a| a.call_index.is_some())
        .collect::<Vec<_>>();
    assert_eq!(
        calls
            .iter()
            .map(|a| (a.point, a.call_index, a.ordinal))
            .collect::<Vec<_>>(),
        [
            (1, Some(0), 0),
            (2, Some(0), 0),
            (5, Some(0), 0),
            (5, Some(1), 0),
            (5, Some(2), 0),
            (5, Some(3), 0)
        ]
    );
    let unique = leaves
        .iter()
        .filter(|l| !l.invocation.is_empty())
        .map(|l| l.invocation.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), 6);
    let inverse = a.iter().rev().cloned().collect::<Vec<_>>();
    assert_eq!(
        inverse
            .iter()
            .filter(|a| a.point == 5)
            .filter_map(|a| a.call_index)
            .collect::<Vec<_>>(),
        [3, 2, 1, 0]
    );
    for input in 0..8 {
        let mut q = QuantumState::new();
        let w = q.allocate(3).unwrap();
        for (bit, wire) in w.iter().enumerate() {
            if input >> bit & 1 != 0 {
                q.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        let before = q.clone();
        apply(&mut q, &leaves, false, &w);
        same_live_state(&q, &before);
        apply(&mut q, &leaves, true, &w);
        same_live_state(&q, &before);
    }
}
