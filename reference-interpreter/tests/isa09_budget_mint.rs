mod support;
use bennett_isa_ref::{Budget, ClassicalMachine, ClassicalValue, Object, QuantumRegisters};

fn program() -> Object {
    extended(&[])
}
fn extended(extra: &[(&str, &str, &str)]) -> Object {
    let mut functions = vec![
        ("id",r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#,
         "oracle @id (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x"),
        ("mint",r#"{"abi":"0.8","kind":"r","params":["i64","i64","i64","i64"],"results":["token"],"tier":"r"}"#,
         "fn r @mint (s:i64, h:i64, st:i64, d:i64) (tk:token) () () ()\n^entry():\ntk = budget.mint @id s h st d\nret tk")
    ];
    functions.extend_from_slice(extra);
    let text = support::object(&functions, "{}", "")
        .replace("=0.8", "=0.9")
        .replace("\"0.8\"", "\"0.9\"");
    support::rehash(&text)
}

#[test]
fn device_mint_validates_all_unsigned_components_before_creating_a_token() {
    let object = program();
    for tuple in [
        [0, 0, 0, 0],
        [1, 0, 0, 0],
        [u64::MAX, 0, 0, 0],
        [1, 1, 0, 0],
        [1, 0, 1, 0],
        [1, 0, 0, 1],
    ] {
        let args = tuple.map(|v| ClassicalValue::int(64, v as u128).unwrap());
        let mut machine = ClassicalMachine::new(&object, "mint", &args, &[]).unwrap();
        let before = machine.snapshot();
        let result = machine.run(20).unwrap();
        if tuple[1..] == [0, 0, 0] {
            assert_eq!(result.fault, None);
            assert_eq!(result.values.len(), 1);
        } else {
            assert_eq!(result.fault, Some(4));
            assert!(result.values.is_empty());
        }
        machine.reverse().unwrap();
        assert_eq!(machine.snapshot(), before);
    }
}

#[test]
fn mint_is_09_only_and_its_four_operands_must_be_i64() {
    let object = program();
    let old = object
        .text
        .replace("=0.9", "=0.8")
        .replace("\"0.9\"", "\"0.8\"");
    assert!(Object::parse(&old).is_err());
    let bad = object.text.replace("s:i64", "s:i32").replace(
        "[\"i64\",\"i64\",\"i64\",\"i64\"]",
        "[\"i32\",\"i64\",\"i64\",\"i64\"]",
    );
    assert!(Object::parse(&bad)
        .unwrap_err()
        .to_string()
        .contains("§3.3"));
}

fn registers(x: u128, t: u128, f: u128) -> (QuantumRegisters, Vec<ClassicalValue>, [u64; 3]) {
    let mut registers = QuantumRegisters::new();
    let mut ids = [0; 3];
    for (index, width, value) in [(0, 1, x), (1, 1, t), (2, 4, f)] {
        let id = registers.allocate(width).unwrap();
        let bits = (0..width).map(|b| value >> b & 1 != 0).collect::<Vec<_>>();
        registers.prepare(id, &bits).unwrap();
        ids[index] = id;
    }
    let args = ids.map(ClassicalValue::Qreg).to_vec();
    (registers, args, ids)
}

#[test]
fn minted_budget_drives_any_oracle_and_reverses_the_entire_successful_or_failed_prefix() {
    let object = extended(&[
        ("other",r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#,
         "oracle @other (x:wire in) (t:wire) (f:wire[4]) () ()\nregs { one:reg[1], r:reg[1] }\n^entry():\none = lift.const.i1 1\nr = lift.xor x one\nlift.retval r"),
        ("invoke",r#"{"abi":"0.8","kind":"r","params":["qreg[1]","qreg[1]","qreg[4]","i64"],"results":[],"tier":"r"}"#,
         "fn r @invoke (x:qreg[1], t:qreg[1], f:qreg[4], s:i64) () () () ()\n^entry():\nz = const.i64 0\ntk = call @mint s z z z\noracle_call @other tk (x) t f () ()\nret")]);
    for steps in 0..=4 {
        for x in 0..2 {
            for target in 0..2 {
                for fault in 0..16 {
                    let (registers, mut args, ids) = registers(x, target, fault);
                    args.push(ClassicalValue::int(64, steps).unwrap());
                    let mut machine = ClassicalMachine::with_quantum_registers(
                        &object,
                        "invoke",
                        &args,
                        &[],
                        registers,
                    )
                    .unwrap();
                    let before = machine.snapshot();
                    assert_eq!(machine.run(40).unwrap().fault, None);
                    let registers = machine.quantum_registers();
                    let state = registers.quantum_state();
                    assert_eq!(
                        state.read_basis(registers.bundle(ids[0]).unwrap()).unwrap(),
                        x
                    );
                    assert_eq!(
                        state.read_basis(registers.bundle(ids[1]).unwrap()).unwrap(),
                        target ^ if steps >= 3 { x ^ 1 } else { 0 }
                    );
                    assert_eq!(
                        state.read_basis(registers.bundle(ids[2]).unwrap()).unwrap(),
                        fault ^ u128::from(steps < 3)
                    );
                    machine.reverse().unwrap();
                    assert_eq!(machine.snapshot(), before);
                }
            }
        }
    }
}

#[test]
fn uncall_revokes_the_mint_but_a_pure_alias_keeps_its_stale_identity() {
    let object = extended(&[("stale",r#"{"abi":"0.8","kind":"r","params":["qreg[1]","qreg[1]","qreg[4]"],"results":[],"tier":"r"}"#,
        "fn r @stale (x:qreg[1], t:qreg[1], f:qreg[4]) () () () ()\n^entry():\ns = const.i64 1\nz = const.i64 0\nc = const.i1 1\ntk = call @mint s z z z\na = select c tk tk\nuncall @mint (s, z, z, z) (tk)\nfresh = call @mint s z z z\noracle_call @id a (x) t f () ()\nret")]);
    let (registers, args, ids) = registers(1, 1, 7);
    let mut machine =
        ClassicalMachine::with_quantum_registers(&object, "stale", &args, &[], registers).unwrap();
    let before = machine.snapshot();
    assert_eq!(machine.run(60).unwrap().fault, Some(4));
    let registers = machine.quantum_registers();
    assert_eq!(
        registers
            .quantum_state()
            .read_basis(registers.bundle(ids[1]).unwrap())
            .unwrap(),
        1
    );
    assert_eq!(
        registers
            .quantum_state()
            .read_basis(registers.bundle(ids[2]).unwrap())
            .unwrap(),
        7
    );
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}

#[test]
fn equal_tuple_tokens_have_distinct_identities_for_matched_classical_uncall() {
    let pass = r#"{"abi":"0.8","kind":"r","params":["token"],"results":["i1"],"tier":"r"}"#;
    let root = r#"{"abi":"0.8","kind":"r","params":["i1"],"results":[],"tier":"r"}"#;
    let object = extended(&[
        ("pass",pass,"fn r @pass (tk:token) (r:i1) () () ()\n^entry():\nr = const.i1 1\nret r"),
        ("compare",root,"fn r @compare (same:i1) () () () ()\n^entry():\ns = const.i64 1\nz = const.i64 0\na = budget.mint @id s z z z\nb = budget.mint @id s z z z\nchosen = select same a b\nr = call @pass a\nuncall @pass (chosen) (r)\nret")]);
    for same in 0..2 {
        let mut machine = ClassicalMachine::new(
            &object,
            "compare",
            &[ClassicalValue::int(1, same).unwrap()],
            &[],
        )
        .unwrap();
        let before = machine.snapshot();
        assert_eq!(
            machine.run(60).unwrap().fault,
            if same == 1 { None } else { Some(4) }
        );
        machine.reverse().unwrap();
        assert_eq!(machine.snapshot(), before);
    }
}

#[test]
fn host_token_aliases_are_explicit_and_foreign_or_run_scoped_launch_tokens_reject() {
    let object = extended(&[(
        "token",
        r#"{"abi":"0.8","kind":"r","params":["token"],"results":[],"tier":"r"}"#,
        "fn r @token (t:token) () () () ()\n^entry():\nret",
    )]);
    let a = object.budget_token("id", Budget::steps(1)).unwrap();
    let b = object.budget_token("id", Budget::steps(1)).unwrap();
    assert_ne!(a, b);
    assert_eq!(a, a.clone());
    assert_eq!(a.tuple(), b.tuple());
    let mut valid = ClassicalMachine::new(
        &object,
        "token",
        &[ClassicalValue::DeviceToken(a.clone())],
        &[],
    )
    .unwrap();
    assert_eq!(valid.run(10).unwrap().fault, None);
    valid.reverse().unwrap();
    let mut mint = ClassicalMachine::new(
        &object,
        "mint",
        &[1, 0, 0, 0].map(|v| ClassicalValue::int(64, v).unwrap()),
        &[],
    )
    .unwrap();
    let minted = mint.run(20).unwrap().values[0].clone();
    let foreign = program().budget_token("id", Budget::steps(1)).unwrap();
    for value in [ClassicalValue::DeviceToken(foreign), minted] {
        let error = ClassicalMachine::new(&object, "token", &[value], &[])
            .err()
            .expect("foreign or run-owned launch token");
        assert!(error.to_string().contains("E_ARG"));
    }
}

#[test]
fn observer_rollback_discards_token_validity_and_retry_mints_a_new_identity() {
    let object = program();
    let args = [1, 0, 0, 0].map(|v| ClassicalValue::int(64, v).unwrap());
    let mut machine = ClassicalMachine::new(&object, "mint", &args, &[]).unwrap();
    let before = machine.snapshot();
    let mut rejected = None;
    let result = machine.step_once_observed(|_, pre, post| {
        let delta = pre.delta(post).unwrap();
        rejected = delta.registers.into_iter().find_map(|r| match r.post {
            Some(ClassicalValue::DeviceToken(t)) => Some(t),
            _ => None,
        });
        assert!(rejected.is_some());
        Err::<(), _>(bennett_isa_ref::Error::host_resource(
            "test: reject minted-token publication".into(),
        ))
    });
    assert!(result.is_err());
    assert_eq!(machine.snapshot(), before);
    let result = machine.run(20).unwrap();
    let ClassicalValue::DeviceToken(accepted) = &result.values[0] else {
        panic!("minted token")
    };
    assert_ne!(*accepted, rejected.unwrap());
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}

#[test]
fn legacy_tuple_only_09_launch_needs_a_partition() {
    let object = extended(&[("tokens",r#"{"abi":"0.8","kind":"r","params":["token","token"],"results":[],"tier":"r"}"#,
        "fn r @tokens (a:token, b:token) () () () ()\n^entry():\nret")]);
    let args = [ClassicalValue::Token(Budget::steps(1)),ClassicalValue::Token(Budget::steps(1))];
    let error = ClassicalMachine::new(&object,"tokens",&args,&[]).err().expect("tuple-only09 request must supply token_aliases");
    assert!(error.to_string().contains("E_ARG"));
}
