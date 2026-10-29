//! Round 2 load-time witnesses; expectations follow §§3.3, 3.5, 16.1.
mod support;
use bennett_isa_ref::Object;
use std::process::Command;
fn reject(text: &str, rule: &str, tag: &str) {
    let e = Object::parse(text).expect_err("malformed object accepted");
    assert!(e.to_string().contains(rule), "{e}");
    let path = format!("{}/.tmp/r2-{tag}.bisa", env!("CARGO_MANIFEST_DIR"));
    std::fs::write(&path, text).unwrap();
    for cmd in ["hash", "check"] {
        let out = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref")).args([cmd, &path]).output().unwrap();
        assert_eq!(out.status.code(), Some(1), "{:?}", out);
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains(rule));
    }
}
#[test]
fn qif_arms_cannot_pop_or_leave_predicates() {
    for (n, arm) in ["qunpred p", "qpred t"].iter().enumerate() {
        let text = support::object(&[("a", support::ACTION_1_1, &format!("action @a (t:wire mut) (p:wire in) () () ()\n^entry():\nqif p {{\n{arm}\n}} {{\nx t\n}}"))], "{}", "");
        reject(&text, "§3.5", &format!("pred-{n}"));
    }
    Object::parse(&support::object(&[("a", support::ACTION_1_1, "action @a (t:wire mut) (p:wire in) () () ()\n^entry():\nqif p {\nqpred p\nx t\nqunpred p\n} {\nqif p {\nx t\n} {\nx t\n}\n}")], "{}", "")).unwrap();
}
#[test]
fn icmp_requires_integer_or_pointer_operands() {
    for ty in ["token", "f64", "qreg[1]"] {
        let sig = format!(r#"{{"abi":"0.8","kind":"cc","params":["{ty}","{ty}"],"results":["i1"],"tier":"cc"}}"#);
        let text = support::object(&[("cmp", &sig, &format!("fn cc @cmp (a:{ty}, b:{ty}) (r:i1) () () ()\n^entry():\nr = icmp.eq a b\nret r"))], "{}", "");
        reject(&text, "§3.3", ty);
    }
}
#[test]
fn lifted_memory_widths_are_canonical_at_load() {
    for (fixture, op) in [("sum_db", "load"), ("heap", "store")] {
        let text = std::fs::read_to_string(format!("{}/tests/fixtures/{fixture}.bisa", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let width = if fixture == "heap" { 16 } else { 8 };
        let old = format!("lift.{op}.i{width}");
        assert!(text.contains(&old));
        reject(&text.replace(&old, &format!("lift.{op}.i0{width}")), "§16.1", fixture);
    }
}

#[test]
fn integer_and_pointer_comparisons_execute_and_reverse() {
    use bennett_isa_ref::{ClassicalMachine, ClassicalValue};
    for ty in ["i8", "ptr"] {
        let sig = format!(r#"{{"abi":"0.8","kind":"cc","params":["{ty}","{ty}"],"results":["i1"],"tier":"cc"}}"#);
        let text = support::object(&[("cmp", &sig, &format!("fn cc @cmp (a:{ty}, b:{ty}) (r:i1) () () ()\n^entry():\nr = icmp.eq a b\nret r"))], "{}", "");
        let o = Object::parse(&text).unwrap();
        for b in [3, 4] {
            let val = |v| if ty == "ptr" { ClassicalValue::Ptr(v) } else { ClassicalValue::int(8, v).unwrap() };
            let mut m = ClassicalMachine::new(&o, "cmp", &[val(3), val(b)], &[]).unwrap();
            let before = m.snapshot();
            assert_eq!(m.run(3).unwrap().values, [ClassicalValue::int(1, u128::from(b == 3)).unwrap()]);
            m.reverse().unwrap();
            assert_eq!(m.snapshot(), before);
        }
    }
}
