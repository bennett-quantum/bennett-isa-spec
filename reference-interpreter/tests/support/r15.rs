//! Clause-derived R15 controls with literal reviewer identity pins.
use super::json;
use bennett_isa_ref::json::Json;

pub const IH: &str = "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93";
pub const OH: &str = "8093da48bad04d6564325faf01ea1304a84b5ac5a120e685eee9951f5daec5d9";
pub const FRAME: &str = include_str!("r15-witnesses/r15-frame-empty.bisa");
pub const EPSILON: &str = include_str!("r15-witnesses/r15-epsilon-0.0.bisa");

pub fn schedule(epsilon: Json) -> String {
    let corpus = json(include_str!("r13_schema_cases.json"));
    let case = corpus
        .array()
        .unwrap()
        .iter()
        .find(|c| c.get("name").unwrap().string().unwrap() == "schedule-cleanupFalse")
        .unwrap();
    let text = case.get("object").unwrap().string().unwrap();
    let (prefix, section) = text.split_once("section schedule\n").unwrap();
    let mut value = json(section.trim());
    let Json::Object(root) = &mut value else {
        unreachable!()
    };
    let Json::Object(bindings) = root.get_mut("bindings").unwrap() else {
        unreachable!()
    };
    bindings.insert("epsilon".into(), epsilon);
    format!("{prefix}section schedule\n{}\n", value.canonical())
}

pub fn zeros() -> Vec<Json> {
    ["0", r#""0""#, r#""0.0""#, r#""0.000""#].map(json).to_vec()
}
pub fn nonzeros() -> Vec<Json> {
    vec![
        json(r#""0.1""#),
        json(r#""0.000001""#),
        // Decimal strings admit no exponent; this is exact 10^-400, below
        // binary64's range. The nonzero mantissa must never round to zero.
        Json::String(format!("0.{}1", "0".repeat(399))),
        json("1e-300"),
    ]
}
