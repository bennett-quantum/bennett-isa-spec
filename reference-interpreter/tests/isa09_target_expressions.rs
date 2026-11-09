use bennett_isa_ref::json::Json;
use bennett_isa_ref::target::{CostExpression, ExpressionBindings, ModelExpression, Rational};
use std::collections::BTreeMap;

fn json(s: &str) -> Json {
    Json::parse(s).unwrap()
}

#[test]
fn cost_expressions_use_resolved_exact_context_and_closed_grammar() {
    let mut bindings = ExpressionBindings::new();
    bindings.insert("k".into(), Rational::integer(1));
    bindings.insert("c".into(), Rational::integer(2));
    bindings.insert("eps".into(), Rational::leaf("999", "10000").unwrap());
    let mut tables = BTreeMap::new();
    tables.insert(
        "cost".into(),
        vec![
            Rational::integer(3),
            Rational::integer(5),
            Rational::integer(7),
        ],
    );
    let expr = CostExpression::parse(&json(r#"{"table":"cost","index":"eps_decade"}"#)).unwrap();
    // 999/10000 * 10 < 1 and *100 >= 1, therefore index 2.
    assert_eq!(
        expr.evaluate(&bindings, &tables).unwrap(),
        Rational::integer(7)
    );
    let log = CostExpression::parse(&json(r#"{"op":"log2inv_eps","args":[]}"#)).unwrap();
    // 999/10000 * 8 < 1 and *16 >= 1.
    assert_eq!(
        log.evaluate(&bindings, &tables).unwrap(),
        Rational::integer(4)
    );
    let zero = CostExpression::parse(&json(r#"{"op":"-","args":[1,1]}"#)).unwrap();
    assert_eq!(
        zero.evaluate(&bindings, &tables).unwrap(),
        Rational::integer(0)
    );
    assert!(!zero.uses_eps());
    assert!(expr.uses_eps());
    for invalid in [
        r#"{"var":"unknown"}"#,
        r#"{"op":"+","args":[1]}"#,
        r#"{"var":"c","extra":0}"#,
        r#"{"table":"cost","index":"c"}"#,
        "\"0.1\"",
    ] {
        assert!(CostExpression::parse(&json(invalid)).is_err(), "{invalid}");
    }
    bindings.remove("eps");
    assert!(expr.evaluate(&bindings, &tables).is_err());
    assert!(log.evaluate(&bindings, &tables).is_err());
    bindings.insert("eps".into(), Rational::integer(0));
    assert!(expr.evaluate(&bindings, &tables).is_err());
    bindings.insert("eps".into(), Rational::integer(2));
    assert!(log.evaluate(&bindings, &tables).is_err());
    let divide = CostExpression::parse(&json(r#"{"op":"/","args":[1,0]}"#)).unwrap();
    assert!(divide.evaluate(&bindings, &tables).is_err());
}

#[test]
fn model_expressions_preserve_binary64_and_complex_phase() {
    let mut bindings = ExpressionBindings::new();
    bindings.insert(
        "theta".into(),
        Rational::binary64(0.1f64.to_bits()).unwrap(),
    );
    let angle = ModelExpression::parse(&json(r#"{"var":"theta"}"#), &["theta"]).unwrap();
    let value = angle.exact(&bindings).unwrap().unwrap();
    assert_eq!(
        value.real().pair(),
        ("3602879701896397".into(), "36028797018963968".into())
    );
    assert_eq!(value.imaginary(), &Rational::integer(0));
    let imaginary_squared = ModelExpression::parse(
        &json(r#"{"op":"*","args":[{"op":"i","args":[]},{"op":"i","args":[]}]}"#),
        &[],
    )
    .unwrap();
    assert_eq!(
        imaginary_squared.exact(&bindings).unwrap().unwrap().real(),
        &Rational::integer(-1)
    );
    let sqrt = ModelExpression::parse(&json(r#"{"op":"sqrt","args":[["9","4"]]}"#), &[]).unwrap();
    assert_eq!(
        sqrt.exact(&bindings).unwrap().unwrap().real(),
        &Rational::leaf("3", "2").unwrap()
    );
    // Unsupported exact identities retain the operation and cannot become zero.
    let pi = ModelExpression::parse(&json(r#"{"op":"pi","args":[]}"#), &[]).unwrap();
    assert_eq!(pi.exact(&bindings).unwrap(), None);
    for invalid in [
        r#"["2","4"]"#,
        r#"{"var":"eps"}"#,
        r#"{"op":"sin","args":[]}"#,
        r#"{"op":"i","args":[["1","1"]]}"#,
        "1",
    ] {
        assert!(
            ModelExpression::parse(&json(invalid), &["theta"]).is_err(),
            "{invalid}"
        );
    }
    bindings.clear();
    assert!(angle.exact(&bindings).is_err());
}
