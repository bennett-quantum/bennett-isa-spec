use bennett_isa_ref::Object;

fn action(kind: &str, body: &str, bounds: &str) -> String {
    let signature = format!(
        r#"{{"abi":"0.8","args":[],"bounds":[{bounds}],"in":[1],"kind":"{kind}","mem":[],"mut":[1],"tier":"q"}}"#
    );
    let regs = if bounds.is_empty() {
        String::new()
    } else {
        format!("regs {{  }} bounds {{{bounds}}}\n")
    };
    format!("bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{{\"0\":{{\"interface_hash\":\"{}\",\"kind\":\"{kind}\",\"name\":\"a\",\"signature\":{signature}}}}}\n\nsection functions\n{kind} @a (t:wire mut) (p:wire in) () () ()\n{regs}^entry():\n{body}\n","0".repeat(64))
}
fn bad(text: &str, rule: &str) {
    let e = Object::parse(text).expect_err("invalid action accepted");
    assert!(e.to_string().contains(rule), "{e} must cite {rule}");
}

#[test]
fn nested_structured_code_and_bounds_parse() {
    let s = action("action", "qloop 3 {\nqif p {\nx t\n} {\nh t\n}\n}", "3");
    let o = Object::parse(&s).unwrap();
    let i = &o.functions["a"].blocks[0].instructions[0];
    assert_eq!(i.bodies[0][0].bodies.len(), 2);
    bad(&s.replace("qloop 3", "qloop 4"), "§5.3");
    bad(&action("action", "qloop 3 {\nx t\n}", ""), "§5.3");
}
#[test]
fn tier_arity_aliasing_and_input_roles() {
    bad(&action("action", "mystery t", ""), "§4.8");
    bad(&action("action", "x t p", ""), "§4.8");
    bad(&action("action", "x absent", ""), "§3.3");
    bad(&action("action", "cx t t", ""), "§3.3");
    bad(&action("action", "qpred p\nx p\nqunpred p", ""), "§3.3");
    bad(&action("action", "qpred t\ncx p t\nqunpred t", ""), "§3.3");
    bad(&action("action", "x p", ""), "§5.1");
    bad(&action("perm_action", "h t", ""), "§5.3");
    bad(&action("action", "qpred p\nx t", ""), "§3.5");
    bad(&action("action", "qpred p\nx t\nqunpred t", ""), "§3.5");
    bad(&action("action", "r = measure t", ""), "§5.1");
}
#[test]
fn cleanup_windows_and_linear_lifetimes() {
    Object::parse(&action("action", "d = qcopy p\nz d\nquncopy p d", "").replace("z d", "s d"))
        .unwrap();
    bad(
        &action("action", "d = qcopy t\nh t\nquncopy t d", ""),
        "§6.2",
    );
    bad(
        &action("action", "d = qcopy t\nx t\nquncopy t d", ""),
        "§6.2",
    );
    Object::parse(&action("action", "d = qcopy t\nx t\nx t\nquncopy t d", "")).unwrap();
    bad(&action("action", "d = qcopy t", ""), "§5.1");
    bad(&action("action", "d = qcopy t\nquncopy p d", ""), "§6.2");
    bad(
        &action("action", "d = qalloc any 1\nqfree d\nx d", ""),
        "§3.3",
    );
}
#[test]
fn metadata_unknown_fields_are_errors() {
    let inc = include_str!("fixtures/inc8.bisa");
    bad(
        &format!("{inc}\nsection coverage\n{{\"entries\":[],\"extra\":0}}\n"),
        "§9.1",
    );
    bad(
        &format!("{inc}\nsection devices\n{{\"models\":{{}},\"services\":{{}},\"unknown\":0}}\n"),
        "§9.1",
    );
    bad(
        &format!("{inc}\nsection schedule\n{{\"unknown\":0}}\n"),
        "§9.1",
    );
}
