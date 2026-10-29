//! Council round 1: fable R1-B3, codex B03, B04, B05, muse B5 (§6.1, §6.2, §4.6, §5.1).
mod support;
use bennett_isa_ref::{Object, QuantumBindings, QuantumState};
use support::object;

const TWO_MUT: &str = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1,1],"tier":"q"}"#;
fn two_mut(body: &str) -> String {
    object(
        &[(
            "a",
            TWO_MUT,
            &format!("action @a (t:wire mut, m:wire mut) () () () ()\n^entry():\n{body}"),
        )],
        "{}",
        "",
    )
}
/// action @a (t:wire mut) (p:wire in), with optional loop bounds.
fn action(body: &str, bounds: &[u64]) -> String {
    let values = bounds.iter().map(u64::to_string).collect::<Vec<_>>();
    let signature = format!(
        r#"{{"abi":"0.8","args":[],"bounds":[{}],"in":[1],"kind":"action","mem":[],"mut":[1],"tier":"q"}}"#,
        values.join(",")
    );
    let regs = if bounds.is_empty() {
        String::new()
    } else {
        format!("regs {{  }} bounds {{{}}}\n", values.join(", "))
    };
    object(
        &[(
            "a",
            &signature,
            &format!("action @a (t:wire mut) (p:wire in) () () ()\n{regs}^entry():\n{body}"),
        )],
        "{}",
        "",
    )
}
fn reject(text: &str, rule: &str) {
    match Object::parse(text) {
        Err(e) => assert!(e.to_string().contains(rule), "{e} must cite {rule}"),
        Ok(_) => panic!("ill-formed cleanup accepted:\n{text}"),
    }
}
fn accept(text: &str) -> Object {
    Object::parse(text).unwrap_or_else(|e| panic!("{e}\n{text}"))
}
/// Executes @a forward and adjoint on every basis input of (t, p) and checks
/// the stated map, the absence of workspace and the exact adjoint.
fn run_basis(o: &Object, map: impl Fn(u128, u128) -> u128) {
    for t in 0..2u128 {
        for p in 0..2u128 {
            let mut q = QuantumState::new();
            let tw = q.allocate(1).unwrap();
            let pw = q.allocate(1).unwrap();
            if t == 1 {
                q.gate("x", &tw, None, &[]).unwrap();
            }
            if p == 1 {
                q.gate("x", &pw, None, &[]).unwrap();
            }
            let before = q.clone();
            let mut b = QuantumBindings::default();
            b.bundles.insert("t".into(), tw.clone());
            b.bundles.insert("p".into(), pw.clone());
            o.execute_action("a", &mut q, &b, false, &[]).unwrap();
            assert_eq!(q.read_basis(&tw).unwrap(), map(t, p), "t={t} p={p}");
            assert_eq!(q.read_basis(&pw).unwrap(), p);
            assert_eq!(q.wire_count(), 2, "no workspace survives");
            o.execute_action("a", &mut q, &b, true, &[]).unwrap();
            assert_eq!(q, before);
        }
    }
}

#[test]
fn fable_r1_b3_predicate_flip_closes_the_window() {
    // §6.1 qcopy: S = {s} ∪ Π; §6.2 window closes on a permutation change to a
    // wire in S not undone before the cleanup. On m=1,t=1: d=1, m flips to 0,
    // the cleanup is disabled and d is released dirty.
    reject(
        &two_mut("qpred m\nd = qcopy t\nqunpred m\nx m\nqpred m\nquncopy t d\nqunpred m"),
        "§6.2",
    );
    reject(
        &two_mut("d = qalloc any 1\nqpred m\ncx t d\nqunpred m\nx m\nqpred m\ncx t d\nqunpred m\nqfree d"),
        "§6.2",
    );
    accept(&two_mut(
        "qpred m\nd = qcopy t\nqunpred m\nx m\nx m\nqpred m\nquncopy t d\nqunpred m",
    ));
    accept(&two_mut(
        "qpred m\nd = qcopy t\nqunpred m\nqpred m\nquncopy t d\nqunpred m\nx m",
    ));
}

#[test]
fn codex_b03_predicate_versions_and_windows_start_at_the_computation() {
    // d = t xor not(t) = 1 for every t: the identical text does not cancel.
    reject(
        &action(
            "d = qalloc any 1\nqpred t\nx d\nqunpred t\nx t\nqpred t\nx d\nqunpred t\nqfree d",
            &[],
        ),
        "§6.2",
    );
    // A non-diagonal gate before the computation does not close a later window.
    let o = accept(&action("h t\nh t\nd = qcopy t\nquncopy t d", &[]));
    run_basis(&o, |t, _| t);
}

#[test]
fn codex_b04_qif_arms_are_complementary_and_loops_compose() {
    // Exactly one arm runs: d = 1 for every p (§4.6 qif expansion).
    reject(
        &action("d = qalloc any 1\nqif p {\nx d\n} {\nx d\n}\nqfree d", &[]),
        "§6.2",
    );
    let o = accept(&action(
        "d = qalloc any 1\nqloop 2 {\nx d\n}\nqfree d",
        &[2],
    ));
    run_basis(&o, |t, _| t);
    // Two qif with the same polarity cancel: X under p, twice.
    let o = accept(&action(
        "d = qalloc any 1\nqif p {\nx d\n} {\ns t\n}\nqif p {\nx d\n} {\nsdg t\n}\nqfree d",
        &[],
    ));
    run_basis(&o, |t, _| t);
}

#[test]
fn codex_b05_quantum_memory_cleanup_and_restoration() {
    reject(
        &action("qregion r 1 1\nd = qload r p\nx d\nqunload r p d", &[]),
        "§6.2",
    );
    reject(&action("qregion r 1 1\nqstore r p t", &[]), "§5.1");
    let o = accept(&action(
        "qregion r 1 1\nd = qload r p\ncx d t\nqunload r p d",
        &[],
    ));
    // The zero image: every cell is 0, so t is unchanged.
    run_basis(&o, |t, _| t);
    // qstore xors t into cell p, cx p t makes t' = t xor p, the second qstore
    // xors t' into the same cell: cell p = p, not the zero image when p = 1.
    reject(
        &action("qregion r 1 1\nqstore r p t\ncx p t\nqstore r p t", &[]),
        "§5.1",
    );
    let o = accept(&action("qregion r 1 1\nqstore r p t\nqstore r p t", &[]));
    run_basis(&o, |t, _| t);
}

#[test]
fn codex_b05_child_call_advances_loaded_cells() {
    let child_sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"action","mem":[{"A":1,"W":1,"role":"mut"}],"mut":[1],"tier":"q"}"#;
    let parent_sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let child =
        "action @child (t:wire mut) (p:wire in) (m:region[1,1] mut) () ()\n^entry():\nqstore m p t";
    let layout = "\nsection layout\n{\"A_tab\":0,\"H\":0,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";
    let parent = |middle: &str| {
        object(
            &[
                ("child", child_sig, child),
                (
                    "a",
                    parent_sig,
                    &format!("action @a (t:wire mut) (p:wire in) () () ()\n^entry():\nqregion r 1 1\nd = qload r p\n{middle}qunload r p d"),
                ),
            ],
            "{}",
            layout,
        )
    };
    reject(
        &parent("action_call @child (t) (p) (r) ()\nadj action_call @child (t) (p) (r) ()\n"),
        "§6.2",
    );
}

#[test]
fn muse_b5_loop_bodies_are_validated_per_iteration() {
    // The first iteration's copy is never cleaned (and d is produced twice).
    let text = action("qloop 2 {\nd = qcopy p\n}\nquncopy p d", &[2]);
    match Object::parse(&text) {
        Err(e) => assert!(
            e.to_string().contains("§6.2") || e.to_string().contains("§3.3"),
            "{e}"
        ),
        Ok(_) => panic!("loop leaks a perm value"),
    }
    let o = accept(&action(
        "qloop 2 {\nd = qcopy p\ncx d t\nquncopy p d\n}",
        &[2],
    ));
    // Two iterations of t ^= p: identity.
    run_basis(&o, |t, _| t);
    let o = accept(&action(
        "qloop 3 {\nd = qcopy p\ncx d t\nquncopy p d\n}",
        &[3],
    ));
    run_basis(&o, |t, p| t ^ p);
}

/// action @a (t:wire mut, m:wire mut) (p:wire in): the fuzzer's interface.
fn tmp(body: &str) -> String {
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"action","mem":[],"mut":[1,1],"tier":"q"}"#;
    object(
        &[(
            "a",
            sig,
            &format!("action @a (t:wire mut, m:wire mut) (p:wire in) () () ()\n^entry():\n{body}"),
        )],
        "{}",
        "",
    )
}
#[test]
fn fuzz_witnesses_of_entangling_reads_are_rejected() {
    // Found by tests/r1_fuzz_discipline.rs; each releases d0 dirty at run time.
    // (1) t reads d0 through the perm value d1 (transitive dependency).
    reject(
        &tmp("d0 = qalloc any 1\nd1 = qcopy p\nh d0\nrz 0.5 d0\nccx d0 p d1\ncx d1 t\nccx d0 p d1\nrz -0.5 d0\nh d0\nquncopy p d1\nqfree d0"),
        "§6.2",
    );
    // (2) the same through a predicate instead of a control.
    reject(
        &tmp("d0 = qalloc any 1\nd1 = qcopy m\nh d0\ncx d0 d1\nqpred d1\nx t\nqunpred d1\ncx d0 d1\nh d0\nquncopy m d1\nqfree d0"),
        "§6.2",
    );
    // (3) a diagonal phase on a perm value that depends on d0 kicks back onto d0.
    reject(
        &tmp("d0 = qalloc any 1\nd1 = qcopy t\nh d0\ncx d0 d1\ncz d1 t\ncx d0 d1\nh d0\nquncopy t d1\nqfree d0"),
        "§6.2",
    );
    // (4) a flipped effective-control wire between t and tdg on an `other` value.
    reject(
        &tmp("d0 = qalloc any 1\nry 0.75 d0\nqpred t\nqpred m\nt d0\nqunpred m\nqunpred t\nx t\nqpred t\nqpred m\ntdg d0\nqunpred m\nqunpred t\nry -0.75 d0\nqfree d0"),
        "§6.2",
    );
    // Positive controls: the readers are undone before the adjoint.
    accept(&tmp("d0 = qalloc any 1\nd1 = qcopy p\nh d0\nccx d0 p d1\ncx d1 t\ncx d1 t\nccx d0 p d1\nh d0\nquncopy p d1\nqfree d0"));
    accept(&tmp("d0 = qalloc any 1\nd1 = qcopy t\nh d0\ncx d0 d1\ncz d1 t\ncz d1 t\ncx d0 d1\nh d0\nquncopy t d1\nqfree d0"));
}

#[test]
fn r2_regions_are_lexical_function_entry_declarations() {
    for (body, bounds) in [
        ("qloop 2 {\nqregion r 1 1\n}", vec![2]),
        ("d = qload r p\nqunload r p d\nqregion r 1 1", vec![]),
        ("d = qload r p\nqunload r p d\nqloop 0 {\nqregion r 1 1\n}", vec![0]),
        ("qif p {\nqregion r 1 1\n} {\nd = qload r p\nqunload r p d\n}", vec![]),
    ] {
        run_basis(&accept(&action(body, &bounds)), |t, _| t);
    }
    reject(&action("qregion r 1 1\nqloop 0 {\nqregion r 1 1\n}", &[0]), "§3.3");
    // Exercise borrow Compiler too: region use before a zero-trip declaration.
    let text = action("b = qborrow any 1\nd = qload r p\nqxor d b\nqxor d b\nqunload r p d\nqreturn b\nqloop 0 {\nqregion r 1 1\n}", &[0]);
    accept(&text);
}

#[test]
fn r2_exchanges_preserve_transitive_entangling_read_dependencies() {
    for exchange in ["swap", "qexch"] {
        let body = format!("d = qalloc any 1\nh d\n{exchange} d m\ncx m t\n{exchange} d m\nh d\nqfree d");
        reject(&tmp(&body), "§6.2");
        let good = body.replace("cx m t", "cx m t\ncx m t");
        run_three_identity(&accept(&tmp(&good)));
    }
    let body = "qregion r 1 1\nd = qalloc any 1\nh d\nqexchm r p d\nz = qload r p\ncx z t\nqunload r p z\nqexchm r p d\nh d\nqfree d";
    reject(&tmp(body), "§6.2");
    run_three_identity(&accept(&tmp(&body.replace("cx z t", "cx z t\ncx z t"))));
}
fn run_three_identity(o: &Object) {
    for input in 0..9u32 {
        let mut q = QuantumState::new();
        let wires = q.allocate(3).unwrap();
        for (bit,w) in wires.iter().enumerate() {
            if input == 8 { q.gate("h", &[*w], None, &[]).unwrap(); }
            else if input >> bit & 1 == 1 { q.gate("x", &[*w], None, &[]).unwrap(); }
        }
        let before = q.clone();
        let mut b = QuantumBindings::default();
        for (n,w) in ["t", "m", "p"].iter().zip(&wires) { b.bundles.insert((*n).into(), vec![*w]); }
        o.execute_action("a", &mut q, &b, false, &[]).unwrap();
        assert_eq!(q, before);
        o.execute_action("a", &mut q, &b, true, &[]).unwrap();
        assert_eq!(q, before);
    }
}

#[test]
fn r2_exchange_inverse_must_cancel_both_outputs_together() {
    reject(&tmp("qregion r 1 1\nd0 = qalloc any 1\nd1 = qalloc any 1\nqexch d0 d1\nqexch d1 t\nx t\nqpred d0\nx t\nqunpred d0\nqexch d1 t\nqexch d0 d1\nqfree d1\nqfree d0"), "§6.2");
}
