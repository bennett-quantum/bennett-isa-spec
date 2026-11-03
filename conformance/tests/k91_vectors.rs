use bennett_isa_conformance::{
    cases_dir, default_bin, json, load_cases, ref_hash::sha256, run_case, Outcome,
};
use std::{collections::BTreeSet, path::PathBuf};
const MANIFEST: &str = include_str!("../fixtures/accepted-k91-09/k91.json");
#[test]
fn all_published_k91_case_commands_pass_the_portable_cli_harness() {
    assert_eq!(
        sha256(MANIFEST.as_bytes()),
        "72b68db2dfbb041fc259f95d3669ff29438bfec5cdcb356fc4b1c713b7dd53fa"
    );
    let m = json::parse(MANIFEST).unwrap();
    let ids = m
        .get("conformance_cases")
        .unwrap()
        .as_arr()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 21);
    let cases = load_cases(&cases_dir()).unwrap();
    let temporary = PathBuf::from(std::env::var("TMPDIR").unwrap()).join("k91-harness");
    std::fs::create_dir_all(&temporary).unwrap();
    let mut seen = BTreeSet::new();
    for case in cases.iter().filter(|c| ids.contains(&c.id)) {
        assert!(
            matches!(run_case(&default_bin(), case, &temporary), Outcome::Pass),
            "{}",
            case.id
        );
        assert!(case.unavailable.is_none());
        seen.insert(case.id.clone());
    }
    assert_eq!(seen, ids);
}
