//! Council round 1: codex B02, B06, B08, B09, B10, B11, B13, B20; muse B3, B4.
mod support;
use bennett_isa_ref::{
    Argument, Budget, ClassicalMachine, ClassicalValue, Object, OracleInput, OracleMachine,
    QuantumBindings, QuantumState,
};
use support::{object, object_with_data, rehash};

fn rejects(text: &str, rule: &str) {
    match Object::parse(text) {
        Err(e) => assert!(e.to_string().contains(rule), "{e} must cite {rule}"),
        Ok(_) => panic!("ill-formed object accepted"),
    }
}
/// F_B and T2 through the small-integer adapters.
fn check(o: &Object, name: &str, input: &OracleInput, budget: Budget) -> (u128, u128, u64) {
    let r = o.run_oracle(name, input, budget).unwrap();
    let mut m = OracleMachine::new(o, name, input, budget).unwrap();
    let entry = m.snapshot();
    for _ in 0..budget.steps {
        m.forward_step().unwrap();
    }
    assert_eq!(m.outcome().unwrap(), r);
    for _ in 0..budget.steps {
        m.inverse_step().unwrap();
    }
    assert_eq!(m.snapshot(), entry);
    (r.payload, r.code, r.steps)
}
const LAYOUT0: &str = "\nsection layout\n{\"A_tab\":0,\"H\":0,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";

#[test]
fn b02_region_recursion_through_lifted_calls_is_rejected() {
    // §5.1 "recursion exists only through lift.call".
    let sig = r#"{"abi":"0.8","args":["token"],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#;
    let a = "oracle @a (x:wire[8] in) (t:wire[8]) (f:wire[4]) () (tk:token)\nregs { r:reg[8], fr:reg[4] }\n^entry():\nlift.oracle_call @a tk (x) r fr () (tk)\nlift.retval r";
    rejects(&object(&[("a", sig, a)], "{}", ""), "§5.1");
    let p_sig = r#"{"abi":"0.8","args":["token"],"bounds":[],"in":[8],"kind":"perm_action","mem":[],"mut":[8,4],"tier":"q"}"#;
    let p = "perm_action @p (t:wire[8] mut, g:wire[4] mut) (x:wire[8] in) () (tk:token) ()\n^entry():\noracle_call @o tk (x) t g () (tk)";
    let o = "oracle @o (x:wire[8] in) (t:wire[8]) (f:wire[4]) () (tk:token)\nregs { m:reg[8], g:reg[4] }\n^entry():\nlift.perm_call @p (m, g) (x) () (tk)\nlift.retval m";
    rejects(&object(&[("p", p_sig, p), ("o", sig, o)], "{}", ""), "§5.1");
    // Recursion through lift.call remains legal (§16.3 example 2).
    Object::parse(include_str!("fixtures/sum_db.bisa")).unwrap();
}

#[test]
fn b06_local_qregion_passed_to_a_child_action() {
    let parent_sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[3],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let child_sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[3],"kind":"action","mem":[{"A":3,"W":1,"role":"in"}],"mut":[1],"tier":"q"}"#;
    let parent = "action @parent (t:wire mut) (p:wire[3] in) () () ()\n^entry():\nqregion r 3 1 @img\naction_call @child (t) (p) (r) ()";
    let child = "action @child (t:wire mut) (p:wire[3] in) (r:region[3,1] in) () ()\n^entry():\nd = qload r p\ncx d t\nqunload r p d";
    let o = rehash(&object_with_data(
        &[("parent", parent_sig, parent), ("child", child_sig, child)],
        &[("img", "a5")],
        LAYOUT0,
    ));
    // Hand derivation: cell i of image a5 is bit i of 0b1010_0101; t ^= cell p.
    for p in 0..8u128 {
        for t in 0..2u128 {
            let mut q = QuantumState::new();
            let tw = q.allocate(1).unwrap();
            let pw = q.allocate(3).unwrap();
            for (bit, wire) in tw.iter().chain(&pw).enumerate() {
                if (t | (p << 1)) >> bit & 1 == 1 {
                    q.gate("x", &[*wire], None, &[]).unwrap();
                }
            }
            let before = q.clone();
            let mut b = QuantumBindings::default();
            b.bundles.insert("t".into(), tw.clone());
            b.bundles.insert("p".into(), pw.clone());
            o.execute_action("parent", &mut q, &b, false, &[]).unwrap();
            assert_eq!(q.read_basis(&tw).unwrap(), t ^ ((0xa5 >> p) & 1), "p={p}");
            assert_eq!(q.read_basis(&pw).unwrap(), p);
            o.execute_action("parent", &mut q, &b, true, &[]).unwrap();
            assert_eq!(q, before);
        }
    }
}

#[test]
fn b08_two_calls_then_two_matched_uncalls() {
    let text = include_str!("fixtures/nested.bisa").replace(
        "lift.oracle_call @inc8 tk (x) r fr () ()\nlift.oracle_uncall @inc8 tk (x) r fr () ()",
        "lift.oracle_call @inc8 tk (x) r fr () ()\nlift.oracle_call @inc8 tk (x) r fr () ()\nlift.oracle_uncall @inc8 tk (x) r fr () ()\nlift.oracle_uncall @inc8 tk (x) r fr () ()",
    );
    let o = rehash(&text);
    let mut input = OracleInput::bits(&[8]);
    input.args.push(Argument::Token(Budget::steps(3)));
    // 9 xor 9 xor 9 xor 9 = 0 in four rows plus lift.retval.
    assert_eq!(check(&o, "cancel", &input, Budget::steps(5)), (0, 0, 5));
    // A third uncall has no outstanding invocation left.
    let extra = text.replace(
        "lift.oracle_uncall @inc8 tk (x) r fr () ()\nlift.retval r",
        "lift.oracle_uncall @inc8 tk (x) r fr () ()\nlift.oracle_uncall @inc8 tk (x) r fr () ()\nlift.retval r",
    );
    rejects(&extra, "§5.5");
}

#[test]
fn b08_disjoint_global_store_between_a_matched_pair() {
    let inner_sig = r#"{"abi":"0.8","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[{"A":2,"W":8,"role":"in"}],"tgt":8,"tier":"q"}"#;
    let outer_sig = r#"{"abi":"0.8","args":["token"],"fault":4,"in":[8],"kind":"oracle","mem":[{"A":2,"W":8,"role":"in"}],"tgt":8,"tier":"q"}"#;
    let inner = "oracle @o (x:wire[8] in) (t:wire[8]) (f:wire[4]) (db:region[2,8] in) ()\nregs { b:reg[32], v:reg[8], r:reg[8] }\n^entry():\nb = lift.addr @db\nv = lift.load.i8 b\nr = lift.add x v\nlift.retval r";
    let outer = |target: &str| {
        format!("oracle @outer (x:wire[8] in) (t:wire[8]) (f:wire[4]) (db:region[2,8] in) (tk:token)\nregs {{ r:reg[8], fr:reg[4], p:reg[32], v:reg[8] }}\n^entry():\nlift.oracle_call @o tk (x) r fr (db) ()\np = lift.addr @{target}\nv = lift.const.i8 7\nlift.store.i8 p v\nlift.oracle_uncall @o tk (x) r fr (db) ()\nlift.retval r")
    };
    let layout = "\nsection layout\n{\"A_tab\":2,\"H\":8,\"S_cells\":0,\"globals\":{\"g\":{\"base\":32,\"image\":\"g\",\"size\":1}},\"heap_base\":64,\"heap_limit_max\":0,\"interface_regions\":{\"o\":{\"db\":{\"base\":16}},\"outer\":{\"db\":{\"base\":16}}},\"null_size\":16,\"ptr_width\":32}\n";
    let build = |target: &str| {
        object_with_data(
            &[
                ("o", inner_sig, inner),
                ("outer", outer_sig, &outer(target)),
            ],
            &[("g", "00")],
            layout,
        )
    };
    let o = rehash(&build("g"));
    let mut input = OracleInput::bits(&[5]);
    input.regions.insert("db".into(), vec![3, 0, 0, 0]);
    input.args.push(Argument::Token(Budget {
        steps: 4,
        heap: 0,
        stack: 0,
        depth: 0,
    }));
    let budget = Budget {
        steps: 6,
        heap: 0,
        stack: 0,
        depth: 0,
    };
    // r = (5 + 3) xor (5 + 3) = 0; the store writes g, never a db cell.
    assert_eq!(check(&o, "outer", &input, budget), (0, 0, 6));
    // A store through the address of db itself may change the supplied cells.
    rejects(&build("db"), "§5.5");
}

#[test]
fn b09_whole_run_reversal_after_an_initial_cancelled_segment() {
    let sig = r#"{"abi":"0.8","kind":"r","params":[],"results":[],"tier":"r"}"#;
    let o = rehash(&object(
        &[
            (
                "main",
                sig,
                "fn r @main () () () () ()\n^entry():\ncall @f\nuncall @f () ()\nret",
            ),
            ("f", sig, "fn r @f () () () () ()\n^entry():\nret"),
        ],
        "{}",
        "",
    ));
    let mut m = ClassicalMachine::new(&o, "main", &[], &[]).unwrap();
    let entry = m.snapshot();
    for _ in 0..2 {
        let r = m.run(4).unwrap();
        assert_eq!((r.fault, r.steps), (None, 4));
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), entry);
    }
}

#[test]
fn b10_pointer_arguments_reach_permutation_actions() {
    let flip_sig = r#"{"abi":"0.8","args":["ptr"],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let o_sig = r#"{"abi":"0.8","args":["ptr"],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let flip = "perm_action @flip (t:wire mut) () () (p:ptr) ()\n^entry():\nx t";
    let o = "oracle @o (x:wire in) (t:wire) (f:wire[4]) () (p:ptr)\nregs { r:reg[1] }\n^entry():\nlift.perm_call @flip (r) () () (p)\nlift.retval r";
    let o = rehash(&object(
        &[("flip", flip_sig, flip), ("o", o_sig, o)],
        "{}",
        "",
    ));
    let mut input = OracleInput::bits(&[0]);
    input.args.push(Argument::Bits(0));
    assert_eq!(check(&o, "o", &input, Budget::steps(2)), (1, 0, 2));
    let mut wide = OracleInput::bits(&[0]);
    wide.args.push(Argument::Bits(1 << 32));
    assert!(o.run_oracle("o", &wide, Budget::steps(2)).is_err());
}

#[test]
fn b11_classical_definitions_need_not_precede_uses_in_text() {
    let sig = r#"{"abi":"0.8","kind":"r","params":[],"results":["i8"],"tier":"r"}"#;
    let f = "fn r @f () (r:i8) () () ()\n^entry():\nbr ^def()\n^use():\nret x\n^def():\nx = const.i8 7\nbr ^use()";
    let o = rehash(&object(&[("f", sig, f)], "{}", ""));
    let mut m = ClassicalMachine::new(&o, "f", &[], &[]).unwrap();
    let entry = m.snapshot();
    let r = m.run(4).unwrap();
    assert_eq!(
        (r.fault, r.values),
        (None, vec![ClassicalValue::int(8, 7).unwrap()])
    );
    m.reverse().unwrap();
    assert_eq!(m.snapshot(), entry);
}

#[test]
fn b13_malformed_callee_is_a_diagnostic_never_a_panic() {
    let z_sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let a = "action @a (t:wire mut) (p:wire in) () () ()\n^entry():\nqpred p\naction_call @z (t) () () ()\nqunpred p";
    let z = "action @z (t:wire mut) () () () ()\n^entry():\naction_call";
    let text = object(&[("a", support::ACTION_1_1, a), ("z", z_sig, z)], "{}", "");
    rejects(&text, "§4.8");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("b13_malformed_callee.bisa");
    std::fs::write(&path, &text).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
        .args(["hash", path.to_str().unwrap()])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("§4.8"));
}

#[test]
fn b20_call_tiers_follow_the_section_3_1_graph() {
    // SPEC-QUESTIONS 19: CC calls CC or R; R calls only R; uncall per §7.4.
    let r_sig = r#"{"abi":"0.8","kind":"r","params":[],"results":["i8"],"tier":"r"}"#;
    let cc_sig = r#"{"abi":"0.8","kind":"cc","params":[],"results":["i8"],"tier":"cc"}"#;
    let helper = "fn cc @h () (r:i8) () () ()\n^entry():\nr = const.i8 3\nret r";
    rejects(
        &object(
            &[
                ("h", cc_sig, helper),
                (
                    "r",
                    r_sig,
                    "fn r @r () (v:i8) () () ()\n^entry():\nv = call @h\nret v",
                ),
            ],
            "{}",
            "",
        ),
        "§3.1",
    );
    let main = "fn cc @main () (z:i8) () () ()\n^entry():\nv = call @h\nuncall @h () (v)\nz = const.i8 1\nret z";
    let o = rehash(&object(
        &[("h", cc_sig, helper), ("main", cc_sig, main)],
        "{}",
        "",
    ));
    let mut m = ClassicalMachine::new(&o, "main", &[], &[]).unwrap();
    let entry = m.snapshot();
    let r = m.run(10).unwrap();
    assert_eq!(
        (r.fault, r.values),
        (None, vec![ClassicalValue::int(8, 1).unwrap()])
    );
    m.reverse().unwrap();
    assert_eq!(m.snapshot(), entry);
}

#[test]
fn muse_b3_effect_operations_are_validated_at_load() {
    let raw = include_str!("fixtures/classical.bisa");
    for bad in [
        "effect clock.read p p",
        "effect clock.read",
        "effect clock.write p",
        "effect console.write p",
    ] {
        rejects(
            &raw.replacen("effect clock.read p\n", &format!("{bad}\n"), 1),
            "§7.3",
        );
    }
    // An unmodelled device is legal text and faults 8 at run time (§17 item 3).
    Object::parse(raw).unwrap();
}

#[test]
fn muse_b4_qstore_value_is_a_qxor_source_and_may_be_in() {
    // Declined: §4.6 defines `qstore name a v` as "for each i: under a == i,
    // qxor v cell_i", and §5.1 admits `in` wires "as qcopy/qxor sources".
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[2],"kind":"action","mem":[],"mut":[2],"tier":"q"}"#;
    let body = "action @a (t:wire[2] mut) (p:wire[2] in) () () ()\n^entry():\nqregion r 2 2\nqstore r t p\nqstore r t p";
    Object::parse(&object(&[("a", sig, body)], "{}", "")).unwrap();
    // qexchm swaps v into a cell, so an `in` value is a modification (§5.1).
    rejects(
        &object(
            &[(
                "a",
                sig,
                &body.replace("qstore r t p\nqstore r t p", "qexchm r t p\nqexchm r t p"),
            )],
            "{}",
            "",
        ),
        "§5.1",
    );
}

#[test]
fn parameter_roles_agree_with_their_header_group() {
    // muse suggestion: (mut..) (in..) for actions, §16.2 fhead comment.
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[2],"kind":"action","mem":[],"mut":[2],"tier":"q"}"#;
    rejects(
        &object(
            &[(
                "a",
                sig,
                "action @a (t:wire[2] in) (p:wire[2] mut) () () ()\n^entry():\nx t[0]",
            )],
            "{}",
            "",
        ),
        "§16.2",
    );
    rejects(
        &include_str!("fixtures/inc8.bisa").replace("(t:wire[8])", "(t:wire[8] mut)"),
        "§16.2",
    );
    rejects(
        &include_str!("fixtures/inc8.bisa").replace(
            "regs { one:reg[8], r:reg[8] }",
            "regs { one:reg[8] in, r:reg[8] }",
        ),
        "§16.2",
    );
}
