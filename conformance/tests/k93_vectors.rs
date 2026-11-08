use bennett_isa_conformance::{cases_dir, default_bin, json, load_cases, run_case, Outcome};
use std::collections::BTreeSet;

#[test]
fn all_k93_portable_vectors_run_against_the_published_cli() {
    let manifest = json::parse(include_str!("../fixtures/accepted-k93-09/k93.json")).unwrap();
    let ids = manifest
        .get("conformance_cases")
        .unwrap()
        .as_arr()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 291);
    let temporary = std::path::PathBuf::from(std::env::var("TMPDIR").unwrap()).join("k93-harness");
    std::fs::create_dir_all(&temporary).unwrap();
    let mut seen = BTreeSet::new();
    for case in load_cases(&cases_dir())
        .unwrap()
        .iter()
        .filter(|c| ids.contains(&c.id))
    {
        let outcome = run_case(&default_bin(), case, &temporary);
        assert!(matches!(outcome, Outcome::Pass), "{}: {outcome:?}", case.id);
        assert!(case.unavailable.is_none());
        seen.insert(case.id.clone());
    }
    assert_eq!(seen, ids);
}
