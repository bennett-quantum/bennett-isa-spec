//! Independent checks of ratified draft witnesses, not 0.9 executor conformance.
use bennett_isa_conformance::{cases_dir, json::{self, Json}};
use std::{fs, path::Path, process::Command};

fn checked_document(path: &Path) -> Json {
    let bytes = fs::read(path).unwrap();
    assert_eq!(bytes.last(), Some(&b'}'), "JCS has no trailing LF");
    let doc = json::parse(std::str::from_utf8(&bytes).unwrap()).unwrap();
    // Witnesses use ASCII strings without escapes and integer numbers only;
    // BTreeMap ordering and Display are a separate canonicalizer for this subset.
    assert_eq!(doc.to_string().as_bytes(), bytes);
    doc
}
fn checked_hash(path: &Path, expected: &str) {
    let output = Command::new("sha256sum").arg(path).output().unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().split_whitespace().next().unwrap(), expected);
}
#[test]
fn grid_topology_bytes_hash_and_adjacency() {
    let dir = cases_dir().join("E/E-topology-grid-09");
    let doc = checked_document(&dir.join("target.json"));
    let vertices = doc.at_path("topology.vertices").unwrap().as_arr().unwrap();
    assert_eq!(vertices.len(), 4);
    assert!(vertices.iter().all(|v| v.get("class").and_then(Json::as_str) == Some("fast")));
    let mut edges = Vec::new();
    for u in 0..4 {
        for v in u+1..4 {
            if (u / 2 == v / 2 && v == u+1) || v == u+2 {
                edges.push(Json::Arr(vec![Json::Num(u as f64), Json::Num(v as f64)]));
            }
        }
    }
    assert_eq!(doc.at_path("topology.edges"), Some(&Json::Arr(edges.clone())));
    assert!(edges.contains(&Json::Arr(vec![Json::Num(0.0), Json::Num(1.0)])));
    assert!(!edges.contains(&Json::Arr(vec![Json::Num(0.0), Json::Num(3.0)])));
    assert_eq!(doc.at_path("classes.fast.capacity"), Some(&Json::Num(vertices.len() as f64)));
    let case = checked_document_case(&dir.join("case.json"));
    checked_hash(&dir.join("target.json"), case.at_path("expected_outcome.target_hash").unwrap().as_str().unwrap());
    let draft = fs::read_to_string(cases_dir().parent().unwrap().parent().unwrap().join("spec/Bennett-ISA-Spec-0.9-draft.md")).unwrap();
    assert!(draft.contains(&doc.to_string()));
    assert!(draft.contains(case.at_path("expected_outcome.target_hash").unwrap().as_str().unwrap()));
}
fn checked_document_case(path: &Path) -> Json {
    json::parse(&fs::read_to_string(path).unwrap()).unwrap()
}
