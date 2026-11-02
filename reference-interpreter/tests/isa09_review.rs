mod support;
use bennett_isa_ref::Object;
fn example(n: usize) -> String {
    let spec = include_str!("../../spec/Bennett-ISA-Spec-0.9.md");
    let heading = format!("**Example {n}**");
    let text = spec.split_once(&heading).unwrap().1;
    let body = text
        .split_once("```\n")
        .unwrap()
        .1
        .split_once("\n```")
        .unwrap()
        .0;
    assert!(body.starts_with("bennett-object"));
    format!("{body}\n")
}
#[test]
fn published_09_layout_hashes_are_admitted_without_rehashing() {
    for n in [1, 2, 8, 9] {
        let object = Object::parse(&example(n)).unwrap();
        object
            .check_hashes()
            .unwrap_or_else(|e| panic!("example {n}: {e}"));
    }
}
#[test]
fn published_09_wire_role_suffixes_are_optional_and_08_stays_strict() {
    Object::parse(&example(3)).unwrap().check_hashes().unwrap();
    let old = support::object(
        &[(
            "a",
            support::ACTION_1_1,
            "action @a (t:wire) (x:wire in) () () ()\n^entry():\ncx x t",
        )],
        "{}",
        "",
    );
    assert!(Object::parse(&old).is_err());
}
#[test]
fn imported_09_lproc_preserves_all_five_header_groups() {
    let import = r#"{"f":{"signature":{"abi":"0.9","args":["i2","ptr","f64"],"kind":"lproc","mem":[{"A":0,"W":8,"role":"in"}],"params":[1],"result":1,"tier":"q","tokens":["token"]}}}"#;
    let main = r#"{"abi":"0.9","kind":"cc","params":[],"results":[],"tier":"cc"}"#;
    let text = support::object(
        &[("main", main, "fn cc @main () () () () ()\n^entry():\nret")],
        import,
        "",
    )
    .replace("=0.8", "=0.9");
    Object::parse(&text).unwrap();
}

#[test]
fn isa09_oracle_input_role_is_optional_but_region_roles_remain_required() {
    let sig =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let source = support::object(
        &[(
            "o",
            sig,
            "oracle @o (x:wire) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x",
        )],
        "{}",
        "",
    )
    .replace("=0.8", "=0.9");
    Object::parse(&source).unwrap();
    assert!(Object::parse(&source.replace("x:wire)", "x:wire mut)")).is_err());
}
