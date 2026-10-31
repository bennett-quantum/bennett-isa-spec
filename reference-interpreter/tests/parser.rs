use bennett_isa_ref::{json::Json, Object};

const INC: &str = include_str!("fixtures/inc8.bisa");
const SUM: &str = include_str!("fixtures/sum_db.bisa");

fn rejects(text: &str, rule: &str) {
    let error = match Object::parse(text) {
        Err(e) => e,
        Ok(_) => panic!("ill-formed object accepted"),
    };
    assert!(error.to_string().contains(rule), "{error} must cite {rule}");
}

#[test]
fn published_objects_parse() {
    let inc = Object::parse(INC).unwrap();
    assert_eq!(inc.ptr_width, 32);
    assert_eq!(inc.functions["inc8"].blocks[0].instructions.len(), 3);
    let sum = Object::parse(SUM).unwrap();
    assert_eq!(sum.functions["sum"].locations().len(), 13);
    assert_eq!(sum.functions["sum"].frame_cells(), 16);
}

#[test]
fn lexical_and_section_errors_cite_rules() {
    for text in [
        INC.replace('\n', "\r\n"),
        INC.replace("lift.add x", "lift.add  x"),
        INC.replace("lift.add x one", "lift.add x one "),
        INC.replace("\n\nsection", "\nsection"),
        INC.replace("^entry():", "\n^entry():"),
    ] {
        rejects(&text, "§16.1");
    }
    rejects(&INC.replace("isa=0.8", "isa=0.9"), "§1");
    rejects(&INC.replace("section symbols", "section unknown"), "§9.1");
    rejects(&format!("{INC}\nsection symbols\n{{}}\n"), "§9.1");
    rejects(&INC.replace("{\"0\":", "{ \"0\":"), "§16.1");
}

#[test]
fn names_widths_opcodes_and_terminators_are_checked() {
    rejects(
        &INC.replace("lift.add x one", "lift.add missing one"),
        "§3.7",
    );
    rejects(&INC.replace("r = lift.add", "x = lift.add"), "§3.7");
    rejects(&INC.replace("r:reg[8]", "r:reg[7]"), "§3.3");
    rejects(
        &INC.replace("one = lift.const.i8 1", "one = lift.magic x"),
        "§4.8",
    );
    rejects(&INC.replace("lift.retval r", "r = lift.mov one"), "§3.6");
    rejects(&INC.replace("lift.retval r", "lift.ret r"), "§4.2");
    rejects(&INC.replace("lift.retval r", "qjump ^missing"), "§3.6");
    rejects(&INC.replace("lift.retval r", "fault 16"), "§3.3");
    rejects(
        &SUM.replace("idx = lift.zext.i32 n2", "idx = lift.trunc.i32 n2"),
        "§3.3",
    );
    rejects(
        &SUM.replace(
            "lift.call @sum (n, acc, base)",
            "lift.call @sum (base, acc, n)",
        ),
        "§3.3",
    );
    rejects(
        &SUM.replace("zp = lift.addr @zero", "zp = lift.addr @db"),
        "§4.2",
    );
    rejects(&SUM.replace("\"base\":4096", "\"base\":16"), "§7.2");
    rejects(&SUM.replace("\"size\":1", "\"size\":2"), "§7.2");
}

#[test]
fn json_is_strict_and_canonical() {
    let raw = r#"{"a":[null,true,false,"\n",-1,1e+30],"z":"é"}"#;
    assert_eq!(Json::parse(raw).unwrap().canonical(), raw);
    for bad in [
        "{\"a\":1,\"a\":2}",
        "[01]",
        "[1.]",
        "[NaN]",
        "[1e999]",
        "\"\u{0001}\"",
        "\"\\ud800\"",
    ] {
        assert!(Json::parse(bad).is_err(), "accepted {bad}");
    }
    assert_eq!(
        Json::parse("[-0,1e-7,1e-6,1e20,1e21]").unwrap().canonical(),
        "[0,1e-7,0.000001,100000000000000000000,1e+21]"
    );
    // RFC 8785 sorts object keys by UTF-16 units: a supplementary character
    // sorts before U+E000, despite its larger Unicode scalar value.
    assert_eq!(
        Json::parse("{\"\u{e000}\":0,\"😀\":1}")
            .unwrap()
            .canonical(),
        "{\"😀\":1,\"\u{e000}\":0}"
    );
}

#[test]
fn empty_hex_data_and_canonical_type_widths() {
    let data=format!("bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{{\"0\":{{\"interface_hash\":\"{}\",\"kind\":\"data\",\"name\":\"empty\",\"signature\":{{\"abi\":\"0.8\",\"kind\":\"data\",\"size\":0,\"tier\":\"-\"}}}}}}\n\nsection data\nempty \n","0".repeat(64));
    assert!(Object::parse(&data).unwrap().data["empty"].is_empty());
    // INT := -?(0|[1-9][0-9]*) (§16.1) governs every width token.
    rejects(&INC.replace("reg[8]", "reg[08]"), "§16.1");
}

#[test]
fn label_width_uses_reachable_closure_not_unrelated_functions() {
    let raw = include_str!("fixtures/arithmetic.bisa");
    let narrow = raw.replace(
        "z:reg[8], p:reg[1], idx:reg[32], r:reg[8]",
        "z:reg[8], p:reg[1], idx:reg[4], r:reg[8]",
    );
    // jump has ten rows, so 2^4 > 10, regardless of other independent oracles.
    let object = Object::parse(&narrow).unwrap();
    let result = object
        .run_oracle(
            "jump",
            &bennett_isa_ref::OracleInput::bits(&[0, 0]),
            bennett_isa_ref::Budget::steps(10),
        )
        .unwrap();
    assert_eq!((result.payload, result.code), (42, 0));
}

#[test]
fn every_operand_uses_canonical_int_float_string_and_index_spellings() {
    let text = include_str!("fixtures/control_flow.bisa");
    for bad in [
        "6.2831853071795860",
        "06.283185307179586",
        "+6.283185307179586",
    ] {
        rejects(&text.replace("6.283185307179586", bad), "§16.1");
    }
    rejects(&INC.replace("lift.const.i8 1", "lift.const.i8 01"), "§16.1");
    assert!(bennett_isa_ref::Type::parse("qreg[02]")
        .unwrap_err()
        .to_string()
        .contains("§16.1"));
    rejects(&text.replace("x t\n}", "cx p[00] t\n}"), "§16.1");
}

#[test]
fn quoted_service_literals_decode_for_binding_execution_and_identity() {
    let raw = include_str!("fixtures/classical.bisa");
    let quoted = raw.replace("hostcall org.bennett.test", "hostcall \"org.bennett.test\"");
    let o = Object::parse(&quoted).unwrap();
    use bennett_isa_ref::{ClassicalMachine, ClassicalValue};
    let mut machine = ClassicalMachine::new(&o, "host", &[], &[]).unwrap();
    assert_eq!(
        machine.run(100).unwrap().request.unwrap().service,
        "org.bennett.test"
    );
    machine.reply(&[7]).unwrap();
    assert_eq!(
        machine.run(100).unwrap().values,
        vec![ClassicalValue::Int(16, 1)]
    );
    o.identity().unwrap();
    // Only the three ISA escapes are available in instruction strings.
    rejects(
        &raw.replace(
            "hostcall org.bennett.test",
            "hostcall \"org\\t.bennett.test\"",
        ),
        "§16.1",
    );
}

#[test]
fn metadata_integers_preserve_exactness_and_canonical_decimal_strings() {
    assert_eq!(
        Json::parse("9007199254740991").unwrap().uint().unwrap(),
        9_007_199_254_740_991
    );
    assert_eq!(
        Json::parse("\"340282366920938463463374607431768211455\"")
            .unwrap()
            .uint()
            .unwrap(),
        u128::MAX
    );
    for bad in ["9007199254740992", "\"01\"", "\"+1\"", "\"-0\"", "\"1.0\""] {
        assert!(Json::parse(bad).unwrap().uint().is_err(), "accepted {bad}");
    }
}

mod support;
#[test]
fn r3_normalized_cc_controls_are_typed_and_disjoint_at_load() {
    let ac = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    for (tag, ty, control) in [
        ("integer", "i8", "c"),
        ("token", "token", "c"),
        ("unknown", "qreg[1]", "missing"),
        ("multiwire", "qreg[2]", "c"),
        ("alias", "qreg[1]", "q"),
    ] {
        let cc = format!(
            r#"{{"abi":"0.8","kind":"cc","params":["{ty}","qreg[1]"],"results":[],"tier":"cc"}}"#
        );
        let text = support::object(&[("a",ac,"action @a (t:wire mut) () () () ()\n^entry():\nx t"),("p",&cc,&format!("fn cc @p (c:{ty}, q:qreg[1]) () () () ()\n^entry():\nctl {control} action_call @a (q) () () ()\nret"))],"{}", "");
        support::reject_cli(&text, "§3.3", &format!("cc-ctl-{tag}"));
    }
}

#[test]
fn r10_function_header_prefix_matches_kind() {
    let failures: Vec<_> = support::r10_bad_headers()
        .iter()
        .enumerate()
        .filter_map(|(n, text)| {
            let result = Object::parse(text).err().map(|e| e.to_string());
            println!("header {n}: {result:?}");
            (!result.is_some_and(|e| e.contains("§16.2"))).then_some(n)
        })
        .collect();
    assert!(
        failures.is_empty(),
        "headers without §16.2 rejection: {failures:?}"
    );
}
#[test]
fn r10_present_bounds_requires_an_integer() {
    rejects(&support::r10_text("empty-bounds"), "§16.2");
    support::r10_pin("omitted-bounds");
}
#[test]
fn r10_exact_fhead_and_nonempty_optional_productions() {
    let failures: Vec<_> = ["fn-oracle", "inc-empty-bounds", "empty-lhs"]
        .into_iter()
        .filter(|tag| {
            let result = Object::parse(&support::r10_text(tag))
                .err()
                .map(|e| e.to_string());
            println!("{tag}: {result:?}");
            !result.is_some_and(|e| e.contains("§16.2"))
        })
        .collect();
    assert!(failures.is_empty(), "{failures:?}");
    rejects(
        &support::r10_text("cc").replace("\nret\n", "\n() = ret\n"),
        "§16.2",
    );
    // Nonempty tuple LHS and empty operand lists remain legal.
    Object::parse(&INC.replace("r = lift.add", "(r) = lift.add")).unwrap();
    for tag in ["cc", "r", "omitted-bounds"] {
        support::r10_pin(tag);
    }
    let sig = r#"{"abi":"0.8","args":[],"bounds":[1],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    Object::parse(&support::object(&[("a",sig,"action @a (t:wire mut) () () () ()\nregs {  } bounds {1}\n^entry():\nqloop 1 {\nx t\n}")],"{}","")).unwrap();
}

#[test]
fn r11_action_regs_do_not_produce_wires() {
    let text = support::r11_text("wire");
    for kind in ["action", "perm_action"] {
        for operand in ["d", "d[0]"] {
            rejects(
                &text
                    .replace("action", kind)
                    .replace("cx d t", &format!("cx {operand} t")),
                "§3.3",
            );
        }
    }
    support::r11_pin("producer");
}

#[test]
fn r11_action_regs_cannot_invent_angles() {
    rejects(&support::r11_text("angle"), "§3.3");
    support::r11_pin("argument");
    // Named angles must be an f64 argument, never a wire, region or token.
    let text = support::r11_text("argument");
    for ty in ["i64", "ptr", "token"] {
        rejects(
            &text
                .replace("angle:f64", &format!("angle:{ty}"))
                .replace(r#""args":["f64"]"#, &format!(r#""args":["{ty}"]"#)),
            "§3.3",
        );
    }
    rejects(&text.replace("rx angle t", "rx missing t"), "§3.3");
    support::r10_pin("one-trip");
    support::r11_pin("producer");
}

#[test]
fn r11_register_types_follow_function_kind() {
    let mut accepted = 0;
    let mut rejected = 0;
    for kind in ["cc", "r", "oracle", "lproc", "action", "perm_action"] {
        let (sig, head, row, tail) = match kind {
            "cc" | "r" => (format!(r#"{{"abi":"0.8","kind":"{kind}","params":[],"results":[],"tier":"{kind}"}}"#),
                format!("fn {kind} @a () () () () ()"), "ret", ""),
            "oracle" => (r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#.into(),
                "oracle @a (x:wire in) (t:wire) (f:wire[4]) () ()".into(), "lift.retval x", ""),
            "lproc" => (r#"{"abi":"0.8","kind":"lproc","params":[1],"result":1,"tier":"q"}"#.into(),
                "lproc @a (x:reg[1]) (r:reg[1]) () () ()".into(), "lift.ret x",
                r#"
section layout
{"A_tab":0,"H":1,"S_cells":0,"globals":{},"heap_base":1,"heap_limit_max":0,"interface_regions":{},"null_size":1,"ptr_width":32}
"#),
            _ => (format!(r#"{{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"{kind}","mem":[],"mut":[1],"tier":"q"}}"#),
                format!("{kind} @a (t:wire mut) () () () ()"), "x t", ""),
        };
        for ty in [
            "reg[1]",
            "wire",
            "wire[2]",
            "f64",
            "i8",
            "ptr",
            "token",
            "qreg[1]",
            "region[0,8]",
        ] {
            let body = format!(
                "{head}
regs {{ d:{ty} }}
^entry():
{row}
"
            );
            let text = support::object(&[("a", &sig, &body)], "{}", tail);
            if matches!(kind, "oracle" | "lproc") && ty == "reg[1]" {
                Object::parse(&text).unwrap();
                accepted += 1;
            } else {
                rejects(&text, "§3.3");
                rejected += 1;
            }
        }
        let text = support::object(
            &[(
                "a",
                &sig,
                &format!(
                    "{head}
regs {{  }}
^entry():
{row}
"
                ),
            )],
            "{}",
            tail,
        );
        Object::parse(&text).unwrap();
        accepted += 1;
    }
    assert_eq!((accepted, rejected), (8, 52));
    eprintln!("R11 regs family: {accepted} accepted / {rejected} rejected");
}
