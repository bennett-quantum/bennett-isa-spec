use bennett_isa_ref::{json::Json, sha256_hex, Object, QinplaceApplicability};
#[test]
fn portable_qinplace_cases_preserve_owner_and_actual_context_scope() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../conformance/fixtures/accepted-k92-09");
    let manifest = Json::parse(include_str!(
        "../../conformance/fixtures/accepted-k92-09/vectors.json"
    ))
    .unwrap();
    for case in manifest.get("cases").unwrap().array().unwrap() {
        let name = case.get("object").unwrap().string().unwrap();
        let source = std::fs::read_to_string(root.join(name)).unwrap();
        assert_eq!(
            sha256_hex(source.as_bytes()),
            case.get("sha256").unwrap().string().unwrap(),
            "{name}"
        );
        let result = Object::parse(&source).and_then(|object| {
            let root = object
                .section("certificate")
                .unwrap()
                .get("bindings")
                .unwrap()
                .get("closure_hash")
                .unwrap()
                .string()
                .unwrap()
                .to_owned();
            object.verify_certificate().map(|report| (root, report))
        });
        match case.get("applicability").unwrap().string().unwrap() {
            "reject" => assert!(result.is_err(), "{name}: {result:?}"),
            "established" => {
                let (root, report) = result.unwrap();
                assert_eq!(
                    report.qinplace_applicability.get(&root),
                    Some(&QinplaceApplicability::Established),
                    "{name}: {report:?}"
                );
            }
            other => panic!("unknown outcome {other}"),
        }
    }
}
