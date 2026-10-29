use bennett_isa_ref::{json::Json, Object};

const INC: &str = include_str!("fixtures/inc8.bisa");
const SUM: &str = include_str!("fixtures/sum_db.bisa");

fn rejects(text: &str, rule: &str) {
    let error = Object::parse(text).expect_err("ill-formed object accepted");
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
    for text in [INC.replace('\n', "\r\n"), INC.replace("lift.add x", "lift.add  x"), INC.replace("lift.add x one", "lift.add x one "), INC.replace("\n\nsection", "\nsection"), INC.replace("^entry():", "\n^entry():")] {
        rejects(&text, "§16.1");
    }
    rejects(&INC.replace("isa=0.8", "isa=0.9"), "§1");
    rejects(&INC.replace("section symbols", "section unknown"), "§9.1");
    rejects(&format!("{INC}\nsection symbols\n{{}}\n"), "§9.1");
    rejects(&INC.replace("{\"0\":", "{ \"0\":"), "§16.1");
}

#[test]
fn names_widths_opcodes_and_terminators_are_checked() {
    rejects(&INC.replace("lift.add x one", "lift.add missing one"), "§3.7");
    rejects(&INC.replace("r = lift.add", "x = lift.add"), "§3.7");
    rejects(&INC.replace("r:reg[8]", "r:reg[7]"), "§3.3");
    rejects(&INC.replace("one = lift.const.i8 1", "one = lift.magic x"), "§4.8");
    rejects(&INC.replace("lift.retval r", "r = lift.mov one"), "§3.6");
    rejects(&INC.replace("lift.retval r", "lift.ret r"), "§4.2");
    rejects(&INC.replace("lift.retval r", "qjump ^missing"), "§3.6");
    rejects(&INC.replace("lift.retval r", "fault 16"), "§3.3");
    rejects(&SUM.replace("idx = lift.zext.i32 n2", "idx = lift.trunc.i32 n2"), "§3.3");
    rejects(&SUM.replace("lift.call @sum (n, acc, base)", "lift.call @sum (base, acc, n)"), "§3.3");
    rejects(&SUM.replace("zp = lift.addr @zero", "zp = lift.addr @db"), "§4.2");
    rejects(&SUM.replace("\"base\":4096", "\"base\":16"), "§7.2");
    rejects(&SUM.replace("\"size\":1", "\"size\":2"), "§7.2");
}

#[test]
fn json_is_strict_and_canonical() {
    let raw = r#"{"a":[null,true,false,"\n",-1,1e+30],"z":"é"}"#;
    assert_eq!(Json::parse(raw).unwrap().canonical(), raw);
    for bad in ["{\"a\":1,\"a\":2}", "[01]", "[1.]", "[NaN]", "[1e999]", "\"\u{0001}\"", "\"\\ud800\""] {
        assert!(Json::parse(bad).is_err(), "accepted {bad}");
    }
    assert_eq!(Json::parse("[-0,1e-7,1e-6,1e20,1e21]").unwrap().canonical(), "[0,1e-7,0.000001,100000000000000000000,1e+21]");
    // RFC 8785 sorts object keys by UTF-16 units: a supplementary character
    // sorts before U+E000, despite its larger Unicode scalar value.
    assert_eq!(Json::parse("{\"\u{e000}\":0,\"😀\":1}").unwrap().canonical(), "{\"😀\":1,\"\u{e000}\":0}");
}
