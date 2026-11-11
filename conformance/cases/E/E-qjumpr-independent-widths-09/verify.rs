use bennett_isa_conformance::{
    json,
    ref_catalog::{Leaf as MachineLeaf, Machine, Operation},
    ref_hash::{canonical, envelope_hash},
    ref_language::{Expander, Registry},
};
use std::{fs, path::Path};

fn catalog() -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../../spec/ref-0.9-r1.json"),
    )
    .expect("missing repaired qjumpr catalog")
}

#[test]
fn complete_catalog_repairs_only_qjumpr_and_binds_its_bytes() {
    let bytes = catalog();
    let doc = json::parse(&bytes).unwrap();
    assert_eq!(canonical(&doc).unwrap(), bytes);
    let old = json::parse(include_str!("../../../../spec/ref-0.9.json")).unwrap();
    let recipes = doc.get("recipes").unwrap().as_arr().unwrap();
    let previous = old.get("recipes").unwrap().as_arr().unwrap();
    assert_eq!(recipes.len(), 219);
    let mut changed = 0;
    for (before, after) in previous.iter().zip(recipes) {
        assert_eq!(before.get("kind"), after.get("kind"));
        if before != after {
            changed += 1;
            assert_eq!(after.get("kind").unwrap().as_str(), Some("qjumpr"));
        }
    }
    assert_eq!(changed, 1);
    let draft = include_str!("../../../../spec/Bennett-ISA-Spec-0.9.md");
    let hash = envelope_hash(&bytes);
    assert!(draft.contains(&hash), "new catalog envelope not published");
    assert!(draft.contains("**Independent qjumpr widths**"));
}

#[test]
fn full_address_checks_and_independent_successor_width_reverse_on_the_full_small_domain() {
    let bytes = catalog();
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
        let leaves = Expander::new(&bytes, Registry::new())
            .unwrap()
            .expand("qjumpr", args.as_arr().unwrap(), &bundles)
            .unwrap();
        let machine_leaves = leaves
            .iter()
            .map(|leaf| {
                let operation = if let Some((id, width, _, _)) = &leaf.allocation {
                    if leaf.gate == "alloc" {
                        Operation::Alloc(id.clone(), *width)
                    } else {
                        assert_eq!(leaf.gate, "release");
                        Operation::Release(id.clone(), *width)
                    }
                } else {
                    assert!(matches!(leaf.gate.as_str(), "x" | "cx" | "ccx" | "mcx"));
                    let mut wires = leaf.controls.clone();
                    wires.extend(leaf.wires.clone());
                    Operation::Gate(
                        if leaf.controls.is_empty() {
                            leaf.gate.clone()
                        } else {
                            "mcx".into()
                        },
                        wires,
                    )
                };
                MachineLeaf {
                    ordinal: leaf.ordinal,
                    operation,
                }
            })
            .collect::<Vec<_>>();
        let machine = Machine::compile(&machine_leaves, &bundles).unwrap();
        for enabled in [false, true] {
            for address in 0..1u64 << a {
                for old_succ in [0, (1u64 << p) - 1] {
                    for old_faulted in [0, 1] {
                        for old_code in [0, 5] {
                            let mut state = machine.state();
                            for (name, value) in [
                                ("a", address),
                                ("succ", old_succ),
                                ("faulted", old_faulted),
                                ("code", old_code),
                            ] {
                                machine.set(&mut state, name, value);
                            }
                            let before = state.clone();
                            let flag = enabled && !(1..3).contains(&address);
                            machine.run(&mut state, enabled, false).unwrap();
                            assert_eq!(machine.get(&state, "a"), address);
                            assert_eq!(machine.get(&state, "flag"), flag as u64);
                            assert_eq!(
                                machine.get(&state, "succ"),
                                old_succ
                                    ^ if enabled {
                                        address & ((1u64 << p.min(a)) - 1)
                                    } else {
                                        0
                                    }
                            );
                            assert_eq!(machine.get(&state, "faulted"), old_faulted ^ flag as u64);
                            assert_eq!(
                                machine.get(&state, "code"),
                                old_code ^ if flag { 4 } else { 0 }
                            );
                            machine.run(&mut state, enabled, true).unwrap();
                            assert_eq!(state, before, "inverse did not restore inputs/scratch");
                        }
                    }
                }
            }
        }
    }
}
