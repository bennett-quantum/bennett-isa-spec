//! Compile both shipped C headers with `cc` as C11 under `-Wall -Werror`
//! (brief task 3). Each header is included twice so a broken include
//! guard fails, and both are included in one translation unit so a
//! clash between them fails. Headless: scratch under `conformance/.tmp`.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn repo_include() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../include")
}

fn tmpdir() -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp/headers");
    fs::create_dir_all(&d).expect("create conformance/.tmp/headers");
    d
}

fn compile(name: &str, source: &str) {
    let dir = tmpdir();
    let src = dir.join(format!("{name}.c"));
    let obj = dir.join(format!("{name}.o"));
    fs::write(&src, source).expect("write TU");
    let out = Command::new("cc")
        .args(["-std=c11", "-Wall", "-Werror", "-I"])
        .arg(repo_include())
        .arg("-c")
        .arg(&src)
        .arg("-o")
        .arg(&obj)
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("DBUS_SESSION_BUS_ADDRESS")
        .env_remove("BROWSER")
        .env("TMPDIR", &dir)
        .output()
        .unwrap_or_else(|e| panic!("cannot run cc (required by brief task 3): {e}"));
    assert!(
        out.status.success(),
        "cc -std=c11 -Wall -Werror failed for {name}:\n{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn bennett_launch_h_compiles() {
    compile(
        "launch",
        "#include \"bennett_launch.h\"\n#include \"bennett_launch.h\"\nint main(void) { return 0; }\n",
    );
}

#[test]
fn bennett_sched_h_compiles() {
    compile(
        "sched",
        "#include \"bennett_sched.h\"\n#include \"bennett_sched.h\"\nint main(void) { return 0; }\n",
    );
}

#[test]
fn both_headers_in_one_tu() {
    compile(
        "both",
        "#include \"bennett_launch.h\"\n#include \"bennett_sched.h\"\nint main(void) { return 0; }\n",
    );
}
