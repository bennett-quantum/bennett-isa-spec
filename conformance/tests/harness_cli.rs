//! End-to-end failure policy: no empty success, and staged work fails loudly.
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn run(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_bennett-isa-conformance"))
        .args(args)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("DBUS_SESSION_BUS_ADDRESS")
        .env_remove("BROWSER")
        .env(
            "TMPDIR",
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp"),
        )
        .output()
        .unwrap()
}

#[test]
fn empty_case_directory_is_an_error() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp/empty-cases");
    fs::create_dir_all(&root).unwrap();
    let out = run(&["--cases", root.to_str().unwrap(), "--bin", "/bin/true"]);
    assert_eq!(
        out.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("no cases"));
}

#[test]
fn unavailable_is_a_nonzero_counted_outcome() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("cases/G/G-copied-arguments");
    let out = run(&["--cases", root.to_str().unwrap(), "--bin", "/bin/true"]);
    assert_eq!(out.status.code(), Some(2));
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("unavailable: deliverable 8"), "{stdout}");
    assert!(stdout.contains("| G | 0 | 0 | 1 | 1 |"), "{stdout}");
}
