//! Independent draft catalog checks; no interpreter source dependency.
use bennett_isa_conformance::ref_catalog::{Catalog, Machine};
const BYTES: &str = include_str!("../../spec/ref-0.9-draft.json");
fn catalog() -> Catalog {
    Catalog::parse(BYTES).unwrap()
}
#[test]
fn catalog_checkpoint_exists_and_is_canonical() {
    assert_eq!(BYTES.as_bytes().last(), Some(&b'}'));
    assert!(!catalog().kinds().is_empty());
}
fn check(m: &Machine, values: &[(&str, u64)], enabled: bool, expected: &[(&str, u64)]) {
    let mut s = m.state();
    for (n, v) in values {
        m.set(&mut s, n, *v);
    }
    let before = s.clone();
    m.run(&mut s, enabled, false).unwrap();
    for (n, v) in expected {
        assert_eq!(m.get(&s, n), *v, "{n}: {values:?}, enable={enabled}");
    }
    m.run(&mut s, enabled, true).unwrap();
    assert_eq!(s, before, "ordinal reversal");
}
#[test]
fn copy_exchange_full_domain() {
    let c = catalog();
    let mut count = 0;
    for n in 1..=4 {
        for kind in ["qxor", "qexch", "qcopy", "quncopy"] {
            let leaves = c.expand(kind, &[n]).unwrap();
            assert!(leaves.iter().enumerate().all(|(i, l)| i == l.ordinal));
            let bundles = match kind {
                "qcopy" => vec![("s", n as usize)],
                "qexch" => vec![("a", n as usize), ("b", n as usize)],
                _ => vec![("s", n as usize), ("t", n as usize)],
            };
            let m = Machine::compile(&leaves, &bundles).unwrap();
            for enabled in [false, true] {
                for a in 0..1 << n {
                    if kind == "qcopy" {
                        check(
                            &m,
                            &[("s", a)],
                            enabled,
                            &[("s", a), ("t", if enabled { a } else { 0 })],
                        );
                        count += 1;
                    } else if kind == "quncopy" {
                        check(
                            &m,
                            &[("s", a), ("t", if enabled { a } else { 0 })],
                            enabled,
                            &[("s", a), ("t", 0)],
                        );
                        count += 1;
                    } else {
                        for b in 0..1 << n {
                            if kind == "qexch" {
                                check(
                                    &m,
                                    &[("a", a), ("b", b)],
                                    enabled,
                                    &[
                                        ("a", if enabled { b } else { a }),
                                        ("b", if enabled { a } else { b }),
                                    ],
                                );
                            } else {
                                check(
                                    &m,
                                    &[("s", a), ("t", b)],
                                    enabled,
                                    &[("s", a), ("t", b ^ if enabled { a } else { 0 })],
                                );
                            }
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(count, 1480);
    println!("copy/exchange widths 1..4: {count} tuples including both controls");
}
#[test]
fn native_permutations_full_domain() {
    let c = catalog();
    let mut count = 0;
    for (kind, n) in [("x", 1), ("cx", 2), ("ccx", 3), ("swap", 2)] {
        let leaves = c.expand(&format!("gate.{kind}"), &[]).unwrap();
        let bundles = if n == 3 {
            vec![("a", 1), ("b", 1), ("t", 1)]
        } else if n == 2 {
            vec![("a", 1), ("b", 1)]
        } else {
            vec![("a", 1)]
        };
        let m = Machine::compile(&leaves, &bundles).unwrap();
        for enabled in [false, true] {
            for v in 0..1u64 << n {
                let values: Vec<_> = bundles
                    .iter()
                    .enumerate()
                    .map(|(i, (n, _))| (*n, (v >> i) & 1))
                    .collect();
                let mut w = v;
                if enabled {
                    match kind {
                        "x" => w ^= 1,
                        "cx" => w ^= (v & 1) << 1,
                        "ccx" => w ^= ((v & 1) & ((v >> 1) & 1)) << 2,
                        "swap" => w = ((v & 1) << 1) | ((v >> 1) & 1),
                        _ => unreachable!(),
                    }
                }
                let expected: Vec<_> = bundles
                    .iter()
                    .enumerate()
                    .map(|(i, (n, _))| (*n, (w >> i) & 1))
                    .collect();
                check(&m, &values, enabled, &expected);
                count += 1;
            }
        }
    }
    assert_eq!(count, 36);
    println!("native fixed permutation gates: {count} tuples");
}
#[test]
fn expander_rejects_unknown_constructors_and_unbound_parameters() {
    assert!(catalog().expand("qxor", &[]).is_err());
    assert!(catalog().expand("qxor", &[-1]).is_err());
    let bad = BYTES.replace("\"gate\":\"cx\"", "\"private_gate\":\"cx\"");
    assert!(Catalog::parse(&bad).unwrap().expand("qxor", &[1]).is_err());
}
fn signed(v: u64, n: u32) -> i64 {
    ((v << (64 - n)) as i64) >> (64 - n)
}
#[test]
fn ripple_full_domain() {
    let c = catalog();
    let mut count = 0u64;
    for n in 1..=4u32 {
        let size = 1u64 << n;
        let mask = size - 1;
        for op in [
            "add", "sub", "icmp.eq", "icmp.ne", "icmp.ult", "icmp.ule", "icmp.slt", "icmp.sle",
        ] {
            let tw = if op.starts_with("icmp") {
                1
            } else {
                n as usize
            };
            let m = Machine::compile(
                &c.expand(&format!("value.{op}"), &[n as i64]).unwrap(),
                &[("a", n as usize), ("b", n as usize), ("t", tw)],
            )
            .unwrap();
            for enabled in [false, true] {
                for a in 0..size {
                    for b in 0..size {
                        for t in 0..1 << tw {
                            let result = match op {
                                "add" => (a + b) & mask,
                                "sub" => a.wrapping_sub(b) & mask,
                                "icmp.eq" => u64::from(a == b),
                                "icmp.ne" => u64::from(a != b),
                                "icmp.ult" => u64::from(a < b),
                                "icmp.ule" => u64::from(a <= b),
                                "icmp.slt" => u64::from(signed(a, n) < signed(b, n)),
                                "icmp.sle" => u64::from(signed(a, n) <= signed(b, n)),
                                _ => unreachable!(),
                            };
                            check(
                                &m,
                                &[("a", a), ("b", b), ("t", t)],
                                enabled,
                                &[
                                    ("a", a),
                                    ("b", b),
                                    ("t", t ^ if enabled { result } else { 0 }),
                                ],
                            );
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(count, 26880);
    println!("ripple widths 1..4: {count} tuples");
}
