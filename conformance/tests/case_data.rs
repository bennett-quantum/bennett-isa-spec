//! Data integrity of the case files themselves: every suite A-H has
//! cases, every case cites the spec sentence it tests and shows its
//! hand derivation, referenced objects exist, ids are unique, and
//! unavailable cases name a staged deliverable (8-11).

use bennett_isa_conformance::{cases_dir, load_cases};
use std::collections::BTreeSet;

#[test]
fn all_suites_present_and_cases_well_formed() {
    let root = cases_dir();
    let cases = load_cases(&root).expect("cases load");
    assert!(!cases.is_empty(), "no cases under {}", root.display());

    let mut ids = BTreeSet::new();
    let mut suites = BTreeSet::new();
    for c in &cases {
        assert!(ids.insert(c.id.clone()), "duplicate case id {}", c.id);
        suites.insert(c.suite.clone());
        assert!(
            c.id.starts_with(&format!("{}-", c.suite)),
            "case id {} does not start with its suite letter",
            c.id
        );
        assert!(!c.spec.trim().is_empty(), "{}: empty spec citation", c.id);
        assert!(
            c.spec.contains('§'),
            "{}: spec citation names no section: {}",
            c.id,
            c.spec
        );
        assert!(
            !c.derivation.trim().is_empty(),
            "{}: empty hand derivation",
            c.id
        );
        if let Some(obj) = &c.object {
            assert!(
                obj.is_file(),
                "{}: object {} does not exist",
                c.id,
                obj.display()
            );
        }
        match &c.unavailable {
            Some(u) => {
                assert!(
                    (8..=11).contains(&u.deliverable),
                    "{}: unavailable deliverable {} is not a staged deliverable 8-11",
                    c.id,
                    u.deliverable
                );
                assert!(!u.reason.trim().is_empty(), "{}: unavailable without reason", c.id);
            }
            None => {
                assert!(
                    c.command.is_some(),
                    "{}: available case has no command to run",
                    c.id
                );
            }
        }
    }
    for s in ["A", "B", "C", "D", "E", "F", "G", "H"] {
        assert!(suites.contains(s), "suite {s} has no cases");
    }
}
