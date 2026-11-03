mod support;
use bennett_isa_ref::{Argument, Budget, Object, OracleInput, OracleMachine};
use std::collections::BTreeMap;
fn object() -> Object {
    let spec = include_str!("../../spec/Bennett-ISA-Spec-0.9.md");
    let body = spec
        .split_once("**Example 10**")
        .unwrap()
        .1
        .split_once("```\n")
        .unwrap()
        .1
        .split_once("\n```")
        .unwrap()
        .0;
    let o = Object::parse(&format!("{body}\n")).unwrap();
    o.check_hashes().unwrap();
    o
}
fn budget(steps: u64, stack: u128, depth: u128) -> Budget {
    Budget {
        steps,
        heap: 0,
        stack,
        depth,
    }
}
fn input(x: u128, a: u128, byte: u8, nested: u64) -> OracleInput {
    OracleInput {
        ins: vec![x],
        args: vec![Argument::Token(Budget::steps(nested)), Argument::Bits(a)],
        regions: BTreeMap::from([("db".into(), vec![byte])]),
    }
}
#[test]
fn published_example10_forwards_every_input_argument_and_cell_without_extra_charge() {
    let o = object();
    for x in 0..4 {
        for a in 0..4 {
            for d in 0..=255 {
                let input = input(x, a, d, 6);
                let want = ((x + a + u128::from(d & 3)) & 3) ^ 1;
                let out = o
                    .run_oracle_with_token_aliases("through", &input, budget(8, 8, 1), &[0, 1])
                    .unwrap();
                assert_eq!((out.payload, out.code), (want, 0), "x={x} a={a} d={d}");
                let mut machine = OracleMachine::new_with_token_aliases(
                    &o,
                    "through",
                    &input,
                    budget(8, 8, 1),
                    &[0, 1],
                )
                .unwrap();
                let entry = machine.snapshot();
                for _ in 0..8 {
                    machine.forward_step().unwrap();
                }
                assert_eq!(machine.commit(2, 13).unwrap(), (want ^ 2, 13));
                for _ in 0..8 {
                    machine.inverse_step().unwrap();
                }
                assert_eq!(machine.snapshot(), entry);
            }
        }
    }
}
#[test]
fn forwarded_nested_faults_and_outer_boundaries_reverse_the_complete_prefix() {
    let o = object();
    for (outer, nested, code) in [
        (budget(7, 8, 1), 6, 1),
        (budget(8, 7, 1), 6, 3),
        (budget(8, 8, 0), 6, 3),
        (budget(5, 8, 1), 5, 9),
    ] {
        let input = input(3, 2, 255, nested);
        let out = o
            .run_oracle_with_token_aliases("through", &input, outer, &[0, 1])
            .unwrap();
        assert_eq!((out.payload, out.code), (0, code));
        let mut machine =
            OracleMachine::new_with_token_aliases(&o, "through", &input, outer, &[0, 1]).unwrap();
        let entry = machine.snapshot();
        for _ in 0..outer.steps {
            machine.forward_step().unwrap();
        }
        assert_eq!(machine.commit(3, 10).unwrap(), (3, 10 ^ code));
        for _ in 0..outer.steps {
            machine.inverse_step().unwrap();
        }
        assert_eq!(machine.snapshot(), entry);
    }
}

#[test]
fn source_constructor_binds_lproc_tokens_values_and_region_cells_per_instance() {
    use bennett_isa_conformance::{json, ref_catalog::Machine, ref_language::permutation};
    use bennett_isa_ref::ref_library::OracleInvocation;
    // Keep the published 32-bit example exhaustive in the literal tests.
    // A compact source circuit separately checks each fixed-binding channel.
    let o = compact_source_object();
    let args=json::parse(r#"["x","t","f",{"address":0,"cell":8,"role":"in","origin":"interface:db","cells":"db"},["2","0","0","0"],"1"]"#).unwrap();
    let context = json::parse(r#"{"token":["6","0","7","1"]}"#).unwrap();
    let call = OracleInvocation::capture_with_token_aliases(
        &o,
        "through",
        Some(&context),
        budget(6, 7, 1),
        args.as_arr().unwrap(),
        &[0, 1],
    )
    .unwrap();
    let leaves = call
        .expand_source_with_leaf_limit(
            &o.bound_ref_library().unwrap(),
            &[("x", 1), ("t", 1), ("f", 4), ("db", 8)],
            2_000_000,
        )
        .unwrap();
    eprintln!(
        "compact forwarded source expansion: {} leaves",
        leaves.len()
    );
    let hash = o.identity().unwrap().closure_hashes["worker"]
        .clone()
        .unwrap();
    assert!(leaves.iter().any(|l| l
        .source
        .as_ref()
        .is_some_and(|s| s.terminal.symbol == hash && s.terminal.instance == [0, 1, 0])));
    assert!(leaves.iter().any(
        |l| l.source.as_ref().is_some_and(|s| s
            .path
            .iter()
            .any(|e| e.point.symbol == hash && e.point.instance == [0, 1, 0]))
    ));
    for leaf in &leaves {
        let source = leaf.source.as_ref().unwrap();
        let root = source.invocations[0].tokens.as_ref().unwrap();
        if let Some(child) = source.invocations.get(1) {
            assert_eq!(
                child.tokens.as_ref().unwrap().primary,
                root.arguments.values().next().copied(),
                "lifted fixed binding retains the root token identity"
            );
            assert_eq!(child.binding.as_ref().unwrap().token_aliases, [0]);
        }
    }
    let m = Machine::compile(
        &permutation(&leaves).unwrap(),
        &[("x", 1), ("t", 1), ("f", 4), ("db", 8)],
    )
    .unwrap();
    for x in 0..2 {
        for d in [0u8, 1, 127, 128, 255] {
            let mut state = m.state();
            m.set(&mut state, "x", x);
            m.set(&mut state, "t", 1);
            m.set(&mut state, "f", 13);
            m.set(&mut state, "db", d as u64);
            let entry = state.clone();
            m.run(&mut state, true, false).unwrap();
            assert_eq!(m.get(&state, "x"), x);
            assert_eq!(m.get(&state, "db"), d as u64);
            assert_eq!(m.get(&state, "t"), x);
            assert_eq!(m.get(&state, "f"), 13);
            m.run(&mut state, true, true).unwrap();
            assert_eq!(state, entry);
        }
    }
}

fn compact_source_object() -> Object {
    let root = r#"{"abi":"0.9","args":["token","i1"],"fault":4,"in":[1],"kind":"oracle","mem":[{"A":0,"W":8,"role":"in"}],"tgt":1,"tier":"q"}"#;
    let worker = r#"{"abi":"0.9","args":["i1"],"kind":"lproc","mem":[{"A":0,"W":8,"role":"in"}],"params":[1],"result":1,"tier":"q","tokens":["token"]}"#;
    let leaf = r#"{"abi":"0.9","args":["i1"],"fault":4,"in":[1],"kind":"oracle","mem":[{"A":0,"W":8,"role":"in"}],"tgt":1,"tier":"q"}"#;
    support::rehash(&support::object(&[("through",root,"oracle @through (x:wire in) (t:wire) (f:wire[4]) (db:region[0,8] in) (tk:token, a:i1)\nregs { r:reg[1] }\n^entry():\nr = lift.call @worker (x) (tk) (a) (db)\nlift.retval r"),("worker",worker,"lproc @worker (x:reg[1]) (r:reg[1]) (tk:token) (a:i1) (cell:region[0,8] in)\nregs { fr:reg[4], bit:reg[1] }\n^entry():\nlift.oracle_call @leaf tk (x) r fr (cell) (a)\nbit = lift.trunc.i1 fr\nr = lift.xor r bit\nlift.ret r"),("leaf",leaf,"oracle @leaf (x:wire in) (t:wire) (f:wire[4]) (db:region[0,8] in) (a:i1)\nregs { r:reg[1] }\n^entry():\nr = lift.xor x a\nlift.retval r")],"{}","\nsection layout\n{\"A_tab\":1,\"H\":2,\"S_cells\":3,\"globals\":{},\"heap_base\":3,\"heap_limit_max\":0,\"interface_regions\":{\"leaf\":{\"db\":{\"base\":2}},\"through\":{\"db\":{\"base\":1}}},\"null_size\":1,\"ptr_width\":3}\n").replace("=0.8","=0.9").replace("ptr_width=32","ptr_width=3"))
}

fn source_object(ptr: usize) -> Object {
    let published = object();
    let layout = support::json(&format!(
        r#"{{"A_tab":1,"H":2,"S_cells":3,"globals":{{}},"heap_base":3,"heap_limit_max":1,"interface_regions":{{"plus_db":{{"db":{{"base":2}}}},"through":{{"db":{{"base":1}}}}}},"null_size":1,"ptr_width":{ptr}}}"#
    ));
    support::rehash(
        &published
            .text
            .replace(
                &published.section("layout").unwrap().canonical(),
                &layout.canonical(),
            )
            .replace("ptr_width=32", &format!("ptr_width={ptr}"))
            .replace("base:reg[32]", &format!("base:reg[{ptr}]")),
    )
}

#[test]
fn source_forwarding_large_materialisation_reports_host_capacity_before_exhausting_memory() {
    use bennett_isa_conformance::json;
    use bennett_isa_ref::ref_library::OracleInvocation;
    let o = source_object(32);
    let args=json::parse(r#"["x","t","f",{"address":0,"cell":8,"role":"in","origin":"interface:db","cells":"db"},["6","0","0","0"],"2"]"#).unwrap();
    let context = json::parse(r#"{"token":["8","0","8","1"]}"#).unwrap();
    let call = OracleInvocation::capture_with_token_aliases(
        &o,
        "through",
        Some(&context),
        budget(8, 8, 1),
        args.as_arr().unwrap(),
        &[0, 1],
    )
    .unwrap();
    let error = call
        .expand_source(
            &o.bound_ref_library().unwrap(),
            &[("x", 2), ("t", 2), ("f", 4), ("db", 8)],
        )
        .unwrap_err();
    assert_eq!(error.kind(), bennett_isa_ref::ErrorKind::HostResource);
    assert!(error.to_string().contains("materialised leaf budget"));
}

fn dual_instance_object() -> Object {
    let root = r#"{"abi":"0.9","args":["token","token"],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let worker = r#"{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[1],"result":1,"tier":"q","tokens":["token"]}"#;
    let id =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    support::rehash(&support::object(&[("o",root,"oracle @o (x:wire in) (t:wire) (f:wire[4]) () (left:token, right:token)\nregs { r:reg[1] }\n^entry():\nqbranch x ^left ^right\n^left():\nr = lift.call @w (x) (left) () ()\nqjump ^done\n^right():\nr = lift.call @w (x) (right) () ()\nqjump ^done\n^done():\nlift.retval r"),("w",worker,"lproc @w (x:reg[1]) (r:reg[1]) (tk:token) () ()\nregs { fr:reg[4], bit:reg[1], zero:reg[4], bad:reg[1] }\n^entry():\nlift.oracle_call @id tk (x) r fr () ()\nbit = lift.trunc.i1 fr\nlift.oracle_uncall @id tk (x) r fr () ()\nzero = lift.const.i4 0\nbad = lift.icmp.ne fr zero\nlift.faultif bad 9\nlift.ret bit"),("id",id,"oracle @id (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x")],"{}","\nsection layout\n{\"A_tab\":1,\"H\":4,\"S_cells\":4,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9"))
}
#[test]
fn coherent_paths_keep_two_lproc_instances_even_when_entry_token_values_are_equal() {
    let o = dual_instance_object();
    let b = budget(11, 9, 1);
    let widths = bennett_isa_ref::DerivedWidths::derive(&o, "o", b).unwrap();
    assert_eq!((widths.p, widths.did), (5, 4));
    for (left, right) in [(0, 1), (1, 1)] {
        for x in 0..2 {
            let input = OracleInput {
                ins: vec![x],
                args: vec![
                    Argument::Token(Budget::steps(left)),
                    Argument::Token(Budget::steps(right)),
                ],
                ..Default::default()
            };
            let expected = if x == 1 {
                u128::from(left == 0)
            } else {
                u128::from(right == 0)
            };
            let out = o
                .run_oracle_with_token_aliases("o", &input, b, &[0, 1, 2])
                .unwrap();
            assert_eq!((out.payload, out.code), (expected, 0));
            let mut machine =
                OracleMachine::new_with_token_aliases(&o, "o", &input, b, &[0, 1, 2]).unwrap();
            let entry = machine.snapshot();
            for _ in 0..b.steps {
                machine.forward_step().unwrap();
            }
            assert_eq!(machine.commit(1, 13).unwrap(), (1 ^ expected, 13));
            for _ in 0..b.steps {
                machine.inverse_step().unwrap();
            }
            assert_eq!(machine.snapshot(), entry);
        }
    }
}
#[test]
fn recursive_forwarding_closes_over_two_permuted_binding_keys() {
    let root = r#"{"abi":"0.9","args":["token","token"],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let worker = r#"{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[1],"result":1,"tier":"q","tokens":["token","token"]}"#;
    let id =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let source=support::object(&[("o",root,"oracle @o (x:wire in) (t:wire) (f:wire[4]) () (left:token, right:token)\nregs { r:reg[1] }\n^entry():\nr = lift.call @w (x) (left, right) () ()\nlift.retval r"),("w",worker,"lproc @w (x:reg[1]) (r:reg[1]) (tk:token, other:token) () ()\nregs { zero:reg[1], z:reg[1], n:reg[1], fr:reg[4], bit:reg[1] }\n^entry():\nzero = lift.const.i1 0\nz = lift.icmp.eq x zero\nqbranch z ^base ^recurse\n^recurse():\nn = lift.const.i1 0\nr = lift.call @w (n) (other, tk) () ()\nlift.ret r\n^base():\nlift.oracle_call @id tk (x) r fr () ()\nbit = lift.trunc.i1 fr\nlift.oracle_uncall @id tk (x) r fr () ()\nlift.ret bit"),("id",id,"oracle @id (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x")],"{}","\nsection layout\n{\"A_tab\":1,\"H\":4,\"S_cells\":5,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9");
    let o = support::rehash(&source);
    let b = budget(15, 20, 2);
    let widths = bennett_isa_ref::DerivedWidths::derive(&o, "o", b).unwrap();
    assert_eq!((widths.p, widths.did), (5, 4));
    for x in 0..2 {
        let input = OracleInput {
            ins: vec![x],
            args: vec![
                Argument::Token(Budget::steps(0)),
                Argument::Token(Budget::steps(1)),
            ],
            ..Default::default()
        };
        let expected = x ^ 1;
        let out = o
            .run_oracle_with_token_aliases("o", &input, b, &[0, 1, 2])
            .unwrap();
        assert_eq!((out.payload, out.code), (expected, 0));
        let mut machine =
            OracleMachine::new_with_token_aliases(&o, "o", &input, b, &[0, 1, 2]).unwrap();
        let entry = machine.snapshot();
        for _ in 0..b.steps {
            machine.forward_step().unwrap();
        }
        assert_eq!(machine.commit(1, 7).unwrap(), (1 ^ expected, 7));
        for _ in 0..b.steps {
            machine.inverse_step().unwrap();
        }
        assert_eq!(machine.snapshot(), entry);
    }
}

#[test]
fn source_expansion_host_stack_limit_is_explicit_and_not_an_isa_fault() {
    use bennett_isa_conformance::json;
    use bennett_isa_ref::ref_library::OracleInvocation;
    let o = object();
    let args=json::parse(r#"["x","t","f",{"address":0,"cell":8,"role":"in","origin":"interface:db","cells":"db"},["6","0","0","0"],"2"]"#).unwrap();
    let context = json::parse(r#"{"token":["600","0","8","1"]}"#).unwrap();
    let call = OracleInvocation::capture_with_token_aliases(
        &o,
        "through",
        Some(&context),
        budget(600, 8, 1),
        args.as_arr().unwrap(),
        &[0, 1],
    )
    .unwrap();
    let error = call
        .expand_source(
            &o.bound_ref_library().unwrap(),
            &[("x", 2), ("t", 2), ("f", 4), ("db", 8)],
        )
        .unwrap_err();
    assert_eq!(error.kind(), bennett_isa_ref::ErrorKind::HostResource);
    assert!(error.to_string().contains("stack budget"));
}

#[test]
fn lproc_address_rows_resolve_the_forwarded_region_and_do_not_hash_it_as_a_symbol() {
    let root = r#"{"abi":"0.9","args":[],"fault":4,"in":[],"kind":"oracle","mem":[{"A":0,"W":8,"role":"in"}],"tgt":1,"tier":"q"}"#;
    let proc = r#"{"abi":"0.9","args":[],"kind":"lproc","mem":[{"A":0,"W":8,"role":"in"}],"params":[],"result":1,"tier":"q","tokens":[]}"#;
    let source=support::object(&[("o",root,"oracle @o () (t:wire) (f:wire[4]) (db:region[0,8] in) ()\nregs { r:reg[1] }\n^entry():\nr = lift.call @w () () () (db)\nlift.retval r"),("w",proc,"lproc @w () (r:reg[1]) () () (cell:region[0,8] in)\nregs { base:reg[32], byte:reg[8] }\n^entry():\nbase = lift.addr @cell\nbyte = lift.load.i8 base\nr = lift.trunc.i1 byte\nlift.ret r")],"{}","\nsection layout\n{\"A_tab\":1,\"H\":2,\"S_cells\":3,\"globals\":{},\"heap_base\":2,\"heap_limit_max\":0,\"interface_regions\":{\"o\":{\"db\":{\"base\":1}}},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9");
    let o = support::rehash(&source);
    o.check_hashes().unwrap();
    for d in 0..=255u8 {
        let input = OracleInput {
            regions: BTreeMap::from([("db".into(), vec![d])]),
            ..Default::default()
        };
        let b = budget(6, 6, 1);
        let result = o.run_oracle("o", &input, b).unwrap();
        assert_eq!((result.payload, result.code), (u128::from(d & 1), 0));
        let mut m = OracleMachine::new(&o, "o", &input, b).unwrap();
        let entry = m.snapshot();
        for _ in 0..b.steps {
            m.forward_step().unwrap();
        }
        assert_eq!(m.commit(1, 13).unwrap(), (1 ^ u128::from(d & 1), 13));
        for _ in 0..b.steps {
            m.inverse_step().unwrap();
        }
        assert_eq!(m.snapshot(), entry);
    }
}

#[test]
fn label_width_covers_all_fixed_binding_instances_in_the_entry_index_space() {
    let o = dual_instance_object();
    let narrow = o
        .text
        .replace("regs { fr:reg[4]", "regs { ip:reg[4], fr:reg[4]")
        .replace(
            "^entry():\nlift.oracle_call @id",
            "^entry():\nip = lift.label ^entry\nlift.oracle_call @id",
        );
    let error = Object::parse(&narrow)
        .expect_err("two distinct worker instances exceed a four-bit index space");
    assert!(error.to_string().contains("index space"));
    let wide = support::rehash(&narrow.replace("ip:reg[4]", "ip:reg[5]"));
    assert_eq!(
        bennett_isa_ref::DerivedWidths::derive(&wide, "o", budget(12, 10, 1))
            .unwrap()
            .p,
        5
    );
}

#[test]
fn forwarding_rejects_missing_lists_locations_literals_and_wrong_fixed_types() {
    let o = object();
    let original = "r = lift.call @worker (x) (tk) (a) (db)";
    for invalid in [
        "r = lift.call @worker (x)",
        "r = lift.call @worker (x) () (a) (db)",
        "r = lift.call @worker (x) (x) (a) (db)",
        "r = lift.call @worker (x) (a) (a) (db)",
        "r = lift.call @worker (x) (tk) (r) (db)",
        "r = lift.call @worker (x) (tk) (0) (db)",
        "r = lift.call @worker (x) (tk) (tk) (db)",
        "r = lift.call @worker (x) (tk) (a) (a)",
    ] {
        assert!(
            Object::parse(&o.text.replace(original, invalid)).is_err(),
            "admitted invalid forwarding: {invalid}"
        );
    }
}

#[test]
fn forwarded_i128_ptr_and_f64_keep_all_bits_including_nan_payloads() {
    let root = r#"{"abi":"0.9","args":["i128","ptr","f64"],"fault":4,"in":[],"kind":"oracle","mem":[],"tgt":128,"tier":"q"}"#;
    let worker = r#"{"abi":"0.9","args":["i128","ptr","f64"],"kind":"lproc","mem":[],"params":[],"result":128,"tier":"q","tokens":[]}"#;
    let o=support::rehash(&support::object(&[("o",root,"oracle @o () (t:wire[128]) (f:wire[4]) () (wide:i128, p:ptr, raw:f64)\nregs { r:reg[128] }\n^entry():\nr = lift.call @w () () (wide, p, raw) ()\nlift.retval r"),("w",worker,"lproc @w () (r:reg[128]) () (wide:i128, p:ptr, raw:f64) ()\nregs { v:reg[128], lo:reg[128], hi:reg[128] }\n^entry():\nv = lift.zext.i128 raw\nlo = lift.zext.i128 p\nhi = lift.xor v wide\nhi = lift.xor hi lo\nlift.ret hi")],"{}","\nsection layout\n{\"A_tab\":1,\"H\":2,\"S_cells\":3,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9"));
    let wide = (1u128 << 127) | 0x0123456789abcdef;
    let pointer = 0xfedcba98u128;
    let b = budget(7, 7, 1);
    for raw in [
        0u64,
        0x8000000000000000,
        0x3ff8000000000000,
        0x7ff0000000000000,
        0xfff0000000000000,
        0x7ff8000000000001,
        0x7ff0000000000042,
    ] {
        let input = OracleInput {
            args: vec![
                Argument::Bits(wide),
                Argument::Bits(pointer),
                Argument::Float(f64::from_bits(raw)),
            ],
            ..Default::default()
        };
        let expected = wide ^ pointer ^ u128::from(raw);
        let out = o.run_oracle("o", &input, b).unwrap();
        assert_eq!((out.payload, out.code), (expected, 0), "raw={raw:016x}");
        let mut m = OracleMachine::new(&o, "o", &input, b).unwrap();
        let entry = m.snapshot();
        for _ in 0..b.steps {
            m.forward_step().unwrap();
        }
        assert_eq!(m.commit(0x55, 13).unwrap(), (expected ^ 0x55, 13));
        for _ in 0..b.steps {
            m.inverse_step().unwrap();
        }
        assert_eq!(m.snapshot(), entry);
    }
}
