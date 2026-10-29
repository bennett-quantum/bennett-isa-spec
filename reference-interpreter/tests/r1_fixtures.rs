//! Council round 1: codex B18 (a fixture audit that audits every fixture), and
//! fable's suggestion that the §16.3 fixtures equal the spec text byte for byte.
use bennett_isa_ref::Object;
use std::path::PathBuf;

fn fixtures() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut out: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "bisa"))
        .collect();
    out.sort();
    out
}

#[test]
fn b18_every_bisa_file_is_audited_and_has_a_comments_file() {
    let all = fixtures();
    // Inventory: the audit covers whatever is committed, never a hand list.
    assert!(all.len() >= 12, "fixture inventory shrank: {all:?}");
    for path in &all {
        let text = std::fs::read_to_string(path).unwrap();
        Object::parse(&text)
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()))
            .check_hashes()
            .unwrap_or_else(|e| panic!("{}: {e}", path.display()));
        assert!(
            path.with_extension("comments").exists(),
            "{} lacks its hand-derived .comments file",
            path.display()
        );
    }
}

#[test]
fn b18_a_corrupted_stored_hash_is_detected_by_the_same_audit() {
    let path = fixtures()
        .into_iter()
        .find(|p| p.ends_with("tiny_memory.bisa"))
        .unwrap();
    let text = std::fs::read_to_string(path).unwrap();
    let start = text.find("\"interface_hash\":\"").unwrap() + 18;
    let mut corrupted = text.clone();
    let flipped = if &text[start..start + 1] == "0" {
        "1"
    } else {
        "0"
    };
    corrupted.replace_range(start..start + 1, flipped);
    let o = Object::parse(&corrupted).unwrap();
    assert!(o.check_hashes().unwrap_err().to_string().contains("§9.2"));
}

/// The n-th fenced block after `marker` in the specification.
fn spec_block(marker: &str, n: usize) -> String {
    let spec = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spec/Bennett-ISA-Spec.md"),
    )
    .unwrap();
    let mut rest = &spec[spec.find(marker).unwrap()..];
    let mut block = String::new();
    for _ in 0..=n {
        let open = rest.find("```\n").unwrap() + 4;
        let close = rest[open..].find("```").unwrap() + open;
        block = rest[open..close].to_string();
        rest = &rest[close + 3..];
    }
    block
}

#[test]
fn published_example_objects_are_the_fixtures_byte_for_byte() {
    assert_eq!(
        spec_block("**Example 1** (`inc8`)", 0),
        include_str!("fixtures/inc8.bisa")
    );
    assert_eq!(
        spec_block("**Example 2** (`sum_db`)", 0),
        include_str!("fixtures/sum_db.bisa")
    );
}
