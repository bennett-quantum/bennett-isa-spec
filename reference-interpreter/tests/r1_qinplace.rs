//! Council round 1: codex B07 (§4.5 qinplace conditions are checked before use).
mod support;
use bennett_isa_ref::{Argument, Budget, Object, QuantumBindings, QuantumState};

const ORACLE1: &str =
    r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
const ACTION: &str = r#"{"abi":"0.8","args":["token"],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1,1],"tier":"q"}"#;
fn object(g: &str, ginv: &str, body: &str) -> Object {
    let zero = "oracle @zero (x:wire in) (t:wire) (f:wire[4]) () ()\nregs { z:reg[1] }\n^entry():\nz = lift.const.i1 0\nlift.retval z";
    let not = "oracle @not (x:wire in) (t:wire) (f:wire[4]) () ()\nregs { one:reg[1], r:reg[1] }\n^entry():\none = lift.const.i1 1\nr = lift.xor x one\nlift.retval r";
    let action = format!(
        "action @a (t:wire mut, c:wire mut) () () (tk:token) ()\n^entry():\n{}",
        body.replace("G", g).replace("H", ginv)
    );
    support::rehash(&support::object(
        &[
            ("zero", ORACLE1, zero),
            ("not", ORACLE1, not),
            ("a", ACTION, &action),
        ],
        "{}",
        "",
    ))
}
/// Runs @a on |t c> with token budget B; returns the error or (t, c).
fn run(o: &Object, t: u128, c: u128, steps: u64, adjoint: bool) -> Result<(u128, u128), String> {
    let mut q = QuantumState::new();
    let tw = q.allocate(1).unwrap();
    let cw = q.allocate(1).unwrap();
    for (w, v) in [(&tw, t), (&cw, c)] {
        if v == 1 {
            q.gate("x", w, None, &[]).unwrap();
        }
    }
    let before = q.clone();
    let mut b = QuantumBindings::default();
    b.bundles.insert("t".into(), tw.clone());
    b.bundles.insert("c".into(), cw.clone());
    b.args
        .insert("tk".into(), Argument::Token(Budget::steps(steps)));
    match o.execute_action("a", &mut q, &b, adjoint, &[]) {
        Err(e) => {
            // Rejected before any state change.
            assert_eq!(q, before);
            Err(e.to_string())
        }
        Ok(()) => Ok((q.read_basis(&tw).unwrap(), q.read_basis(&cw).unwrap())),
    }
}

#[test]
fn b07_zero_zero_pair_is_rejected_before_mutation() {
    let o = object("@zero", "@zero", "qinplace G H tk t");
    for t in 0..2 {
        // B = 1: zero needs two rows, so FAULT_BUDGET_STEPS on every input.
        assert!(run(&o, t, 0, 1, false).unwrap_err().contains("§4.5"));
        // B = 2: fault-free, but zero∘zero is not the identity.
        assert!(run(&o, t, 0, 2, false).unwrap_err().contains("§4.5"));
    }
}
#[test]
fn b07_not_not_pair_executes_controlled_and_adjoint() {
    let o = object("@not", "@not", "qinplace G H tk t");
    for t in 0..2 {
        // not needs three rows: B = 2 faults, B = 3 inverts t.
        assert!(run(&o, t, 0, 2, false).unwrap_err().contains("§4.5"));
        assert_eq!(run(&o, t, 0, 3, false).unwrap(), (1 - t, 0));
        assert_eq!(run(&o, t, 0, 3, true).unwrap(), (1 - t, 0));
    }
    let o = object("@not", "@not", "qpred c\nqinplace G H tk t\nqunpred c");
    for t in 0..2 {
        for c in 0..2 {
            assert_eq!(run(&o, t, c, 3, false).unwrap(), (t ^ c, c));
        }
    }
}
#[test]
fn b07_interfaces_are_checked_at_load() {
    let wide =
        r#"{"abi":"0.8","args":[],"fault":5,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let five = "oracle @five (x:wire in) (t:wire) (f:wire[5]) () ()\n^entry():\nlift.retval x";
    let not = "oracle @not (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x";
    let action = "action @a (t:wire mut, c:wire mut) () () (tk:token) ()\n^entry():\nqinplace @not @five tk t";
    let e = Object::parse(&support::object(
        &[
            ("five", wide, five),
            ("not", ORACLE1, not),
            ("a", ACTION, action),
        ],
        "{}",
        "",
    ))
    .unwrap_err()
    .to_string();
    assert!(e.contains("§4.5"), "{e}");
}

#[test]
fn r2_qinplace_explicit_adjoint_cleans_workspace() {
    for (oracle_body, steps) in [
        ("lift.retval x", 1),
        (
            "one = lift.const.i1 1\nr = lift.xor x one\nlift.retval r",
            3,
        ),
    ] {
        let g = format!("oracle @g (x:wire in) (t:wire) (f:wire[4]) () ()\nregs {{ one:reg[1], r:reg[1] }}\n^entry():\n{oracle_body}");
        let gi = g.replace("@g ", "@gi ");
        let body = "action @a (t:wire mut, c:wire mut) () () (tk:token) ()\n^entry():\nd = qalloc any 1\nqxor t d\nqinplace @g @gi tk d\nqinplace @gi @g tk d\nqxor t d\nqfree d";
        let build = |body: &str| {
            support::object(
                &[
                    ("g", ORACLE1, &g),
                    ("gi", ORACLE1, &gi),
                    ("a", ACTION, body),
                ],
                "{}",
                "",
            )
        };
        let o = support::rehash(&build(body));
        for t in 0..2 {
            for c in 0..2 {
                assert_eq!(run(&o, t, c, steps, false).unwrap(), (t, c));
                assert_eq!(run(&o, t, c, steps, true).unwrap(), (t, c));
            }
        }
        // Same text pair but different captured effective-control versions.
        let bad = body.replace("qinplace @g @gi tk d\nqinplace @gi @g tk d", "qpred c\nqinplace @g @gi tk d\nqunpred c\nx c\nqpred c\nqinplace @gi @g tk d\nqunpred c");
        let e = Object::parse(&build(&bad)).unwrap_err();
        assert!(e.to_string().contains("§6.2"), "{e}");
    }
}

#[test]
fn r2_qinplace_complementary_arms_do_not_cancel() {
    let g = "oracle @g (x:wire in) (t:wire) (f:wire[4]) () ()\nregs { one:reg[1], r:reg[1] }\n^entry():\none = lift.const.i1 1\nr = lift.xor x one\nlift.retval r";
    let a = "action @a (t:wire mut, c:wire mut) () () (tk:token) ()\n^entry():\nd = qalloc any 1\nqif c {\nqinplace @g @g tk d\n} {\nqinplace @g @g tk d\n}\nqfree d";
    let e = Object::parse(&support::object(
        &[("g", ORACLE1, g), ("a", ACTION, a)],
        "{}",
        "",
    ))
    .unwrap_err();
    assert!(e.to_string().contains("§6.2"), "{e}");
}
