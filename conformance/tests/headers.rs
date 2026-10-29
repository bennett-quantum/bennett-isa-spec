//! Compile every type, enumerator and function in the normative §10/§11
//! C blocks, with signature/value checks, as C11 and C++17. The spec is
//! the independent symbol list; empty headers must fail. Scratch stays
//! under conformance/.tmp and compiler children run headless.

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
    for (compiler, standard, extension) in [("cc", "c11", "c"), ("c++", "c++17", "cpp")] {
        compile_as(name, source, compiler, standard, extension);
    }
}

fn compile_as(name: &str, source: &str, compiler: &str, standard: &str, extension: &str) {
    let dir = tmpdir();
    let src = dir.join(format!("{name}.{extension}"));
    let obj = dir.join(format!("{name}.{extension}.o"));
    fs::write(&src, source).expect("write TU");
    let out = Command::new(compiler)
        .arg(format!("-std={standard}"))
        .args(["-Wall", "-Werror", "-pedantic-errors", "-I"])
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
        .unwrap_or_else(|e| panic!("cannot run required compiler {compiler}: {e}"));
    assert!(
        out.status.success(),
        "{compiler} -std={standard} -Wall -Werror failed for {name}:\n{}",
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
    compile(
        "both_sched_first",
        "#include \"bennett_sched.h\"\n#include \"bennett_launch.h\"\nint main(void) { return 0; }\n",
    );
}

// Fail loudly if the normative declaration grammar changes. Never silently
// drop a declaration because this deliberately small parser cannot handle it.
fn declarations(section: u8) -> Vec<String> {
    let spec = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../spec/Bennett-ISA-Spec.md"),
    )
    .expect("read normative ISA");
    let heading = format!("\n## {section}. ");
    let section = spec.split_once(&heading).expect("normative section").1;
    let section = section.split("\n## ").next().unwrap();
    let block = section.split_once("```c\n").expect("normative C block").1;
    let block = block.split_once("\n```").expect("closed C block").0;
    let mut text = block.to_owned();
    while let Some(start) = text.find("/*") {
        let end = start + text[start..].find("*/").expect("closed C comment") + 2;
        text.replace_range(start..end, "");
    }
    let mut parts: Vec<_> = text.split(';').collect();
    assert!(
        parts.pop().unwrap().trim().is_empty(),
        "unterminated declaration"
    );
    parts
        .into_iter()
        .map(|part| part.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect()
}

fn contract(section: u8, header: &str, counts: (usize, usize, usize)) -> String {
    let mut source = format!(
        "#include <stddef.h>\n#include <stdint.h>\n#include \"{header}\"\n#include \"{header}\"\n\
         #ifdef __cplusplus\n#include <type_traits>\n\
         #define CHECK(c, m) static_assert(c, m)\n\
         #define SAME(expr, type) (std::is_same<decltype(expr), type>::value)\n\
         #else\n#define CHECK(c, m) _Static_assert(c, m)\n\
         #define SAME(expr, type) _Generic((expr), type: 1, default: 0)\n#endif\n"
    );
    let (mut functions, mut types, mut constants) = (0, 0, 0);
    let mut prototypes = String::new();
    for decl in declarations(section) {
        if decl.starts_with("typedef enum {") {
            types += 1;
            let (entries, name) = decl["typedef enum {".len()..].split_once('}').unwrap();
            let name = name.trim();
            source.push_str(&format!("CHECK(sizeof({name}) > 0, \"{name}\");\n"));
            let mut next_value = 0;
            for entry in entries.split(',') {
                let (constant, value) = match entry.trim().split_once('=') {
                    Some((constant, value)) => (constant.trim(), value.trim().parse().unwrap()),
                    None => (entry.trim(), next_value),
                };
                assert!(constant.starts_with("BENNETT_"), "unknown enum: {decl}");
                source.push_str(&format!("CHECK({constant} == {value}, \"{constant}\");\n"));
                next_value = value + 1;
                constants += 1;
            }
        } else if let Some(rest) = decl.strip_prefix("typedef ") {
            types += 1;
            let (underlying, name) = rest.rsplit_once(' ').unwrap();
            assert!(name.starts_with("bennett_"), "unknown typedef: {decl}");
            let expression = if underlying.starts_with("struct ") {
                format!("SAME(({name}*)0, {underlying}*)")
            } else {
                assert_eq!(underlying, "uint64_t", "unknown typedef: {decl}");
                format!("SAME(({name})0, {underlying})")
            };
            source.push_str(&format!("CHECK({expression}, \"{name}\");\n"));
        } else {
            let (head, args) = decl.split_once('(').expect("function declaration");
            let (result, name) = head.trim_end().rsplit_once(' ').unwrap();
            assert!(name.starts_with("bennett_"), "unknown function: {decl}");
            assert!(args.ends_with(')'), "unknown arguments: {decl}");
            source.push_str(&format!(
                "typedef {result} (*expected_{name})({args};\n\
                 CHECK(SAME(&{name}, expected_{name}), \"{name} signature\");\n"
            ));
            // A second declaration with explicit C linkage detects a missing
            // extern "C" wrapper in C++ (type_traits alone cannot do that).
            prototypes.push_str(&format!("{decl};\n"));
            functions += 1;
        }
    }
    assert_eq!(
        (functions, types, constants),
        counts,
        "§{section} symbol inventory changed; review all declarations"
    );
    source.push_str("#ifdef __cplusplus\nextern \"C\" {\n#endif\n");
    source.push_str(&prototypes);
    source.push_str("#ifdef __cplusplus\n}\n#endif\nint main(void) { return 0; }\n");
    source
}

#[test]
fn every_launch_symbol_matches_section_10() {
    compile(
        "launch_contract",
        &contract(10, "bennett_launch.h", (41, 12, 17)),
    );
}

#[test]
fn every_sched_symbol_matches_section_11() {
    compile(
        "sched_contract",
        &contract(11, "bennett_sched.h", (15, 3, 3)),
    );
}
