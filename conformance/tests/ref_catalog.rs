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
type Tuple<'a> = (Vec<(&'a str, u64)>, bool, Vec<(&'a str, u64)>);
fn packed(m: &Machine, tuples: &[Tuple<'_>]) {
    let mut s = m.packed_state();
    let mut enable = 0;
    for (lane, (inputs, en, _)) in tuples.iter().enumerate() {
        for (n, v) in inputs {
            m.set_lane(&mut s, n, *v, lane);
        }
        if *en {
            enable |= 1 << lane;
        }
    }
    let before = s.clone();
    m.run_packed(&mut s, enable, false).unwrap();
    for (lane, (inputs, en, outputs)) in tuples.iter().enumerate() {
        for (n, v) in outputs {
            assert_eq!(m.get_lane(&s, n, lane), *v, "{n}: {inputs:?}, enabled={en}");
        }
    }
    m.run_packed(&mut s, enable, true).unwrap();
    assert_eq!(s, before, "packed ordinal reversal");
}
#[test]
fn other_arithmetic_full_domain() {
    let c = catalog();
    let mut count = 0u64;
    for n in 1..=4u32 {
        let size = 1u64 << n;
        let mask = size - 1;
        for op in [
            "and", "xor", "or", "mul", "shl", "lshr", "ashr", "udiv", "urem", "sdiv", "srem",
        ] {
            let div = op.ends_with("div") || op.ends_with("rem");
            let kind = if ["shl", "lshr", "ashr"].contains(&op) {
                format!("value.{op}.w{n}")
            } else {
                format!("value.{op}")
            };
            let args = if kind.contains(".w") {
                vec![]
            } else {
                vec![n as i64]
            };
            let mut bundles = vec![("a", n as usize), ("b", n as usize), ("t", n as usize)];
            if div {
                bundles.push(("f", 4));
            }
            let m = Machine::compile(&c.expand(&kind, &args).unwrap(), &bundles).unwrap();
            let mut batch = Vec::new();
            for enabled in [false, true] {
                for a in 0..size {
                    for b in 0..size {
                        for t in 0..size {
                            for f in 0..if div { 16 } else { 1 } {
                                let code = if div && b == 0 {
                                    5
                                } else if op.starts_with('s') && div && a == size / 2 && b == mask {
                                    6
                                } else {
                                    0
                                };
                                let v = if code != 0 {
                                    0
                                } else {
                                    match op {
                                        "and" => a & b,
                                        "xor" => a ^ b,
                                        "or" => a | b,
                                        "mul" => (a * b) & mask,
                                        "shl" => {
                                            if b >= n as u64 {
                                                0
                                            } else {
                                                (a << b) & mask
                                            }
                                        }
                                        "lshr" => {
                                            if b >= n as u64 {
                                                0
                                            } else {
                                                a >> b
                                            }
                                        }
                                        "ashr" => {
                                            if b >= n as u64 {
                                                0
                                            } else {
                                                (signed(a, n) >> b) as u64 & mask
                                            }
                                        }
                                        "udiv" => a / b,
                                        "urem" => a % b,
                                        "sdiv" => (signed(a, n) / signed(b, n)) as u64 & mask,
                                        "srem" => (signed(a, n) % signed(b, n)) as u64 & mask,
                                        _ => unreachable!(),
                                    }
                                };
                                let mut inputs = vec![("a", a), ("b", b), ("t", t)];
                                if div {
                                    inputs.push(("f", f));
                                }
                                let mut outputs = vec![
                                    ("a", a),
                                    ("b", b),
                                    ("t", t ^ if enabled { v } else { 0 }),
                                ];
                                if div {
                                    outputs.push(("f", f ^ if enabled { code } else { 0 }));
                                }
                                batch.push((inputs, enabled, outputs));
                                if batch.len() == 64 {
                                    packed(&m, &batch);
                                    batch.clear();
                                }
                                count += 1;
                            }
                        }
                    }
                }
            }
            if !batch.is_empty() {
                packed(&m, &batch);
            }
        }
    }
    println!("other arithmetic widths 1..4: {count} tuples");
}
#[test]
fn casts_and_select_full_domain() {
    let c = catalog();
    let mut count = 0;
    for n in 1..=4u32 {
        let size = 1u64 << n;
        let m = Machine::compile(
            &c.expand("value.select", &[n as i64]).unwrap(),
            &[
                ("a", n as usize),
                ("b", n as usize),
                ("p", 1),
                ("t", n as usize),
            ],
        )
        .unwrap();
        for en in [false, true] {
            for a in 0..size {
                for b in 0..size {
                    for p in 0..2 {
                        for t in 0..size {
                            check(
                                &m,
                                &[("a", a), ("b", b), ("p", p), ("t", t)],
                                en,
                                &[
                                    ("a", a),
                                    ("b", b),
                                    ("p", p),
                                    (
                                        "t",
                                        t ^ if en {
                                            if p == 1 {
                                                a
                                            } else {
                                                b
                                            }
                                        } else {
                                            0
                                        },
                                    ),
                                ],
                            );
                            count += 1;
                        }
                    }
                }
            }
        }
        for dest in 1..=4u32 {
            for op in if dest >= n {
                vec!["zext", "sext"]
            } else {
                vec!["trunc"]
            } {
                let m = Machine::compile(
                    &c.expand(&format!("value.{op}"), &[n as i64, dest as i64])
                        .unwrap(),
                    &[("a", n as usize), ("t", dest as usize)],
                )
                .unwrap();
                for en in [false, true] {
                    for a in 0..size {
                        for t in 0..1 << dest {
                            let v = if op == "sext" { signed(a, n) as u64 } else { a }
                                & ((1 << dest) - 1);
                            check(
                                &m,
                                &[("a", a), ("t", t)],
                                en,
                                &[("a", a), ("t", t ^ if en { v } else { 0 })],
                            );
                            count += 1;
                        }
                    }
                }
            }
        }
    }
    println!("casts and select widths 1..4: {count} tuples");
}
#[test]
fn independent_canonical_bytes_and_envelope_hashes() {
    use bennett_isa_conformance::{
        json,
        ref_hash::{canonical, envelope_hash, sha256},
    };
    assert_eq!(
        sha256(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    assert_eq!(
        sha256(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    let draft = include_str!("../../spec/Bennett-ISA-Spec-0.9-draft.md");
    let witness = draft
        .split("**ISA-REF-LIB witness**")
        .nth(1)
        .unwrap()
        .split("```json\n")
        .nth(1)
        .unwrap()
        .split("\n```")
        .next()
        .unwrap();
    assert_eq!(canonical(&json::parse(witness).unwrap()).unwrap(), witness);
    assert_eq!(
        envelope_hash(witness),
        "299510fd67cc626fdcbcc10adcd40730f9af294ba596d86fb25fd31be08a5c7f"
    );
    assert_eq!(canonical(&json::parse(BYTES).unwrap()).unwrap(), BYTES);
    let output = std::process::Command::new("python3")
        .args([
            "-I",
            concat!(env!("CARGO_MANIFEST_DIR"), "/tools/ref_catalog.py"),
            "--families",
            "5",
            "--check",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let hash = envelope_hash(BYTES);
    assert!(String::from_utf8(output.stdout).unwrap().contains(&hash));
    println!("Rust and Python CHECKPOINT ONLY envelope SHA-256 {hash}");
}
#[test]
fn linear_scan_full_domain() {
    let c = catalog();
    let mut counts = [0u64; 2];
    for entries in 1..=2u32 {
        for n in 1..=if entries == 1 { 4 } else { 2 } {
            let size = 1u64 << n;
            let tag = if entries == 1 {
                "e1".into()
            } else {
                format!("e2.w{n}")
            };
            let args = if entries == 1 { vec![n as i64] } else { vec![] };
            for (oi, op) in ["inrange", "free.flag"].iter().enumerate() {
                let mut bundles = vec![
                    ("p", n as usize),
                    ("base", (n * entries) as usize),
                    ("live", entries as usize),
                    ("atop", if entries == 1 { 1 } else { 2 }),
                    ("t", 1),
                ];
                if oi == 0 {
                    bundles.extend([("n", n as usize), ("size", (n * entries) as usize)]);
                } else {
                    bundles.push(("pinned", entries as usize));
                }
                let m = Machine::compile(
                    &c.expand(&format!("value.{op}.{tag}"), &args).unwrap(),
                    &bundles,
                )
                .unwrap();
                let mut batch = Vec::new();
                for p in 0..size {
                    for base in 0..1 << (n * entries) {
                        for data in 0..if oi == 0 {
                            1 << (n * entries)
                        } else {
                            1 << entries
                        } {
                            for len in 0..if oi == 0 { size } else { 1 } {
                                for live in 0..1 << entries {
                                    for atop in 0..=entries {
                                        for en in [false, true] {
                                            for t in 0..2 {
                                                let mut valid = false;
                                                for i in 0..atop {
                                                    let b = (base >> (i * n)) & (size - 1);
                                                    valid |= live >> i & 1 != 0
                                                        && if oi == 0 {
                                                            b <= p
                                                                && p + len
                                                                    <= b + ((data >> (i * n))
                                                                        & (size - 1))
                                                        } else {
                                                            b == p && data >> i & 1 == 0
                                                        };
                                                }
                                                let mut inputs = vec![
                                                    ("p", p),
                                                    ("base", base),
                                                    ("live", live),
                                                    ("atop", atop as u64),
                                                    ("t", t),
                                                ];
                                                if oi == 0 {
                                                    inputs.extend([("n", len), ("size", data)]);
                                                } else {
                                                    inputs.push(("pinned", data));
                                                }
                                                let mut outputs = inputs.clone();
                                                outputs
                                                    .iter_mut()
                                                    .find(|(k, _)| *k == "t")
                                                    .unwrap()
                                                    .1 = t ^ u64::from(
                                                    en && (if oi == 0 { valid } else { !valid }),
                                                );
                                                batch.push((inputs, en, outputs));
                                                if batch.len() == 64 {
                                                    packed(&m, &batch);
                                                    batch.clear();
                                                }
                                                counts[oi] += 1;
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if !batch.is_empty() {
                    packed(&m, &batch);
                }
            }
        }
    }
    println!(
        "linear scans: inrange {} and free flag {} full-domain tuples",
        counts[0], counts[1]
    );
}
#[test]
fn allocation_and_call_flags_full_domain() {
    let c = catalog();
    let mut counts = [0u64; 2];
    for n in 1..=4u32 {
        let size = 1u64 << n;
        for (oi, op) in ["alloc", "call"].iter().enumerate() {
            let m = if oi == 0 {
                Machine::compile(
                    &c.expand("value.alloc.flag", &[n as i64, 1]).unwrap(),
                    &[
                        ("n", n as usize),
                        ("used", n as usize),
                        ("limit", n as usize + 1),
                        ("atop", 2),
                        ("t", 1),
                    ],
                )
                .unwrap()
            } else {
                Machine::compile(
                    &c.expand(&format!("value.call.flag.rf4.w{n}"), &[]).unwrap(),
                    &[
                        ("depth", n as usize),
                        ("depth_limit", n as usize + 1),
                        ("sp", n as usize),
                        ("stack_limit", n.max(3) as usize + 1),
                        ("t", 1),
                    ],
                )
                .unwrap()
            };
            let mut batch = Vec::new();
            for a in 0..size {
                for b in 0..size {
                    for lim in 0..size * 2 {
                        for other in 0..if oi == 0 { 4 } else { 1 << (n.max(3) + 1) } {
                            for en in [false, true] {
                                for t in 0..2 {
                                    let inputs = if oi == 0 {
                                        vec![
                                            ("n", a),
                                            ("used", b),
                                            ("limit", lim),
                                            ("atop", other),
                                            ("t", t),
                                        ]
                                    } else {
                                        vec![
                                            ("depth", a),
                                            ("sp", b),
                                            ("depth_limit", lim),
                                            ("stack_limit", other),
                                            ("t", t),
                                        ]
                                    };
                                    let flag = if oi == 0 {
                                        b + a.max(1) > lim || other >= 2
                                    } else {
                                        a + 1 > lim || b + 4 > other
                                    };
                                    let mut outputs = inputs.clone();
                                    outputs.iter_mut().find(|(k, _)| *k == "t").unwrap().1 =
                                        t ^ u64::from(en && flag);
                                    batch.push((inputs, en, outputs));
                                    if batch.len() == 64 {
                                        packed(&m, &batch);
                                        batch.clear();
                                    }
                                    counts[oi] += 1;
                                }
                            }
                        }
                    }
                }
            }
            if !batch.is_empty() {
                packed(&m, &batch);
            }
            println!("{op} flag width {n} done");
        }
    }
    println!(
        "allocator {} and call {} full-domain flag tuples",
        counts[0], counts[1]
    );
}
#[test]
fn address_dispatch_full_domain() {
    let c = catalog();
    let mut count = 0;
    for n in 1..=4u32 {
        let size = 1u64 << n;
        let mask = size - 1;
        for op in ["copy", "store", "exchange"] {
            let m = Machine::compile(
                &c.expand(&format!("dispatch.{op}.a1"), &[n as i64]).unwrap(),
                &[("a", 1), ("cells", 2 * n as usize), ("v", n as usize)],
            )
            .unwrap();
            let mut batch = Vec::new();
            for a in 0..2 {
                for cells in 0..size * size {
                    for v in 0..size {
                        for en in [false, true] {
                            let old = (cells >> (a * n)) & mask;
                            let (next, result) = if en {
                                match op {
                                    "copy" => (cells, v ^ old),
                                    "store" => (cells ^ (v << (a * n)), v),
                                    "exchange" => {
                                        ((cells & !(mask << (a * n))) | (v << (a * n)), old)
                                    }
                                    _ => unreachable!(),
                                }
                            } else {
                                (cells, v)
                            };
                            batch.push((
                                vec![("a", a as u64), ("cells", cells), ("v", v)],
                                en,
                                vec![("a", a as u64), ("cells", next), ("v", result)],
                            ));
                            if batch.len() == 64 {
                                packed(&m, &batch);
                                batch.clear();
                            }
                            count += 1;
                        }
                    }
                }
            }
            if !batch.is_empty() {
                packed(&m, &batch);
            }
        }
    }
    assert_eq!(count, 56160);
    println!("two-cell address dispatch widths 1..4: {count} tuples");
}
