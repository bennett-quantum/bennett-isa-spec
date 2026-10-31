//! R20 recurring arithmetic/allocation class. Expected outcomes are assigned
//! before production is called. Text has canonical unsigned INT spelling;
//! metadata has number <= 2^53-1 and decimal string otherwise (§9.1).
use super::support;
use bennett_isa_ref::{Object, Result};
use std::panic::{catch_unwind, UnwindSafe};
const VALUES: [u64; 6] = [0, 1, (1 << 63) - 1, 1 << 63, u64::MAX - 1, u64::MAX];

#[derive(Clone, Copy, Debug)]
enum Expected {
    Admit,
    Range,
    Resource,
}
fn check<T>(label: &str, expected: Expected, run: impl FnOnce() -> Result<T> + UnwindSafe) {
    let result = catch_unwind(run).unwrap_or_else(|_| panic!("panic: {label}"));
    match (expected, result) {
        (Expected::Admit, Ok(_)) => (),
        (Expected::Range | Expected::Resource, Err(e)) => {
            let s = e.to_string();
            assert!(s.contains('§'), "{label}: {s}");
            assert_eq!(
                s.contains("host limit"),
                matches!(expected, Expected::Resource),
                "{label}: {s}"
            );
        }
        (expected, result) => panic!(
            "{label}: expected {expected:?}, got {}",
            result.err().map_or("admission".into(), |e| e.to_string())
        ),
    }
}
fn oracle(width: u64, literal: &str) -> String {
    support::object(&[("o", r#"{"abi":"0.8","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#,
       &format!("oracle @o (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()\nregs {{ r:reg[{width}] }}\n^entry():\nr = lift.const.i{width} {literal}\nlift.retval x"))], "{}", "")
}
fn action(width: u64, body: &str, bound: Option<u64>) -> String {
    let bounds = bound.map_or(String::new(), |n| {
        if n <= 9_007_199_254_740_991 {
            n.to_string()
        } else {
            format!("\"{n}\"")
        }
    });
    let meta_width = if width <= 9_007_199_254_740_991 {
        width.to_string()
    } else {
        format!("\"{width}\"")
    };
    let sig = format!(
        r#"{{"abi":"0.8","args":[],"bounds":[{bounds}],"in":[],"kind":"action","mem":[],"mut":[{meta_width}],"tier":"q"}}"#
    );
    let regs = bound.map_or(String::new(), |n| format!("regs {{  }} bounds {{{n}}}\n"));
    support::object(
        &[(
            "a",
            &sig,
            &format!("action @a (t:wire[{width}] mut) () () () ()\n{regs}^entry():\n{body}"),
        )],
        "{}",
        "",
    )
}
#[test]
fn r20_generated_text_width_count_index_extremes() {
    use Expected::*;
    let mut cases = 0;
    for n in VALUES {
        for literal in ["0", "1", "-1", "-2", "255", "256", "-128", "-129"] {
            let expected = if n == 0 || (n == 1 && !["0", "1", "-1"].contains(&literal)) {
                Range
            } else {
                Admit
            };
            check(&format!("const N={n} c={literal}"), expected, || {
                Object::parse(&oracle(n, literal))
            });
            cases += 1;
        }
        // A whole bundle invokes both disjointness expansion and borrow compiler.
        for (path, text, expected) in [
            (
                "action-wire",
                action(n, "x t\nx t", None),
                if n == 0 {
                    Range
                } else if n == 1 {
                    Admit
                } else {
                    Resource
                },
            ),
            (
                "qalloc",
                action(1, &format!("b = qalloc any {n}\nqfree b"), None),
                if n == 0 {
                    Range
                } else if n == 1 {
                    Admit
                } else {
                    Resource
                },
            ),
            (
                "qborrow",
                action(1, &format!("b = qborrow any {n}\nqreturn b"), None),
                if n == 0 {
                    Range
                } else if n == 1 {
                    Admit
                } else {
                    Resource
                },
            ),
            (
                "wire-index",
                action(1, &format!("x t[{n}]\nx t[{n}]"), None),
                if n == 0 { Admit } else { Range },
            ),
            (
                "qloop-bounds",
                action(1, &format!("qloop {n} {{\nx t\nx t\n}}"), Some(n)),
                Admit,
            ),
            (
                "qregion-A",
                action(
                    1,
                    &format!("qregion r {n} 1\nb = qborrow any 1\nqreturn b"),
                    None,
                ),
                if n <= 1 { Admit } else { Resource },
            ),
            (
                "qregion-W",
                action(
                    1,
                    &format!("qregion r 0 {n}\nb = qborrow any 1\nqreturn b"),
                    None,
                ),
                if n == 0 {
                    Range
                } else if n == 1 {
                    Admit
                } else {
                    Resource
                },
            ),
        ] {
            check(&format!("{path}={n}"), expected, || Object::parse(&text));
            cases += 1;
        }
    }
    assert_eq!(cases, 90);
    eprintln!("R20 text: 8 paths, {cases} cases");
}

/// Independent §7.1–7.2 range formulas, evaluated BEFORE Object::parse.
/// No production layout validator or DerivedWidths helper is called here.
pub(super) fn layout_expected(l: &bennett_isa_ref::json::Json, o: &Object) -> &'static str {
    use bennett_isa_ref::{json::Json, Type};
    fn n(v: &Json) -> u128 {
        match v {
            Json::Number(s) | Json::String(s) => s.parse().unwrap(),
            _ => panic!("scalar"),
        }
    }
    let val = |key| n(l.get(key).unwrap());
    let h = val("H");
    if val("ptr_width") != o.ptr_width as u128 || h > o.ptr_width as u128 || val("null_size") == 0 {
        return "layout";
    }
    let mut shared = vec![
        (0, val("null_size")),
        (val("heap_base"), val("heap_limit_max")),
    ];
    let globals = l.get("globals").unwrap().object().unwrap();
    let capacity = (h < 128).then(|| 1u128 << h);
    let entries = (val("A_tab") < 128).then(|| 1u128 << val("A_tab"));
    for (name, g) in globals {
        let size = n(g.get("size").unwrap());
        if o.data[name].len() as u128 != size {
            return "layout";
        }
        shared.push((n(g.get("base").unwrap()), size));
    }
    let valid = |ranges: &[(u128, u128)]| {
        let mut sorted = Vec::new();
        for &(base, size) in ranges {
            let end = base.checked_add(size);
            if end.is_none() && base.wrapping_add(size) != 0 {
                return false;
            }
            if capacity.is_some_and(|cap| end.is_none_or(|end| end > cap)) {
                return false;
            }
            if size > 0 {
                sorted.push((base, end));
            }
        }
        sorted.sort();
        sorted
            .windows(2)
            .all(|p| p[0].1.is_some_and(|end| end <= p[1].0))
    };
    if entries.is_some_and(|e| globals.len() as u128 > e) || !valid(&shared) {
        return "layout";
    }
    for (name, regions) in l.get("interface_regions").unwrap().object().unwrap() {
        let mut ranges = shared.clone();
        for p in &o.functions[name].params[3] {
            let Type::Region(a, _) = p.ty else {
                panic!("region")
            };
            if a >= 128 {
                return "layout";
            }
            ranges.push((
                n(regions.get(&p.name).unwrap().get("base").unwrap()),
                1u128 << a,
            ));
        }
        if entries
            .is_some_and(|e| globals.len() as u128 + o.functions[name].params[3].len() as u128 > e)
            || !valid(&ranges)
        {
            return "layout";
        }
    }
    let widths = [h + 2, val("A_tab") + 1, val("S_cells") + 1];
    if widths
        .iter()
        .any(|w| *w > usize::MAX as u128 || *w > usize::MAX as u128 - 63)
    {
        "representation"
    } else if widths.iter().any(|w| *w > (1u128 << 32)) {
        "allocation"
    } else {
        "legal"
    }
}

#[test]
fn r20_generated_descriptor_budget_storage_extremes() {
    use bennett_isa_ref::{
        BitVector, Budget, DerivedWidths, OracleInput, OracleVectorInput, OracleVectorMachine,
        QuantumState, Type,
    };
    use Expected::*;
    let mut cases = 0;
    for n in VALUES {
        for (ty, zero_legal) in [
            (format!("i{n}"), false),
            (format!("reg[{n}]"), false),
            (format!("wire[{n}]"), false),
            (format!("qreg[{n}]"), false),
            (format!("region[{n},1]"), true),
            (format!("region[0,{n}]"), false),
        ] {
            let expected = if (!zero_legal && n == 0) || (ty.starts_with('i') && n > 128) {
                Range
            } else {
                Admit
            };
            check(&ty, expected, || Type::parse(&ty));
            cases += 1;
        }
        check(
            &format!("bit-storage={n}"),
            if n == 0 {
                Range
            } else if n == 1 {
                Admit
            } else {
                Resource
            },
            || BitVector::zero(n as usize),
        );
        cases += 1;
        check(
            &format!("quantum-storage={n}"),
            if n <= 1 { Admit } else { Resource },
            || QuantumState::new().allocate(n as usize),
        );
        cases += 1;
        let o = Object::parse(include_str!("../fixtures/inc8.bisa")).unwrap();
        let input = OracleVectorInput {
            ins: vec![BitVector::from_u128(8, 8).unwrap()],
            ..Default::default()
        };
        // B frames are storage, whereas B in the sequential engine is a bound:
        // its three rows terminate even at B=max. No huge loop is executed.
        check(
            &format!("budget-B-machine={n}"),
            if n <= 1 { Admit } else { Resource },
            || OracleVectorMachine::new(&o, "inc8", &input, Budget::steps(n)),
        );
        check(&format!("budget-B-sequential={n}"), Admit, || {
            o.run_oracle("inc8", &OracleInput::bits(&[8]), Budget::steps(n))
        });
        cases += 2;
        let text = support::r13_text("S_cells")
            .replace(r#""S_cells":"18446744073709551615""#, r#""S_cells":128"#);
        let o = Object::parse(&text).unwrap();
        for component in ["heap", "stack", "depth"] {
            let mut b = Budget::steps(0);
            match component {
                "heap" => b.heap = n as u128,
                "stack" => b.stack = n as u128,
                _ => b.depth = n as u128,
            }
            let expected = if component == "heap"
                && n as u128
                    > o.section("layout")
                        .unwrap()
                        .get("heap_limit_max")
                        .unwrap()
                        .uint()
                        .unwrap()
            {
                Range
            } else {
                Admit
            };
            check(&format!("budget-{component}={n}"), expected, || {
                o.validate_budget(b)?;
                DerivedWidths::derive(&o, "inc8", b)
            });
            cases += 1;
        }
        // All canonical decimal widths/counts are descriptor-bounded only by
        // the host. This explicitly probes beyond the descriptor, with § citation.
        for (label, text, expected) in [
            ("fault-width", support::object(&[("o",&format!(r#"{{"abi":"0.8","args":[],"fault":{},"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}}"#, if n <= 9_007_199_254_740_991 { n.to_string() } else { format!("\"{n}\"") }), &format!("oracle @o (x:wire[8] in) (t:wire[8]) (f:wire[{n}]) () ()\n^entry():\nfault 1"))],"{}",""), if n < 4 { Range } else { Admit }),
            ("alloc-length", include_str!("../fixtures/heap.bisa").replace("p = lift.alloc 0", &format!("p = lift.alloc {n}")), if n < 65536 { Admit } else { Range }),
        ] { check(&format!("{label}={n}"),expected,|| Object::parse(&text)); cases += 1; }
    }
    for spelling in ["00", "+1", "-0", "1.0", "18446744073709551616"] {
        for ty in [
            format!("reg[{spelling}]"),
            format!("wire[{spelling}]"),
            format!("qreg[{spelling}]"),
            format!("region[{spelling},1]"),
        ] {
            check(
                &ty,
                if spelling == "18446744073709551616" {
                    Resource
                } else {
                    Range
                },
                || Type::parse(&ty),
            );
            cases += 1;
        }
    }
    assert_eq!(cases, 110);
    eprintln!("R20 descriptors/budgets/storage: 15 paths, {cases} cases (20 spelling controls)");
}

#[test]
fn r20_generated_qreg_descriptor_arithmetic_extremes() {
    use Expected::*;
    let mut cases = 0;
    for n in VALUES {
        for (path, params, body, expected) in [
            (
                "new",
                String::new(),
                format!("q = qreg.new any {n}\nqreg.free q\nret"),
                if n == 0 { Range } else { Admit },
            ),
            (
                "slice-offset",
                "q:qreg[18446744073709551615]".into(),
                format!("s = qreg.slice q {n} 1\naction_call @a (s) () () ()\nret"),
                if n == u64::MAX { Range } else { Admit },
            ),
            (
                "slice-length",
                "q:qreg[18446744073709551615]".into(),
                format!("s = qreg.slice q 0 {n}\naction_call @a (s) () () ()\nret"),
                if n == 0 { Range } else { Admit },
            ),
            (
                "concat",
                format!("q:qreg[{n}], p:qreg[1]"),
                "s = qreg.concat q p\nqreg.free s\nret".into(),
                if n == 0 {
                    Range
                } else if n == u64::MAX {
                    Resource
                } else {
                    Admit
                },
            ),
        ] {
            let types = if params.is_empty() {
                String::new()
            } else {
                params
                    .split(", ")
                    .map(|p| format!("\"{}\"", p.split_once(':').unwrap().1))
                    .collect::<Vec<_>>()
                    .join(",")
            };
            let sig = format!(
                r#"{{"abi":"0.8","kind":"cc","params":[{types}],"results":[],"tier":"cc"}}"#
            );
            let fbody = format!("fn cc @f ({params}) () () () ()\n^entry():\n{body}");
            let w = if path == "slice-length" { n } else { 1 };
            let mw = if w <= 9_007_199_254_740_991 {
                w.to_string()
            } else {
                format!("\"{w}\"")
            };
            let asig = format!(
                r#"{{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[{mw}],"tier":"q"}}"#
            );
            let abody =
                format!("action @a (t:wire[{w}] mut) () () () ()\n^entry():\nx t[0]\nx t[0]");
            let mut defs = vec![("f", sig.as_str(), fbody.as_str())];
            if path.starts_with("slice") {
                defs.push(("a", &asig, &abody));
            }
            let text = support::object(&defs, "{}", "");
            check(&format!("qreg-{path}={n}"), expected, || {
                Object::parse(&text)
            });
            cases += 1;
        }
    }
    assert_eq!(cases, 24);
    eprintln!("R20 qreg arithmetic: 4 paths, {cases} cases");
}

#[test]
fn r20_generated_cast_length_region_product_extremes() {
    use Expected::*;
    let mut cases = 0;
    for n in VALUES {
        for cast in ["zext", "trunc"] {
            let text = oracle(n, "1").replace(
                &format!("lift.const.i{n} 1"),
                &format!("lift.{cast}.i{n} x"),
            );
            let expected = if n == 0 || (cast == "zext" && n < 8) || (cast == "trunc" && n > 8) {
                Range
            } else {
                Admit
            };
            check(&format!("{cast}={n}"), expected, || Object::parse(&text));
            cases += 1;
        }
        let text = include_str!("../fixtures/heap.bisa").replace(
            "p = lift.alloc 0",
            &format!("lift.boundscheck p {n}\np = lift.alloc 0"),
        );
        check(
            &format!("bounds-length={n}"),
            if n < 65536 { Admit } else { Range },
            || Object::parse(&text),
        );
        cases += 1;
        let text = action(
            1,
            &format!("qregion r 1 {n}\nb = qborrow any 1\nqreturn b"),
            None,
        );
        check(
            &format!("region-product={n}"),
            if n == 0 {
                Range
            } else if n == 1 {
                Admit
            } else {
                Resource
            },
            || Object::parse(&text),
        );
        cases += 1;
        let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
        let text = support::object_with_data(
            &[(
                "a",
                sig,
                &format!(
                    "action @a (t:wire mut) () () () ()\n^entry():\nqregion r {n} 8 @img\nx t\nx t"
                ),
            )],
            &[("img", "00")],
            "",
        );
        check(
            &format!("region-image={n}"),
            if n == 0 { Admit } else { Range },
            || Object::parse(&text),
        );
        cases += 1;
    }
    assert_eq!(cases, 30);
    eprintln!("R20 casts/lengths/region products: 5 paths, {cases} cases");
}

#[test]
fn r20_generated_borrow_capacity_sum_extremes() {
    for n in VALUES {
        let mut o = Object::parse(&action(
            1,
            "b = qborrow any 1\nqreturn b\nc = qborrow any 1\nqreturn c",
            None,
        ))
        .unwrap();
        // Exercise the public capacity consumer independently of concrete wire
        // expansion. Every positive n is an ISA width; usize limits this host sum.
        o.functions.get_mut("a").unwrap().blocks[0].instructions[0].operands[1] = n.to_string();
        check(
            &format!("borrow-capacity={n}"),
            if n == u64::MAX {
                Expected::Resource
            } else {
                Expected::Admit
            },
            || o.borrow_capacity("a"),
        );
    }
    eprintln!("R20 borrow capacity: 1 path, 6 cases");
}
