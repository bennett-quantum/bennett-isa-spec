//! Pass IX / 7js: exactly A coherent address wires, including the empty actual.
mod support;
use bennett_isa_ref::{Object, QuantumBindings, QuantumRegion, QuantumState};

fn text(op: &str, a: u64, address: &str) -> String {
    let good = match a {
        0 => "()",
        1 => "a[0]",
        2 => "a",
        _ => unreachable!(),
    };
    let body = match op {
        "qload" => format!("d = qload r {address}\nqxor d t\nqunload r {address} d"),
        "qunload" => format!("d = qload r {good}\nqxor d t\nqunload r {address} d"),
        _ => format!("{op} r {address} t"),
    };
    let sig = format!(
        r#"{{"abi":"0.8","args":[],"bounds":[],"in":[2],"kind":"perm_action","mem":[{{"A":{a},"W":1,"role":"mut"}}],"mut":[1],"tier":"q"}}"#
    );
    support::object(&[("m", &sig, &format!("perm_action @m (t:wire mut) (a:wire[2] in) (r:region[{a},1] mut) () ()\n^entry():\n{body}"))], "{}", r#"
section layout
{"A_tab":2,"H":8,"S_cells":0,"globals":{},"heap_base":16,"heap_limit_max":0,"interface_regions":{},"null_size":1,"ptr_width":32}
"#)
}
fn check_cli(text: &str, tag: &str) -> Object {
    let o = support::rehash(text);
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("pass-ix-{tag}.bisa"));
    std::fs::write(&path, &o.text).unwrap();
    for cmd in ["hash", "check"] {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
            .arg(cmd)
            .arg(&path)
            .output()
            .unwrap();
        assert!(out.status.success(), "{tag}/{cmd}: {out:?}");
    }
    o
}
fn semantics(op: &str, a: u64, address: &str) {
    let o = check_cli(&text(op, a, address), &format!("{op}-{a}"));
    let cells = 1usize << a;
    // Includes arbitrary address spectators at A=0, every cell/target entry,
    // and false/true inherited control. Check literal truth, then exact inverse.
    for input in 0..1usize << (4 + cells) {
        let mut q = QuantumState::new();
        let wires = q.allocate(4 + cells).unwrap();
        for (i, w) in wires.iter().enumerate() {
            if input >> i & 1 != 0 {
                q.gate("x", &[*w], None, &[]).unwrap();
            }
        }
        let mut b = QuantumBindings::default();
        b.bundles.insert("t".into(), vec![wires[0]]);
        b.bundles.insert("a".into(), wires[1..3].to_vec());
        b.regions.insert(
            "r".into(),
            QuantumRegion {
                address_width: a,
                cell_width: 1,
                cells: wires[4..].iter().map(|w| vec![*w]).collect(),
            },
        );
        let initial = q.clone();
        let mut expected = input;
        if input >> 3 & 1 != 0 {
            let index = (input >> 1) & (cells - 1);
            let cell_bit = 4 + index;
            let (t, cell) = (input & 1, (input >> cell_bit) & 1);
            match op {
                "qload" | "qunload" => expected ^= cell,
                "qstore" => expected ^= t << cell_bit,
                "qexchm" => expected ^= (t ^ cell) | ((t ^ cell) << cell_bit),
                _ => unreachable!(),
            }
        }
        o.execute_action("m", &mut q, &b, false, &[wires[3]])
            .unwrap();
        assert_eq!(
            q.read_basis(&wires).unwrap(),
            expected as u128,
            "{op}/A={a}/input={input}"
        );
        o.execute_action("m", &mut q, &b, true, &[wires[3]])
            .unwrap();
        assert_eq!(q, initial, "{op}/A={a}/inverse/input={input}");
    }
}
#[test]
fn pass_ix_existing_nonzero_addresses_are_exact_width() {
    for op in ["qload", "qunload", "qstore", "qexchm"] {
        semantics(op, 1, "a[0]");
        semantics(op, 2, "a");
        for (a, bad) in [(1, "a"), (2, "a[0]")] {
            let e = Object::parse(&text(op, a, bad)).unwrap_err();
            assert!(
                e.to_string().contains("memory address width mismatch"),
                "{op}: {e}"
            );
        }
    }
}
#[test]
fn pass_ix_zero_address_qload() {
    semantics("qload", 0, "()");
}
#[test]
fn pass_ix_zero_address_qunload() {
    semantics("qunload", 0, "()");
}
#[test]
fn pass_ix_zero_address_qstore() {
    semantics("qstore", 0, "()");
}
#[test]
fn pass_ix_zero_address_qexchm() {
    semantics("qexchm", 0, "()");
}
#[test]
fn pass_ix_empty_actual_is_confined_to_zero_address_position() {
    for op in ["qload", "qunload", "qstore", "qexchm"] {
        for (a, bad) in [(0, "a[0]"), (0, "a"), (1, "()"), (2, "()"), (0, "(a)")] {
            let e = Object::parse(&text(op, a, bad)).unwrap_err();
            assert!(e.to_string().contains("§3.3"), "{op}/A={a}: {e}");
        }
    }
    for bad in [
        "t = qalloc any 0",
        "qxor () t",
        "qexch () t",
        "d = qcopy ()",
        "qstore r () ()",
        "qexchm r () ()",
    ] {
        let t = text("qstore", 0, "()");
        Object::parse(&t.replace("qstore r () t", bad)).expect_err(bad);
    }
    let t = text("qstore", 0, "()");
    Object::parse(&t.replace("a:wire[2]", "a:wire[0]")).expect_err("wire[0]");
}

#[test]
fn pass_ix_empty_address_compiles_with_borrow_frames_and_local_regions() {
    for op in ["qload", "qunload", "qstore", "qexchm"] {
        let t = text(op, 0, "()").replace(
            "^entry():",
            "^entry():\nb = qborrow any 1\ncx t b\ncx t b\nqreturn b",
        );
        check_cli(&t, &format!("borrow-{op}"));
    }
    let t = text("qstore", 0, "()").replace("qstore r () t", "qregion local 0 1\nqstore local () t\nqstore local () t\nqexchm local () t\nqexchm local () t\nd = qload local ()\nqunload local () d");
    let o = check_cli(&t, "local-zero");
    for input in 0..4 {
        let mut q = QuantumState::new();
        let w = q.allocate(4).unwrap();
        if input & 1 != 0 {
            q.gate("x", &[w[0]], None, &[]).unwrap();
        }
        if input & 2 != 0 {
            q.gate("h", &[w[1]], None, &[]).unwrap();
        }
        let mut b = QuantumBindings::default();
        b.bundles.insert("t".into(), vec![w[0]]);
        b.bundles.insert("a".into(), w[1..3].to_vec());
        b.regions.insert(
            "r".into(),
            QuantumRegion {
                address_width: 0,
                cell_width: 1,
                cells: vec![vec![w[3]]],
            },
        );
        let before = q.clone();
        o.execute_action("m", &mut q, &b, false, &[]).unwrap();
        assert_eq!(q, before);
        o.execute_action("m", &mut q, &b, true, &[]).unwrap();
        assert_eq!(q, before);
    }
}

#[test]
fn pass_ix_catalog_zero_address_binds_empty_wire_actual() {
    use bennett_isa_conformance::{
        json,
        ref_bdd::Bdd,
        ref_catalog::Machine,
        ref_language::{permutation, Expander, Registry},
    };
    let mut e = Expander::new(include_str!("../../spec/ref-0.9.json"), Registry::new()).unwrap();
    for kind in ["dispatch.copy", "qstore", "qexchm"] {
        for width in 1..=3 {
            let bundles = [("cells", width), ("v", width)];
            let args = json::parse(&format!(r#"[0,1,{width},[],"cells","v"]"#)).unwrap();
            let leaves = e.expand(kind, args.as_arr().unwrap(), &bundles).unwrap();
            let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
            let mut b = Bdd::new();
            let mut state = m.symbolic_state();
            for name in ["cells", "v"] {
                for id in &m.inputs[name] {
                    state[*id] = b.variable();
                }
            }
            let enabled = b.variable();
            let old = state.clone();
            let mut expected = old.clone();
            for bit in 0..width {
                let (cell, v) = (m.inputs["cells"][bit], m.inputs["v"][bit]);
                match kind {
                    "dispatch.copy" => {
                        let d = b.and(enabled, old[cell]);
                        expected[v] = b.xor(old[v], d);
                    }
                    "qstore" => {
                        let d = b.and(enabled, old[v]);
                        expected[cell] = b.xor(old[cell], d);
                    }
                    "qexchm" => {
                        let d = b.xor(old[v], old[cell]);
                        let d = b.and(enabled, d);
                        expected[cell] = b.xor(old[cell], d);
                        expected[v] = b.xor(old[v], d);
                    }
                    _ => unreachable!(),
                }
            }
            m.run_symbolic(&mut b, &mut state, enabled, false, 1)
                .unwrap();
            assert_eq!(state, expected, "{kind}/W={width}");
            m.run_symbolic(&mut b, &mut state, enabled, true, 1)
                .unwrap();
            assert_eq!(state, old);
        }
    }
}

#[path = "support/round8.rs"]
mod round8;
#[test]
fn portable_k79_vectors_exact_bytes() {
    let mut rows = Vec::new();
    for op in ["qload", "qunload", "qstore", "qexchm"] {
        for (a, address, admitted) in [(0, "()", true), (0, "a[0]", false), (1, "()", false)] {
            let source = text(op, a, address);
            let parsed = Object::parse(&source);
            assert_eq!(parsed.is_ok(), admitted);
            let mut tuples = Vec::new();
            if let Ok(o) = parsed {
                for input in 0..32u128 {
                    let mut q = QuantumState::new();
                    let w = q.allocate(5).unwrap();
                    for (i, wire) in w.iter().enumerate() {
                        if input >> i & 1 != 0 {
                            q.gate("x", &[*wire], None, &[]).unwrap();
                        }
                    }
                    let mut b = QuantumBindings::default();
                    b.bundles.insert("t".into(), w[0..1].to_vec());
                    b.bundles.insert("a".into(), w[1..3].to_vec());
                    b.regions.insert(
                        "r".into(),
                        QuantumRegion {
                            address_width: 0,
                            cell_width: 1,
                            cells: vec![w[4..5].to_vec()],
                        },
                    );
                    let before = q.clone();
                    o.execute_action("m", &mut q, &b, false, &[w[3]]).unwrap();
                    let after = q.read_basis(&w).unwrap();
                    let mut exact = QuantumState::new();
                    let v = exact.allocate(5).unwrap();
                    for (i, wire) in v.iter().enumerate() {
                        if after >> i & 1 != 0 {
                            exact.gate("x", &[*wire], None, &[]).unwrap();
                        }
                    }
                    assert_eq!(q.amplitudes().unwrap(), exact.amplitudes().unwrap());
                    o.execute_action("m", &mut q, &b, true, &[w[3]]).unwrap();
                    assert_eq!(q, before);
                    tuples.push(format!(r#"{{"before":{input},"after":{after},"inverse_after":{input},"phase":[1,0],"live_wires":5,"scratch":[]}}"#));
                }
            }
            rows.push(format!(r#"{{"operation":{},"address_width":{a},"address_actual":{},"source":{},"source_admission":{admitted},"catalog_address":{},"tuple_order":["t","a[0]","a[1]","control","r[0][0]"],"tuples":[{}]}}"#, round8::quote(op), round8::quote(address), round8::quote(&source), if admitted {"[]"} else {"null"}, tuples.join(",")));
        }
    }
    round8::lock("k79.json", rows);
}
