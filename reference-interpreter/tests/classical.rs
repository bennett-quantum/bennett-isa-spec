use bennett_isa_ref::{ClassicalMachine, ClassicalValue, Object};

#[test]
fn dynamic_instances_loop_and_whole_run_reversal() {
    let o = Object::parse(include_str!("fixtures/classical.bisa")).unwrap();
    for n in 0..=8 {
        let mut m =
            ClassicalMachine::new(&o, "sum", &[ClassicalValue::int(8, n).unwrap()], &[]).unwrap();
        let before = m.snapshot();
        let r = m.run(1000).unwrap();
        assert_eq!(r.fault, None);
        assert_eq!(
            r.values,
            vec![ClassicalValue::int(8, n * (n + 1) / 2).unwrap()]
        );
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), before);
    }
}
#[test]
fn nested_uncall_and_effects_are_reversed_exactly_once() {
    let o = Object::parse(include_str!("fixtures/classical.bisa")).unwrap();
    for name in ["nested", "effects"] {
        let mut m = ClassicalMachine::new(&o, name, &[], &[11, 22, 33]).unwrap();
        let before = m.snapshot();
        let result = m.run(1000).unwrap();
        assert_eq!(result.fault, None);
        assert_eq!(result.values, vec![ClassicalValue::int(64, 0).unwrap()]);
        if name == "nested" {
            assert_eq!(m.clock(), 0);
            assert_eq!(m.effect_tape_len(), 0);
        } else {
            assert_eq!(m.clock(), 1);
            assert_eq!(m.console(), &[11, 22, 33]);
            assert_eq!(m.effect_tape_len(), 5);
        }
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), before);
    }
}
#[test]
fn faults_are_atomic_terminal_and_reversible() {
    let o = Object::parse(include_str!("fixtures/classical.bisa")).unwrap();
    for (name, code) in [
        ("impure", 4),
        ("entropy_short", 10),
        ("unknown_effect", 8),
        ("memfault", 7),
        ("divzero", 5),
    ] {
        let mut m = ClassicalMachine::new(&o, name, &[], &[]).unwrap();
        let before = m.snapshot();
        let result = m.run(1000).unwrap();
        assert_eq!(result.fault, Some(code), "{name}");
        assert!(result.values.is_empty());
        if name == "entropy_short" {
            assert_eq!(m.effect_tape_len(), 0);
        }
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), before, "{name}");
    }
}

#[test]
fn host_request_reply_records_full_length_and_reverses_bytes() {
    let o = Object::parse(include_str!("fixtures/classical.bisa")).unwrap();
    let mut m = ClassicalMachine::new(&o, "host", &[], &[]).unwrap();
    let before = m.snapshot();
    let waiting = m.run(100).unwrap();
    let request = waiting.request.unwrap();
    assert_eq!(request.service, "org.bennett.test");
    assert_eq!(request.payload, [165, 0, 0]);
    m.reply(&[17, 34, 51, 68, 85]).unwrap();
    assert_eq!(m.memory_bytes(request.address, 3).unwrap(), [17, 34, 51]);
    let result = m.run(100).unwrap();
    assert_eq!(result.values, vec![ClassicalValue::int(16, 5).unwrap()]);
    m.reverse().unwrap();
    assert_eq!(m.snapshot(), before);
    assert!(m.reply(&[]).is_err());
    let mut invalid = ClassicalMachine::new(&o, "host_invalid", &[], &[]).unwrap();
    let result = invalid.run(100).unwrap();
    assert_eq!(result.fault, Some(7));
    assert!(result.request.is_none());
}
