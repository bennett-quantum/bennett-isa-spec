mod support;
use bennett_isa_conformance::{json, ref_catalog::Machine, ref_language::permutation};
use bennett_isa_ref::{
    ref_library::OracleInvocation, Argument, Budget, Object, OracleInput, OracleMachine,
};
#[test]
fn k65_lifted_intrinsic_cleanup_recomputes_the_delivery_in_fresh_targets() {
    let root = r#"{"abi":"0.9","args":["token"],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let inner =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let o=support::rehash(&support::object(&[("o",root,"oracle @o (x:wire in) (t:wire) (f:wire[4]) () (tk:token)\nregs { r:reg[1], fr:reg[4] }\n^entry():\nlift.oracle_call @inner tk (x) r fr () ()\nlift.oracle_uncall @inner tk (x) r fr () ()\nlift.retval x"),("inner",inner,"oracle @inner (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x")],"{}","").replace("=0.8","=0.9"));
    let id = o.identity().unwrap();
    let h = &id.interface_hashes["o"];
    let closure = id.closure_hashes["o"].as_ref().unwrap();
    let params = bennett_isa_ref::sha256_hex(br#"[["1","0","0","0"]]"#);
    let schedule = format!(
        r#"{{"bindings":{{"budget":[3,0,0,0],"closure_hash":"{closure}","epsilon":0,"inherited_controls":0,"params_hash":"{params}","region":"o","scheduler":{{"name":"lifted-cleanup","version":"1"}},"target_hash":"{}","token_aliases":[0,1],"variant":"forward"}},"entries":[{{"cleanup":{{"realisation":"recompute"}},"generated":0,"kind":"cleanup","path":[{{"instance":[],"iteration":[],"point":"entry:1","symbol":"{h}"}}],"phase":"*","step":2}}]}}"#,
        "0".repeat(64)
    );
    let o = Object::parse(&format!("{}\nsection schedule\n{schedule}\n", o.text)).unwrap();
    let args = json::parse(r#"["x","t","f",["1","0","0","0"]]"#).unwrap();
    let context = json::parse(r#"{"token":["3","0","0","0"]}"#).unwrap();
    let call = OracleInvocation::capture_with_token_aliases(
        &o,
        "o",
        Some(&context),
        Budget::steps(3),
        args.as_arr().unwrap(),
        &[0, 1],
    )
    .unwrap();
    let leaves = call
        .expand_source(
            &o.bound_ref_library().unwrap(),
            &[("x", 1), ("t", 1), ("f", 4)],
        )
        .unwrap();
    let calls = leaves
        .iter()
        .filter(|l| {
            l.source
                .as_ref()
                .unwrap()
                .steps
                .first()
                .is_some_and(|s| s.step == 2 && !s.reverse)
        })
        .flat_map(|l| &l.source.as_ref().unwrap().path)
        .filter(|e| e.point.point == "entry:1" && e.point.symbol == *h)
        .map(|e| e.call_index)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(calls, std::collections::BTreeSet::from([0, 1]));
    let m = Machine::compile(
        &permutation(&leaves).unwrap(),
        &[("x", 1), ("t", 1), ("f", 4)],
    )
    .unwrap();
    for x in 0..2 {
        for t in 0..2 {
            for f in 0..16 {
                let mut state = m.state();
                m.set(&mut state, "x", x);
                m.set(&mut state, "t", t);
                m.set(&mut state, "f", f);
                let before = state.clone();
                m.run(&mut state, true, false).unwrap();
                assert_eq!(m.get(&state, "t"), t ^ x);
                assert_eq!(m.get(&state, "f"), f);
                m.run(&mut state, true, true).unwrap();
                assert_eq!(state, before);
                let input = OracleInput {
                    ins: vec![x as u128],
                    args: vec![Argument::Token(Budget::steps(1))],
                    ..Default::default()
                };
                let result = o
                    .run_oracle_with_token_aliases("o", &input, Budget::steps(3), &[0, 1])
                    .unwrap();
                assert_eq!((result.payload, result.code), (x as u128, 0));
                let mut literal = OracleMachine::new_with_token_aliases(
                    &o,
                    "o",
                    &input,
                    Budget::steps(3),
                    &[0, 1],
                )
                .unwrap();
                let before = literal.snapshot();
                for _ in 0..3 {
                    literal.forward_step().unwrap();
                }
                assert_eq!(
                    literal.commit(t as u128, f as u128).unwrap(),
                    ((t ^ x) as u128, f as u128)
                );
                for _ in 0..3 {
                    literal.inverse_step().unwrap();
                }
                assert_eq!(literal.snapshot(), before);
            }
        }
    }
}
