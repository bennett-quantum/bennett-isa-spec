use std::process::Command;
fn fixture(name: &str) -> String {
    format!("{}/tests/fixtures/{name}.bisa", env!("CARGO_MANIFEST_DIR"))
}
fn cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
        .args(args)
        .output()
        .unwrap()
}
#[test]
fn check_and_run_print_machine_readable_json() {
    let output = cli(&["check", &fixture("inc8")]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value =
        bennett_isa_ref::json::Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert_eq!(value.get("status").unwrap().string().unwrap(), "ok");
    for (budget, payload, code, steps) in [("3", 0, 0, 3), ("2", 0, 1, 2), ("0", 0, 1, 0)] {
        let output = cli(&[
            "run-oracle",
            &fixture("inc8"),
            "inc8",
            "--input",
            "255",
            "--budget",
            budget,
        ]);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let value =
            bennett_isa_ref::json::Json::parse(std::str::from_utf8(&output.stdout).unwrap())
                .unwrap();
        for (k, v) in [("payload", payload), ("code", code), ("steps", steps)] {
            assert_eq!(value.get(k).unwrap().uint().unwrap(), v);
        }
    }
}
#[test]
fn exhaustive_step_check_and_diagnostics() {
    let output = cli(&[
        "step-check",
        &fixture("inc8"),
        "inc8",
        "--exhaustive",
        "--budget",
        "4",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value =
        bennett_isa_ref::json::Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert_eq!(value.get("cases").unwrap().uint().unwrap(), 256);
    for args in [
        vec!["nonsense"],
        vec!["check", "/no/such/object"],
        vec!["run-oracle"],
        vec!["run-oracle", &fixture("inc8"), "inc8", "--input", "256"],
    ] {
        let output = cli(&args);
        assert!(!output.status.success());
        assert!(!output.stderr.is_empty());
    }
}
