//! Literal R16 reviewer witness and descriptor-only mutations; no rehash.
use super::json;
pub const IH: &str = super::r15::IH;
pub const OH: &str = super::r15::OH;
use bennett_isa_ref::json::Json;
pub const DEFAULT: &str = include_str!("r16-default-expansion.bisa");
pub const SH: &str = "d5ba44577e812d158265a048d8e4f8843b5a472cad120f47a893c6bbc9dfb99d";

pub fn expansion(value: Json) -> String {
    let (prefix, line) = DEFAULT.split_once("section schedule\n").unwrap();
    let mut schedule = json(line.trim());
    let Json::Object(s) = &mut schedule else {
        unreachable!()
    };
    let Json::Object(b) = s.get_mut("bindings").unwrap() else {
        unreachable!()
    };
    b.insert("expansion".into(), value);
    format!("{prefix}section schedule\n{}\n", schedule.canonical())
}
pub fn malformed() -> Vec<Json> {
    [
        "null", "{}",
        r#"{"name":"ref","version":"0.8","hash":"bad"}"#,
        r#"{"name":"ref","version":"0.8"}"#,
        r#"{"name":"ref","version":8,"hash":"0000000000000000000000000000000000000000000000000000000000000000"}"#,
        r#"{"name":"ref","version":"0.8","hash":"0000000000000000000000000000000000000000000000000000000000000000","extra":0}"#,
    ].map(json).to_vec()
}
