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
