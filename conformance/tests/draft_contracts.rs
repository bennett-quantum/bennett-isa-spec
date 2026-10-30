//! Draft-only witness checks: these do not claim 0.9 executor conformance.
#[test]
fn draft_contract_witnesses() {
    let result = std::process::Command::new("python3")
        .arg("-I")
        .arg(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/draft_witnesses.py"
        ))
        .output()
        .expect("python3 is required for independent draft witness checks");
    assert!(
        result.status.success(),
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
