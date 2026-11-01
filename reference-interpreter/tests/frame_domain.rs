//! Pass V / 73y: normative entry analysis versus stronger prefix reasoning.
mod support;
use bennett_isa_ref::Object;

#[test]
fn pass_v_four_council_zero_entry_fixtures_still_check() {
    for case in ["sol-oracle", "codex-oracle", "loop", "sol-qinplace"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join(format!("tests/fixtures/r9-{case}.bisa"));
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
            .arg("check")
            .arg(path)
            .output()
            .unwrap();
        assert!(out.status.success(), "{case}: {out:?}");
    }
}

#[test]
fn pass_v_polynomial_only_prefix_zero_is_outside_entry_analysis() {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1,1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let text = support::object(&[("a", signature,
        "perm_action @a (t:wire mut) (a:wire in, x:wire in) () () ()\n^entry():\nc = qalloc any 1\ncx a c\ncx x c\ncx a c\ncx x c\nb = qborrow any 1\ncx c b\nqreturn b\ncx x c\ncx a c\ncx x c\ncx a c\nqfree c")], "{}", "");
    let error = Object::parse(&text)
        .expect_err("stronger prefix algebra admitted")
        .to_string();
    assert!(error.contains("§5.6"), "{error}");
    support::reject_cli(&text, "§5.6", "pass-v-polynomial-prefix");
}

#[test]
fn pass_v_interface_and_unknown_local_are_independent_at_frame_entry() {
    let text = support::borrow_lifetime_object(
        "c = qcopy t\nb = qborrow any 1\ncx t b\ncx c b\nqreturn b\nquncopy t c",
        &[],
    );
    let error = Object::parse(&text).unwrap_err().to_string();
    assert!(error.contains("§5.6"), "{error}");
}

#[test]
fn pass_v_whole_region_certificate_cannot_waive_entry_domain() {
    let body = "d = qcopy t\nc = qalloc any 1\ncx t c\ncx d c\ncx t c\ncx d c\nb = qborrow any 1\ncx c b\nqreturn b\ncx d c\ncx t c\ncx d c\ncx t c\nqfree c\nquncopy t d";
    let invalid = support::certified_borrow_identity(body, &[]);
    assert!(Object::parse(&invalid)
        .unwrap_err()
        .to_string()
        .contains("§5.6"));
    // Exercise public certificate verification independently of parse admission.
    let mut o = Object::parse(&support::borrow_lifetime_object(
        &body.replace("cx c b", "cx c b\ncx c b"),
        &[],
    ))
    .unwrap();
    let f = o.functions.get_mut("a").unwrap();
    let at = f.blocks[0]
        .instructions
        .iter()
        .position(|i| i.op == "cx" && i.operands == ["c", "b"])
        .unwrap();
    f.blocks[0].instructions.remove(at);
    f.lines = invalid
        .split("section functions\n")
        .nth(1)
        .unwrap()
        .split("\n\nsection certificate")
        .next()
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    o.metadata.insert(
        "symbols".into(),
        support::json(
            invalid
                .split("section symbols\n")
                .nth(1)
                .unwrap()
                .lines()
                .next()
                .unwrap(),
        ),
    );
    o.metadata.insert(
        "certificate".into(),
        support::json(invalid.lines().last().unwrap()),
    );
    o.text = invalid;
    let closure = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
    assert_eq!(
        o.verify_claim(&closure, 0).unwrap(),
        bennett_isa_ref::ClaimVerdict::Proof
    );
    let error = o.verify_certificate().unwrap_err().to_string();
    assert!(error.contains("§5.6"), "{error}");
}
