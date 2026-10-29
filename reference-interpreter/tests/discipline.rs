mod support;
use bennett_isa_ref::Object;
use support::object;
const ACTION: &str =
    r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
const ORACLE: &str = r#"{"abi":"0.8","args":["token"],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#;
const INC: &str =
    r#"{"abi":"0.8","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#;
fn action(body: &str) -> String {
    object(
        &[(
            "a",
            ACTION,
            &format!("action @a (t:wire mut) (p:wire in) () () ()\n^entry():\n{body}"),
        )],
        "{}",
        "",
    )
}
fn reject(text: &str, rule: &str) {
    let error = match Object::parse(text){Err(error)=>error,Ok(_)=>panic!("ill-formed cleanup accepted")};
    assert!(error.to_string().contains(rule), "{error}");
}
#[test]
fn other_scratch_requires_the_exact_adjoint_but_accepts_h_h() {
    Object::parse(&action("d = qalloc any 1\nh d\nh d\nqfree d")).unwrap();
    reject(&action("d = qalloc any 1\nh d\ns d\nh d\nqfree d"), "§6.2");
    Object::parse(&action("d = qalloc any 1\nh d\ns d\nsdg d\nh d\nqfree d")).unwrap();
    Object::parse(&action("d = qalloc any 1\nrx 0.7 d\nrx -0.7 d\nqfree d")).unwrap();
}
#[test]
fn inherited_control_cannot_be_a_callee_diagonal_target() {
    let parent="action @parent (t:wire mut) (p:wire in) () () ()\n^entry():\nqpred p\naction_call @child (t) (p) () ()\nqunpred p";
    let child = "action @child (t:wire mut) (p:wire in) () () ()\n^entry():\ns p\nx t";
    reject(
        &object(
            &[("parent", ACTION, parent), ("child", ACTION, child)],
            "{}",
            "",
        ),
        "§3.3",
    );
    let legal = child.replace("s p\nx t", "cx p t");
    Object::parse(&object(
        &[("parent", ACTION, parent), ("child", ACTION, &legal)],
        "{}",
        "",
    ))
    .unwrap();
}
#[test]
fn matched_oracle_cleanup_rejects_input_writes_and_unmatched_invocations() {
    let inc="oracle @inc (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()\nregs { r:reg[8] }\n^entry():\nr = lift.mov x\nlift.retval r";
    let build = |middle: &str| {
        object(&[("inc",INC,inc),("outer",ORACLE,&format!("oracle @outer (x:wire[8] in) (t:wire[8]) (f:wire[4]) () (tk:token)\nregs {{ input:reg[8], r:reg[8], fr:reg[4] }}\n^entry():\ninput = lift.mov x\nlift.oracle_call @inc tk (input) r fr () ()\n{middle}lift.oracle_uncall @inc tk (input) r fr () ()\nlift.retval r"))],"{}","")
    };
    Object::parse(&build("")).unwrap();
    reject(&build("input = lift.mov x\n"), "§5.5");
    reject(
        &build("").replace("lift.oracle_call @inc tk (input) r fr () ()\n", ""),
        "§5.5",
    );
    reject(
        &build("lift.oracle_uncall @inc tk (input) r fr () ()\n"),
        "§5.5",
    );
}

#[test]
fn release_cannot_consume_its_effective_control() {
    reject(
        &action("d = qalloc any 1\nqpred d\nqfree d\nqunpred d"),
        "§3.3",
    );
    reject(
        &action("d = qcopy p\nqpred d\nquncopy p d\nqunpred d"),
        "§3.3",
    );
}

#[test]
fn angles_and_function_interfaces_require_classical_argument_types(){
    reject(&action("rx p t"),"§3.3");
    let signature=r#"{"abi":"0.8","kind":"cc","params":["wire[8]"],"results":[],"tier":"cc"}"#;
    reject(&object(&[("bad",signature,"fn cc @bad (x:wire[8]) () () () ()\n^entry():\nret")],"{}",""),"§3.3");
}
#[test]
fn lproc_fault_literals_use_the_enclosing_oracles_fault_width(){
    use bennett_isa_ref::{Budget,OracleInput};
    let oracle=r#"{"abi":"0.8","args":[],"fault":5,"in":[],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let proc=r#"{"abi":"0.8","kind":"lproc","params":[],"result":1,"tier":"q"}"#;
    let layout="\nsection layout\n{\"A_tab\":0,\"H\":8,\"S_cells\":4,\"globals\":{},\"heap_base\":16,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";
    let text=object(&[("main",oracle,"oracle @main () (t:wire) (f:wire[5]) () ()\nregs { r:reg[1] }\n^entry():\nr = lift.call @p ()\nlift.retval r"),("p",proc,"lproc @p () (r:reg[1]) () () ()\n^entry():\nfault 31")],"{}",layout);
    let o=Object::parse(&text).unwrap();let r=o.run_oracle("main",&OracleInput::bits(&[]),Budget{steps:2,heap:0,stack:16,depth:8}).unwrap();assert_eq!((r.payload,r.code),(0,31));
    reject(&text.replace("fault 31","fault 32"),"§3.3");
}
