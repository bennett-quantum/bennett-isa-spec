//! The harness is implementation-agnostic: it must judge any binary
//! that speaks the reference CLI, including this shell-script fake.
//! A fake that returns the derived outcome passes; one that returns a
//! wrong payload fails; an unavailable case whose probe is silently
//! accepted (exit 0) also fails, loudly.

use bennett_isa_conformance::{load_cases, run_case, Outcome};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

fn scratch() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp/fake-bin-test");
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).expect("create scratch");
    d
}

fn write_fake(dir: &Path, name: &str, body: &str) -> PathBuf {
    let p = dir.join(name);
    fs::write(&p, body).expect("write fake bin");
    fs::set_permissions(&p, fs::Permissions::from_mode(0o755)).expect("chmod fake bin");
    p
}

fn write_case(root: &Path, json: &str) {
    let d = root.join("B-fake");
    fs::create_dir_all(&d).expect("case dir");
    fs::write(d.join("object.bisa"), "bennett-object fake\n").expect("object");
    fs::write(d.join("case.json"), json).expect("case.json");
}

const RUNNABLE: &str = r#"{
  "id": "B-fake",
  "suite": "B",
  "spec": "§16.3 example 1: with B >= 3 every x yields (x + 1 mod 256, 0)",
  "derivation": "x=8: one=1, r=8+1=9, retval in row 3, so payload 9 code 0",
  "object": "object.bisa",
  "command": ["run-oracle", "{object}", "inc8", "--input", "8", "--budget", "3"],
  "expect": {"exit": 0, "stdout_json": {"code": 0, "payload": 9, "steps": 3}}
}"#;

const UNAVAILABLE: &str = r#"{
  "id": "B-fake",
  "suite": "B",
  "spec": "§8 target descriptions (suite E vocabulary, probed here)",
  "derivation": "no outcome derivable until the staged deliverable lands",
  "object": "object.bisa",
  "command": ["run-program", "{object}", "main", "--target", "{dir}/target.json"],
  "expect": {"exit": 1, "stdout_empty": true, "stderr_contains": ["deliverable 9"]},
  "unavailable": {"deliverable": 9, "reason": "target descriptions are staged"}
}"#;

#[test]
fn fake_bin_pass_fail_and_unavailable_probe() {
    let dir = scratch();
    let good = write_fake(
        &dir,
        "good",
        "#!/bin/sh\necho '{\"code\":0,\"payload\":9,\"steps\":3}'\n",
    );
    let wrong = write_fake(
        &dir,
        "wrong",
        "#!/bin/sh\necho '{\"code\":0,\"payload\":10,\"steps\":3}'\n",
    );
    let silent = write_fake(&dir, "silent", "#!/bin/sh\necho '{\"status\":\"ok\"}'\n");
    let refusing = write_fake(
        &dir,
        "refusing",
        "#!/bin/sh\necho 'staged deliverable 9: not yet' >&2\nexit 1\n",
    );

    let root = dir.join("cases-runnable");
    write_case(&root, RUNNABLE);
    let cases = load_cases(&root).expect("load runnable");
    assert_eq!(cases.len(), 1);
    match run_case(&good, &cases[0], &dir) {
        Outcome::Pass => {}
        other => panic!("good fake should pass, got {other:?}"),
    }
    match run_case(&wrong, &cases[0], &dir) {
        Outcome::Fail(msg) => assert!(msg.contains("payload"), "diff names field: {msg}"),
        other => panic!("wrong fake should fail, got {other:?}"),
    }

    let root2 = dir.join("cases-unavailable");
    write_case(&root2, UNAVAILABLE);
    let cases2 = load_cases(&root2).expect("load unavailable");
    match run_case(&refusing, &cases2[0], &dir) {
        Outcome::Unavailable { deliverable, .. } => assert_eq!(deliverable, 9),
        other => panic!("refusing fake should confirm unavailable, got {other:?}"),
    }
    match run_case(&silent, &cases2[0], &dir) {
        Outcome::Fail(_) => {}
        other => panic!("silently-accepting fake must fail the probe, got {other:?}"),
    }
}
