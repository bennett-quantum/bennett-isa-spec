//! Run the whole suite against a real binary (default: the reference
//! interpreter, or `$BENNETT_CONFORMANCE_BIN`). Every available case
//! must pass; unavailable cases must be reported as unavailable (with
//! their probes, where a probe exists, confirming a loud refusal).
//! A missing binary is a hard failure with build instructions, never
//! a skip (CLAUDE.md rule 1).

use bennett_isa_conformance::{cases_dir, default_bin, load_cases, run_all};
use std::path::PathBuf;

#[test]
fn suite_against_reference_binary() {
    let bin = default_bin();
    assert!(
        bin.is_file(),
        "binary under test not found at {} — build it \
         (cargo build --manifest-path ../reference-interpreter/Cargo.toml --offline) \
         or set BENNETT_CONFORMANCE_BIN",
        bin.display()
    );
    let root = cases_dir();
    let cases = load_cases(&root).expect("cases load");
    let tmp = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp/run");
    std::fs::create_dir_all(&tmp).expect("create conformance/.tmp/run");
    let report = run_all(&bin, &cases, &tmp);
    for line in report.lines() {
        println!("{line}");
    }
    assert_eq!(
        report.failed(),
        0,
        "{} case(s) failed against {}",
        report.failed(),
        bin.display()
    );
    assert!(
        report.passed() > 0,
        "no case passed; harness is not running cases"
    );
}
