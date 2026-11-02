mod support;
use bennett_isa_conformance::{json, ref_catalog::Machine, ref_language::permutation};
use bennett_isa_ref::{Object, QuantumBindings, QuantumRegion, QuantumState};

fn object(op: &str, a: usize, address: &str) -> Object {
    let body = match op {
        "qload" | "qunload" => format!("d = qload r {address}\nqxor d t\nqunload r {address} d"),
        _ => format!("{op} r {address} t"),
    };
    let sig = format!(
        r#"{{"abi":"0.9","args":[],"bounds":[],"in":[2,1],"kind":"perm_action","mem":[{{"A":{a},"W":1,"role":"mut"}}],"mut":[1],"tier":"q"}}"#
    );
    support::rehash(&support::object(&[("m",&sig,&format!("perm_action @m (t:wire mut) (a:wire[2] in, p:wire in) (r:region[{a},1] mut) () ()\n^entry():\nqpred p\n{body}\nqunpred p"))],"{}","\nsection layout\n{\"A_tab\":1,\"H\":4,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9"))
}
#[test]
fn k79_admitted_09_source_and_native_memory_cover_every_address_and_control_value() {
    for op in ["qload", "qunload", "qstore", "qexchm"] {
        for a in 0..=2 {
            let address = match a {
                0 => "()",
                1 => "a[0]",
                _ => "a",
            };
            let o = object(op, a, address);
            o.check_hashes().unwrap();
            let cells = 1usize << a;
            let args=json::parse(&format!(r#"["t","a","p",{{"address":{a},"cell":1,"role":"mut","origin":"interface:r","cells":"cells"}}]"#)).unwrap();
            let leaves = o
                .expand_action_source(
                    "m",
                    args.as_arr().unwrap(),
                    &[("t", 1), ("a", 2), ("p", 1), ("cells", cells)],
                    false,
                )
                .unwrap();
            let machine = Machine::compile(
                &permutation(&leaves).unwrap(),
                &[("t", 1), ("a", 2), ("p", 1), ("cells", cells)],
            )
            .unwrap();
            for bits in 0..1u64 << (4 + cells) {
                let mut state = machine.state();
                for (n, v) in [
                    ("t", bits & 1),
                    ("a", (bits >> 1) & 3),
                    ("p", (bits >> 3) & 1),
                    ("cells", bits >> 4),
                ] {
                    machine.set(&mut state, n, v);
                }
                let entry = state.clone();
                let mut expected = bits;
                if bits & 8 != 0 {
                    let index = ((bits >> 1) as usize) & (cells - 1);
                    let cell = (bits >> (4 + index)) & 1;
                    let target = bits & 1;
                    expected ^= match op {
                        "qload" | "qunload" => cell,
                        "qstore" => target << (4 + index),
                        "qexchm" => (target ^ cell) | ((target ^ cell) << (4 + index)),
                        _ => unreachable!(),
                    };
                }
                machine.run(&mut state, true, false).unwrap();
                assert_eq!(machine.get(&state, "t"), expected & 1, "{op}/A{a}/{bits}");
                assert_eq!(
                    machine.get(&state, "cells"),
                    expected >> 4,
                    "{op}/A{a}/{bits}"
                );
                machine.run(&mut state, true, true).unwrap();
                assert_eq!(state, entry);
                let mut q = QuantumState::new();
                let w = q.allocate(4 + cells).unwrap();
                for (n, wire) in w.iter().enumerate() {
                    if bits >> n & 1 != 0 {
                        q.gate("x", &[*wire], None, &[]).unwrap();
                    }
                }
                let b = QuantumBindings {
                    bundles: std::collections::BTreeMap::from([
                        ("t".into(), vec![w[0]]),
                        ("a".into(), w[1..3].to_vec()),
                        ("p".into(), vec![w[3]]),
                    ]),
                    regions: std::collections::BTreeMap::from([(
                        "r".into(),
                        QuantumRegion {
                            address_width: a as u64,
                            cell_width: 1,
                            cells: w[4..].iter().map(|w| vec![*w]).collect(),
                        },
                    )]),
                    ..Default::default()
                };
                let before = q.clone();
                o.execute_action("m", &mut q, &b, false, &[]).unwrap();
                assert_eq!(
                    q.read_basis(&w).unwrap(),
                    expected as u128,
                    "{op}/A{a}/{bits}"
                );
                o.execute_action("m", &mut q, &b, true, &[]).unwrap();
                assert_eq!(q, before);
            }
        }
    }
}

#[test]
fn k65_zero_address_qunload_recomputes_at_its_own_carrier() {
    let o = object("qunload", 0, "()");
    let identity = o.identity().unwrap();
    let h = &identity.interface_hashes["m"];
    let closure = identity.closure_hashes["m"].as_ref().unwrap();
    let schedule = format!(
        r#"{{"bindings":{{"budget":[0,0,0,0],"closure_hash":"{closure}","epsilon":0,"inherited_controls":0,"params_hash":"{}","region":"m","scheduler":{{"name":"memory-cleanup","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[{{"cleanup":{{"realisation":"recompute"}},"generated":0,"kind":"cleanup","path":[{{"iteration":[],"point":"entry:3","symbol":"{h}"}}],"phase":"*","step":"*"}}]}}"#,
        bennett_isa_ref::sha256_hex(b"[]"),
        "0".repeat(64)
    );
    let o = Object::parse(&format!("{}\nsection schedule\n{schedule}\n", o.text)).unwrap();
    let args=json::parse(r#"["t","a","p",{"address":0,"cell":1,"role":"mut","origin":"interface:r","cells":"cells"}]"#).unwrap();
    let bundles = [("t", 1), ("a", 2), ("p", 1), ("cells", 1)];
    let leaves = o
        .expand_action_source("m", args.as_arr().unwrap(), &bundles, false)
        .unwrap();
    let cleanup = leaves
        .iter()
        .filter(|l| l.source.as_ref().unwrap().terminal.point == "entry:3")
        .collect::<Vec<_>>();
    // The immutable dispatch recipe retains its own zero-address support
    // leaves; source composition does not fold those away to a single CX.
    let direct_args = json::parse(r#"[0,1,1,[],"cells","v"]"#).unwrap();
    let direct = bennett_isa_ref::ref_library::RefLibrary::published()
        .unwrap()
        .expand(
            "dispatch.copy",
            direct_args.as_arr().unwrap(),
            &[("cells", 1), ("v", 1)],
            Default::default(),
        )
        .unwrap();
    assert_eq!(cleanup.len(), 2 * direct.len() + 4);
    for (n, l) in cleanup.iter().enumerate() {
        assert_eq!(l.ordinal, n);
    }
    let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
    for bits in 0..32 {
        let mut state = m.state();
        for (n, v) in [
            ("t", bits & 1),
            ("a", (bits >> 1) & 3),
            ("p", (bits >> 3) & 1),
            ("cells", bits >> 4),
        ] {
            m.set(&mut state, n, v);
        }
        let before = state.clone();
        m.run(&mut state, true, false).unwrap();
        assert_eq!(
            m.get(&state, "t"),
            (bits & 1) ^ ((bits >> 3) & (bits >> 4) & 1)
        );
        m.run(&mut state, true, true).unwrap();
        assert_eq!(state, before);
    }
}
