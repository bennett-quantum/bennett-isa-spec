use bennett_isa_ref::{
    json::Json,
    sha256_hex,
    target::{ModelContext, ModelVerification, NativeTarget, Rational},
};

fn target(epsilon: Json) -> String {
    let mut target = Json::parse(include_str!(
        "../../conformance/fixtures/controlled-action-09/target.json"
    ))
    .unwrap();
    let Json::Object(t) = &mut target else {
        unreachable!()
    };
    let Json::Array(gates) = t.get_mut("gates").unwrap() else {
        unreachable!()
    };
    let Json::Object(g) = &mut gates[0] else {
        unreachable!()
    };
    g.insert("epsilon".into(), epsilon);
    target.canonical()
}

#[test]
fn evaluated_epsilon_requires_models_only_in_positive_contexts() {
    let zero = target(Json::parse(r#"{"op":"-","args":[1,1]}"#).unwrap());
    let document = NativeTarget::parse(&zero).unwrap();
    assert_eq!(document.hash(), sha256_hex(zero.as_bytes()));
    let ctx = ModelContext {
        own_controls: 0,
        total_controls: 1,
        requested_epsilon: Some(Rational::integer(2)),
        theta_bits: None,
    };
    let resolved = document.verify_entry(0, &ctx, 64).unwrap();
    assert_eq!(resolved.achieved_epsilon, Rational::integer(0));
    assert!(matches!(resolved.model, ModelVerification::Checked(_)));
    let variable = target(Json::parse(r#"{"var":"c"}"#).unwrap());
    let resolved = NativeTarget::parse(&variable)
        .unwrap()
        .verify_entry(0, &ctx, 64)
        .unwrap();
    assert_eq!(resolved.achieved_epsilon, Rational::integer(1));
    assert!(
        matches!(resolved.model,ModelVerification::Unchecked(reason) if reason.contains("missing"))
    );
    assert!(NativeTarget::parse(&(zero + "\n")).is_err());
    assert!(document.verify_entry(1, &ctx, 64).is_err());
}

#[test]
fn zero_epsilon_cost_cannot_reintroduce_requested_eps() {
    let source = target(Json::parse(r#"{"op":"-","args":[1,1]}"#).unwrap());
    let mut t = Json::parse(&source).unwrap();
    let Json::Object(fields) = &mut t else {
        unreachable!()
    };
    let Json::Array(gates) = fields.get_mut("gates").unwrap() else {
        unreachable!()
    };
    let Json::Object(g) = &mut gates[0] else {
        unreachable!()
    };
    g.insert(
        "cost".into(),
        Json::parse(r#"{"time":{"var":"eps"}}"#).unwrap(),
    );
    let ctx = ModelContext {
        own_controls: 0,
        total_controls: 1,
        requested_epsilon: Some(Rational::integer(1)),
        theta_bits: None,
    };
    let error = NativeTarget::parse(&t.canonical())
        .unwrap()
        .verify_entry(0, &ctx, 64)
        .unwrap_err();
    assert!(error.to_string().contains("zero-epsilon"), "{error}");
    g_invalid_fields(&source);
}

fn g_invalid_fields(source: &str) {
    let t = Json::parse(source).unwrap();
    let Json::Object(mut fields) = t else {
        unreachable!()
    };
    fields.insert("unknown".into(), Json::Bool(true));
    assert!(NativeTarget::parse(&Json::Object(fields).canonical()).is_err());
}
