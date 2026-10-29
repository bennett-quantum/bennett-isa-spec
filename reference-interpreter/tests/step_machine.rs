use bennett_isa_ref::{Budget, Object, OracleInput, OracleMachine};

fn compare(o: &Object, name: &str, input: &OracleInput, budget: Budget) {
    let expected = o.run_oracle(name, input, budget).unwrap();
    let mut m = OracleMachine::new(o, name, input, budget).unwrap();
    let initial = m.snapshot();
    for _ in 0..budget.steps {
        m.forward_step().unwrap();
    }
    assert_eq!(m.outcome(), expected, "{name}:B={}", budget.steps);
    for _ in 0..budget.steps {
        m.inverse_step().unwrap();
    }
    assert_eq!(m.snapshot(), initial, "{name}: prefix reversal");
    assert!(m.frames_clean());
}

#[test]
fn inc8_every_input_and_budget_cutoff_including_idle_halt() {
    let o = Object::parse(include_str!("fixtures/inc8.bisa")).unwrap();
    for x in 0..256 {
        for b in 0..=5 {
            compare(&o, "inc8", &OracleInput::bits(&[x]), Budget::steps(b));
        }
    }
}
#[test]
fn arithmetic_parallel_assignment_control_and_every_fault() {
    let o = Object::parse(include_str!("fixtures/arithmetic.bisa")).unwrap();
    for name in o.functions.keys() {
        for a in [0, 1, 2, 127, 128, 255] {
            for b in [0, 1, 2, 8, 128, 255] {
                for steps in 0..=4 {
                    compare(&o, name, &OracleInput::bits(&[a, b]), Budget::steps(steps));
                }
            }
        }
    }
}
#[test]
fn heap_cells_and_allocation_table_restore_on_success_and_faults() {
    let o = Object::parse(include_str!("fixtures/heap.bisa")).unwrap();
    for name in o.functions.keys() {
        for heap in [0, 1, 2, 32] {
            for steps in 0..=10 {
                compare(
                    &o,
                    name,
                    &OracleInput::bits(&[]),
                    Budget {
                        steps,
                        heap,
                        stack: 0,
                        depth: 0,
                    },
                );
            }
        }
    }
}
#[test]
fn recursive_call_and_ret_each_invert_the_complete_pre_step_tuple() {
    let o = Object::parse(include_str!("fixtures/sum_db.bisa")).unwrap();
    let mut input = OracleInput::bits(&[2]);
    input.regions.insert("db".into(), (1..=16).collect());
    let budget = Budget {
        steps: 40,
        heap: 4080,
        stack: 512,
        depth: 64,
    };
    let mut m = OracleMachine::new(&o, "sum_db", &input, budget).unwrap();
    for _ in 0..budget.steps {
        let before = m.snapshot();
        m.forward_step().unwrap();
        let after = m.snapshot();
        m.inverse_step().unwrap();
        assert_eq!(m.snapshot(), before);
        m.forward_step().unwrap();
        assert_eq!(m.snapshot(), after);
    }
    for stack in [0, 15, 16, 47, 48, 512] {
        for depth in [0, 1, 2, 3] {
            compare(
                &o,
                "sum_db",
                &input,
                Budget {
                    stack,
                    depth,
                    ..budget
                },
            );
        }
    }
}
#[test]
#[ignore = "all prefixes of recursive sum for each 8-bit n"]
fn exhaustive_recursive_prefixes() {
    let o = Object::parse(include_str!("fixtures/sum_db.bisa")).unwrap();
    for n in 0..=255 {
        let mut input = OracleInput::bits(&[n]);
        input.regions.insert("db".into(), (1..=16).collect());
        let stop = if n <= 16 { 14 * n as u64 + 11 } else { 14 };
        for steps in 0..=stop {
            compare(
                &o,
                "sum_db",
                &input,
                Budget {
                    steps,
                    heap: 4080,
                    stack: 512,
                    depth: 64,
                },
            );
        }
    }
}
