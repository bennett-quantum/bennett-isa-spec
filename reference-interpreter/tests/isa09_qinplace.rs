mod support;
use bennett_isa_ref::Object;
fn published_source() -> &'static str {
    include_str!("../../spec/Bennett-ISA-Spec-0.9.md").split_once("**Example 7**").unwrap().1
        .split_once("```\n").unwrap().1.split_once("\n```").unwrap().0
}
#[test]
fn published_parameterised_qinplace_source_is_admitted_without_edits() {
    let object=Object::parse(&(published_source().to_string()+"\n")).unwrap();
    object.check_hashes().unwrap();
    assert_eq!(object.functions["addya"].instructions().next().unwrap().operands.len(),7);
}
#[test]
fn parameterised_qinplace_checks_both_signatures_and_full_operand_lists() {
    for (from,to) in [
        ("tk x (y) () (a)","tk x (x) () (a)"),
        ("tk x (y) () (a)","tk x () () (a)"),
        ("tk x (y) () (a)","tk x (y) () ()"),
        ("tk x (y) () (a)","a x (y) () (a)"),
        ("tk x (y) () (a)","tk x"),
        ("oracle @ginv (x:wire[2] in, y:wire[2] in)","oracle @ginv (x:wire[2] in, y:wire in)"),
    ] {
        let bad=published_source().replace(from,to)+"\n";
        assert!(Object::parse(&bad).is_err(),"admitted mutation {from} -> {to}");
    }
}
