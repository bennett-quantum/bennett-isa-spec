mod support;
use bennett_isa_conformance::{
    json::{self, Json},
    ref_catalog::Machine,
    ref_language::{invert_trace, permutation, Leaf},
};
use bennett_isa_ref::Object;

fn source(calls: bool, intrinsic: bool) -> Object {
    let width = if intrinsic { 2 } else { 1 };
    let sig = format!(
        r#"{{"abi":"0.9","args":[],"bounds":[],"in":[{width},1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}}"#
    );
    let child = if intrinsic {
        r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[2],"tier":"q"}"#
    } else {
        r#"{"abi":"0.9","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#
    };
    let body = if intrinsic {
        "d = qcopy x\naction_call @mix (d) () () ()\nadj action_call @mix (d) () () ()\nquncopy x d"
    } else if calls {
        "d = qalloc any 1\naction_call @mix (d) (x) () ()\naction_call @mix (d) (z) () ()\nadj action_call @mix (d) (z) () ()\nadj action_call @mix (d) (x) () ()\nqfree d"
    } else {
        "d = qalloc any 1\ncx x d\ncx z d\ncx z d\ncx x d\nqfree d"
    };
    let child_body = if intrinsic {
        "perm_action @mix (a:wire[2] mut) () () () ()\n^entry():\ncx a[0] a[1]"
    } else {
        "perm_action @mix (a:wire mut) (b:wire in) () () ()\n^entry():\ncx b a"
    };
    support::rehash(&support::object(&[("outer", &sig, &format!("perm_action @outer (t:wire mut) (x:wire[{width}] in, z:wire in) () () ()\n^entry():\n{body}")), ("mix", child, child_body)], "{}", "").replace("=0.8", "=0.9"))
}
fn scheduled(o: &Object, modes: &[(usize, &str)]) -> Object {
    scheduled_for(o, modes, "[]")
}
fn scheduled_for(o: &Object, modes: &[(usize, &str)], iteration: &str) -> Object {
    let id = o.identity().unwrap();
    let h = &id.interface_hashes["outer"];
    let closure = id.closure_hashes["outer"].as_ref().unwrap();
    let entries = modes.iter().map(|(point, mode)| format!(r#"{{"cleanup":{{"realisation":"{mode}"}},"generated":0,"kind":"cleanup","path":[{{"iteration":{iteration},"point":"entry:{point}","symbol":"{h}"}}],"phase":"*","step":"*"}}"#)).collect::<Vec<_>>().join(",");
    let schedule = format!(
        r#"{{"bindings":{{"budget":[0,0,0,0],"closure_hash":"{closure}","epsilon":0,"inherited_controls":0,"params_hash":"{}","region":"outer","scheduler":{{"name":"cleanup-test","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[{entries}]}}"#,
        bennett_isa_ref::sha256_hex(b"[]"),
        "0".repeat(64)
    );
    Object::parse(&format!("{}\nsection schedule\n{schedule}\n", o.text)).unwrap()
}
fn expand(o: &Object, width: usize) -> Vec<Leaf> {
    let args = json::parse(r#"["t","x","z"]"#).unwrap();
    o.expand_action_source(
        "outer",
        args.as_arr().unwrap(),
        &[("t", 1), ("x", width), ("z", 1)],
        false,
    )
    .unwrap()
}
fn restored(leaves: &[Leaf], width: usize) {
    let bundles = [("t", 1), ("x", width), ("z", 1)];
    let m = Machine::compile(&permutation(leaves).unwrap(), &bundles).unwrap();
    let mut raw = leaves.to_vec();
    invert_trace(&mut raw).unwrap();
    for (inverse, forward) in raw.iter().zip(leaves.iter().rev()) {
        assert_eq!(inverse.source, forward.source);
        assert_eq!(inverse.ordinal, forward.ordinal);
        assert_eq!(inverse.invocation, forward.invocation);
        assert_eq!(inverse.allocation, forward.allocation);
    }
    let inv = Machine::compile(&permutation(&raw).unwrap(), &bundles).unwrap();
    for bits in 0..1u64 << (width + 2) {
        let mut state = m.state();
        m.set(&mut state, "t", bits & 1);
        m.set(&mut state, "x", (bits >> 1) & ((1 << width) - 1));
        m.set(&mut state, "z", bits >> (width + 1));
        let before = state.clone();
        m.run(&mut state, true, false).unwrap();
        assert_eq!(state, before, "all private wires clean, input {bits}");
        let mut reverse = inv.state();
        inv.set(&mut reverse, "t", bits & 1);
        inv.set(&mut reverse, "x", (bits >> 1) & ((1 << width) - 1));
        inv.set(&mut reverse, "z", bits >> (width + 1));
        let before = reverse.clone();
        inv.run(&mut reverse, true, false).unwrap();
        assert_eq!(reverse, before);
    }
}
#[test]
fn k80_actual_source_recompute_owns_all_eight_caller_leaves() {
    let o = source(false, false);
    let reverse = expand(&scheduled(&o, &[(5, "reverse")]), 1);
    assert_eq!(reverse.len(), 6);
    restored(&reverse, 1);
    let leaves = expand(&scheduled(&o, &[(5, "recompute")]), 1);
    assert_eq!(
        leaves.len(),
        11,
        "three unchanged computing leaves plus eight at carrier"
    );
    for (n, leaf) in leaves[3..].iter().enumerate() {
        assert_eq!(leaf.source.as_ref().unwrap().terminal.point, "entry:5");
        assert_eq!(leaf.ordinal, n);
    }
    assert_eq!(
        leaves[3..]
            .iter()
            .map(|l| l.gate.as_str())
            .collect::<Vec<_>>(),
        ["alloc", "cx", "cx", "cx", "cx", "cx", "release", "release"]
    );
    restored(&leaves, 1);
}
#[test]
fn k80_actual_calls_have_fresh_carrier_edges_and_separate_callee_spaces() {
    let leaves = expand(&scheduled(&source(true, false), &[(5, "recompute")]), 1);
    let mut calls = Vec::new();
    let mut caller = Vec::new();
    for leaf in &leaves {
        let s = leaf.source.as_ref().unwrap();
        if s.terminal.symbol == s.invocations[0].symbol && s.terminal.point == "entry:5" {
            caller.push((leaf.gate.as_str(), leaf.ordinal));
        } else if let Some(edge) = s.path.last().filter(|e| e.point.point == "entry:5") {
            calls.push(edge.call_index);
            assert_eq!(s.terminal.point, "entry:0");
            assert_eq!(leaf.ordinal, 0);
        }
    }
    assert_eq!(calls, [0, 1, 2, 3]);
    assert_eq!(
        caller,
        [("alloc", 0), ("cx", 1), ("release", 2), ("release", 3)]
    );
    restored(&leaves, 1);
}
#[test]
fn k81_actual_intrinsic_and_nonzero_call_history_select_independently() {
    for call in ["reverse", "recompute"] {
        for copy in ["reverse", "recompute"] {
            let leaves = expand(&scheduled(&source(true, true), &[(2, call), (3, copy)]), 2);
            let call_edges = leaves
                .iter()
                .filter_map(|l| l.source.as_ref()?.path.last())
                .filter(|e| e.point.point == "entry:2")
                .map(|e| e.call_index)
                .collect::<Vec<_>>();
            assert_eq!(
                call_edges,
                if call == "recompute" {
                    vec![0, 1]
                } else {
                    vec![0]
                }
            );
            restored(&leaves, 2);
        }
    }
}
#[test]
fn k81_replays_the_complete_nonzero_entry_history_before_an_independent_call() {
    let original = source(true, true);
    let text = original.text.replace(
        "action_call @mix (d) () () ()\nadj action_call @mix (d) () () ()",
        "cx z d[1]\naction_call @mix (d) () () ()\nadj action_call @mix (d) () () ()\ncx z d[1]",
    );
    let o = support::rehash(&text);
    for copy in ["reverse", "recompute"] {
        let leaves = expand(&scheduled(&o, &[(3, "recompute"), (5, copy)]), 2);
        restored(&leaves, 2);
        let calls = leaves
            .iter()
            .filter_map(|l| l.source.as_ref()?.path.last())
            .filter(|e| e.point.point == "entry:3")
            .map(|e| e.call_index)
            .collect::<Vec<_>>();
        assert_eq!(calls, [0, 1]);
    }
}
#[test]
fn k81_joint_inverse_is_owned_once_by_the_first_released_lifetime() {
    let root = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[],"tier":"q"}"#;
    let child = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1,1],"tier":"q"}"#;
    for release in ["qfree a\nqfree b", "qfree b\nqfree a"] {
        let o=support::rehash(&support::object(&[("outer",root,&format!("perm_action @outer () () () () ()\n^entry():\na = qalloc any 1\nb = qalloc any 1\naction_call @joint (a, b) () () ()\nadj action_call @joint (a, b) () () ()\n{release}")),("joint",child,"perm_action @joint (a:wire mut, b:wire mut) () () () ()\n^entry():\nx a\ncx a b")],"{}","").replace("=0.8","=0.9"));
        let o = scheduled(&o, &[(4, "recompute"), (5, "recompute")]);
        let leaves = o.expand_action_source("outer", &[], &[], false).unwrap();
        let mut edges = std::collections::BTreeSet::new();
        for l in &leaves {
            if let Some(e) = l.source.as_ref().unwrap().path.last() {
                if e.point.point != "entry:2" {
                    edges.insert((e.point.point.as_str(), e.call_index));
                }
            }
        }
        assert_eq!(
            edges,
            std::collections::BTreeSet::from([("entry:4", 0), ("entry:4", 1)])
        );
        let m = Machine::compile(&permutation(&leaves).unwrap(), &[]).unwrap();
        let mut state = m.state();
        let before = state.clone();
        m.run(&mut state, true, false).unwrap();
        assert_eq!(state, before);
    }
}
#[test]
fn k81_repeated_source_iterations_keep_fresh_allocations_and_carrier_coordinates() {
    let old = "d = qalloc any 1\ncx x d\ncx z d\ncx z d\ncx x d\nqfree d";
    let sig = r#"{"abi":"0.9","args":[],"bounds":[2],"in":[1,1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let o=support::rehash(&support::object(&[("outer",sig,&format!("perm_action @outer (t:wire mut) (x:wire in, z:wire in) () () ()\nregs {{  }} bounds {{2}}\n^entry():\nqloop 2 {{\n{old}\n}}"))],"{}","").replace("=0.8","=0.9"));
    let bound = scheduled_for(&o, &[(6, "recompute")], "\"*\"");
    let leaves = expand(&bound, 1);
    let at = leaves
        .iter()
        .filter(|l| l.source.as_ref().unwrap().terminal.point == "entry:6")
        .collect::<Vec<_>>();
    assert_eq!(at.len(), 16);
    for (n, l) in at.iter().enumerate() {
        assert_eq!(l.source.as_ref().unwrap().terminal.iteration, [n / 8]);
        assert_eq!(l.ordinal, n % 8);
    }
    let ids = leaves
        .iter()
        .filter(|l| l.gate == "alloc")
        .map(|l| l.allocation.as_ref().unwrap().0.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(ids.len(), 4);
    restored(&leaves, 1);
}
#[test]
fn k65_ties_require_an_actual_cleanup_carrier_and_its_zero_ordinal() {
    for calls in [false, true] {
        let o = source(calls, false);
        for point in [0, 1, 2, 3, 4] {
            let single = scheduled(&o, &[(point, "recompute")]);
            let mut s = single.section("schedule").unwrap().clone();
            if let bennett_isa_ref::json::Json::Object(fields) = &mut s {
                if let bennett_isa_ref::json::Json::Array(entries) =
                    fields.get_mut("entries").unwrap()
                {
                    entries.push(entries[0].clone());
                }
            }
            Object::parse(&format!(
                "{}\nsection schedule\n{}\n",
                o.text,
                s.canonical()
            ))
            .expect("no cleanup carrier at a computing or absorbed inverse row");
        }
        let single = scheduled(&o, &[(5, "recompute")]);
        let mut s = single.section("schedule").unwrap().clone();
        if let bennett_isa_ref::json::Json::Object(fields) = &mut s {
            if let bennett_isa_ref::json::Json::Array(entries) = fields.get_mut("entries").unwrap()
            {
                if let bennett_isa_ref::json::Json::Object(e) = &mut entries[0] {
                    e.insert("generated".into(), support::json("1"));
                }
                entries.push(entries[0].clone());
            }
        }
        Object::parse(&format!(
            "{}\nsection schedule\n{}\n",
            o.text,
            s.canonical()
        ))
        .expect("carrier ordinal one does not exist");
        assert!(
            Object::parse(&format!("{}\nsection schedule\n{}\n", o.text, {
                let mut s = single.section("schedule").unwrap().clone();
                if let bennett_isa_ref::json::Json::Object(fields) = &mut s {
                    if let bennett_isa_ref::json::Json::Array(entries) =
                        fields.get_mut("entries").unwrap()
                    {
                        entries.push(entries[0].clone());
                    }
                }
                s.canonical()
            }))
            .unwrap_err()
            .to_string()
            .contains("multiple maximal")
        );
    }
}
#[test]
fn k81_independent_call_preserves_an_arbitrary_interface_entry_value() {
    let sig = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[2],"tier":"q"}"#;
    let o=support::rehash(&support::object(&[("outer",sig,"perm_action @outer (t:wire[2] mut) () () () ()\n^entry():\naction_call @mix (t) () () ()\nadj action_call @mix (t) () () ()"),("mix",sig,"perm_action @mix (t:wire[2] mut) () () () ()\n^entry():\ncx t[0] t[1]")],"{}","").replace("=0.8","=0.9"));
    let o = scheduled(&o, &[(1, "recompute")]);
    let leaves = o
        .expand_action_source("outer", &[Json::Str("t".into())], &[("t", 2)], false)
        .unwrap();
    let m = Machine::compile(&permutation(&leaves).unwrap(), &[("t", 2)]).unwrap();
    for t in 0..4 {
        let mut state = m.state();
        m.set(&mut state, "t", t);
        let before = state.clone();
        m.run(&mut state, true, false).unwrap();
        assert_eq!(state, before, "entry t={t}");
    }
}
#[test]
fn k81_an_intervening_read_is_not_a_disjoint_row() {
    let o = source(false, false);
    let o = support::rehash(
        &o.text
            .replace("cx z d\ncx x d\nqfree d", "cx z d\ncx d t\ncx x d\nqfree d"),
    );
    let leaves = expand(&scheduled(&o, &[(6, "recompute")]), 1);
    let m = Machine::compile(
        &permutation(&leaves).unwrap(),
        &[("t", 1), ("x", 1), ("z", 1)],
    )
    .unwrap();
    for bits in 0..8 {
        let mut state = m.state();
        m.set(&mut state, "t", bits & 1);
        m.set(&mut state, "x", (bits >> 1) & 1);
        m.set(&mut state, "z", bits >> 2);
        let before = state.clone();
        m.run(&mut state, true, false).unwrap();
        assert_eq!(
            m.get(&state, "t"),
            (bits & 1) ^ ((bits >> 1) & 1),
            "input {bits}"
        );
        m.run(&mut state, true, true).unwrap();
        assert_eq!(state, before);
    }
    assert!(leaves
        .iter()
        .any(|l| l.source.as_ref().unwrap().terminal.point == "entry:3"));
}

#[test]
fn k81_history_replay_does_not_read_a_changed_external_input_version() {
    let o = source(true, true);
    let text = o
        .text
        .replace("\"in\":[2,1]", "\"in\":[1]")
        .replace("\"mut\":[1]", "\"mut\":[1,2]")
        .replace(
            "(t:wire mut) (x:wire[2] in, z:wire in)",
            "(t:wire mut, x:wire[2] mut) (z:wire in)",
        )
        .replace(
            "d = qcopy x\naction_call",
            "d = qcopy x\nx x[0]\naction_call",
        )
        .replace("\nquncopy x d", "\nx x[0]\nquncopy x d");
    let o = support::rehash(&text);
    let leaves = expand(&scheduled(&o, &[(3, "recompute"), (5, "reverse")]), 2);
    restored(&leaves, 2);
}

#[test]
fn k81_borrowed_targets_restore_every_entry_bit_including_internal_controls() {
    let sig = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[],"tier":"q"}"#;
    let o=support::rehash(&support::object(&[("outer",sig,"perm_action @outer () () () () ()\n^entry():\nb = qborrow any 2\ncx b[1] b[0]\ncx b[1] b[0]\nqreturn b")],"{}","").replace("=0.8","=0.9"));
    let o = scheduled(&o, &[(3, "recompute")]);
    let leaves = o.expand_action_source("outer", &[], &[], false).unwrap();
    // A borrowed lifetime is supplied by the caller, so expose its actual
    // source-generated identity as an explicit arbitrary witness input.
    let loan = leaves
        .iter()
        .find_map(|l| {
            l.allocation
                .as_ref()
                .filter(|a| a.3 == "borrowed")
                .map(|a| a.0.clone())
        })
        .unwrap();
    let mut gates = Vec::new();
    for mut l in leaves {
        if l.allocation.as_ref().is_some_and(|a| a.0 == loan) {
            continue;
        }
        for (name, _) in l.wires.iter_mut().chain(&mut l.controls) {
            if *name == loan {
                *name = "loan".into();
            }
        }
        gates.push(l);
    }
    let m = Machine::compile(&permutation(&gates).unwrap(), &[("loan", 2)]).unwrap();
    for value in 0..4 {
        let mut state = m.state();
        m.set(&mut state, "loan", value);
        let before = state.clone();
        m.run(&mut state, true, false).unwrap();
        assert_eq!(state, before, "borrowed entry {value}");
    }
}

#[test]
fn k65_other_provenance_retains_reverse_and_rejects_recompute() {
    let sig = r#"{"abi":"0.9","args":[],"bounds":[],"in":[1,1],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let o=support::rehash(&support::object(&[("outer",sig,"action @outer (t:wire mut) (x:wire in, z:wire in) () () ()\n^entry():\nd = qalloc any 1\nh d\nh d\nqfree d")],"{}","").replace("=0.8","=0.9"));
    let reverse = expand(&scheduled(&o, &[(3, "reverse")]), 1);
    assert_eq!(
        reverse.iter().map(|l| l.gate.as_str()).collect::<Vec<_>>(),
        ["alloc", "h", "h", "release"]
    );
    let o = scheduled(&o, &[(3, "recompute")]);
    let args = json::parse(r#"["t","x","z"]"#).unwrap();
    let error = o
        .expand_action_source(
            "outer",
            args.as_arr().unwrap(),
            &[("t", 1), ("x", 1), ("z", 1)],
            false,
        )
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("recompute requires permutation provenance"));
}
