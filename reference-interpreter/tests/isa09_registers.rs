mod support;
use bennett_isa_ref::{
    ClassicalMachine, ClassicalSnapshot, ClassicalValue, Object, QuantumRegisters,
};

fn program(functions: &[(&str, &str, &str)]) -> Object {
    let source = support::object(functions, "{}", "")
        .replace("=0.8", "=0.9")
        .replace("\"0.8\"", "\"0.9\"");
    support::rehash(&source)
}
fn allocated_id(before: &ClassicalSnapshot, machine: &ClassicalMachine<'_>) -> u64 {
    before
        .delta(&machine.snapshot())
        .unwrap()
        .registers
        .into_iter()
        .find_map(|r| match r.post {
            Some(ClassicalValue::Qreg(id)) if r.name == "q" => Some(id),
            _ => None,
        })
        .expect("allocation's dynamic q binding")
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
                machine
                    .quantum_registers()
                    .bundle(allocated_id(&before, &machine))
                    .unwrap()
                    .len() as u128,
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
            assert!(machine
                .quantum_registers()
                .bundle(allocated_id(&before, &machine))
                .is_err());
            assert!(machine.reverse().is_err());
        } else {
            assert!(machine.measurements().is_empty());
            assert_eq!(
                machine
                    .quantum_registers()
                    .bundle(allocated_id(&before, &machine))
                    .unwrap()
                    .len() as u128,
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

#[test]
fn an_uncalled_allocation_never_reassigns_its_id_or_resurrects_an_alias() {
    let make = signature("r", "[]", r#"["qreg[*]"]"#);
    let flag = signature("r", "[]", r#"["i1"]"#);
    let size = signature("r", "[]", r#"["i64"]"#);
    let prefix = "q = call @make\nid = qreg.id q\nalias = qreg.from_id id\nuncall @make () (q)\nnext = call @make\n";
    let compare = format!("fn r @different () (v:i1) () () ()\n^entry():\n{prefix}id2 = qreg.id next\nv = icmp.ne id id2\nqreg.free next\nret v");
    let stale =
        format!("fn r @stale () (v:i64) () () ()\n^entry():\n{prefix}v = qreg.width alias\nret v");
    let object = program(&[
        ("make",&make,"fn r @make () (q:qreg[*]) () () ()\n^entry():\nn = const.i64 1\nq = qreg.new \"A\" n\nret q"),
        ("different",&flag,&compare),("stale",&size,&stale)]);
    let mut machine = ClassicalMachine::new(&object, "different", &[], &[]).unwrap();
    let before = machine.snapshot();
    let result = machine.run(100).unwrap();
    assert_eq!(result.fault, None);
    assert_eq!(result.values, vec![ClassicalValue::int(1, 1).unwrap()]);
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
    let mut machine = ClassicalMachine::new(&object, "stale", &[], &[]).unwrap();
    let before = machine.snapshot();
    assert_eq!(machine.run(100).unwrap().fault, Some(4));
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
    let mut first = ClassicalMachine::new(&object, "make", &[], &[]).unwrap();
    let mut second = ClassicalMachine::new(&object, "make", &[], &[]).unwrap();
    let a = first.run(20).unwrap().values;
    let b = second.run(20).unwrap().values;
    assert_ne!(a, b, "ids are unique for the entire loaded program");
}

#[test]
fn dynamic_width_checks_cover_branches_and_returns() {
    let root = signature("r", r#"["i64"]"#, r#"["i64"]"#);
    let branch = signature("r", r#"["qreg[*]"]"#, r#"["i64"]"#);
    let cast = signature("r", r#"["qreg[*]"]"#, r#"["qreg[2]"]"#);
    for (name,callee,body) in [
        ("branch",&branch,"fn r @branch (q:qreg[*]) (w:i64) () () ()\n^entry():\nbr ^next(q)\n^next(p:qreg[2]):\nw = qreg.width p\nret w"),
        ("cast",&cast,"fn r @cast (q:qreg[*]) (r:qreg[2]) () () ()\n^entry():\nret q"),
    ] {
        let call = if name=="branch" { "w = call @branch q\nqreg.free q" }
                   else { "p = call @cast q\nw = qreg.width p\nqreg.free p" };
        let invoke = format!("fn r @invoke (n:i64) (w:i64) () () ()\n^entry():\nq = qreg.new \"A\" n\n{call}\nret w");
        let object = program(&[(name,callee,body),("invoke",&root,&invoke)]);
        for n in [1,2,3] {
            let mut machine = ClassicalMachine::new(&object,"invoke",&[ClassicalValue::int(64,n).unwrap()],&[]).unwrap();
            let before = machine.snapshot();
            assert_eq!(machine.run(40).unwrap().fault,if n==2 {None} else {Some(4)},"{name}/{n}");
            machine.reverse().unwrap();
            assert_eq!(machine.snapshot(),before);
        }
    }
}

#[test]
fn concat_and_runtime_slice_preserve_per_wire_classes_and_reverse() {
    let root = signature("r", "[]", r#"["i64"]"#);
    let object = program(&[
        ("same",r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#,
         "perm_action @same (t:wire mut) () () () ()\n^entry():\nx t\nx t"),
        ("join",&root,"fn r @join () (w:i64) () () ()\n^entry():\na = qreg.new \"A\" 1\nb = qreg.new \"B\" 2\nc = qreg.concat a b\nlo = const.i64 1\nlen = const.i64 1\ns = qreg.slice c lo len\naction_call @same (s) () () ()\nw = qreg.width c\nqreg.free c\nret w")]);
    let mut machine = ClassicalMachine::new(&object, "join", &[], &[]).unwrap();
    let before = machine.snapshot();
    let mut joined = None;
    let mut view = None;
    while let Some(point) = machine.step_once().unwrap() {
        let delta = before.delta(&machine.snapshot()).unwrap();
        for register in delta.registers {
            if let Some(ClassicalValue::Qreg(id)) = register.post {
                if register.name == "c" {
                    joined = Some(id);
                }
                if register.name == "s" {
                    view = Some(id);
                }
            }
        }
        match point.instruction.op.as_str() {
            "qreg.concat" => assert_eq!(
                machine
                    .quantum_registers()
                    .classes(joined.unwrap())
                    .unwrap(),
                ["A", "B", "B"]
            ),
            "qreg.slice" => {
                assert!(machine.quantum_registers().bundle(joined.unwrap()).is_err());
                assert_eq!(
                    machine.quantum_registers().classes(view.unwrap()).unwrap(),
                    ["B"]
                );
            }
            "action_call" => {
                assert!(machine.quantum_registers().bundle(view.unwrap()).is_err());
                assert_eq!(
                    machine
                        .quantum_registers()
                        .classes(joined.unwrap())
                        .unwrap(),
                    ["A", "B", "B"]
                );
            }
            _ => {}
        }
    }
    assert_eq!(machine.outcome().fault, None);
    assert_eq!(
        machine.outcome().values,
        vec![ClassicalValue::int(64, 3).unwrap()]
    );
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}

#[test]
fn literal_out_of_bounds_slice_faults_at_execution_without_a_view() {
    let root = signature("r", "[]", "[]");
    let object = program(&[
        ("same",r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#,
         "perm_action @same (t:wire mut) () () () ()\n^entry():\nx t\nx t"),
        ("bad",&root,"fn r @bad () () () () ()\n^entry():\nq = qreg.new \"A\" 2\ns = qreg.slice q 3 1\naction_call @same (s) () () ()\nqreg.free q\nret")]);
    let mut machine = ClassicalMachine::new(&object, "bad", &[], &[]).unwrap();
    let before = machine.snapshot();
    assert_eq!(machine.run(30).unwrap().fault, Some(4));
    let id = allocated_id(&before, &machine);
    assert_eq!(machine.quantum_registers().width(id).unwrap(), 2);
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}

#[test]
fn launch_rejects_aliases_and_public_cli_accepts_dynamic_register_arguments() {
    let pair = signature("r", r#"["qreg[*]","qreg[*]"]"#, "[]");
    let single = signature("r", r#"["qreg[*]"]"#, r#"["i64"]"#);
    let object = program(&[
        (
            "pair",
            &pair,
            "fn r @pair (a:qreg[*], b:qreg[*]) () () () ()\n^entry():\nret",
        ),
        (
            "width",
            &single,
            "fn r @width (a:qreg[*]) (w:i64) () () ()\n^entry():\nw = qreg.width a\nret w",
        ),
    ]);
    let mut registers = QuantumRegisters::new();
    let id = registers.allocate(2).unwrap();
    let error = ClassicalMachine::with_quantum_registers(
        &object,
        "pair",
        &[ClassicalValue::Qreg(id), ClassicalValue::Qreg(id)],
        &[],
        registers,
    )
    .err()
    .expect("aliased launch");
    assert!(error.to_string().contains("E_ARG"));
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(".tmp/isa09-dynamic-register-cli.bisa");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, &object.text).unwrap();
    for (bits, width) in [("", 0), ("01", 2), ("10101", 5)] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
            .args([
                "run-cc",
                path.to_str().unwrap(),
                "width",
                "--arg",
                &format!("qreg:{bits}"),
            ])
            .output()
            .unwrap();
        assert!(output.status.success(), "{output:?}");
        let value =
            bennett_isa_ref::json::Json::parse(std::str::from_utf8(&output.stdout).unwrap())
                .unwrap();
        assert_eq!(
            value.get("results").unwrap().array().unwrap()[0]
                .get("value")
                .unwrap()
                .string()
                .unwrap(),
            width.to_string()
        );
    }
}

#[test]
fn unrepresentable_device_allocation_faults_heap_without_register_changes() {
    let sig = signature("r", r#"["i64"]"#, "[]");
    let object = program(&[(
        "heap",
        &sig,
        "fn r @heap (n:i64) () () () ()\n^entry():\nq = qreg.new \"A\" n\nqreg.free q\nret",
    )]);
    let mut machine = ClassicalMachine::new(
        &object,
        "heap",
        &[ClassicalValue::int(64, u64::MAX as u128).unwrap()],
        &[],
    )
    .unwrap();
    let before = machine.snapshot();
    assert_eq!(machine.run(20).unwrap().fault, Some(2));
    assert_eq!(machine.quantum_registers().quantum_state().wire_count(), 0);
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
}

#[test]
fn static_measure_suffix_is_checked_and_r_measurement_stays_forbidden() {
    let sig = signature("cc", "[]", r#"["i2"]"#);
    let object = program(&[("measure",&sig,"fn cc @measure () (v:i2) () () ()\n^entry():\nq = qreg.new \"A\" 2\nv = measure.i2 q\nret v")]);
    let mut machine = ClassicalMachine::new(&object, "measure", &[], &[]).unwrap();
    assert_eq!(
        machine.run(10).unwrap().values,
        vec![ClassicalValue::int(2, 0).unwrap()]
    );
    assert!(Object::parse(&object.text.replace("measure.i2", "measure.i1")).is_err());
    let r = object
        .text
        .replace("fn cc @measure", "fn r @measure")
        .replace("\"kind\":\"cc\"", "\"kind\":\"r\"")
        .replace("\"tier\":\"cc\"", "\"tier\":\"r\"");
    assert!(Object::parse(&r)
        .unwrap_err()
        .to_string()
        .contains("measurement is CC only"));
}

#[test]
fn concat_same_name_rejects_statically_and_a_distinct_alias_faults_at_runtime() {
    let sig = signature("r", "[]", "[]");
    let body = "fn r @concat () () () () ()\n^entry():\nq = qreg.new \"A\" 1\nid = qreg.id q\na = qreg.from_id id\nc = qreg.concat q a\nqreg.free c\nret";
    let object = program(&[("concat", &sig, body)]);
    let mut machine = ClassicalMachine::new(&object, "concat", &[], &[]).unwrap();
    let before = machine.snapshot();
    assert_eq!(machine.run(20).unwrap().fault, Some(4));
    assert_eq!(
        machine
            .quantum_registers()
            .width(allocated_id(&before, &machine))
            .unwrap(),
        1
    );
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), before);
    let repeated = object.text.replace("qreg.concat q a", "qreg.concat q q");
    let error = Object::parse(&repeated).expect_err("same-name concat must reject at load");
    assert!(error.to_string().contains("§3.3"));
}
