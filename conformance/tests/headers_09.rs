//! Published header contracts and ABI-family selection, checked independently
//! against the normative C blocks with LLVM 18 and 23.
#[test]
fn published_header_families() {
    let result = std::process::Command::new("python3")
        .arg("-I")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/headers_09.py"))
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("DBUS_SESSION_BUS_ADDRESS")
        .env_remove("BROWSER")
        .output()
        .expect("python3 is required for published header contract checks");
    assert!(
        result.status.success(),
        "{}{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
}
