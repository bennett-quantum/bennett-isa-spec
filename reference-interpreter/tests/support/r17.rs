//! Literal reviewer witnesses; mutations preserve the original semantic object.
use super::json;
use bennett_isa_ref::json::Json;
pub const EPS: &str = include_str!("r17-default-epsilon-share.bisa");
pub const TIE: &str = include_str!("r17-shadowed-tie.bisa");
pub const EPS_SH: &str = "d9025ef24bfd8db01e4e3e0f5b28be776c4e5585b000fd5bd2333122c0d4845d";
pub const TIE_SH: &str = "e13ea989795d205fcf983b033f24bf1945d8be197c241b3a440a67b2489c287f";
pub fn schedule(source: &str) -> Json {
    json(source.split_once("section schedule\n").unwrap().1.trim())
}
pub fn document(source: &str, schedule: &Json) -> String {
    format!(
        "{}section schedule\n{}\n",
        source.split_once("section schedule\n").unwrap().0,
        schedule.canonical()
    )
}
pub fn malformed_gate() -> Vec<Json> {
    [
        r#"{"realisation":"native","epsilon_share":null}"#,
        r#"{"realisation":"native","epsilon_share":{}}"#,
        r#"{"realisation":"native","epsilon_share":-1}"#,
        r#"{"realisation":"native","epsilon_share":"invalid"}"#,
        r#"{"epsilon_share":0}"#,
    ]
    .map(json)
    .to_vec()
}
pub fn gate(payload: Json) -> String {
    let mut s = schedule(EPS);
    let Json::Object(fields) = &mut s else {
        unreachable!()
    };
    let Json::Array(entries) = fields.get_mut("entries").unwrap() else {
        unreachable!()
    };
    let Json::Object(e) = &mut entries[0] else {
        unreachable!()
    };
    e.insert("gate".into(), payload);
    document(EPS, &s)
}
