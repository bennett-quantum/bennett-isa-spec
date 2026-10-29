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

#[test]
fn wide_oracle_cli_preserves_exact_decimal_inputs_and_outputs() {
    let output=cli(&["run-oracle",&fixture("wide"),"div257","--input","1606938044258990275541962092341162602522202993782792835301379,1267650600228229401496703205381","--budget","2,0,16,4"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value =
        bennett_isa_ref::json::Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert_eq!(
        value.get("payload").unwrap().string().unwrap(),
        "1267650600228229401496703205371"
    );
    assert_eq!(value.get("code").unwrap().uint().unwrap(), 0);
}

#[test]
fn step_check_accepts_a_small_domain_with_wide_output_and_fault_wires() {
    let output = cli(&[
        "step-check",
        &fixture("wide"),
        "fault_small257",
        "--exhaustive",
        "--budget",
        "2,0,16,4",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value =
        bennett_isa_ref::json::Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert_eq!(value.get("cases").unwrap().uint().unwrap(), 2);
}

#[test]
fn step_check_modes_follow_the_suite_size_default() {
    // §17 item 13: exhaustive for <= 16 interface bits (inc8: 8 bits).
    let output = cli(&[
        "step-check",
        &fixture("inc8"),
        "inc8",
        "--suite",
        "--budget",
        "4",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v =
        bennett_isa_ref::json::Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert_eq!(v.get("mode").unwrap().string().unwrap(), "exhaustive");
    assert_eq!(v.get("cases").unwrap().uint().unwrap(), 256);
    // sum_db's domain is 8 + 128 bits: explicit sampling, reported as such.
    let output = cli(&[
        "step-check",
        &fixture("sum_db"),
        "sum_db",
        "--samples",
        "40",
        "--budget",
        "300,4080,512,64",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v =
        bennett_isa_ref::json::Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert_eq!(v.get("mode").unwrap().string().unwrap(), "sampled");
    assert_eq!(v.get("cases").unwrap().uint().unwrap(), 40);
    // --max-bits is a user guard with no fixed ceiling (the former limit was 24).
    let output = cli(&[
        "step-check",
        &fixture("inc8"),
        "inc8",
        "--exhaustive",
        "--max-bits",
        "40",
        "--budget",
        "3",
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = cli(&[
        "step-check",
        &fixture("sum_db"),
        "sum_db",
        "--exhaustive",
        "--budget",
        "300,4080,512,64",
    ]);
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("§17.13"));
}
#[test]
fn check_reports_unlinked_objects() {
    let sig =
        r#"{"abi":"0.8","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#;
    let lproc = r#"{"abi":"0.8","kind":"lproc","params":[8],"result":8,"tier":"q"}"#;
    let text = format!(
        "bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{{\"0\":{{\"interface_hash\":\"{}\",\"kind\":\"oracle\",\"name\":\"main\",\"signature\":{sig}}}}}\n\nsection imports\n{{\"inc\":{{\"signature\":{lproc}}}}}\n\nsection functions\noracle @main (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()\nregs {{ r:reg[8] }}\n^entry():\nr = lift.call @inc (x)\nlift.retval r\n\nsection layout\n{{\"A_tab\":0,\"H\":8,\"S_cells\":4,\"globals\":{{}},\"heap_base\":16,\"heap_limit_max\":0,\"interface_regions\":{{}},\"null_size\":1,\"ptr_width\":32}}\n",
        "0".repeat(64)
    );
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("unlinked.bisa");
    std::fs::write(&path, text).unwrap();
    let output = cli(&["hash", path.to_str().unwrap()]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let v =
        bennett_isa_ref::json::Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap();
    assert_eq!(v.get("status").unwrap().string().unwrap(), "unlinked");
    assert_eq!(
        v.get("unresolved_imports").unwrap().array().unwrap().len(),
        1
    );
}
