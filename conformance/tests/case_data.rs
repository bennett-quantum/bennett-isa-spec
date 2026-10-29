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
                assert!(
                    !u.reason.trim().is_empty(),
                    "{}: unavailable without reason",
                    c.id
                );
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

/// These named witnesses were absent in the resumed 182-case suite.
#[test]
fn acceptance_rule_witnesses_exist() {
    let cases = load_cases(&cases_dir()).unwrap();
    for id in [
        "A-recursion",
        "A-recursion-reversal",
        "F-version",
        "F-name-resolution",
        "F-permutation-only-oracle",
        "F-window-closed",
        "F-adjoint-table",
        "F-index-space-flattening",
        "E-imports",
        "H-unsupported",
        "G-suspend-resume",
        "G-argument-kinds",
        "G-accessor-lifetimes",
    ] {
        assert!(
            cases.iter().any(|c| c.id == id),
            "missing acceptance witness {id}"
        );
    }
}

/// Independent expected tuples, rather than two engines agreeing with each other.
#[test]
fn small_t3_domains_are_exhaustive() {
    let cases = load_cases(&cases_dir()).unwrap();
    for (prefix, count) in [
        ("C-domain-oracle-", 256),
        ("C-domain-flip-", 16),
        ("C-domain-qinplace-", 4),
        ("C-domain-borrow-", 8),
    ] {
        let domain: Vec<_> = cases.iter().filter(|c| c.id.starts_with(prefix)).collect();
        assert_eq!(domain.len(), count, "incomplete T3 domain {prefix}");
        let commands: BTreeSet<_> = domain.iter().map(|c| c.command.as_ref().unwrap()).collect();
        assert_eq!(commands.len(), count, "duplicate input in {prefix}");
        assert!(domain
            .iter()
            .all(|c| c.expect.stdout_json.is_some() && c.unavailable.is_none()));
    }
}

#[test]
fn staged_cases_have_structured_inputs_and_expected_outcomes() {
    use bennett_isa_conformance::json::{self, Json};
    for case in load_cases(&cases_dir()).unwrap() {
        if case.unavailable.is_none() {
            continue;
        }
        let doc =
            json::parse(&std::fs::read_to_string(case.dir.join("case.json")).unwrap()).unwrap();
        for key in ["scenario", "expected_outcome"] {
            assert!(
                matches!(doc.get(key), Some(Json::Obj(m)) if !m.is_empty()),
                "{}: missing nonempty staged {key}",
                case.id
            );
        }
    }
}
