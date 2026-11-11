mod support;
use bennett_isa_conformance::{json, ref_catalog::Machine, ref_language::permutation};
use bennett_isa_ref::{
    ref_library::{OracleInvocation, RefLibrary, Registry},
    Budget, Object, OracleInput,
};

const R1_HASH: &str = "0be00e64de17699516a9b5bb5daf308ee0c59af7e39b6bf934bad5c628f48e7a";
const ORIGINAL_HASH: &str = "5de4e4995350313e50217880ba1b95de39cd043283b726337d4894efb005d197";
const ORIGINAL: &str = include_str!("../../spec/ref-0.9.json");
const REPAIRED: &str = include_str!("../../spec/ref-0.9-r1.json");
const SOURCE: &str = include_str!("fixtures/jump-width-source-gap.bisa");

#[test]
fn r1_is_default_and_original_hash_still_selects_original_bytes() {
    assert_eq!(RefLibrary::published().unwrap().hash(), R1_HASH);
    let original = RefLibrary::bind("ref", "0.9", ORIGINAL_HASH, ORIGINAL).unwrap();
    assert_eq!(original.hash(), ORIGINAL_HASH);
    assert_eq!(
        RefLibrary::bind("ref", "0.9", R1_HASH, REPAIRED)
            .unwrap()
            .hash(),
        R1_HASH
    );
    assert!(RefLibrary::bind("ref", "0.9", R1_HASH, ORIGINAL).is_err());
    assert!(RefLibrary::bind("ref", "0.9", ORIGINAL_HASH, REPAIRED).is_err());
    let source = include_str!("../../conformance/fixtures/controlled-action-09/controlled.bisa");
    let object = Object::parse(source).unwrap();
    let identity = object.identity().unwrap();
    assert_eq!(object.bound_ref_library().unwrap().hash(), ORIGINAL_HASH);
    assert_eq!(object.identity().unwrap().object_hash, identity.object_hash);
    // The original signature remains executable, rather than being relabelled r1.
    let args = json::parse(r#"[2,4,0,3,"a","succ","flag","faulted","code"]"#).unwrap();
    original
        .expand(
            "qjumpr",
            args.as_arr().unwrap(),
            &[
                ("a", 2),
                ("succ", 2),
                ("flag", 1),
                ("faulted", 1),
                ("code", 4),
            ],
            Registry::new(),
        )
        .unwrap();
}

#[test]
fn mixed_address_source_widths_expand_and_match_all_small_inputs() {
    let object = Object::parse(SOURCE).unwrap();
    let library = object.bound_ref_library().unwrap();
    let context = json::parse(r#"{"token":["3","0","0","0"]}"#).unwrap();
    let args = json::parse(r#"["a","b","x","t","f"]"#).unwrap();
    let call = OracleInvocation::capture(
        &object,
        "root",
        Some(&context),
        Budget::steps(3),
        args.as_arr().unwrap(),
    )
    .unwrap();
    let bundles = [("a", 3), ("b", 4), ("x", 1), ("t", 1), ("f", 4)];
    let leaves = call.expand_source(&library, &bundles).unwrap();
    let machine = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
    for a in 0..8 {
        for b in 0..16 {
            for x in 0..2 {
                let expected = object
                    .run_oracle("root", &OracleInput::bits(&[a, b, x]), Budget::steps(3))
                    .unwrap();
                let mut state = machine.state();
                for (name, value) in [
                    ("a", a as u64),
                    ("b", b as u64),
                    ("x", x as u64),
                    ("t", 1),
                    ("f", 9),
                ] {
                    machine.set(&mut state, name, value);
                }
                let entry = state.clone();
                machine.run(&mut state, true, false).unwrap();
                assert_eq!(
                    machine.get(&state, "t"),
                    1 ^ expected.payload as u64,
                    "a={a} b={b}"
                );
                assert_eq!(
                    machine.get(&state, "f"),
                    9 ^ expected.code as u64,
                    "a={a} b={b}"
                );
                for (name, value) in [("a", a), ("b", b), ("x", x)] {
                    assert_eq!(machine.get(&state, name), value as u64);
                }
                machine.run(&mut state, true, true).unwrap();
                assert_eq!(state, entry);
            }
        }
    }
}

#[test]
fn independent_successor_width_preserves_full_address_fault_checks() {
    let library = RefLibrary::published().unwrap();
    for (p, a) in [(2usize, 3usize), (2, 4), (4, 2)] {
        let args = json::parse(&format!(
            r#"[{p},{a},{},4,1,3,"a","succ","flag","faulted","code"]"#,
            p.min(a)
        ))
        .unwrap();
        let bundles = [
            ("a", a),
            ("succ", p),
            ("flag", 1),
            ("faulted", 1),
            ("code", 4),
        ];
        let leaves = library
            .expand("qjumpr", args.as_arr().unwrap(), &bundles, Registry::new())
            .unwrap();
        let machine = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
        for enabled in [false, true] {
            for address in 0..1u64 << a {
                let mut state = machine.state();
                machine.set(&mut state, "a", address);
                machine.set(&mut state, "succ", (1 << p) - 1);
                machine.set(&mut state, "code", 5);
                let entry = state.clone();
                let fault = enabled && !(1..3).contains(&address);
                machine.run(&mut state, enabled, false).unwrap();
                assert_eq!(machine.get(&state, "a"), address);
                assert_eq!(machine.get(&state, "flag"), fault as u64);
                assert_eq!(machine.get(&state, "faulted"), fault as u64);
                assert_eq!(machine.get(&state, "code"), 5 ^ if fault { 4 } else { 0 });
                assert_eq!(
                    machine.get(&state, "succ"),
                    ((1 << p) - 1)
                        ^ if enabled {
                            address & ((1 << p.min(a)) - 1)
                        } else {
                            0
                        }
                );
                machine.run(&mut state, enabled, true).unwrap();
                assert_eq!(state, entry);
            }
        }
    }
}

#[test]
fn original_catalog_source_builder_retains_original_qjumpr_arity() {
    let object = support::rehash(
        &SOURCE
            .replace("\"in\":[3,4,1]", "\"in\":[2,2,1]")
            .replace("a:wire[3]", "a:wire[2]")
            .replace("b:wire[4]", "b:wire[2]"),
    );
    let library = RefLibrary::bind("ref", "0.9", ORIGINAL_HASH, ORIGINAL).unwrap();
    let context = json::parse(r#"{"token":["3","0","0","0"]}"#).unwrap();
    let args = json::parse(r#"["a","b","x","t","f"]"#).unwrap();
    let call = OracleInvocation::capture(
        &object,
        "root",
        Some(&context),
        Budget::steps(3),
        args.as_arr().unwrap(),
    )
    .unwrap();
    let bundles = [("a", 2), ("b", 2), ("x", 1), ("t", 1), ("f", 4)];
    let leaves = call.expand_source(&library, &bundles).unwrap();
    let machine = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
    let mut state = machine.state();
    for (name, value) in [("a", 1), ("b", 2), ("x", 1)] {
        machine.set(&mut state, name, value);
    }
    let entry = state.clone();
    machine.run(&mut state, true, false).unwrap();
    assert_eq!((machine.get(&state, "t"), machine.get(&state, "f")), (1, 0));
    machine.run(&mut state, true, true).unwrap();
    assert_eq!(state, entry);
}
