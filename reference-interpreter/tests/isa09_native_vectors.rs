#[path = "support/native_operands.rs"]
mod native_operands;
use bennett_isa_ref::{
    json::Json,
    target::{verify_numerical_bound, ModelContext, ModelVerification, NativeTarget, Rational},
    Object,
};
use std::{fs, path::Path};

#[test]
fn portable_vectors_check_every_model_outcome_and_phase_sensitive_process_bound() {
    let root =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../conformance/fixtures/accepted-k86-09");
    let manifest = Json::parse(include_str!(
        "../../conformance/fixtures/accepted-k86-09/vectors.json"
    ))
    .unwrap();
    let mut checked = std::collections::BTreeMap::new();
    for case in manifest.get("cases").unwrap().array().unwrap() {
        let name = case.get("name").unwrap().string().unwrap();
        let bytes =
            fs::read_to_string(root.join(case.get("target").unwrap().string().unwrap())).unwrap();
        let target = NativeTarget::parse(&bytes).unwrap();
        assert_eq!(
            target.hash(),
            case.get("target_hash").unwrap().string().unwrap()
        );
        let c = case.get("context").unwrap();
        let eps = c.get("requested_epsilon").unwrap().array().unwrap();
        let own = c.get("own_controls").unwrap().uint().unwrap() as usize;
        let total = c.get("total_controls").unwrap().uint().unwrap() as usize;
        let context = ModelContext {
            operands: native_operands::operands(own, total, 1),
            own_controls: c.get("own_controls").unwrap().uint().unwrap() as usize,
            total_controls: c.get("total_controls").unwrap().uint().unwrap() as usize,
            requested_epsilon: Some(
                Rational::leaf(eps[0].string().unwrap(), eps[1].string().unwrap()).unwrap(),
            ),
            theta_bits: c
                .object()
                .unwrap()
                .get("theta_bits")
                .map(|bits| u64::from_str_radix(bits.string().unwrap(), 16).unwrap()),
        };
        let result = target.verify_entry(0, &context, 64);
        match (case.get("outcome").unwrap().string().unwrap(), result) {
            ("checked", Ok(entry)) => match entry.model {
                ModelVerification::Checked(matrix) => {
                    checked.insert(name.to_owned(), matrix);
                }
                other => panic!("{name}: {other:?}"),
            },
            ("unchecked", Ok(entry)) => assert!(
                matches!(entry.model, ModelVerification::Unchecked(_)),
                "{name}: {:?}",
                entry.model
            ),
            ("reject", Err(_)) => {}
            (outcome, result) => panic!("{name}: expected {outcome}, found {result:?}"),
        }
    }
    let x = &checked["controlled-x"];
    let minus = &checked["negative-x-bound-two"];
    assert!(verify_numerical_bound(x, minus, &Rational::integer(2)).is_err());
    verify_numerical_bound(x, minus, &Rational::integer(3)).unwrap();
    verify_numerical_bound(x, x, &Rational::integer(0)).unwrap();
    // Existing source and actual bound schedule admission stay independent of
    // target model identity. This does not claim scheduled process execution.
    let source = fs::read_to_string(root.join("source.bisa")).unwrap();
    let object = Object::parse(&source).unwrap();
    let identity = object.identity().unwrap();
    assert_eq!(
        identity.object_hash,
        manifest
            .get("source_identity")
            .unwrap()
            .get("object_hash")
            .unwrap()
            .string()
            .unwrap()
    );
    let schedule = fs::read_to_string(root.join("controlled-x.schedule.json")).unwrap();
    let bound = Object::parse(&format!("{source}\nsection schedule\n{schedule}\n")).unwrap();
    assert_eq!(bound.identity().unwrap().object_hash, identity.object_hash);
}
