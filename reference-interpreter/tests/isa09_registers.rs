mod support;
use bennett_isa_ref::{ClassicalMachine, ClassicalValue, Object};

fn program(functions: &[(&str, &str, &str)]) -> Object {
    let source = support::object(functions, "{}", "")
        .replace("=0.8", "=0.9")
        .replace("\"0.8\"", "\"0.9\"");
    support::rehash(&source)
}
fn signature(tier: &str, params: &str, results: &str) -> String {
    format!(
        r#"{{"abi":"0.8","kind":"{tier}","params":{params},"results":{results},"tier":"{tier}"}}"#
    )
}

#[test]
fn runtime_allocation_in_r_includes_zero_and_reverses() {
    let sig = signature("r", r#"["i64"]"#, r#"["i64"]"#);
    let object = program(&[("size",&sig,"fn r @size (n:i64) (w:i64) () () ()\n^entry():\nq = qreg.new \"A\" n\nw = qreg.width q\nqreg.free q\nret w")]);
    for width in [0, 1, 2, 65] {
        let mut machine = ClassicalMachine::new(
            &object,
            "size",
            &[ClassicalValue::int(64, width).unwrap()],
            &[],
        )
        .unwrap();
        let before = machine.snapshot();
        let outcome = machine.run(20).unwrap();
        assert_eq!(outcome.fault, None);
        assert_eq!(
            outcome.values,
            vec![ClassicalValue::int(64, width).unwrap()]
        );
        machine.reverse().unwrap();
        assert_eq!(machine.snapshot(), before);
    }
}

#[test]
fn dynamic_to_static_call_checks_width_before_entering_the_callee() {
    let child = signature("r", r#"["qreg[2]"]"#, r#"["i64"]"#);
    let root = signature("r", r#"["i64"]"#, r#"["i64"]"#);
    let object = program(&[
        ("two",&child,"fn r @two (q:qreg[2]) (w:i64) () () ()\n^entry():\nw = qreg.width q\nret w"),
        ("invoke",&root,"fn r @invoke (n:i64) (w:i64) () () ()\n^entry():\nq = qreg.new \"A\" n\nw = call @two q\nqreg.free q\nret w")]);
    for width in [0, 1, 2, 3] {
        let mut machine = ClassicalMachine::new(
            &object,
            "invoke",
            &[ClassicalValue::int(64, width).unwrap()],
            &[],
        )
        .unwrap();
        let before = machine.snapshot();
        let outcome = machine.run(30).unwrap();
        assert_eq!(outcome.fault, if width == 2 { None } else { Some(4) });
        if width != 2 {
            assert!(outcome.values.is_empty());
            assert_eq!(
                machine.quantum_registers().bundle(1).unwrap().len() as u128,
                width
            );
        }
        machine.reverse().unwrap();
        assert_eq!(machine.snapshot(), before);
    }
}

#[test]
fn dynamic_measurement_checks_width_before_consumption_or_a_draw() {
    let sig = signature("cc", r#"["i64"]"#, r#"["i2"]"#);
    let object = program(&[("measure",&sig,"fn cc @measure (n:i64) (v:i2) () () ()\n^entry():\nq = qreg.new \"A\" n\nv = measure.i2 q\nret v")]);
    for width in [0, 1, 2, 3] {
        let mut machine = ClassicalMachine::new(
            &object,
            "measure",
            &[ClassicalValue::int(64, width).unwrap()],
            &[],
        )
        .unwrap();
        let before = machine.snapshot();
        let outcome = machine.run(20).unwrap();
        assert_eq!(outcome.fault, if width == 2 { None } else { Some(4) });
        if width == 2 {
            assert_eq!(outcome.values, vec![ClassicalValue::int(2, 0).unwrap()]);
            assert!(machine.quantum_registers().bundle(1).is_err());
            assert!(machine.reverse().is_err());
        } else {
            assert!(machine.measurements().is_empty());
            assert_eq!(
                machine.quantum_registers().bundle(1).unwrap().len() as u128,
                width
            );
            machine.reverse().unwrap();
            assert_eq!(machine.snapshot(), before);
        }
    }
}

#[test]
fn dynamic_qreg_header_is_version_gated() {
    let sig = signature("r", r#"["qreg[*]"]"#, r#"["qreg[*]"]"#);
    let object = program(&[(
        "alias",
        &sig,
        "fn r @alias (q:qreg[*]) (r:qreg[*]) () () ()\n^entry():\nret q",
    )]);
    object.check_hashes().unwrap();
    let old = object
        .text
        .replace("=0.9", "=0.8")
        .replace("\"0.9\"", "\"0.8\"");
    assert!(Object::parse(&old).is_err());
}
