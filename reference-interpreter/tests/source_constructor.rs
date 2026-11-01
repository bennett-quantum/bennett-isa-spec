mod support;
use bennett_isa_conformance::{json, ref_catalog::Machine, ref_language::permutation};
use bennett_isa_ref::{
    ref_library::{OracleInvocation, Value},
    Budget, OracleInput, OracleMachine,
};
#[test]
fn source_derived_inc8_constructs_each_budget_and_restores_every_private_bit() {
    let o = support::rehash(&include_str!("fixtures/inc8.bisa").replace("0.8", "0.9"));
    let library = o.bound_ref_library().unwrap();
    for b in 0..=4 {
        let context = json::parse(&format!(r#"{{"token":["{b}","0","0","0"]}}"#)).unwrap();
        let args = json::parse(r#"["x","t","f"]"#).unwrap();
        let invocation = OracleInvocation::capture(
            &o,
            "inc8",
            Some(&context),
            Budget::steps(b),
            args.as_arr().unwrap(),
        )
        .unwrap();
        let leaves = invocation
            .expand_source(&library, &[("x", 8), ("t", 8), ("f", 4)])
            .unwrap();
        let machine = Machine::compile(
            &permutation(&leaves).unwrap(),
            &[("x", 8), ("t", 8), ("f", 4)],
        )
        .unwrap();
        for x in 0..256 {
            let mut state = machine.state();
            machine.set(&mut state, "x", x);
            machine.set(&mut state, "t", 0xa5);
            machine.set(&mut state, "f", 11);
            let entry = state.clone();
            let expected = o
                .run_oracle("inc8", &OracleInput::bits(&[x as u128]), Budget::steps(b))
                .unwrap();
            let mut literal = OracleMachine::new(
                &o,
                "inc8",
                &OracleInput::bits(&[x as u128]),
                Budget::steps(b),
            )
            .unwrap();
            for _ in 0..b {
                literal.forward_step().unwrap();
            }
            assert_eq!(
                literal.commit(0xa5, 11).unwrap(),
                (expected.payload ^ 0xa5, expected.code ^ 11)
            );
            machine.run(&mut state, true, false).unwrap();
            assert_eq!(machine.get(&state, "x"), x);
            assert_eq!(
                machine.get(&state, "t"),
                expected.payload as u64 ^ 0xa5,
                "B={b} x={x}"
            );
            assert_eq!(
                machine.get(&state, "f"),
                expected.code as u64 ^ 11,
                "B={b} x={x}"
            );
            machine.run(&mut state, true, true).unwrap();
            assert_eq!(state, entry);
        }
        assert!(leaves.iter().any(|l| l.allocation.is_some()));
        let _typed_context: &Value = invocation.context();
    }
}

#[test]
fn source_derived_memory_uses_mapped_cells_and_preserves_them() {
    let o = support::rehash(&include_str!("fixtures/tiny_memory.bisa").replace("0.8", "0.9"));
    let lib = o.bound_ref_library().unwrap();
    let b = 8;
    let context = json::parse(r#"{"token":["8","0","0","0"]}"#).unwrap();
    let args = json::parse(
        r#"["x","t","f",{"address":0,"cell":8,"role":"in","origin":"interface:db","cells":"db"}]"#,
    )
    .unwrap();
    let invocation = OracleInvocation::capture(
        &o,
        "tiny",
        Some(&context),
        Budget::steps(b),
        args.as_arr().unwrap(),
    )
    .unwrap();
    let leaves = invocation
        .expand_source(&lib, &[("x", 1), ("t", 1), ("f", 4), ("db", 8)])
        .unwrap();
    let machine = Machine::compile(
        &permutation(&leaves).unwrap(),
        &[("x", 1), ("t", 1), ("f", 4), ("db", 8)],
    )
    .unwrap();
    for x in 0..2 {
        for byte in [0, 1, 2, 127, 128, 255] {
            let input = bennett_isa_ref::OracleInput {
                ins: vec![x as u128],
                regions: std::collections::BTreeMap::from([("db".into(), vec![byte])]),
                ..Default::default()
            };
            let expected = o.run_oracle("tiny", &input, Budget::steps(b)).unwrap();
            let mut state = machine.state();
            machine.set(&mut state, "x", x);
            machine.set(&mut state, "t", 1);
            machine.set(&mut state, "f", 9);
            machine.set(&mut state, "db", byte as u64);
            let entry = state.clone();
            machine.run(&mut state, true, false).unwrap();
            assert_eq!(machine.get(&state, "t"), expected.payload as u64 ^ 1);
            assert_eq!(machine.get(&state, "f"), expected.code as u64 ^ 9);
            assert_eq!(machine.get(&state, "db"), byte as u64);
            machine.run(&mut state, true, true).unwrap();
            assert_eq!(state, entry);
        }
    }
}

#[test]
fn source_derived_lproc_captures_stack_and_complete_global_delivery() {
    let oracle =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let lp = r#"{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[1],"result":1,"tier":"q","tokens":[]}"#;
    let source=support::object(&[("caller",oracle,"oracle @caller (x:wire in) (t:wire) (f:wire[4]) () ()\nregs { r:reg[1], unused:reg[3] }\n^entry():\nr = lift.call @id (x)\nlift.retval r"),("id",lp,"lproc @id (x:reg[1]) (r:reg[1]) () () ()\n^entry():\nr = lift.mov x\nlift.ret r")],"{}","\nsection layout\n{\"A_tab\":0,\"H\":5,\"S_cells\":3,\"globals\":{},\"heap_base\":24,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9");
    let o = support::rehash(&source);
    let library = o.bound_ref_library().unwrap();
    let budget = Budget {
        steps: 4,
        heap: 0,
        stack: 5,
        depth: 1,
    };
    let context = json::parse(r#"{"token":["4","0","5","1"]}"#).unwrap();
    let args = json::parse(r#"["x","t","f"]"#).unwrap();
    let call =
        OracleInvocation::capture(&o, "caller", Some(&context), budget, args.as_arr().unwrap())
            .unwrap();
    let leaves = call
        .expand_source(&library, &[("x", 1), ("t", 1), ("f", 4)])
        .unwrap();
    let m = Machine::compile(
        &permutation(&leaves).unwrap(),
        &[("x", 1), ("t", 1), ("f", 4)],
    )
    .unwrap();
    for x in 0..2 {
        let mut state = m.state();
        m.set(&mut state, "x", x);
        m.set(&mut state, "f", 15);
        let entry = state.clone();
        let expected = o
            .run_oracle("caller", &OracleInput::bits(&[x as u128]), budget)
            .unwrap();
        assert_eq!(expected.code, 0);
        m.run(&mut state, true, false).unwrap();
        assert_eq!(m.get(&state, "x"), x);
        assert_eq!(m.get(&state, "t"), x);
        assert_eq!(m.get(&state, "f"), 15);
        m.run(&mut state, true, true).unwrap();
        assert_eq!(state, entry);
    }
}
