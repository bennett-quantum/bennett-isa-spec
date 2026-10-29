#![allow(dead_code)]
use bennett_isa_ref::{json::Json, Object};
use std::collections::BTreeMap;

pub fn json(s: &str) -> Json {
    Json::parse(s).unwrap()
}
pub fn object(functions: &[(&str, &str, &str)], imports: &str, extra: &str) -> String {
    let symbols = functions
        .iter()
        .enumerate()
        .map(|(i, (name, signature, _))| {
            let sig = json(signature);
            let value = Json::Object(BTreeMap::from([
                ("name".into(), Json::String((*name).into())),
                ("kind".into(), sig.get("kind").unwrap().clone()),
                ("signature".into(), sig),
                ("interface_hash".into(), Json::String("0".repeat(64))),
            ]));
            (i.to_string(), value)
        })
        .collect();
    let mut text=format!("bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{}\n",Json::Object(symbols).canonical());
    if imports != "{}" {
        text.push_str(&format!(
            "\nsection imports\n{}\n",
            json(imports).canonical()
        ));
    }
    text.push_str("\nsection functions\n");
    for (_, _, body) in functions {
        text.push_str(body.trim_end_matches('\n'));
        text.push('\n');
    }
    text.push_str(extra);
    text
}
pub fn rehash(text: &str) -> Object {
    let parsed = Object::parse(text).unwrap();
    let hashes = parsed.identity().unwrap().interface_hashes;
    let mut lines: Vec<_> = text.lines().map(str::to_string).collect();
    let i = lines.iter().position(|s| s == "section symbols").unwrap() + 1;
    let Json::Object(mut symbols) = json(&lines[i]) else {
        unreachable!()
    };
    for symbol in symbols.values_mut() {
        let name = symbol.get("name").unwrap().string().unwrap().to_string();
        let Json::Object(value) = symbol else {
            unreachable!()
        };
        value.insert("interface_hash".into(), Json::String(hashes[&name].clone()));
    }
    lines[i] = Json::Object(symbols).canonical();
    Object::parse(&(lines.join("\n") + "\n")).unwrap()
}
/// An object with functions, data symbols (name, hex) and trailing sections
/// (layout and later, each starting with "\nsection ..."). Hashes are zero;
/// pass the text through `rehash` to obtain canonical identities.
pub fn object_with_data(
    functions: &[(&str, &str, &str)],
    data: &[(&str, &str)],
    tail: &str,
) -> String {
    let mut symbols = BTreeMap::new();
    for (i, (name, signature, _)) in functions.iter().enumerate() {
        let sig = json(signature);
        symbols.insert(
            i.to_string(),
            Json::Object(BTreeMap::from([
                ("name".into(), Json::String((*name).into())),
                ("kind".into(), sig.get("kind").unwrap().clone()),
                ("signature".into(), sig),
                ("interface_hash".into(), Json::String("0".repeat(64))),
            ])),
        );
    }
    for (j, (name, hex)) in data.iter().enumerate() {
        let sig = json(&format!(
            r#"{{"abi":"0.8","kind":"data","size":{},"tier":"-"}}"#,
            hex.len() / 2
        ));
        symbols.insert(
            (functions.len() + j).to_string(),
            Json::Object(BTreeMap::from([
                ("name".into(), Json::String((*name).into())),
                ("kind".into(), Json::String("data".into())),
                ("signature".into(), sig),
                ("interface_hash".into(), Json::String("0".repeat(64))),
            ])),
        );
    }
    let mut text = format!(
        "bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{}\n\nsection functions\n",
        Json::Object(symbols).canonical()
    );
    for (_, _, body) in functions {
        text.push_str(body.trim_end_matches('\n'));
        text.push('\n');
    }
    if !data.is_empty() {
        text.push_str("\nsection data\n");
        for (name, hex) in data {
            text.push_str(&format!("{name} {hex}\n"));
        }
    }
    text.push_str(tail);
    text
}
pub const ACTION_1_1: &str =
    r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
