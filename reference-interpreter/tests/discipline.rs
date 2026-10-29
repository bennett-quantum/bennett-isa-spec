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
    let error = match Object::parse(text) {
        Err(error) => error,
        Ok(_) => panic!("ill-formed cleanup accepted"),
    };
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
fn angles_and_function_interfaces_require_classical_argument_types() {
    reject(&action("rx p t"), "§3.3");
    let signature = r#"{"abi":"0.8","kind":"cc","params":["wire[8]"],"results":[],"tier":"cc"}"#;
    reject(
        &object(
            &[(
                "bad",
                signature,
                "fn cc @bad (x:wire[8]) () () () ()\n^entry():\nret",
            )],
            "{}",
            "",
        ),
        "§3.3",
    );
}
#[test]
fn lproc_fault_literals_use_the_enclosing_oracles_fault_width() {
    use bennett_isa_ref::{Budget, OracleInput};
    let oracle =
        r#"{"abi":"0.8","args":[],"fault":5,"in":[],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let proc = r#"{"abi":"0.8","kind":"lproc","params":[],"result":1,"tier":"q"}"#;
    let layout="\nsection layout\n{\"A_tab\":0,\"H\":8,\"S_cells\":4,\"globals\":{},\"heap_base\":16,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";
    let text=object(&[("main",oracle,"oracle @main () (t:wire) (f:wire[5]) () ()\nregs { r:reg[1] }\n^entry():\nr = lift.call @p ()\nlift.retval r"),("p",proc,"lproc @p () (r:reg[1]) () () ()\n^entry():\nfault 31")],"{}",layout);
    let o = Object::parse(&text).unwrap();
    let r = o
        .run_oracle(
            "main",
            &OracleInput::bits(&[]),
            Budget {
                steps: 2,
                heap: 0,
                stack: 16,
                depth: 8,
            },
        )
        .unwrap();
    assert_eq!((r.payload, r.code), (0, 31));
    reject(&text.replace("fault 31", "fault 32"), "§3.3");
}

#[test]
fn loop_cleanup_uses_all_iterations_and_lexical_bounds_once() {
    fn loop_action(body: &str, bounds: &[u64]) -> String {
        let mut text = action(body);
        let values = bounds
            .iter()
            .map(u64::to_string)
            .collect::<Vec<_>>()
            .join(",");
        text = text.replace("\"bounds\":[]", &format!("\"bounds\":[{values}]"));
        text.replace(
            "^entry():",
            &format!(
                "regs {{  }} bounds {{{}}}\n^entry():",
                values.replace(',', ", ")
            ),
        )
    }
    // X^(2^40) is identity. The verifier must not expand 2^40 iterations.
    Object::parse(&loop_action(
        "d = qalloc any 1\nqloop 1099511627776 {\nx d\n}\nqfree d",
        &[1099511627776],
    ))
    .unwrap();
    Object::parse(&loop_action(
        "qloop 3 {\nd = qalloc any 1\nqloop 2 {\nx d\n}\nqfree d\n}",
        &[3, 2],
    ))
    .unwrap();
    Object::parse(&loop_action(
        "d = qalloc any 1\nqloop 0 {\nx d\n}\nqfree d",
        &[0],
    ))
    .unwrap();
    reject(
        &loop_action("d = qalloc any 1\nqloop 3 {\nx d\n}\nqfree d", &[3]),
        "§6.2",
    );
    reject(
        &loop_action("qloop 2 {\nd = qalloc any 1\n}\nqfree d", &[2]),
        "§3.3",
    );
    reject(&loop_action("qloop 0 {\nmystery t\n}", &[0]), "§4.8");
}

#[test]
fn cleanup_adjoint_cannot_skip_an_entangling_reader() {
    reject(
        &action("d = qalloc any 1\nh d\ncx d t\nh d\nqfree d"),
        "§6.2",
    );
    Object::parse(&action(
        "d = qalloc any 1\nh d\ncx d t\ncx d t\nh d\nqfree d",
    ))
    .unwrap();
    reject(
        &action("d = qalloc any 1\nh d\nqxor d t\nh d\nqfree d"),
        "§6.2",
    );
    Object::parse(&action(
        "d = qalloc any 1\nh d\nqxor d t\nqxor d t\nh d\nqfree d",
    ))
    .unwrap();
}

#[test]
fn cleanup_pins_predicate_versions_and_distinguishes_qif_polarity() {
    reject(
        &action("qpred t\nd = qcopy p\nqunpred t\nx t\nqpred t\nquncopy p d\nqunpred t"),
        "§6.2",
    );
    Object::parse(&action(
        "qpred t\nd = qcopy p\nqunpred t\nx t\nx t\nqpred t\nquncopy p d\nqunpred t",
    ))
    .unwrap();
    // The mutually exclusive branches each flip d once: together they are X, not I.
    reject(
        &action("d = qalloc any 1\nqif p {\nx d\n} {\nx d\n}\nqfree d"),
        "§6.2",
    );
}

#[test]
fn local_regions_require_explicit_restoration_and_load_pins_cell_versions() {
    reject(&action("qregion scratch 1 1\nqstore scratch p t"), "§5.1");
    Object::parse(&action(
        "qregion scratch 1 1\nqstore scratch p t\nqstore scratch p t",
    ))
    .unwrap();
    reject(&action("qregion scratch 1 1\nd = qload scratch p\nqstore scratch p t\nqstore scratch p t\nqunload scratch p d"), "§6.2");
    reject(
        &action("qregion scratch 1 1\nd = qload scratch p\nx d\nqunload scratch p d"),
        "§6.2",
    );
}

#[test]
fn region_calls_transfer_memory_versions_and_oracles_are_self_adjoint() {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"action","mem":[{"A":1,"W":1,"role":"mut"}],"mut":[1],"tier":"q"}"#;
    let child =
        "action @child (t:wire mut) (p:wire in) (m:region[1,1] mut) () ()\n^entry():\nqstore m p t";
    let make = |tail: &str| {
        object(&[("child",signature,child),("a",ACTION,&format!("action @a (t:wire mut) (p:wire in) () () ()\n^entry():\nqregion local 1 1\naction_call @child (t) (p) (local) ()\n{tail}"))],"{}","\nsection layout\n{\"A_tab\":0,\"H\":4,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n")
    };
    reject(&make(""), "§5.1");
    Object::parse(&make("adj action_call @child (t) (p) (local) ()")).unwrap();
    let inc="oracle @inc (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()\nregs { r:reg[8] }\n^entry():\nr = lift.mov x\nlift.retval r";
    let caller = r#"{"abi":"0.8","args":["token"],"bounds":[],"in":[8],"kind":"action","mem":[],"mut":[],"tier":"q"}"#;
    let body="action @a () (x:wire[8] in) () (tk:token) ()\n^entry():\nd = qalloc any 8\nf = qalloc any 4\noracle_call @inc tk (x) d f () ()\noracle_call @inc tk (x) d f () ()\nqfree f\nqfree d";
    Object::parse(&object(&[("inc", INC, inc), ("a", caller, body)], "{}", "")).unwrap();
}

#[test]
fn coherent_while_cannot_write_selector_dependencies_even_if_restored() {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[1],"in":[1],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let text=object(&[("a",signature,"action @a (t:wire mut) (p:wire in) () () ()\nregs {  } bounds {1}\n^entry():\ncond = qcopy t\nqloop 1 while cond {\nx t\nx t\n}\nquncopy t cond")],"{}","");
    reject(&text, "§4.6");
}

#[test]
fn large_exact_loop_bounds_use_decimal_strings_in_the_signature() {
    let signature=r#"{"abi":"0.8","args":[],"bounds":["1152921504606846976"],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let body="perm_action @a (t:wire mut) () () () ()\nregs {  } bounds {1152921504606846976}\n^entry():\nqloop 1152921504606846976 {\nx t\n}";
    Object::parse(&object(&[("a",signature,body)],"{}","")).unwrap();
}
