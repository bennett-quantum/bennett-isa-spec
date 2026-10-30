//! Independent checks of ratified draft witnesses, not 0.9 executor conformance.
use bennett_isa_conformance::{
    cases_dir,
    json::{self, Json},
};
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
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .split_whitespace()
            .next()
            .unwrap(),
        expected
    );
}
#[test]
fn grid_topology_bytes_hash_and_adjacency() {
    let dir = cases_dir().join("E/E-topology-grid-09");
    let doc = checked_document(&dir.join("target.json"));
    let vertices = doc.at_path("topology.vertices").unwrap().as_arr().unwrap();
    assert_eq!(vertices.len(), 4);
    assert!(vertices
        .iter()
        .all(|v| v.get("class").and_then(Json::as_str) == Some("fast")));
    let mut edges = Vec::new();
    for u in 0..4 {
        for v in u + 1..4 {
            if (u / 2 == v / 2 && v == u + 1) || v == u + 2 {
                edges.push(Json::Arr(vec![Json::Num(u as f64), Json::Num(v as f64)]));
            }
        }
    }
    assert_eq!(
        doc.at_path("topology.edges"),
        Some(&Json::Arr(edges.clone()))
    );
    assert!(edges.contains(&Json::Arr(vec![Json::Num(0.0), Json::Num(1.0)])));
    assert!(!edges.contains(&Json::Arr(vec![Json::Num(0.0), Json::Num(3.0)])));
    assert_eq!(
        doc.at_path("classes.fast.capacity"),
        Some(&Json::Num(vertices.len() as f64))
    );
    let case = checked_document_case(&dir.join("case.json"));
    checked_hash(
        &dir.join("target.json"),
        case.at_path("expected_outcome.target_hash")
            .unwrap()
            .as_str()
            .unwrap(),
    );
    let draft = fs::read_to_string(
        cases_dir()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("spec/Bennett-ISA-Spec-0.9-draft.md"),
    )
    .unwrap();
    assert!(draft.contains(&doc.to_string()));
    assert!(draft.contains(
        case.at_path("expected_outcome.target_hash")
            .unwrap()
            .as_str()
            .unwrap()
    ));
}
fn checked_document_case(path: &Path) -> Json {
    json::parse(&fs::read_to_string(path).unwrap()).unwrap()
}

fn width(expr: &Json, c: u64) -> u64 {
    match expr {
        Json::Num(n) => {
            assert!(*n >= 0.0 && n.fract() == 0.0);
            *n as u64
        }
        Json::Obj(m) if m.contains_key("var") => {
            assert_eq!(m.len(), 1);
            assert_eq!(expr.get("var").and_then(Json::as_str), Some("c"));
            c
        }
        _ => {
            assert_eq!(expr.get("op").and_then(Json::as_str), Some("+"));
            let args = expr.get("args").unwrap().as_arr().unwrap();
            assert_eq!(args.len(), 2);
            width(&args[0], c) + width(&args[1], c)
        }
    }
}
#[test]
fn typed_frame_bytes_widths_and_disjoint_lifetimes() {
    let dir = cases_dir().join("F/F-frame-records-09");
    let doc = checked_document(&dir.join("frame.json"));
    let case = checked_document_case(&dir.join("case.json"));
    checked_hash(
        &dir.join("frame.json"),
        case.at_path("expected_outcome.frame_hash")
            .unwrap()
            .as_str()
            .unwrap(),
    );
    for (category, expected) in [
        ("slots", 3),
        ("scratch", 4),
        ("history", 3),
        ("borrow", 1),
        ("ancillae", 2),
    ] {
        let records = doc.get(category).unwrap().as_arr().unwrap();
        let actual: u64 = records
            .iter()
            .map(|r| width(r.get("width").unwrap(), 2))
            .sum();
        assert_eq!(actual, expected);
        assert_eq!(
            case.get("expected_outcome").unwrap().get(category),
            Some(&Json::Num(expected as f64))
        );
    }
    let ranges = doc.get("live_ranges").unwrap().as_arr().unwrap();
    for (i, a) in ranges.iter().enumerate() {
        let start = width(a.get("start").unwrap(), 2);
        let end = width(a.get("end").unwrap(), 2);
        assert!(start < end);
        for b in &ranges[i + 1..] {
            if a.get("frame") == b.get("frame") {
                assert!(
                    end <= width(b.get("start").unwrap(), 2)
                        || width(b.get("end").unwrap(), 2) <= start
                );
            }
        }
    }
    let draft = fs::read_to_string(
        cases_dir()
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("spec/Bennett-ISA-Spec-0.9-draft.md"),
    )
    .unwrap();
    assert!(draft.contains(&doc.to_string()));
    assert!(draft.contains(
        case.at_path("expected_outcome.frame_hash")
            .unwrap()
            .as_str()
            .unwrap()
    ));
}

#[test]
fn measurement_fifo_and_lexicographic_cdf_ties() {
    let dir = cases_dir().join("A/A-measure-draw-order-09");
    let case = checked_document_case(&dir.join("case.json"));
    let outcomes = ["00", "01", "10", "11"];
    // All weights = 1/4; compare integer quarter-units without floating point.
    let values: Vec<_> = [1, 3]
        .iter()
        .map(|draw| {
            let bits = outcomes
                .iter()
                .enumerate()
                .find(|(i, _)| i + 1 > *draw)
                .unwrap()
                .1;
            bits.bytes()
                .enumerate()
                .map(|(i, b)| ((b - b'0') as u64) << i)
                .sum::<u64>()
        })
        .collect();
    assert_eq!(values, [2, 3]);
    assert_ne!(values, [3, 2], "FIFO is observable");
    assert_ne!(
        values,
        [1, 3],
        "lexicographic order differs from integer order"
    );
    assert_eq!(
        case.at_path("expected_outcome.values"),
        Some(&Json::Arr(
            values.iter().map(|v| Json::Num(*v as f64)).collect()
        ))
    );
}
