//! Shared deterministic portable-vector writer/checker. Updates are explicit.
pub fn quote(s: &str) -> String {
    bennett_isa_conformance::json::Json::Str(s.into()).to_string()
}
pub fn lock(name: &str, rows: Vec<String>) {
    let raw = format!(
        r#"{{"schema":"bennett.k78-k80.v1","vectors":[{}]}}"#,
        rows.join(",")
    );
    let bytes = format!("{}\n", bennett_isa_conformance::json::parse(&raw).unwrap());
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../conformance/fixtures/accepted-k78-k80-09")
        .join(name);
    if std::env::var_os("BENNETT_UPDATE_ROUND8_VECTORS").is_some() {
        std::fs::write(&path, &bytes).unwrap();
    }
    assert_eq!(
        std::fs::read_to_string(path).unwrap(),
        bytes,
        "portable vector bytes: {name}"
    );
}
