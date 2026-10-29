mod support;
use bennett_isa_ref::{BitVector, Budget, Object, OracleVectorInput};
fn object(body: &str) -> Object {
    let signature = r#"{"abi":"0.8","args":[],"fault":4,"in":[257,257],"kind":"oracle","mem":[],"tgt":257,"tier":"q"}"#;
    Object::parse(&support::object(&[("wide",signature,&format!("oracle @wide (x:wire[257] in, y:wire[257] in) (t:wire[257]) (f:wire[4]) () ()\nregs {{ r:reg[257] }}\n^entry():\n{body}\nlift.retval r"))],"{}","")).unwrap()
}
#[test]
fn arbitrary_width_oracle_arithmetic_and_total_faults() {
    // x=2^200+3, y=2^100+5. Quotient=2^100-5, remainder=28.
    let x = BitVector::parse_decimal(
        257,
        "1606938044258990275541962092341162602522202993782792835301379",
    )
    .unwrap();
    let y = BitVector::parse_decimal(257, "1267650600228229401496703205381").unwrap();
    let input = OracleVectorInput {
        ins: vec![x.clone(), y.clone()],
        ..Default::default()
    };
    for (op, expected) in [
        ("add", x.binary("add", &y).unwrap()),
        (
            "udiv",
            BitVector::parse_decimal(257, "1267650600228229401496703205371").unwrap(),
        ),
        ("urem", BitVector::from_u128(257, 28).unwrap()),
    ] {
        let o = object(&format!("r = lift.{op} x y"));
        let r = o
            .run_oracle_vectors("wide", &input, Budget::steps(2))
            .unwrap();
        assert_eq!(r.payload, expected);
        assert!(r.code.is_zero());
        assert_eq!(r.steps, 2);
        let r = o
            .run_oracle_vectors("wide", &input, Budget::steps(1))
            .unwrap();
        assert!(r.payload.is_zero());
        assert_eq!(r.code.to_u128().unwrap(), 1);
    }
    let o = object("r = lift.udiv x y");
    let input = OracleVectorInput {
        ins: vec![x, BitVector::zero(257).unwrap()],
        ..Default::default()
    };
    let r = o
        .run_oracle_vectors("wide", &input, Budget::steps(2))
        .unwrap();
    assert_eq!((r.code.to_u128().unwrap(), r.steps), (5, 1));
    assert!(r.payload.is_zero());
}

#[test]
fn vector_execution_matches_narrow_reference_for_recursion_memory_and_all_rows() {
    use bennett_isa_ref::OracleInput;
    for fixture in [
        "sum_db",
        "arithmetic",
        "heap",
        "nested",
        "actions",
        "tiny_memory",
    ] {
        let path = format!(
            "{}/tests/fixtures/{fixture}.bisa",
            env!("CARGO_MANIFEST_DIR")
        );
        let o = Object::parse(&std::fs::read_to_string(path).unwrap()).unwrap();
        for f in o.functions.values().filter(|f| f.kind == "oracle") {
            for x in 0..4 {
                let mut input = OracleInput::bits(
                    &f.params[0]
                        .iter()
                        .map(|p| {
                            x & if p.ty.width(o.ptr_width).unwrap() == 1 {
                                1
                            } else {
                                3
                            }
                        })
                        .collect::<Vec<_>>(),
                );
                for p in &f.params[3] {
                    let bennett_isa_ref::Type::Region(a, _) = p.ty else {
                        unreachable!()
                    };
                    input.regions.insert(
                        p.name.clone(),
                        (0..1usize << a).map(|i| i as u8 + 1).collect(),
                    );
                }
                for p in &f.params[4] {
                    input.args.push(match p.ty {
                        bennett_isa_ref::Type::Token => {
                            bennett_isa_ref::Argument::Token(o.default_budget(100).unwrap())
                        }
                        _ => bennett_isa_ref::Argument::Bits(1),
                    });
                }
                let wide = OracleVectorInput::from_input(&o, &f.name, &input).unwrap();
                for steps in [0, 1, 2, 3, 5, 15, 60] {
                    let budget = o.default_budget(steps).unwrap();
                    let small = o.run_oracle(&f.name, &input, budget).unwrap();
                    let large = o.run_oracle_vectors(&f.name, &wide, budget).unwrap();
                    let mut machine =
                        bennett_isa_ref::OracleVectorMachine::new(&o, &f.name, &wide, budget)
                            .unwrap();
                    let before = machine.snapshot();
                    for _ in 0..steps {
                        machine.forward_step().unwrap_or_else(|e| {
                            panic!("{fixture} @{} x={x} B={steps}: {e}", f.name)
                        });
                    }
                    assert_eq!(
                        machine.outcome().unwrap(),
                        large,
                        "vector machine {fixture} @{} B={steps}",
                        f.name
                    );
                    for _ in 0..steps {
                        machine.inverse_step().unwrap();
                    }
                    assert_eq!(machine.snapshot(), before);
                    assert!(machine.frames_clean());
                    assert_eq!(
                        (
                            large.payload.to_u128().unwrap(),
                            large.code.to_u128().unwrap(),
                            large.steps
                        ),
                        (small.payload, small.code, small.steps),
                        "{fixture} @{} x={x} B={steps}",
                        f.name
                    );
                }
            }
        }
    }
}

#[test]
fn wide_fault_literals_and_quantum_oracle_delivery_preserve_high_bits() {
    use bennett_isa_ref::{Argument, QuantumBindings, QuantumState};
    let signature = r#"{"abi":"0.8","args":[],"fault":257,"in":[257],"kind":"oracle","mem":[],"tgt":257,"tier":"q"}"#;
    let action = r#"{"abi":"0.8","args":["token"],"bounds":[],"in":[257],"kind":"action","mem":[],"mut":[257,257],"tier":"q"}"#;
    let code = "1606938044258990275541962092341162602522202993782792835301379";
    let text=support::object(&[("fault",signature,&format!("oracle @fault (x:wire[257] in) (t:wire[257]) (f:wire[257]) () ()\n^entry():\nfault {code}")),("invoke",action,"action @invoke (t:wire[257] mut, f:wire[257] mut) (x:wire[257] in) () (tk:token) ()\n^entry():\noracle_call @fault tk (x) t f () ()")],"{}","");
    let o = Object::parse(&text).unwrap();
    let input = OracleVectorInput {
        ins: vec![BitVector::zero(257).unwrap()],
        ..Default::default()
    };
    let result = o
        .run_oracle_vectors("fault", &input, Budget::steps(1))
        .unwrap();
    assert_eq!(result.code.to_decimal(), code);
    let mut q = QuantumState::new();
    let t = q.allocate(257).unwrap();
    let f = q.allocate(257).unwrap();
    let x = q.allocate(257).unwrap();
    q.gate("x", &[f[200]], None, &[]).unwrap();
    let mut b = QuantumBindings::default();
    b.bundles.insert("t".into(), t.clone());
    b.bundles.insert("f".into(), f.clone());
    b.bundles.insert("x".into(), x);
    b.args
        .insert("tk".into(), Argument::Token(Budget::steps(1)));
    let before = q.clone();
    o.execute_action("invoke", &mut q, &b, false, &[]).unwrap();
    assert_eq!(q.read_basis(&[f[0], f[1], f[200]]).unwrap(), 3);
    assert_eq!(q.read_basis(&[t[200]]).unwrap(), 0);
    o.execute_action("invoke", &mut q, &b, true, &[]).unwrap();
    assert_eq!(q, before);
}

#[test]
fn wide_literal_step_machine_restores_every_prefix_and_activation_cell() {
    use bennett_isa_ref::OracleVectorMachine;
    let o = Object::parse(include_str!("fixtures/wide.bisa")).unwrap();
    let x = BitVector::parse_decimal(
        257,
        "1606938044258990275541962092341162602522202993782792835301379",
    )
    .unwrap();
    for name in ["add257", "div257", "call257", "fault257"] {
        let count = o.functions[name].params[0].len();
        let input = OracleVectorInput {
            ins: vec![x.clone(); count],
            ..Default::default()
        };
        for stack in [0, 4, 5, 16] {
            for steps in 0..6 {
                let budget = Budget {
                    steps,
                    heap: 0,
                    stack,
                    depth: 4,
                };
                let result = o.run_oracle_vectors(name, &input, budget).unwrap();
                let mut machine = OracleVectorMachine::new(&o, name, &input, budget).unwrap();
                let entry = machine.snapshot();
                for _ in 0..steps {
                    let before = machine.snapshot();
                    machine.forward_step().unwrap();
                    machine.inverse_step().unwrap();
                    assert_eq!(machine.snapshot(), before);
                    machine.forward_step().unwrap();
                }
                assert_eq!(
                    machine.outcome().unwrap(),
                    result,
                    "@{name} stack={stack} B={steps}"
                );
                let target = BitVector::parse_decimal(257, "-1").unwrap();
                let fw = o.functions[name].params[2][0]
                    .ty
                    .width(o.ptr_width)
                    .unwrap() as usize;
                let fault = BitVector::from_u128(fw, 3).unwrap();
                let committed = machine.commit(&target, &fault).unwrap();
                assert_eq!(committed.0, target.binary("xor", &result.payload).unwrap());
                assert_eq!(committed.1, fault.binary("xor", &result.code).unwrap());
                for _ in 0..steps {
                    machine.inverse_step().unwrap();
                }
                assert_eq!(machine.snapshot(), entry);
                assert!(machine.frames_clean());
            }
        }
    }
}

#[test]
fn u128_compatibility_api_reports_its_representation_limit_without_panicking() {
    use bennett_isa_ref::{OracleInput, OracleMachine};
    let o = Object::parse(include_str!("fixtures/wide.bisa")).unwrap();
    let input = OracleInput::bits(&[3, 5]);
    let budget = Budget {
        steps: 2,
        heap: 0,
        stack: 16,
        depth: 4,
    };
    assert_eq!(o.run_oracle("add257", &input, budget).unwrap().payload, 8);
    let error = match OracleMachine::new(&o, "add257", &input, budget) {
        Err(e) => e,
        Ok(_) => panic!("u128 machine accepted wide locations"),
    };
    assert!(error.to_string().contains("OracleVectorMachine"));
}
