use bennett_isa_conformance::{
    json,
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
#[test]
fn generic_dispatch_full_symbolic_domains() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    for a in 1..=4 {
        let cells = 1 << a;
        for w in 1..=4 {
            for kind in [
                "dispatch.copy",
                "dispatch.store",
                "dispatch.exchange",
                "qstore",
                "qexchm",
            ] {
                let op = match kind {
                    "qstore" => "store",
                    "qexchm" => "exchange",
                    other => other.strip_prefix("dispatch.").unwrap(),
                };
                let bundles = [("a", a), ("cells", cells * w), ("v", w)];
                let args = json::parse(&format!(r#"[{a},{cells},{w},"a","cells","v"]"#)).unwrap();
                let m = Machine::compile(
                    &permutation(&e.expand(kind, args.as_arr().unwrap(), &bundles).unwrap())
                        .unwrap(),
                    &bundles,
                )
                .unwrap();
                let mut b = Bdd::new();
                let mut s = m.symbolic_state();
                for name in ["a", "cells", "v"] {
                    for id in &m.inputs[name] {
                        s[*id] = b.variable();
                    }
                }
                let enabled = b.variable();
                let old = s.clone();
                let mut expected = old.clone();
                for address in 0..cells {
                    let equal = b.equal(
                        &m.inputs["a"].iter().map(|i| old[*i]).collect::<Vec<_>>(),
                        &Bdd::constant(address, a),
                    );
                    let active = b.and(enabled, equal);
                    for bit in 0..w {
                        let cell = m.inputs["cells"][address * w + bit];
                        let value = m.inputs["v"][bit];
                        match op {
                            "copy" => {
                                let d = b.and(active, old[cell]);
                                expected[value] = b.xor(expected[value], d);
                            }
                            "store" => {
                                let d = b.and(active, old[value]);
                                expected[cell] = b.xor(expected[cell], d);
                            }
                            "exchange" => {
                                let d = b.xor(old[cell], old[value]);
                                let d = b.and(active, d);
                                expected[cell] = b.xor(expected[cell], d);
                                expected[value] = b.xor(expected[value], d);
                            }
                            _ => unreachable!(),
                        }
                    }
                }
                m.run_symbolic(&mut b, &mut s, enabled, false, 1).unwrap();
                assert_eq!(s, expected, "{op} A={a} W={w}");
                m.run_symbolic(&mut b, &mut s, enabled, true, 1).unwrap();
                assert_eq!(s, old);
                println!(
                    "dispatch {op}: A={a} W={w}, full 2^{} tuples, {} BDD nodes",
                    a + cells * w + w + 1,
                    b.nodes()
                );
            }
        }
    }
}

#[test]
fn generic_scans_full_symbolic_domains() {
    let bytes = include_str!("../../spec/ref-0.9-draft.json");
    let mut e = Expander::new(bytes, Registry::new()).unwrap();
    for n in 1..=4 {
        for length_width in 1..=4 {
            for cells in [1, 2] {
                let t = if cells == 1 { 1 } else { 2 };
                for op in [
                    "scan.inrange",
                    "scan.free.flag",
                    "lift.boundscheck",
                    "lift.free",
                ] {
                    let row = op.starts_with("lift.");
                    let target = if row { "flag" } else { "t" };
                    let mut bundles = vec![
                        ("p", n),
                        ("n", length_width),
                        ("base", cells * n),
                        ("size", cells * n),
                        ("live", cells),
                        ("pinned", cells),
                        ("atop", t),
                        (target, 1),
                    ];
                    if row {
                        bundles.extend([("faulted", 1), ("code", 4)]);
                    }
                    let f_arg = if row { ",4" } else { "" };
                    let args=json::parse(&format!(r#"[{n},{length_width},{},{cells},{t}{f_arg},"p","n","base","size","live","pinned","atop","{target}"{}]"#,n.max(length_width)+1,if row{",\"faulted\",\"code\""}else{""})).unwrap();
                    let m = Machine::compile(
                        &permutation(&e.expand(op, args.as_arr().unwrap(), &bundles).unwrap())
                            .unwrap(),
                        &bundles,
                    )
                    .unwrap();
                    let mut b = Bdd::new();
                    let mut s = m.symbolic_state();
                    for name in ["p", "n", "atop"] {
                        for id in &m.inputs[name] {
                            s[*id] = b.variable();
                        }
                    }
                    for entry in 0..cells {
                        for name in ["base", "size"] {
                            for bit in 0..n {
                                s[m.inputs[name][entry * n + bit]] = b.variable();
                            }
                        }
                        for name in ["live", "pinned"] {
                            s[m.inputs[name][entry]] = b.variable();
                        }
                    }
                    if !row {
                        s[m.inputs[target][0]] = b.variable();
                    } else {
                        for name in ["faulted", "code"] {
                            for id in &m.inputs[name] {
                                s[*id] = b.variable();
                            }
                        }
                    }
                    let enabled = b.variable();
                    let old = s.clone();
                    let mut expected = old.clone();
                    let atop = m.inputs["atop"].iter().map(|i| old[*i]).collect::<Vec<_>>();
                    let invalid = b.less(&Bdd::constant(cells, t), &atop);
                    let domain = b.not(invalid);
                    let p = m.inputs["p"].iter().map(|i| old[*i]).collect::<Vec<_>>();
                    let len = m.inputs["n"].iter().map(|i| old[*i]).collect::<Vec<_>>();
                    let mut px = p.clone();
                    px.resize(n.max(length_width) + 1, 0);
                    let mut nx = len;
                    nx.resize(n.max(length_width) + 1, 0);
                    let endp = b.add(&px, &nx);
                    let mut valid = 0;
                    for entry in 0..cells {
                        let base = m.inputs["base"][entry * n..(entry + 1) * n]
                            .iter()
                            .map(|i| old[*i])
                            .collect::<Vec<_>>();
                        let size = m.inputs["size"][entry * n..(entry + 1) * n]
                            .iter()
                            .map(|i| old[*i])
                            .collect::<Vec<_>>();
                        let mut pred = b.less(&Bdd::constant(entry, t), &atop);
                        pred = b.and(pred, old[m.inputs["live"][entry]]);
                        if op == "scan.free.flag" || op == "lift.free" {
                            let eq = b.equal(&p, &base);
                            pred = b.and(pred, eq);
                            let unpinned = b.not(old[m.inputs["pinned"][entry]]);
                            pred = b.and(pred, unpinned);
                        } else {
                            let below = b.less(&p, &base);
                            let ge = b.not(below);
                            pred = b.and(pred, ge);
                            let mut bx = base;
                            bx.resize(n.max(length_width) + 1, 0);
                            let mut sx = size;
                            sx.resize(n.max(length_width) + 1, 0);
                            let endb = b.add(&bx, &sx);
                            let above = b.less(&endb, &endp);
                            let le = b.not(above);
                            pred = b.and(pred, le);
                        }
                        valid = b.or(valid, pred);
                    }
                    let result = if op == "scan.inrange" {
                        valid
                    } else {
                        b.not(valid)
                    };
                    let active = b.and(enabled, result);
                    expected[m.inputs[target][0]] = b.xor(expected[m.inputs[target][0]], active);
                    if row {
                        let id = m.inputs["faulted"][0];
                        expected[id] = b.xor(expected[id], active);
                        for id in &m.inputs["code"][..3] {
                            expected[*id] = b.xor(expected[*id], active);
                        }
                    }
                    if op == "lift.free" {
                        let success = b.not(result);
                        let success = b.and(enabled, success);
                        for entry in 0..cells {
                            let base = m.inputs["base"][entry * n..(entry + 1) * n]
                                .iter()
                                .map(|i| old[*i])
                                .collect::<Vec<_>>();
                            let eq = b.equal(&p, &base);
                            let active_entry = b.less(&Bdd::constant(entry, t), &atop);
                            let matched = b.and(eq, active_entry);
                            let unpinned = b.not(old[m.inputs["pinned"][entry]]);
                            let matched = b.and(matched, unpinned);
                            let flip = b.and(success, matched);
                            let id = m.inputs["live"][entry];
                            expected[id] = b.xor(expected[id], flip);
                        }
                    }
                    m.run_symbolic(&mut b, &mut s, enabled, false, domain)
                        .unwrap();
                    for (i, v) in s.iter().enumerate() {
                        assert!(
                            b.equivalent(*v, expected[i], domain),
                            "{op} n={n} C={cells} bit={i}"
                        );
                    }
                    m.run_symbolic(&mut b, &mut s, enabled, true, domain)
                        .unwrap();
                    for (i, v) in s.iter().enumerate() {
                        assert!(b.equivalent(*v, old[i], domain));
                    }
                    let exponent =
                        n + length_width + 2 * cells * n + 2 * cells + if row { 6 } else { 2 };
                    println!(
                    "{op} P={n} L={length_width} C={cells}: full {}*2^{exponent} tuples; {} BDD nodes",
                    cells + 1,
                    b.nodes()
                );
                }
            }
        }
    }
}
