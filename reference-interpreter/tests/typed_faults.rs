mod support;
use bennett_isa_ref::{ClassicalMachine, ClassicalValue, Object};
use support::{object, rehash};
const TAIL: &str = "\nsection devices\n{\"models\":{\"clock\":{\"version\":\"0.9\"},\"console\":{\"version\":\"0.9\"},\"entropy\":{\"version\":\"0.9\"}},\"services\":{\"org.bennett.test\":{\"version\":\"1\"}}}\n\nsection layout\n{\"A_tab\":4,\"H\":12,\"S_cells\":16,\"globals\":{},\"heap_base\":16,\"heap_limit_max\":1024,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";
fn sig(kind: &str, params: &str, results: &str) -> String {
    format!(
        r#"{{"abi":"0.9","kind":"{kind}","params":{params},"results":{results},"tier":"{kind}"}}"#
    )
}
fn program(kind: &str, body: &str) -> Object {
    rehash(
        &object(
            &[(
                "main",
                &sig(kind, "[]", "[]"),
                &format!("fn {kind} @main () () () () ()\n^entry():\n{body}"),
            )],
            "{}",
            TAIL,
        )
        .replace("=0.8", "=0.9"),
    )
}
fn codes() -> Vec<u64> {
    let mut out: Vec<_> = (0..=256).collect();
    for bit in 0..64 {
        let x = 1u64 << bit;
        out.extend([x - 1, x, x.saturating_add(1), u64::MAX - x]);
    }
    out.push(u64::MAX);
    out.sort_unstable();
    out.dedup();
    out
}
#[test]
fn typed_fault_codes_preserve_every_bit_and_reverse_for_both_tiers() {
    for kind in ["cc", "r"] {
        for c in codes() {
            let o = program(kind, &format!("fault {c}"));
            o.check_hashes().unwrap();
            let mut m = ClassicalMachine::new(&o, "main", &[], &[]).unwrap();
            let entry = m.snapshot();
            let out = m.run(1).unwrap();
            assert_eq!(
                out.fault,
                Some(if c == 0 { 4 } else { c as u128 }),
                "{kind} {c}"
            );
            assert_eq!(out.steps, 1);
            assert!(out.values.is_empty());
            let terminal = m.snapshot();
            assert!(m.step_once().unwrap().is_none());
            assert_eq!(m.run(0).unwrap().steps, 0);
            assert_eq!(m.snapshot(), terminal);
            m.reverse().unwrap();
            assert_eq!(m.snapshot(), entry);
        }
    }
}
#[test]
fn fault_admission_is_version_tier_width_and_terminator_checked() {
    let valid = program("r", "fault 1").text;
    for bad in [
        "-1",
        "18446744073709551616",
        "340282366920938463463374607431768211455",
        "01",
        "code",
        "",
    ] {
        assert!(
            Object::parse(&valid.replace("fault 1", &format!("fault {bad}"))).is_err(),
            "{bad}"
        );
    }
    for bad in [
        "x = fault 1",
        "fault 1 2",
        "fault 1\nret",
        "fault 1 {\nret\n}",
    ] {
        assert!(
            Object::parse(&valid.replace("fault 1", bad)).is_err(),
            "{bad}"
        );
    }
    assert!(Object::parse(&valid.replace("0.9", "0.8")).is_err());
}
#[test]
fn nested_terminal_fault_retains_prefix_and_never_returns_to_callers() {
    for kind in ["cc", "r"] {
        for c in [0, 1, 2, 3, 4, 10, 11, 1u64 << 63, u64::MAX] {
            let rcaller=format!("fn r @rmain () () () () ()\n^entry():\nn = const.i32 16\np = alloc n\nlen = const.i32 1\nv = const.i8 65\nstore.i8 p v\neffect console.write p len\neffect entropy.read p len\ncall @middle p len\neffect console.write p len\nret");
            let caller = if kind == "r" {
                rcaller.replace("@rmain", "@main")
            } else {
                "fn cc @main () () () () ()\n^entry():\ncall @rmain\nret".into()
            };
            let middle="fn r @middle (p:ptr, len:i32) () () () ()\n^entry():\ncall @fail p len\neffect console.write p len\nret";
            let fail=format!("fn r @fail (p:ptr, len:i32) () () () ()\n^entry():\nfault {c}\n^dead():\neffect console.write p len\nfault 99");
            let o = rehash(
                &object(
                    &[
                        ("main", &sig(kind, "[]", "[]"), &caller),
                        ("rmain", &sig("r", "[]", "[]"), &rcaller),
                        ("middle", &sig("r", r#"["ptr","i32"]"#, "[]"), middle),
                        ("fail", &sig("r", r#"["ptr","i32"]"#, "[]"), &fail),
                    ],
                    "{}",
                    TAIL,
                )
                .replace("=0.8", "=0.9"),
            );
            let mut m = ClassicalMachine::new(&o, "main", &[], &[66]).unwrap();
            let entry = m.snapshot();
            for _ in 0..(9 + usize::from(kind == "cc")) {
                m.step_once().unwrap().unwrap();
            }
            let prefix = m.snapshot();
            let tape = m.effect_tape().to_vec();
            assert_eq!(m.console(), b"A");
            assert_eq!(m.memory_bytes(16, 1).unwrap(), [66]);
            let out = m.run(1).unwrap();
            assert_eq!(out.fault, Some(if c == 0 { 4 } else { c as u128 }));
            assert!(out.values.is_empty());
            assert!(m.snapshot().memory_equals(&prefix));
            assert!(prefix.delta(&m.snapshot()).unwrap().registers.is_empty());
            assert!(prefix
                .first_difference(&m.snapshot())
                .unwrap()
                .starts_with("fault:"));
            assert_eq!(m.effect_tape(), tape);
            assert_eq!(m.console(), b"A");
            let failed = m.snapshot();
            assert_eq!(m.run(100).unwrap().steps, 0);
            assert_eq!(m.snapshot(), failed);
            m.reverse().unwrap();
            assert_eq!(m.snapshot(), entry);
        }
    }
}
fn instrument(name: &str, ops: &[&str]) -> String {
    let mut s=format!("fn r @{name} (remaining:i64, p:ptr, byte:i8, len:i32) (left:i64) () () ()\n^entry():\nzero = const.i64 0\none = const.i64 1\nbr ^check0()\n");
    let mut previous = "remaining".to_owned();
    for (i, op) in ops.iter().enumerate() {
        let next = format!("next{i}");
        let row = if let Some(call) = op.strip_prefix("remaining = ") {
            format!("after{i} = {}", call.replace("remaining", &next))
        } else {
            op.replace("remaining", &next)
        };
        s.push_str(&format!("^check{i}():\nempty{i} = icmp.eq {previous} zero\nbr empty{i} ^exhausted() ^do{i}()\n^do{i}():\n{next} = sub {previous} one\n{row}\n"));
        previous = if op.starts_with("remaining = ") {
            format!("after{i}")
        } else {
            next
        };
        if !op.starts_with("ret") {
            s.push_str(&format!("br ^check{}()\n", i + 1));
        }
    }
    s.push_str("^exhausted():\nfault 1");
    s
}
#[test]
fn source_counter_checks_every_boundary_including_zero_and_both_returns() {
    let wrapper="fn cc @entry (steps:i64) (left:i64) () () ()\n^entry():\nn = const.i32 16\np = alloc n\nbyte = const.i8 65\nlen = const.i32 1\nleft = call @main steps p byte len\nret left";
    let main = instrument(
        "main",
        &[
            "store.i8 p byte",
            "remaining = call @inner remaining p byte len",
            "effect console.write p len",
            "ret remaining",
        ],
    );
    let inner = instrument(
        "inner",
        &[
            "effect clock.read p",
            "effect entropy.read p len",
            "ret remaining",
        ],
    );
    let rsig = sig("r", r#"["i64","ptr","i8","i32"]"#, r#"["i64"]"#);
    let o = rehash(
        &object(
            &[
                ("entry", &sig("cc", r#"["i64"]"#, r#"["i64"]"#), wrapper),
                ("main", &rsig, &main),
                ("inner", &rsig, &inner),
            ],
            "{}",
            TAIL,
        )
        .replace("=0.8", "=0.9"),
    );
    let mut zero = ClassicalMachine::new(
        &o,
        "main",
        &[
            ClassicalValue::Int(64, 0),
            ClassicalValue::Ptr(0),
            ClassicalValue::Int(8, 65),
            ClassicalValue::Int(32, 1),
        ],
        &[],
    )
    .unwrap();
    let zero_entry = zero.snapshot();
    assert_eq!(zero.run(100).unwrap().fault, Some(1));
    assert_eq!(zero.effect_tape_len(), 0);
    assert!(zero.snapshot().memory_equals(&zero_entry));
    zero.reverse().unwrap();
    assert_eq!(zero.snapshot(), zero_entry);
    for steps in 0..=9 {
        let mut m =
            ClassicalMachine::new(&o, "entry", &[ClassicalValue::Int(64, steps)], &[90]).unwrap();
        let entry = m.snapshot();
        let out = m.run(1000).unwrap();
        assert_eq!(
            out.fault,
            if steps < 7 { Some(1) } else { None },
            "source budget {steps}"
        );
        assert_eq!(m.clock(), u64::from(steps >= 3));
        assert_eq!(
            m.effect_tape_len(),
            usize::from(steps >= 3) + usize::from(steps >= 4) + usize::from(steps >= 6)
        );
        assert_eq!(m.console(), if steps >= 6 { b"Z".as_slice() } else { &[] });
        if steps < 7 {
            assert!(out.values.is_empty());
        } else {
            assert_eq!(out.values, vec![ClassicalValue::Int(64, steps - 7)]);
        }
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), entry, "source budget {steps}");
    }
    // The executor's fuel limit stops instrumentation with a diagnostic. It
    // neither invents a source counter nor turns a host limit into fault 1.
    let mut m = ClassicalMachine::new(&o, "entry", &[ClassicalValue::Int(64, 0)], &[90]).unwrap();
    let entry = m.snapshot();
    assert!(m.run(0).unwrap_err().to_string().contains("fuel"));
    assert_eq!(m.snapshot(), entry);
    assert_eq!(m.outcome().fault, None);
    assert!(m.run(8).unwrap_err().to_string().contains("fuel"));
    assert_eq!(m.outcome().fault, None);
    assert_eq!(m.run(1000).unwrap().fault, Some(1));
    m.reverse().unwrap();
    assert_eq!(m.snapshot(), entry);
}
#[test]
fn implicit_cause_mapping_is_atomic_terminal_and_reversible_on_09() {
    for kind in ["cc", "r"] {
        for (body, code) in [
            ("a = const.i8 1\nz = const.i8 0\nx = udiv a z\nret", 5),
            ("a = const.i8 1\nz = const.i8 0\nx = urem a z\nret", 5),
            ("a = const.i8 128\nz = const.i8 255\nx = sdiv a z\nret", 6),
            ("a = const.i8 128\nz = const.i8 255\nx = srem a z\nret", 6),
            ("z = const.i32 0\np = inttoptr z\nx = load.i8 p\nret", 7),
            (
                "z = const.i32 0\np = inttoptr z\nv = const.i8 4\nstore.i8 p v\nret",
                7,
            ),
            ("z = const.i32 0\np = inttoptr z\nfree p\nret", 7),
            ("n = const.i32 1025\np = alloc n\nret", 2),
            (
                "z = const.i32 0\np = inttoptr z\neffect clock.read p\nret",
                7,
            ),
            (
                "n = const.i32 8\np = alloc n\neffect entropy.read p n\nret",
                10,
            ),
            ("effect unknown.read\nret", 8),
            ("uncall @main () ()\nret", 4),
            (
                "br ^use()\n^never():\nv = const.i8 0\nret\n^use():\nx = add v v\nret",
                4,
            ),
            ("z = const.i32 0\np = inttoptr z\nn = const.i32 1\nr = hostcall \"org.bennett.test\" p n\nret", 7),
            ("trap", 4),
        ] {
            if (kind == "cc" && body.contains("effect ")) || (kind == "r" && body.contains("hostcall ")) {
                continue;
            }
            let o = program(kind, body);
            let mut m = ClassicalMachine::new(&o, "main", &[], &[]).unwrap();
            let entry = m.snapshot();
            let mut prefix = entry.clone();
            while m.is_running() {
                prefix = m.snapshot();
                m.step_once().unwrap();
            }
            assert_eq!(m.outcome().fault, Some(code), "{kind} {body}");
            assert!(m.outcome().values.is_empty());
            assert!(m.snapshot().memory_equals(&prefix));
            assert!(prefix.delta(&m.snapshot()).unwrap().registers.is_empty());
            assert!(prefix
                .first_difference(&m.snapshot())
                .unwrap()
                .starts_with("fault:"));
            assert_eq!(m.effect_tape(), prefix.effect_tape());
            m.reverse().unwrap();
            assert_eq!(m.snapshot(), entry);
        }
    }
}

#[test]
fn cli_exposes_one_exact_fault_no_results_and_reverses_failed_09_launch() {
    use bennett_isa_ref::json::Json;
    use std::process::Command;
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(format!("../.tmp/typed-fault-{}.bisa", std::process::id()));
    for c in [0, 1, 2, 3, 10, 11, 1u64 << 53, 1u64 << 63, u64::MAX] {
        let o = program("cc", &format!("fault {c}"));
        std::fs::write(&path, &o.text).unwrap();
        for (command, reverse) in [
            ("run-cc", false),
            ("reverse", false),
            ("run-program", false),
            ("run-program", true),
        ] {
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"));
            cmd.args([command, path.to_str().unwrap(), "main"]);
            if reverse {
                cmd.arg("--reverse");
            }
            let out = cmd.output().unwrap();
            assert!(
                out.status.success(),
                "{command} {reverse}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            let j = Json::parse(std::str::from_utf8(&out.stdout).unwrap().trim()).unwrap();
            let effective = if c == 0 { 4 } else { c as u128 };
            assert_eq!(j.get("fault").unwrap().uint().unwrap(), effective);
            let faults = j.get("faults").unwrap().array().unwrap();
            assert_eq!(faults.len(), 1);
            assert_eq!(faults[0].get("code").unwrap().uint().unwrap(), effective);
            assert!(j.get("results").unwrap().array().unwrap().is_empty());
            if reverse || command == "reverse" {
                assert_eq!(j.get("status").unwrap().string().unwrap(), "REVERSED");
                assert_eq!(j.get("equals_entry").unwrap(), &Json::Bool(true));
            } else {
                assert_eq!(j.get("status").unwrap().string().unwrap(), "FAILED");
            }
        }
    }
    let o = program("cc", "fault 1");
    std::fs::write(&path, &o.text).unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
        .args(["run-cc", path.to_str().unwrap(), "main", "--budget", "0"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("fuel"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn implicit_division_fault_boundaries_cover_all_integer_widths() {
    for width in 1..=128 {
        let max = if width == 128 {
            u128::MAX
        } else {
            (1u128 << width) - 1
        };
        let min = 1u128 << (width - 1);
        for op in ["udiv", "urem", "sdiv", "srem"] {
            let signature = sig(
                "r",
                &format!(r#"["i{width}","i{width}"]"#),
                &format!(r#"["i{width}"]"#),
            );
            let body=format!("fn r @main (a:i{width}, b:i{width}) (x:i{width}) () () ()\n^entry():\nx = {op} a b\nret x");
            let o =
                rehash(&object(&[("main", &signature, &body)], "{}", "").replace("=0.8", "=0.9"));
            for a in [0, 1, min - 1, min, max] {
                for b in [0, 1, min, max] {
                    let expected = if b == 0 {
                        Some(5)
                    } else if op.starts_with('s') && a == min && b == max {
                        Some(6)
                    } else {
                        None
                    };
                    let mut m = ClassicalMachine::new(
                        &o,
                        "main",
                        &[ClassicalValue::Int(width, a), ClassicalValue::Int(width, b)],
                        &[],
                    )
                    .unwrap();
                    let entry = m.snapshot();
                    let out = m.run(2).unwrap();
                    assert_eq!(out.fault, expected, "i{width} {op} {a} {b}");
                    assert_eq!(out.values.is_empty(), expected.is_some());
                    m.reverse().unwrap();
                    assert_eq!(m.snapshot(), entry);
                }
            }
        }
    }
}

#[test]
fn q_fault_effective_cause_retains_distinct_budget_and_inverse_semantics() {
    use bennett_isa_ref::{Budget, OracleInput, OracleMachine};
    for v in ["0.8", "0.9"] {
        for c in [0, 1, 2, 3, 4, 11, u64::MAX] {
            for conditional in [false, true] {
                let signature = format!(
                    r#"{{"abi":"{v}","args":[],"fault":64,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}}"#
                );
                let rows = if conditional {
                    format!("one = lift.const.i1 1\nlift.faultif one {c}\nlift.retval x")
                } else {
                    format!("fault {c}")
                };
                let body=format!("oracle @main (x:wire[8] in) (t:wire[8]) (f:wire[64]) () ()\nregs {{ one:reg[1] }}\n^entry():\n{rows}");
                let o = rehash(
                    &object(&[("main", &signature, &body)], "{}", "")
                        .replace("=0.8", &format!("={v}")),
                );
                let charged = if conditional { 2 } else { 1 };
                for b in 0..=5 {
                    let input = OracleInput::bits(&[73]);
                    let budget = Budget::steps(b);
                    let out = o.run_oracle("main", &input, budget).unwrap();
                    let code = if b < charged {
                        1
                    } else if c == 0 && v == "0.9" {
                        4
                    } else {
                        c as u128
                    };
                    assert_eq!(
                        (out.code, out.payload),
                        (code, 0),
                        "{v} code={c} conditional={conditional} B={b}"
                    );
                    assert_eq!(out.steps, b.min(charged));
                    let mut m = OracleMachine::new(&o, "main", &input, budget).unwrap();
                    let entry = m.snapshot();
                    for _ in 0..b {
                        m.forward_step().unwrap();
                    }
                    assert_eq!(m.outcome().unwrap(), out);
                    for _ in 0..b {
                        m.inverse_step().unwrap();
                    }
                    assert_eq!(m.snapshot(), entry);
                    assert!(m.frames_clean());
                }
            }
        }
    }
}
