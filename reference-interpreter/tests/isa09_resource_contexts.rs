use bennett_isa_ref::{
    json::Json,
    target::{charged_wire_occupancy, resource_display, verify_resource_rows, Rational},
};
use std::collections::BTreeMap;
#[test]
fn resource_displays_round_up_once_at_34_significant_digits() {
    for (n, d, expected) in [
        ("1", "3", "0.3333333333333333333333333333333334"),
        ("2", "3", "0.6666666666666666666666666666666667"),
        ("4", "3", "1.333333333333333333333333333333334"),
        ("0", "1", "0"),
        ("1", "8", "0.125"),
        ("1000", "1", "1000"),
    ] {
        assert_eq!(
            resource_display(&Rational::leaf(n, d).unwrap()).unwrap(),
            expected
        );
    }
    let third = Rational::leaf("1", "3").unwrap();
    let exact = BTreeMap::from([("time".into(), third)]);
    let good = Json::parse(r#"{"time":"0.3333333333333333333333333333333334"}"#).unwrap();
    verify_resource_rows(&good, None, &exact).unwrap();
    for value in [
        "0.3333333333333333333333333333333333",
        "+0.3333333333333333333333333333333334",
        "3.333333333333333333333333333333334e-1",
        "0.33333333333333333333333333333333340",
    ] {
        let bad = Json::Object(BTreeMap::from([(
            "time".into(),
            Json::String(value.into()),
        )]));
        assert!(verify_resource_rows(&bad, None, &exact).is_err());
    }
    let estimate = Json::parse(
        r#"{"precision_digits":34,"exact_resources":{"time":{"numerator":"1","denominator":"3"}}}"#,
    )
    .unwrap();
    verify_resource_rows(&good, Some(&estimate), &exact).unwrap();
    let wrong=Json::parse(r#"{"precision_digits":34,"exact_resources":{"time":{"numerator":"3333333333333333333333333333333334","denominator":"10000000000000000000000000000000000"}}}"#).unwrap();
    assert!(verify_resource_rows(&good, Some(&wrong), &exact).is_err());
}
#[test]
fn borrowed_intervals_replace_owner_charge_instead_of_adding_to_it() {
    let q = Rational::integer;
    let occupancy = charged_wire_occupancy(
        &q(0),
        &q(2),
        &q(1),
        &[(q(0), q(1), Rational::leaf("1", "4").unwrap())],
    )
    .unwrap();
    assert_eq!(occupancy, Rational::leaf("5", "4").unwrap());
    assert!(charged_wire_occupancy(
        &q(0),
        &q(2),
        &q(1),
        &[(q(0), q(2), q(1)), (q(1), q(2), q(1))]
    )
    .is_err());
}

#[test]
fn context_coverage_projection_and_exact_estimate_are_checked_without_rounding_children() {
    use bennett_isa_ref::target::{verify_resource_contexts, ResolvedResourceContext};
    let key = |point: &str| {
        Json::parse(&format!(r#"[{{"symbol":"b98f9111e8533f00a8e5ac195e1b335678d14efcabb653d754d9999e21297c16","point":"{point}","iteration":[],"instance":[],"call_index":0}}]"#)).unwrap()
    };
    let gates = Json::parse(r#"{"x":{"count":1}}"#).unwrap();
    let expected = vec![
        ResolvedResourceContext {
            context_key: key("entry:0"),
            resources: BTreeMap::from([("time".into(), Rational::leaf("1", "3").unwrap())]),
            gates_used: gates.clone(),
        },
        ResolvedResourceContext {
            context_key: key("entry:2"),
            resources: BTreeMap::from([("time".into(), Rational::leaf("1", "3").unwrap())]),
            gates_used: gates.clone(),
        },
    ];
    let rows = Json::parse(r#"{"time":"0.3333333333333333333333333333333334"}"#).unwrap();
    let records = expected
        .iter()
        .map(|c| {
            Json::Object(BTreeMap::from([
                ("context_key".into(), c.context_key.clone()),
                ("resources".into(), rows.clone()),
                ("gates_used".into(), gates.clone()),
            ]))
        })
        .collect::<Vec<_>>();
    let region = Json::Object(BTreeMap::from([
        ("resources".into(), rows.clone()),
        ("gates_used".into(), gates.clone()),
        ("resource_contexts".into(), Json::Array(records.clone())),
    ]));
    verify_resource_contexts(&region, &expected).unwrap();
    for bad in [
        vec![records[1].clone(), records[0].clone()],
        vec![records[0].clone()],
        vec![records[0].clone(), records[0].clone()],
    ] {
        let mut invalid = region.clone();
        let Json::Object(m) = &mut invalid else {
            unreachable!()
        };
        m.insert("resource_contexts".into(), Json::Array(bad));
        assert!(verify_resource_contexts(&invalid, &expected).is_err());
    }
    let mut wrong = region.clone();
    let Json::Object(m) = &mut wrong else {
        unreachable!()
    };
    m.insert(
        "resources".into(),
        Json::parse(r#"{"time":"0.6666666666666666666666666666666667"}"#).unwrap(),
    );
    assert!(verify_resource_contexts(&wrong, &expected).is_err());
    let singleton = Json::Object(BTreeMap::from([
        ("resources".into(), rows),
        ("gates_used".into(), gates),
        (
            "resource_contexts".into(),
            Json::Array(vec![records[0].clone()]),
        ),
    ]));
    verify_resource_contexts(&singleton, &expected[..1]).unwrap();
}

#[test]
fn portable_two_flip_context_rows_match_independent_fraction_arithmetic() {
    use bennett_isa_conformance::json::Json as Value;
    use bennett_isa_ref::{
        target::{verify_resource_contexts, ResolvedResourceContext},
        Object,
    };
    for source in [
        include_str!("../../conformance/fixtures/accepted-k87-09/two-flips.bisa"),
        include_str!("../../conformance/fixtures/accepted-k87-09/two-flips-estimate.bisa"),
    ] {
        let object = Object::parse(source).unwrap();
        let contexts = object
            .source_action_invocations(
                "twice",
                &[Value::Str("t".into()), Value::Str("c".into())],
                &[("t", 1), ("c", 1)],
                false,
                None,
            )
            .unwrap();
        assert_eq!(contexts.len(), 3);
        for (closure, region) in object
            .section("certificate")
            .unwrap()
            .get("regions")
            .unwrap()
            .object()
            .unwrap()
        {
            let computed = contexts
                .iter()
                .filter(|c| c.closure_hash == *closure)
                .map(|context| {
                    let root = context.context_key.array().unwrap().is_empty();
                    let controls = context.binding.inherited_controls as i128;
                    let resources = BTreeMap::from([
                        (
                            "time".into(),
                            Rational::leaf(if root { "2" } else { "1" }, "3").unwrap(),
                        ),
                        (
                            "qubit_second[fast]".into(),
                            if root {
                                Rational::leaf("4", "3").unwrap()
                            } else {
                                Rational::integer(1 + controls)
                                    .div(&Rational::integer(3))
                                    .unwrap()
                            },
                        ),
                        (
                            "peak_qubits[fast]".into(),
                            Rational::integer(if root { 2 } else { 1 + controls }),
                        ),
                    ]);
                    ResolvedResourceContext {
                        context_key: context.context_key.clone(),
                        resources,
                        gates_used: Json::parse(if root {
                            r#"{"x":{"count":2}}"#
                        } else {
                            r#"{"x":{"count":1}}"#
                        })
                        .unwrap(),
                    }
                })
                .collect::<Vec<_>>();
            verify_resource_contexts(region, &computed).unwrap();
        }
    }
}
