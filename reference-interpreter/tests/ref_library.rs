mod support;
use bennett_isa_ref::{
    ref_library::{RefLibrary, Registry, Value, REF09_HASH},
    Object,
};

fn j(s: &str) -> Value {
    bennett_isa_conformance::json::parse(s).unwrap()
}
const CATALOG: &str = include_str!("../../spec/ref-0.9.json");

#[test]
fn bound_catalog_checks_the_full_canonical_envelope() {
    RefLibrary::bind("ref", "0.9", REF09_HASH, CATALOG).unwrap();
    for (name, version, hash, bytes) in [
        ("other", "0.9", REF09_HASH, CATALOG.to_owned()),
        ("ref", "0.8", REF09_HASH, CATALOG.to_owned()),
        ("ref", "0.9", &"0".repeat(64), CATALOG.to_owned()),
        ("ref", "0.9", REF09_HASH, format!("{CATALOG}\n")),
        (
            "ref",
            "0.9",
            REF09_HASH,
            CATALOG.replace("0.8 §4.5", "0.9 §4.5"),
        ),
    ] {
        let e = RefLibrary::bind(name, version, hash, &bytes)
            .err()
            .unwrap()
            .to_string();
        assert!(e.contains("E_SCHEDULE_INFEASIBLE"), "{e}");
    }
}

#[test]
fn default_and_explicit_09_binding_use_full_library_without_rewriting_schedule() {
    let source = include_str!("../../conformance/fixtures/controlled-action-09/controlled.bisa");
    // Check the complete published object. The implicit-binding variant below
    // is a different object, so it cannot retain this certificate's hash binding.
    let explicit = Object::parse(source).unwrap();
    explicit.check_hashes().unwrap();
    let before = explicit.identity().unwrap();
    let lib = explicit.bound_ref_library().unwrap();
    let binding = format!(r#","expansion":{{"hash":"{REF09_HASH}","name":"ref","version":"0.9"}}"#);
    assert!(source.contains(&binding));
    let implicit_source = source.split("\nsection certificate\n").next().unwrap();
    let implicit = Object::parse(&implicit_source.replace(&binding, "")).unwrap();
    assert_eq!(
        implicit.schedule_expansion().unwrap().unwrap().hash,
        Some(REF09_HASH)
    );
    implicit.bound_ref_library().unwrap();
    assert_eq!(explicit.identity().unwrap().object_hash, before.object_hash);
    assert_eq!(lib.hash(), REF09_HASH);
    // Accepted 0.8 keeps its existing admission/default behavior.
    let old = Object::parse(include_str!("fixtures/inc8.bisa")).unwrap();
    assert!(old.bound_ref_library().is_err());
}

#[test]
fn production_expansion_uses_published_recipes_and_lexical_actuals() {
    let lib = RefLibrary::published().unwrap();
    let hash = "a".repeat(64);
    let registry = Registry::from([(
        hash.clone(),
        j(
            r#"{"parameters":[{"name":"t","type":"wire","width":1}],"body":[{"gate":"x","wires":["t"]}]}"#,
        ),
    )]);
    let args = j(&format!(
        r#"["c",[{{"invoke":"action_call","args":["{hash}",["t"]]}}]]"#
    ));
    let out = lib
        .expand(
            "ctl",
            args.as_arr().unwrap(),
            &[("c", 1), ("t", 1)],
            registry,
        )
        .unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(
        (&out[0].gate, &out[0].wires, &out[0].controls),
        (
            &"x".to_owned(),
            &vec![("t".into(), 0)],
            &vec![("c".into(), 0)]
        )
    );
    assert_eq!(out[0].ordinal, 0);
    assert!(!out[0].invocation.is_empty());
    assert!(lib.expand("private", &[], &[], Registry::new()).is_err());
    assert!(lib
        .expand("ctl", &[j("[]"), j("[]")], &[], Registry::new())
        .is_err());
}

#[test]
fn oracle_context_is_validated_captured_and_built_per_invocation() {
    use bennett_isa_ref::{ref_library::OracleInvocation, Budget};
    let object = support::rehash(&include_str!("fixtures/inc8.bisa").replace("0.8", "0.9"));
    let args = j(r#"["x","t","f"]"#);
    let mut captures = Vec::new();
    for b in [2, 3] {
        let record = j(&format!(r#"{{"token":["{b}","0","0","0"]}}"#));
        let call = OracleInvocation::capture(
            &object,
            "inc8",
            Some(&record),
            Budget::steps(b),
            args.as_arr().unwrap(),
        )
        .unwrap();
        assert_eq!(call.budget(), Budget::steps(b));
        assert_eq!(call.arguments(), args.as_arr().unwrap());
        captures.push(call);
    }
    assert_eq!(captures[0].closure_hash(), captures[1].closure_hash());
    assert_ne!(captures[0].context(), captures[1].context());
    let lib = object.bound_ref_library().unwrap();
    // Bodies are intentionally a finite inc8 builder, not a general lowering.
    // They must be requested twice even with identical closure and actuals.
    let mut built = Vec::new();
    let expansions = captures
        .iter()
        .map(|call| {
            call.expand(&lib, &[("x", 8), ("t", 8), ("f", 4)], |source| {
                built.push(source.budget().steps);
                if source.budget().steps == 2 {
                    return Ok(j(r#"[{"gate":"x","wires":["f[0]"]}]"#));
                }
                let mut body = vec![j(r#"{"gate":"x","wires":["t[0]"]}"#)];
                for bit in 0..8 {
                    body.push(j(&format!(
                        r#"{{"gate":"cx","wires":["x[{bit}]","t[{bit}]"]}}"#
                    )));
                    if bit > 0 {
                        let controls = (0..bit)
                            .map(|i| format!("\"x[{i}]\""))
                            .collect::<Vec<_>>()
                            .join(",");
                        body.push(j(&format!(
                            r#"{{"gate":"mcx","wires":[{controls},"t[{bit}]"]}}"#
                        )));
                    }
                }
                Ok(Value::Arr(body))
            })
            .unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(built, [2, 3]);
    for x in 0..256 {
        let mut q = bennett_isa_ref::QuantumState::new();
        let wires = [
            q.allocate(8).unwrap(),
            q.allocate(8).unwrap(),
            q.allocate(4).unwrap(),
        ];
        for bit in 0..8 {
            if x >> bit & 1 != 0 {
                q.gate("x", &[wires[0][bit]], None, &[]).unwrap();
            }
        }
        let initial = q.clone();
        let apply = |q: &mut bennett_isa_ref::QuantumState,
                     leaves: &[bennett_isa_ref::ref_library::Leaf]| {
            for leaf in leaves {
                let operands = leaf
                    .wires
                    .iter()
                    .map(|(n, i)| {
                        wires[match n.as_str() {
                            "x" => 0,
                            "t" => 1,
                            "f" => 2,
                            _ => panic!(),
                        }][*i]
                    })
                    .collect::<Vec<_>>();
                q.gate(&leaf.gate, &operands, None, &[]).unwrap();
            }
        };
        for leaves in &expansions {
            apply(&mut q, leaves);
        }
        assert_eq!(
            (
                q.read_basis(&wires[1]).unwrap(),
                q.read_basis(&wires[2]).unwrap()
            ),
            ((x + 1) & 255, 1)
        );
        // Raw inverse retains the already-instantiated bodies and their tuples.
        for leaves in expansions.iter().rev() {
            apply(&mut q, &leaves.iter().rev().cloned().collect::<Vec<_>>());
        }
        assert_eq!(q, initial);
    }
}

#[test]
fn oracle_context_rejects_malformed_inconsistent_and_extra_positional_tokens() {
    use bennett_isa_ref::{ref_library::OracleInvocation, Budget};
    let object = support::rehash(&include_str!("fixtures/inc8.bisa").replace("0.8", "0.9"));
    let args = j(r#"["x","t","f"]"#);
    let good = j(r#"{"token":["2","0","0","0"]}"#);
    for record in [
        None,
        Some(j("[]")),
        Some(j("{}")),
        Some(j(r#"{"token":["2","0","0"]}"#)),
        Some(j(r#"{"token":[2,"0","0","0"]}"#)),
        Some(j(r#"{"token":["02","0","0","0"]}"#)),
        Some(j(r#"{"token":["3","0","0","0"]}"#)),
        Some(j(r#"{"token":["2","0","0","0"],"extra":0}"#)),
    ] {
        let e = OracleInvocation::capture(
            &object,
            "inc8",
            record.as_ref(),
            Budget::steps(2),
            args.as_arr().unwrap(),
        )
        .err()
        .unwrap()
        .to_string();
        assert!(e.contains("K77"), "{e}");
    }
    let mut extra = args.as_arr().unwrap().to_vec();
    extra.push(j(r#"["2","0","0","0"]"#));
    assert!(
        OracleInvocation::capture(&object, "inc8", Some(&good), Budget::steps(2), &extra).is_err()
    );
    // A failed nested capture cannot replace an immutable outer capture.
    let outer = OracleInvocation::capture(
        &object,
        "inc8",
        Some(&good),
        Budget::steps(2),
        args.as_arr().unwrap(),
    )
    .unwrap();
    let lib = object.bound_ref_library().unwrap();
    assert!(outer
        .expand(&lib, &[("x", 8), ("t", 8), ("f", 4)], |_| {
            OracleInvocation::capture(
                &object,
                "inc8",
                None,
                Budget::steps(3),
                args.as_arr().unwrap(),
            )?;
            Ok(j("[]"))
        })
        .is_err());
    assert_eq!(outer.budget(), Budget::steps(2));
    outer
        .expand(&lib, &[("x", 8), ("t", 8), ("f", 4)], |_| Ok(j("[]")))
        .unwrap();
}

#[test]
fn published_controlled_leaves_are_byte_exact_and_execute_every_basis_tuple() {
    use bennett_isa_conformance::ref_hash::canonical;
    use bennett_isa_ref::{QuantumBindings, QuantumState};
    let source = include_str!("../../conformance/fixtures/controlled-action-09/controlled.bisa");
    // Certificate admission is a separately retained .3 red; do not call this
    // a successful acceptance of the complete unmodified .bisa fixture.
    let source = source.split("\nsection certificate\n").next().unwrap();
    let object = Object::parse(source).unwrap();
    object.check_hashes().unwrap();
    let identity = object.identity().unwrap();
    let hash = identity.closure_hashes["flip"].as_ref().unwrap();
    let registry = Registry::from([(
        hash.clone(),
        j(
            r#"{"parameters":[{"name":"t","type":"wire","width":1}],"body":[{"gate":"x","wires":["t"]}]}"#,
        ),
    )]);
    let args = j(&format!(
        r#"["c",[{{"invoke":"action_call","args":["{hash}",["t"]]}}]]"#
    ));
    let library = object.bound_ref_library().unwrap();
    let leaves = library
        .expand(
            "ctl",
            args.as_arr().unwrap(),
            &[("t", 1), ("c", 1)],
            registry.clone(),
        )
        .unwrap();
    assert_eq!(
        leaves,
        library
            .expand(
                "ctl",
                args.as_arr().unwrap(),
                &[("t", 1), ("c", 1)],
                registry
            )
            .unwrap()
    );
    assert_eq!(leaves.len(), 1);
    let leaf = &leaves[0];
    let wires = |ws: &[(String, usize)]| {
        Value::Arr(
            ws.iter()
                .map(|(n, i)| Value::Str(format!("{n}[{i}]")))
                .collect(),
        )
    };
    let root = identity.closure_hashes["controlled"].as_ref().unwrap();
    let path = j(&format!(
        r#"[{{"iteration":[],"point":"entry:0","symbol":"{root}"}},{{"iteration":[],"point":"entry:0","symbol":"{hash}"}}]"#
    ));
    let rendered = Value::Arr(vec![Value::Obj(std::collections::BTreeMap::from([
        ("class".into(), j(r#""fast""#)),
        ("controls".into(), wires(&leaf.controls)),
        ("end".into(), j("1")),
        ("gate".into(), Value::Str(leaf.gate.clone())),
        ("generated".into(), Value::Num(leaf.ordinal as f64)),
        ("path".into(), path),
        ("phase".into(), j(r#""*""#)),
        ("start".into(), j("0")),
        ("step".into(), j(r#""*""#)),
        ("target_entry".into(), j("0")),
        ("wires".into(), wires(&leaf.wires)),
    ]))]);
    assert_eq!(
        canonical(&rendered).unwrap(),
        include_str!("../../conformance/fixtures/controlled-action-09/leaves.json")
    );
    for input in 0..4 {
        let mut q = QuantumState::new();
        let w = q.allocate(2).unwrap();
        for bit in 0..2 {
            if input >> bit & 1 != 0 {
                q.gate("x", &[w[bit]], None, &[]).unwrap();
            }
        }
        let mut bindings = QuantumBindings::default();
        bindings.bundles.insert("t".into(), vec![w[0]]);
        bindings.bundles.insert("c".into(), vec![w[1]]);
        let initial = q.clone();
        object
            .execute_action("controlled", &mut q, &bindings, false, &[])
            .unwrap();
        assert_eq!(q.read_basis(&w).unwrap(), input ^ (input >> 1));
        object
            .execute_action("controlled", &mut q, &bindings, true, &[])
            .unwrap();
        assert_eq!(q, initial);
    }
}

#[test]
fn captured_oracle_actuals_are_checked_before_builder_side_effects() {
    use bennett_isa_ref::{ref_library::OracleInvocation, Budget};
    let object = support::rehash(&include_str!("fixtures/inc8.bisa").replace("0.8", "0.9"));
    let record = j(r#"{"token":["3","0","0","0"]}"#);
    let args = j(r#"["x","t","f"]"#);
    let call = OracleInvocation::capture(
        &object,
        "inc8",
        Some(&record),
        Budget::steps(3),
        args.as_arr().unwrap(),
    )
    .unwrap();
    let lib = object.bound_ref_library().unwrap();
    let mut called = false;
    assert!(call
        .expand(&lib, &[("x", 7), ("t", 8), ("f", 4)], |_| {
            called = true;
            Ok(j("[]"))
        })
        .is_err());
    assert!(!called);
    let valid = call
        .expand(&lib, &[("x", 8), ("t", 8), ("f", 4)], |_| Ok(j("[]")))
        .unwrap();
    assert!(valid.is_empty());
}

#[test]
fn complete_published_controlled_fixture_is_admitted_and_executes() {
    use bennett_isa_ref::{QuantumBindings, QuantumState};
    let source = include_str!("../../conformance/fixtures/controlled-action-09/controlled.bisa");
    let o = Object::parse(source).unwrap();
    o.check_hashes().unwrap();
    assert_eq!(
        o.identity().unwrap().object_hash,
        "45bbb506541e3a8bb28a18e1bb0aef16ba01ee1e7cac74d5a190deb3ad36ef5a"
    );
    for t in 0..2 {
        for c in 0..2 {
            let mut q = QuantumState::new();
            let wires = q.allocate(2).unwrap();
            for (value, wire) in [(t, wires[0]), (c, wires[1])] {
                if value == 1 {
                    q.gate("x", &[wire], None, &[]).unwrap();
                }
            }
            let entry = q.clone();
            let mut bindings = QuantumBindings::default();
            bindings.bundles.insert("t".into(), vec![wires[0]]);
            bindings.bundles.insert("c".into(), vec![wires[1]]);
            o.execute_action("controlled", &mut q, &bindings, false, &[])
                .unwrap();
            assert_eq!(q.read_basis(&[wires[0]]).unwrap(), t ^ c);
            assert_eq!(q.read_basis(&[wires[1]]).unwrap(), c);
            o.execute_action("controlled", &mut q, &bindings, true, &[])
                .unwrap();
            assert_eq!(q, entry);
        }
    }
}

#[test]
fn published_certificate_rejects_conflicting_invocations_and_invalid_frames() {
    let source = include_str!("../../conformance/fixtures/controlled-action-09/controlled.bisa");
    for (from, to) in [
        (
            r#""invocation":{"args":[],"budget":["0","0","0","0"]},"#,
            "",
        ),
        (
            r#""budget":["0","0","0","0"]"#,
            r#""budget":["00","0","0","0"]"#,
        ),
        (
            r#""invocation":{"args":[]"#,
            r#""invocation":{"args":["1"]"#,
        ),
        (
            r#""budget":["0","0","0","0"]"#,
            r#""budget":["1","0","0","0"]"#,
        ),
        (
            r#""ancillae":[],"borrow":[],"history":[],"scratch":[],"slots""#,
            r#""ancillae":[],"borrow":[],"history":[],"slots""#,
        ),
        (r#""id":"in:0""#, r#""id":"mut:0""#),
        (
            r#""id":"mut:0","role":"mut","width":1"#,
            r#""id":"mut:0","role":"in","width":1"#,
        ),
        (
            r#""id":"mut:0","role":"mut","width":1"#,
            r#""id":"mut:0","role":"mut","width":2"#,
        ),
        (
            r#""id":"mut:0","role":"mut","width":1"#,
            r#""id":"mut:0","role":"mut","width":{"var":"k"}"#,
        ),
        (
            r#""class":"fast","id":"mut:0""#,
            r#""class":"","id":"mut:0""#,
        ),
        (
            r#""history":[],"scratch":[],"slots""#,
            r#""history":[],"scratch":[],"live_ranges":[{"frame":"absent","start":0,"end":1}],"slots""#,
        ),
        (
            r#""history":[],"scratch":[],"slots""#,
            r#""history":[],"scratch":[{"id":"s","class":"fast","state":"clean","width":1}],"live_ranges":[{"frame":"s","start":0,"end":2},{"frame":"s","start":1,"end":3}],"slots""#,
        ),
        (
            r#""history":[],"scratch":[],"slots""#,
            r#""history":[],"scratch":[{"id":"s","class":"fast","state":"clean","width":{"op":"/","args":[1,2]}}],"slots""#,
        ),
        (
            r#""before":{"borrowed":[],"bundles":["0","0"]"#,
            r#""before":{"borrowed":[],"bundles":["00","0"]"#,
        ),
        (
            r#""before":{"borrowed":[],"bundles":["0","0"]"#,
            r#""before":{"borrowed":[],"bundles":["0"]"#,
        ),
        (
            r#""before":{"borrowed":[],"bundles":["0","0"]"#,
            r#""before":{"borrowed":[],"bundles":["0","2"]"#,
        ),
        (
            r#""before":{"borrowed":[],"bundles":["0","0"]"#,
            r#""before":{"borrowed":["1"],"bundles":["0","0"]"#,
        ),
        (
            r#""inherited_controls":0,"schedule_hash""#,
            r#""inherited_controls":"340282366920938463463374607431768211455","schedule_hash""#,
        ),
    ] {
        assert!(source.contains(from), "missing mutation {from}");
        let bad = source.replacen(from, to, 1);
        assert!(Object::parse(&bad).is_err(), "accepted {from} -> {to}");
    }
}

#[test]
fn published_08_certificate_retains_legacy_schema_and_identity() {
    let source = include_str!("../../conformance/fixtures/controlled-action-09/compat-0.8.bisa");
    let o = Object::parse(source).unwrap();
    o.check_hashes().unwrap();
    for (from, to) in [
        (r#""interface":["#, r#""slots":["#),
        (
            r#""dialect_mapping_version":"worked-ctl.1""#,
            r#""dialect_mapping_version":"worked-ctl.1","invocation":{"args":[],"budget":["0","0","0","0"]}"#,
        ),
    ] {
        assert!(source.contains(from), "missing 0.8 mutation {from}");
        assert!(Object::parse(&source.replacen(from, to, 1)).is_err());
    }
}
