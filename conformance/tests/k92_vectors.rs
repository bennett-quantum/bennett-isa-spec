use bennett_isa_conformance::{
    json::{self, Json},
    ref_hash::sha256,
};
#[test]
fn qinplace_vector_sources_pin_actual_distinct_token_budgets_and_default_schedules() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/accepted-k92-09");
    let manifest = json::parse(include_str!("../fixtures/accepted-k92-09/vectors.json")).unwrap();
    let cases = manifest.get("cases").unwrap().as_arr().unwrap();
    assert_eq!(cases.len(), 8);
    for case in cases {
        let file = case.get("object").unwrap().as_str().unwrap();
        let source = std::fs::read_to_string(root.join(file)).unwrap();
        assert_eq!(
            sha256(source.as_bytes()),
            case.get("sha256").unwrap().as_str().unwrap()
        );
        if file.starts_with("outer-3-") {
            let cert =
                json::parse(source.split("section certificate\n").nth(1).unwrap().trim()).unwrap();
            let invocation = cert.at_path("bindings.invocation").unwrap();
            let args = invocation.get("args").unwrap().as_arr().unwrap();
            assert_eq!(args[0].as_arr().unwrap()[0].as_str(), Some("3"));
            assert_eq!(
                args[1].as_arr().unwrap()[0].as_str(),
                Some(if file.contains("3-2") { "2" } else { "3" })
            );
            assert_eq!(
                invocation.get("budget").unwrap().as_arr().unwrap()[0].as_str(),
                Some("9")
            );
            assert_eq!(
                invocation.get("token_aliases").unwrap().as_arr().unwrap(),
                &[Json::Num(0.0), Json::Num(1.0)]
            );
            if file.contains("scheduled") || file.contains("adjoint") {
                assert!(source.contains("\"entries\":[]"));
            }
        }
    }
}
