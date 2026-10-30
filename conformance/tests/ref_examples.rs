//! Exact component witnesses at the normative §16.3 object widths. These do
//! not certify the still-blocked heterogeneous step/activation construction.
use bennett_isa_conformance::{
    json,
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
fn machine(e: &mut Expander, k: &str, args: &str, bundles: &[(&str, usize)]) -> Machine {
    let args = json::parse(args).unwrap();
    Machine::compile(
        &permutation(&e.expand(k, args.as_arr().unwrap(), bundles).unwrap()).unwrap(),
        bundles,
    )
    .unwrap()
}
fn variables(m: &Machine, b: &mut Bdd, names: &[&str]) -> Vec<usize> {
    let mut state = m.symbolic_state();
    for name in names {
        for i in &m.inputs[*name] {
            state[*i] = b.variable();
        }
    }
    state
}
fn mux(b: &mut Bdd, c: usize, x: usize, y: usize) -> usize {
    let diff = b.xor(x, y);
    let diff = b.and(c, diff);
    b.xor(x, diff)
}
fn verify(m: &Machine, b: &mut Bdd, old: &[usize], expected: &[usize], en: usize) {
    let mut state = old.to_vec();
    m.run_symbolic(b, &mut state, en, false, 1).unwrap();
    assert_eq!(state, expected);
    m.run_symbolic(b, &mut state, en, true, 1).unwrap();
    assert_eq!(state, old);
}
#[test]
fn inc8_and_sum_db_arithmetic_cast_and_successor_widths() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    for (op, n) in [("add", 8), ("sub", 8), ("add", 32), ("icmp.eq", 8)] {
        let width = if op == "icmp.eq" { 1 } else { n };
        let bundles = [("a", n), ("b", n), ("slot", width), ("w", width)];
        let m = machine(
            &mut e,
            &format!("lift.{op}"),
            &format!(r#"[{n},"a","b","slot","w"]"#),
            &bundles,
        );
        let mut b = Bdd::new();
        let mut old = m.symbolic_state();
        for bit in 0..n {
            for name in ["a", "b"] {
                old[m.inputs[name][bit]] = b.variable();
            }
        }
        for id in &m.inputs["w"] {
            old[*id] = b.variable();
        }
        let en = b.variable();
        let a = m.inputs["a"].iter().map(|i| old[*i]).collect::<Vec<_>>();
        let rhs = m.inputs["b"].iter().map(|i| old[*i]).collect::<Vec<_>>();
        let value = if op == "icmp.eq" {
            vec![b.equal(&a, &rhs)]
        } else if op == "sub" {
            let neg = rhs.iter().map(|v| b.not(*v)).collect::<Vec<_>>();
            let neg = b.add(&neg, &Bdd::constant(1, n));
            b.add(&a, &neg)
        } else {
            b.add(&a, &rhs)
        };
        let mut expected = old.clone();
        for bit in 0..width {
            let w = m.inputs["w"][bit];
            expected[w] = mux(&mut b, en, old[w], value[bit]);
            expected[m.inputs["slot"][bit]] = b.and(en, old[w]);
        }
        verify(&m, &mut b, &old, &expected, en);
        println!("§16.3 lift.{op} N={n}: full 2^{} tuples", 2 * n + width + 1);
    }
    let m = machine(
        &mut e,
        "lift.zext",
        r#"[8,32,"a","slot","w"]"#,
        &[("a", 8), ("slot", 32), ("w", 32)],
    );
    let mut b = Bdd::new();
    let old = variables(&m, &mut b, &["a", "w"]);
    let en = b.variable();
    let mut expected = old.clone();
    for bit in 0..32 {
        let w = m.inputs["w"][bit];
        let value = if bit < 8 { old[m.inputs["a"][bit]] } else { 0 };
        expected[w] = mux(&mut b, en, old[w], value);
        expected[m.inputs["slot"][bit]] = b.and(en, old[w]);
    }
    verify(&m, &mut b, &old, &expected, en);
    println!("§16.3 lift.zext 8->32: full 2^41 tuples");
    let m = machine(
        &mut e,
        "lift.retval",
        r#"[8,"a","t"]"#,
        &[("a", 8), ("t", 8)],
    );
    let mut b = Bdd::new();
    let old = variables(&m, &mut b, &["a", "t"]);
    let en = b.variable();
    let mut expected = old.clone();
    for bit in 0..8 {
        let flip = b.and(en, old[m.inputs["a"][bit]]);
        let t = m.inputs["t"][bit];
        expected[t] = b.xor(expected[t], flip);
    }
    verify(&m, &mut b, &old, &expected, en);
    println!("§16.3 lift.retval N=8: full 2^17 tuples");
    let m = machine(
        &mut e,
        "qbranch",
        r#"[5,18,9,"p","succ"]"#,
        &[("p", 1), ("succ", 5)],
    );
    let mut b = Bdd::new();
    let old = variables(&m, &mut b, &["p", "succ"]);
    let en = b.variable();
    let p = old[m.inputs["p"][0]];
    let notp = b.not(p);
    let mut expected = old.clone();
    for bit in 0..5 {
        let then = if 18 >> bit & 1 == 1 { p } else { 0 };
        let otherwise = if 9 >> bit & 1 == 1 { notp } else { 0 };
        let value = b.or(then, otherwise);
        let flip = b.and(en, value);
        let id = m.inputs["succ"][bit];
        expected[id] = b.xor(expected[id], flip);
    }
    verify(&m, &mut b, &old, &expected, en);
    println!("§16.3 qbranch P=5, labels18/9: full 2^7 tuples");
}
#[test]
fn example_literal_pointer_bases_and_byte_constants() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    for (kind, n, c) in [
        ("lift.addr", 32, 16),
        ("lift.addr", 32, 4096),
        ("lift.const", 8, 0),
        ("lift.const", 8, 1),
    ] {
        let m = machine(
            &mut e,
            kind,
            &format!(r#"[{n},{c},"slot","w"]"#),
            &[("slot", n), ("w", n)],
        );
        let mut b = Bdd::new();
        let old = variables(&m, &mut b, &["w"]);
        let en = b.variable();
        let mut expected = old.clone();
        for bit in 0..n {
            let id = m.inputs["w"][bit];
            expected[id] = mux(&mut b, en, old[id], (c >> bit) & 1);
            expected[m.inputs["slot"][bit]] = b.and(en, old[id]);
        }
        verify(&m, &mut b, &old, &expected, en);
        println!("§16.3 {kind} width{n} C={c}: full 2^{} tuples", n + 1);
    }
}
#[test]
fn example_byte_region_dispatch_full_domain() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let bundles = [("address", 4), ("cells", 128), ("value", 8)];
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
            s => s.strip_prefix("dispatch.").unwrap(),
        };
        let m = machine(
            &mut e,
            kind,
            r#"[4,16,8,"address","cells","value"]"#,
            &bundles,
        );
        let mut b = Bdd::new();
        let old = variables(&m, &mut b, &["address", "cells", "value"]);
        let en = b.variable();
        let address = m.inputs["address"]
            .iter()
            .map(|i| old[*i])
            .collect::<Vec<_>>();
        let mut expected = old.clone();
        for cell in 0..16 {
            let at = b.equal(&address, &Bdd::constant(cell, 4));
            let active = b.and(en, at);
            for bit in 0..8 {
                let src = m.inputs["cells"][cell * 8 + bit];
                let val = m.inputs["value"][bit];
                if op == "exchange" {
                    expected[src] = mux(&mut b, active, old[src], old[val]);
                    let d = b.xor(old[src], old[val]);
                    let flip = b.and(active, d);
                    expected[val] = b.xor(expected[val], flip);
                } else if op == "copy" {
                    let flip = b.and(active, old[src]);
                    expected[val] = b.xor(expected[val], flip);
                } else {
                    let flip = b.and(active, old[val]);
                    expected[src] = b.xor(expected[src], flip);
                }
            }
        }
        verify(&m, &mut b, &old, &expected, en);
        println!("§16.3 byte-region shape {kind} A=4 W=8: full 2^141 tuples");
    }
}
#[test]
fn example_pinned_layout_full_pointer_domain() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let bundles = [
        ("p", 32),
        ("n", 1),
        ("base", 512),
        ("size", 512),
        ("live", 16),
        ("pinned", 16),
        ("atop", 5),
        ("flag", 1),
        ("faulted", 1),
        ("code", 4),
    ];
    let m = machine(
        &mut e,
        "lift.boundscheck",
        r#"[32,1,33,16,5,4,"p","n","base","size","live","pinned","atop","flag","faulted","code"]"#,
        &bundles,
    );
    let mut b = Bdd::new();
    let mut old = variables(&m, &mut b, &["p", "faulted", "code"]);
    old[m.inputs["n"][0]] = 1;
    old[m.inputs["atop"][1]] = 1;
    for (entry, base, size) in [(0, 16usize, 1usize), (1, 4096, 16)] {
        for bit in 0..32 {
            old[m.inputs["base"][entry * 32 + bit]] = (base >> bit) & 1;
            old[m.inputs["size"][entry * 32 + bit]] = (size >> bit) & 1;
        }
        old[m.inputs["live"][entry]] = 1;
        old[m.inputs["pinned"][entry]] = 1;
    }
    let en = b.variable();
    let p = m.inputs["p"].iter().map(|i| old[*i]).collect::<Vec<_>>();
    let zero = b.equal(&p, &Bdd::constant(16, 32));
    let below = b.less(&p, &Bdd::constant(4096, 32));
    let above = b.less(&Bdd::constant(4111, 32), &p);
    let outside = b.or(below, above);
    let db = b.not(outside);
    let valid = b.or(zero, db);
    let invalid = b.not(valid);
    let failed = b.and(en, invalid);
    let mut expected = old.clone();
    expected[m.inputs["flag"][0]] = failed;
    let id = m.inputs["faulted"][0];
    expected[id] = b.xor(expected[id], failed);
    for id in &m.inputs["code"][..3] {
        expected[*id] = b.xor(expected[*id], failed);
    }
    verify(&m, &mut b, &old, &expected, en);
    println!("§16.3 pinned layout p32, length1, 16 entries: full 2^38 tuples");
}
