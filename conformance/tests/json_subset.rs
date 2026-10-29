use bennett_isa_conformance::json::{exact_diff, parse, subset_diff};

fn ok(expected: &str, actual: &str) {
    let e = parse(expected).expect("expected parses");
    let a = parse(actual).expect("actual parses");
    assert_eq!(subset_diff(&e, &a), None, "{expected} vs {actual}");
}

fn bad(expected: &str, actual: &str) {
    let e = parse(expected).expect("expected parses");
    let a = parse(actual).expect("actual parses");
    assert!(subset_diff(&e, &a).is_some(), "{expected} vs {actual}");
}

#[test]
fn object_subset_ignores_extra_fields() {
    ok(
        r#"{"code":0,"payload":9}"#,
        r#"{"code":0,"payload":9,"steps":3}"#,
    );
    ok(
        r#"{"a":{"b":[1,2]}}"#,
        r#"{"a":{"b":[1,2],"c":true},"d":null}"#,
    );
}

#[test]
fn mismatches_are_reported() {
    bad(r#"{"code":1}"#, r#"{"code":0}"#);
    bad(r#"{"code":0}"#, r#"{"payload":9}"#);
    bad(r#"{"a":[1,2]}"#, r#"{"a":[1]}"#);
    bad(r#"{"a":[1]}"#, r#"{"a":[1,2]}"#);
    bad(r#"{"a":null}"#, r#"{"a":0}"#);
    bad(r#"{"a":"1"}"#, r#"{"a":1}"#);
}

#[test]
fn numbers_compare_by_value_integers_and_floats() {
    ok(r#"{"a":3}"#, r#"{"a":3}"#);
    ok(r#"{"a":[0.5,0]}"#, r#"{"a":[0.5,0]}"#);
    ok(r#"{"a":0.7071067811865476}"#, r#"{"a":0.7071067811865476}"#);
    bad(r#"{"a":0.5}"#, r#"{"a":0.25}"#);
}

#[test]
fn exact_diff_rejects_extra_keys() {
    let e = parse(r#"{"00":[1,0]}"#).unwrap();
    let a = parse(r#"{"00":[1,0],"11":[1,0]}"#).unwrap();
    assert_eq!(subset_diff(&e, &a), None);
    assert!(exact_diff(&e, &a).is_some());
    assert_eq!(exact_diff(&e, &e), None);
}

#[test]
fn parser_handles_escapes_and_nested() {
    ok(r#"{"s":"a\"b\\c\nd"}"#, r#"{"s":"a\"b\\c\nd"}"#);
    ok("{}", r#"{"x":[{"y":[[[true]]]}]}"#);
    assert!(parse("{").is_err());
    assert!(parse(r#"{"a":}"#).is_err());
    assert!(parse("").is_err());
    assert!(parse("{} trailing").is_err());
}

#[test]
fn rejects_invalid_json_output() {
    for text in [
        "01",
        "-01",
        "1.",
        "1.e2",
        "1e999",
        "\"raw\nnewline\"",
        r#""\uD800\u0041""#,
    ] {
        assert!(
            bennett_isa_conformance::json::parse(text).is_err(),
            "accepted invalid JSON: {text:?}"
        );
    }
}
