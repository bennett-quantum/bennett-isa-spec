use bennett_isa_ref::json::Json;
use bennett_isa_ref::target::{verify_native_model, ModelContext, ModelVerification, Rational};

fn model(sign: &str, controls: usize, own: usize) -> Json {
    let inherited = controls - own;
    let map: Vec<_> = (inherited..controls)
        .chain(0..inherited)
        .chain(controls..controls + 1)
        .map(|v| v.to_string())
        .collect();
    Json::parse(&format!(r#"{{"version":"bennett.native-model/1","kind":"unitary","arity":1,"basis":["t"],"parameter_bindings":{{"k":"k","c":"c"}},"control_models":{{"kind":"controlled_U","controls":"c","disabled":"identity"}},"operand_map":[{}],"data":{{"matrix":[[[["0","1"],["0","1"]],[["{sign}","1"],["0","1"]]],[[["{sign}","1"],["0","1"]],[["0","1"],["0","1"]]]]}}}}"#, map.join(","))).unwrap()
}

fn context(own: usize, total: usize) -> ModelContext {
    ModelContext {
        own_controls: own,
        total_controls: total,
        requested_epsilon: Some(Rational::integer(3)),
        theta_bits: None,
    }
}

fn checked(
    model: &Json,
    opcode: &str,
    context: &ModelContext,
    epsilon: i128,
) -> bennett_isa_ref::target::IntervalMatrix {
    match verify_native_model(
        Some(model),
        opcode,
        context,
        &Rational::integer(epsilon),
        64,
    )
    .unwrap()
    {
        ModelVerification::Checked(m) => m,
        ModelVerification::Unchecked(reason) => panic!("{reason}"),
    }
}

#[test]
fn controlled_x_is_reconstructed_with_its_disabled_identity_and_phase() {
    let c = context(0, 1);
    let x = checked(&model("1", 1, 0), "x", &c, 0);
    assert_eq!(x.rows(), 4);
    // Inherited controls are the leading allocation-basis bits (§12.1).
    for row in 0..4 {
        for col in 0..4 {
            let expected = i128::from([0, 1, 3, 2][col] == row);
            assert_eq!(
                x.entry(row, col).unwrap().re.lower(),
                &Rational::integer(expected)
            );
            assert_eq!(
                x.entry(row, col).unwrap().re.upper(),
                &Rational::integer(expected)
            );
        }
    }
    let negative = model("-1", 1, 0);
    assert!(verify_native_model(Some(&negative), "x", &c, &Rational::integer(0), 64).is_err());
    assert!(verify_native_model(Some(&negative), "x", &c, &Rational::integer(1), 64).is_err());
    // ||controlled(-X)-controlled(X)||op=2, not sqrt(8).
    checked(&negative, "x", &c, 2);
    let minus = checked(&negative, "x", &c, 3);
    assert_eq!(minus.entry(0, 0).unwrap().re.lower(), &Rational::integer(1));
    assert_eq!(
        minus.entry(2, 3).unwrap().re.lower(),
        &Rational::integer(-1)
    );
}

#[test]
fn supplied_model_must_be_normalized_even_under_a_loose_accuracy_bound() {
    let c = context(0, 0);
    let invalid = model("2", 0, 0);
    let error =
        verify_native_model(Some(&invalid), "x", &c, &Rational::integer(3), 64).unwrap_err();
    assert!(error.to_string().contains("unitary"), "{error}");
    let missing = verify_native_model(None, "x", &c, &Rational::integer(1), 64).unwrap();
    assert!(matches!(missing, ModelVerification::Unchecked(reason) if reason.contains("missing")));
    assert!(matches!(
        verify_native_model(None, "x", &c, &Rational::integer(0), 64).unwrap(),
        ModelVerification::Checked(_)
    ));
    let mut unknown = model("2", 0, 0);
    if let Json::Object(m) = &mut unknown {
        m.insert(
            "version".into(),
            Json::String("bennett.native-model/99".into()),
        );
    }
    assert!(
        matches!(verify_native_model(Some(&unknown), "x", &c, &Rational::integer(0), 64).unwrap(), ModelVerification::Unchecked(reason) if reason.contains("unsupported"))
    );
}

#[test]
fn own_and_inherited_control_order_is_checked_not_repaired() {
    let c = context(1, 2);
    let valid = model("1", 2, 1);
    assert!(matches!(
        verify_native_model(Some(&valid), "cx", &c, &Rational::integer(0), 64).unwrap(),
        ModelVerification::Checked(_)
    ));
    for map in ["[0,1,2]", "[1,1,2]", "[1,0]", "[1,0,3]"] {
        let mut invalid = valid.clone();
        if let Json::Object(m) = &mut invalid {
            m.insert("operand_map".into(), Json::parse(map).unwrap());
        }
        assert!(
            verify_native_model(Some(&invalid), "cx", &c, &Rational::integer(0), 64).is_err(),
            "{map}"
        );
    }
    let mut wrong_binding = valid.clone();
    if let Json::Object(m) = &mut wrong_binding {
        m.insert(
            "parameter_bindings".into(),
            Json::parse(r#"{"k":"c","c":"k"}"#).unwrap(),
        );
    }
    assert!(
        verify_native_model(Some(&wrong_binding), "cx", &c, &Rational::integer(0), 64).is_err()
    );
}

#[test]
fn reordered_base_basis_is_applied_through_its_operand_map() {
    let mut m = model("1", 0, 0);
    let Json::Object(fields) = &mut m else {
        unreachable!()
    };
    fields.insert("arity".into(), Json::Number("2".into()));
    fields.insert("basis".into(), Json::parse(r#"["t2","t"]"#).unwrap());
    fields.insert("operand_map".into(), Json::parse("[1,0]").unwrap());
    // X on the first model bit becomes X on allocation bit t2 after mapping.
    let permutation = [2, 3, 0, 1];
    let rows = (0..4)
        .map(|r| {
            Json::Array(
                (0..4)
                    .map(|c| {
                        Json::parse(&format!(
                            r#"[["{}","1"],["0","1"]]"#,
                            i128::from(permutation[c] == r)
                        ))
                        .unwrap()
                    })
                    .collect(),
            )
        })
        .collect();
    fields.insert(
        "data".into(),
        Json::Object(std::collections::BTreeMap::from([(
            "matrix".into(),
            Json::Array(rows),
        )])),
    );
    let actual = checked(&m, "swap", &context(0, 0), 2);
    for col in 0..4 {
        assert_eq!(
            actual.entry(col ^ 1, col).unwrap().re.lower(),
            &Rational::integer(1)
        );
    }
}

#[test]
fn unsupported_control_representation_is_not_validated_as_a_base_matrix() {
    let mut opaque = model("1", 1, 0);
    let Json::Object(fields) = &mut opaque else {
        unreachable!()
    };
    fields.insert(
        "control_models".into(),
        Json::parse(r#"{"kind":"unpublished"}"#).unwrap(),
    );
    // Explicit controls describe a full context; applying controlled_U's
    // base-arity rules to an unsupported representation would reject it.
    fields.insert("arity".into(), Json::Number("2".into()));
    let verdict = verify_native_model(
        Some(&opaque),
        "x",
        &context(0, 1),
        &Rational::integer(0),
        64,
    )
    .unwrap();
    assert!(
        matches!(verdict, ModelVerification::Unchecked(reason) if reason.contains("unsupported"))
    );
}
