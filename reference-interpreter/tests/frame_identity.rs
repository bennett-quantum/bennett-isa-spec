//! Pass VIII / 9eq: exact causal matching after primitive resolution.
mod support;
use bennett_isa_ref::{Object, QuantumBindings, QuantumState};

fn object(prefix: &str, cleanup: &str) -> String {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1,1,2],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    support::object(&[("a", signature, &format!(
        "perm_action @a (t:wire mut) (a:wire in, x:wire in, s:wire[2] in) () () ()\n^entry():\nc = qalloc any 2\nd = qalloc any 2\n{prefix}\nb = qborrow any 1\ncx c[0] b\ncx c[1] b\nqreturn b\n{cleanup}\nqfree d\nqfree c"
    ))], "{}", "")
}

fn accepts(prefix: &str, cleanup: &str, tag: &str) {
    let text = object(prefix, cleanup);
    let o = Object::parse(&text).unwrap_or_else(|e| panic!("{tag}: {e}"));
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("pass-viii-{tag}.bisa"));
    let checked = support::rehash(&text);
    std::fs::write(&path, &checked.text).unwrap();
    for cmd in ["hash", "check"] {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
            .arg(cmd)
            .arg(&path)
            .output()
            .unwrap();
        assert!(out.status.success(), "{tag}/{cmd}: {out:?}");
    }
    // These are exact identities. Check forward execution independently of
    // admission, on every interface basis input and both initial target bits.
    for input in 0..64 {
        let mut q = QuantumState::new();
        let t = q.allocate(1).unwrap();
        let a = q.allocate(1).unwrap();
        let x = q.allocate(1).unwrap();
        let s = q.allocate(2).unwrap();
        let borrowed = q.allocate(1).unwrap();
        let wires = [&t[..], &a[..], &x[..], &s[..], &borrowed[..]].concat();
        for (bit, wire) in wires.iter().enumerate() {
            if input >> bit & 1 != 0 {
                q.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        let mut bindings = QuantumBindings::default();
        bindings.borrowed.push(borrowed);
        for (name, wires) in [("t", t), ("a", a), ("x", x), ("s", s)] {
            bindings.bundles.insert(name.into(), wires);
        }
        let before = q.clone();
        o.execute_action("a", &mut q, &bindings, false, &[])
            .unwrap();
        assert_eq!(q, before, "{tag}, input={input}: forward identity");
    }
}

#[test]
fn pass_viii_resolved_x_family_ignores_source_spelling() {
    for (prefix, cleanup, tag) in [
        (
            "cx a c[0]\nmcx (a) c[0]",
            "mcx (a) c[0]\ncx a c[0]",
            "cx-mcx",
        ),
        (
            "mcx (a) c[0]\ncx a c[0]",
            "cx a c[0]\nmcx (a) c[0]",
            "mcx-cx",
        ),
        (
            "ccx a x c[0]\nmcx (x, a) c[0]",
            "mcx (x, a) c[0]\nccx a x c[0]",
            "ccx-mcx-control-set",
        ),
        (
            "qpred a\nx c[0]\nqunpred a\ncx a c[0]",
            "cx a c[0]\nqpred a\nx c[0]\nqunpred a",
            "context-own-control",
        ),
    ] {
        accepts(prefix, cleanup, tag);
    }
}

#[test]
fn pass_viii_bundle_operations_record_their_primitive_events() {
    accepts(
        "qxor s c\ncx s[1] c[1]\ncx s[0] c[0]",
        "cx s[0] c[0]\ncx s[1] c[1]\nqxor s c",
        "qxor-per-bit",
    );
    accepts(
        "qxor s d\nqexch c d\nswap c[1] d[1]\nswap c[0] d[0]",
        "swap c[0] d[0]\nswap c[1] d[1]\nqexch c d\nqxor s d",
        "qexch-per-bit",
    );
}

#[test]
fn pass_viii_resolution_keeps_controls_and_all_operand_causality() {
    for (prefix, cleanup, tag) in [
        (
            "cx a c[0]\nmcx (x) c[0]",
            "mcx (x) c[0]\ncx a c[0]",
            "different-controls",
        ),
        (
            "cx a c[0]\ncx a t\nmcx (a) c[0]",
            "mcx (a) c[0]\ncx a t\ncx a c[0]",
            "intervening-reader",
        ),
        (
            "cx a c[0]\ncx x c[0]\nmcx (a) c[0]\nmcx (x) c[0]",
            "mcx (x) c[0]\nmcx (a) c[0]\ncx x c[0]\ncx a c[0]",
            "polynomial-only",
        ),
    ] {
        support::reject_cli(
            &object(prefix, cleanup),
            "§5.6",
            &format!("pass-viii-{tag}"),
        );
    }
}
