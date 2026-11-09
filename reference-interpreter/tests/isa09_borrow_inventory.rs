mod support;
use bennett_isa_ref::Object;

#[test]
fn checked_lifetimes_keep_repeated_transitive_and_adjoint_allocations_in_order() {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[3],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let child_signature = signature.replace("[3]", "[]");
    let root = "perm_action @root (t:wire mut) () () () ()\nregs {  } bounds {3}\n^entry():\naction_call @child (t) () () ()\nqloop 3 {\naction_call @child (t) () () ()\n}\nadj action_call @child (t) () () ()";
    let child = "perm_action @child (t:wire mut) () () () ()\n^entry():\nb = qborrow any 1\nqreturn b\nc = qborrow any 2\nqreturn c";
    let source = support::object(
        &[
            ("root", signature, root),
            ("child", &child_signature, child),
        ],
        "{}",
        "",
    )
    .replace("0.8", "0.9");
    let object = support::rehash(&source);
    let before = object.text.clone();
    // First child [1,2], three repeated children [1,2] each, adjoint [2,1].
    assert_eq!(
        object.checked_borrow_lifetimes("root").unwrap().unwrap(),
        vec![1, 2, 1, 2, 1, 2, 1, 2, 2, 1]
    );
    assert_eq!(
        object.checked_borrow_lifetimes("child").unwrap().unwrap(),
        vec![1, 2]
    );
    assert_eq!(object.text, before);
}

#[test]
fn zero_trip_has_no_lifetimes_and_qif_visits_both_unconditional_arms() {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[0],"in":[1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let body = "perm_action @a (t:wire mut) (p:wire in) () () ()\nregs {  } bounds {0}\n^entry():\nqloop 0 {\nb = qborrow any 7\nqreturn b\n}\nqif p {\nc = qborrow any 1\nqreturn c\n} {\nd = qborrow any 2\nqreturn d\n}";
    let source = support::object(&[("a", signature, body)], "{}", "").replace("0.8", "0.9");
    let object = support::rehash(&source);
    assert_eq!(
        object.checked_borrow_lifetimes("a").unwrap().unwrap(),
        vec![1, 2]
    );
}
