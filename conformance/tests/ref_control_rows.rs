use bennett_isa_conformance::{
    json,
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
fn machine(e: &mut Expander, kind: &str, args: &str, bundles: &[(&str, usize)]) -> Machine {
    let args = json::parse(args).unwrap();
    Machine::compile(
        &permutation(&e.expand(kind, args.as_arr().unwrap(), bundles).unwrap()).unwrap(),
        bundles,
    )
    .unwrap()
}
fn variables(m: &Machine, b: &mut Bdd, names: &[&str]) -> Vec<usize> {
    let mut s = m.symbolic_state();
    for name in names {
        for id in &m.inputs[*name] {
            s[*id] = b.variable();
        }
    }
    s
}
fn verify(m: &Machine, b: &mut Bdd, old: &[usize], expected: &[usize], en: usize) {
    let mut s = old.to_vec();
    m.run_symbolic(b, &mut s, en, false, 1).unwrap();
    assert_eq!(s, expected);
    m.run_symbolic(b, &mut s, en, true, 1).unwrap();
    assert_eq!(s, old);
}
#[test]
fn canonical_branches_and_successor_fragments() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut count = 0;
    for p in 1..=4 {
        let size = 1 << p;
        for a in 0..size {
            for c in 0..size {
                let m = machine(
                    &mut e,
                    "qbranch",
                    &format!(r#"[{p},{a},{c},"p","next"]"#),
                    &[("p", 1), ("next", p)],
                );
                let mut b = Bdd::new();
                let old = variables(&m, &mut b, &["p", "next"]);
                let en = b.variable();
                let mut expected = old.clone();
                let pred = old[m.inputs["p"][0]];
                let no = b.not(pred);
                for i in 0..p {
                    let bit = b.or(
                        if a >> i & 1 == 1 { pred } else { 0 },
                        if c >> i & 1 == 1 { no } else { 0 },
                    );
                    let flip = b.and(en, bit);
                    let id = m.inputs["next"][i];
                    expected[id] = b.xor(expected[id], flip);
                }
                verify(&m, &mut b, &old, &expected, en);
                count += 1u64 << (p + 2);
            }
        }
        for c in 0..size {
            let args = json::parse(&format!("[{c}]")).unwrap();
            assert!(e
                .expand("qjump", args.as_arr().unwrap(), &[])
                .unwrap()
                .is_empty());
            let m = machine(
                &mut e,
                "successor.literal",
                &format!(r#"[{p},{c},"next"]"#),
                &[("next", p)],
            );
            let mut b = Bdd::new();
            let old = variables(&m, &mut b, &["next"]);
            let en = b.variable();
            let mut expected = old.clone();
            for i in 0..p {
                if c >> i & 1 == 1 {
                    let id = m.inputs["next"][i];
                    expected[id] = b.xor(expected[id], en);
                }
            }
            verify(&m, &mut b, &old, &expected, en);
            count += 1 << (p + 1);
            let m = machine(
                &mut e,
                "successor.flagged",
                &format!(r#"[{p},{c},{},"flag","next"]"#, size - 1),
                &[("flag", 1), ("next", p)],
            );
            let mut b = Bdd::new();
            let old = variables(&m, &mut b, &["flag", "next"]);
            let en = b.variable();
            let flag = old[m.inputs["flag"][0]];
            let mut expected = old.clone();
            for i in 0..p {
                let bit = if c >> i & 1 == 1 { 1 } else { flag };
                let flip = b.and(en, bit);
                let id = m.inputs["next"][i];
                expected[id] = b.xor(expected[id], flip);
            }
            verify(&m, &mut b, &old, &expected, en);
            count += 1 << (p + 2);
        }
        let m = machine(
            &mut e,
            "successor.dynamic",
            &format!(r#"[{p},{},"flag","succ","next"]"#, size - 1),
            &[("flag", 1), ("succ", p), ("next", p)],
        );
        let mut b = Bdd::new();
        let old = variables(&m, &mut b, &["flag", "succ", "next"]);
        let en = b.variable();
        let flag = old[m.inputs["flag"][0]];
        let mut expected = old.clone();
        for i in 0..p {
            let bit = b.or(flag, old[m.inputs["succ"][i]]);
            let flip = b.and(en, bit);
            let id = m.inputs["next"][i];
            expected[id] = b.xor(expected[id], flip);
        }
        verify(&m, &mut b, &old, &expected, en);
        count += 1 << (2 * p + 2);
    }
    assert_eq!(count, 22120);
    println!("branch/successor fragments: {count} tuples");
}
#[test]
fn function_local_indirect_jump_intervals() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut count = 0;
    for p in 1..=4 {
        let halt = (1 << p) - 1;
        for first in 0..halt {
            for end in first + 1..=halt {
                let bundles = [
                    ("a", p),
                    ("succ", p),
                    ("flag", 1),
                    ("faulted", 1),
                    ("code", 4),
                ];
                let m = machine(
                    &mut e,
                    "qjumpr",
                    &format!(r#"[{p},4,{first},{end},"a","succ","flag","faulted","code"]"#),
                    &bundles,
                );
                let mut b = Bdd::new();
                let old = variables(&m, &mut b, &["a", "faulted", "code"]);
                let en = b.variable();
                let mut expected = old.clone();
                let a = m.inputs["a"].iter().map(|i| old[*i]).collect::<Vec<_>>();
                let below = b.less(&a, &Bdd::constant(first, p));
                let lessend = b.less(&a, &Bdd::constant(end, p));
                let above = b.not(lessend);
                let invalid = b.or(below, above);
                let active = b.and(en, invalid);
                expected[m.inputs["flag"][0]] = active;
                let id = m.inputs["faulted"][0];
                expected[id] = b.xor(expected[id], active);
                let id = m.inputs["code"][2];
                expected[id] = b.xor(expected[id], active);
                for (bit, id) in m.inputs["succ"].iter().enumerate() {
                    expected[*id] = b.and(en, a[bit]);
                }
                verify(&m, &mut b, &old, &expected, en);
                count += 1 << (p + 6);
            }
        }
    }
    assert_eq!(count, 138880);
    println!("qjumpr function intervals: {count} tuples");
}
#[test]
fn effective_causes_and_wide_code_registers() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut count = 0;
    for f in [4, 5, 8] {
        for c in [0, 1, 10, (1 << f) - 1] {
            for kind in ["fault", "lift.faultif"] {
                let bundles = [("p", 1), ("flag", 1), ("faulted", 1), ("code", f)];
                let args = if kind == "fault" {
                    format!(r#"[{f},{c},"faulted","code"]"#)
                } else {
                    format!(r#"[{f},{c},"p","flag","faulted","code"]"#)
                };
                let m = machine(&mut e, kind, &args, &bundles);
                let mut b = Bdd::new();
                let old = variables(
                    &m,
                    &mut b,
                    if kind == "fault" {
                        &["faulted", "code"]
                    } else {
                        &["p", "faulted", "code"]
                    },
                );
                let en = b.variable();
                let active = if kind == "fault" {
                    en
                } else {
                    b.and(en, old[m.inputs["p"][0]])
                };
                let mut expected = old.clone();
                if kind != "fault" {
                    expected[m.inputs["flag"][0]] = active;
                }
                let id = m.inputs["faulted"][0];
                expected[id] = b.xor(expected[id], active);
                let cause = if c == 0 { 4 } else { c };
                for bit in 0..f {
                    if cause >> bit & 1 == 1 {
                        let id = m.inputs["code"][bit];
                        expected[id] = b.xor(expected[id], active);
                    }
                }
                verify(&m, &mut b, &old, &expected, en);
                count += 1 << (f + 2 + usize::from(kind != "fault"));
            }
        }
    }
    assert_eq!(count, 14592);
    println!("wide effective causes: {count} tuples");
}
