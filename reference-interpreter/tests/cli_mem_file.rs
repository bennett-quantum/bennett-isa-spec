use bennett_isa_ref::json::Json;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

struct Scratch(PathBuf);

impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "cli-mem-file-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, name: &str, bytes: &[u8]) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, bytes).unwrap();
        path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[derive(Clone, Copy)]
struct Case {
    command: &'static str,
    fixture: &'static str,
    entry: &'static str,
    args: &'static [&'static str],
    region: &'static str,
    bytes: &'static [u8],
    hex: &'static str,
    expected: &'static str,
}

fn cases() -> Vec<Case> {
    let mut cases = vec![Case {
        command: "run-oracle",
        fixture: "tiny_memory",
        entry: "tiny",
        args: &["--input", "1", "--budget", "20"],
        region: "db",
        bytes: &[0xa5],
        hex: "a5",
        // tiny computes the low bit of db[0] / x in eight steps.
        expected: r#"{"code":0,"payload":1,"steps":8}"#,
    }];
    cases.push(Case {
        command: "step-check",
        args: &["--exhaustive", "--budget", "20"],
        expected: r#"{"cases":2,"fixed_regions":["db"],"mode":"exhaustive"}"#,
        ..cases[0]
    });
    cases.push(Case {
        command: "reverse",
        expected: r#"{"equals_entry":true,"status":"REVERSED"}"#,
        ..cases[0]
    });
    for command in ["run-action", "reverse"] {
        cases.push(Case {
            command,
            fixture: "control_flow",
            entry: "exchange",
            args: &["--mut", "2", "--in", "1"],
            region: "m",
            bytes: &[1, 3],
            hex: "0103",
            // exchange swaps t=2 with m[1]=3; reversal restores both.
            expected: match command {
                "run-action" => r#"{"mut":[3],"mem":{"m":[1,2]}}"#,
                _ => r#"{"equals_entry":true,"status":"REVERSED"}"#,
            },
        });
    }
    for command in ["run-cc", "run-program", "reverse"] {
        cases.push(Case {
            command,
            fixture: "cli_mem_file/global",
            entry: "read",
            args: &["--args", "8"],
            region: "seed",
            // A NUL, newline and non-UTF-8 bytes must survive raw loading.
            bytes: &[0, 10, 128, 255],
            hex: "000a80ff",
            // Little-endian i32: 255*2^24 + 128*2^16 + 10*2^8.
            expected: match command {
                "reverse" => r#"{"equals_entry":true,"status":"REVERSED"}"#,
                _ => r#"{"results":[{"type":"i32","value":"4286581248"}],"status":"COMPLETED"}"#,
            },
        });
    }
    cases
}

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(format!("{name}.bisa"))
}

fn cli(case: &Case, memory: &[(&str, &str)]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"));
    command
        .arg(case.command)
        .arg(fixture(case.fixture))
        .arg(case.entry)
        .args(case.args);
    for (option, value) in memory {
        command.args([option, value]);
    }
    command.output().unwrap()
}

fn success(output: &Output) -> Json {
    assert_eq!(
        output.status.code(),
        Some(0),
        "stdout: {}\nstderr: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.stderr.is_empty(), "{output:?}");
    Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
}

fn failure(output: &Output, needles: &[&str]) {
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty(), "{output:?}");
    let stderr = String::from_utf8_lossy(&output.stderr);
    for needle in needles {
        assert!(stderr.contains(needle), "{stderr:?} must name {needle:?}");
    }
}

#[test]
fn mem_file_matches_inline_bytes_for_every_command_and_reverse_kind() {
    let scratch = Scratch::new();
    for case in cases() {
        // Splitting name=path must preserve spaces and subsequent '=' signs.
        let path = scratch.write("raw region=bytes.bin", case.bytes);
        let file = format!("{}={}", case.region, path.display());
        let inline = format!("{}={}", case.region, case.hex);
        let expected = cli(&case, &[("--mem", &inline)]);
        let actual = cli(&case, &[("--mem-file", &file)]);
        let value = success(&actual);
        success(&expected);
        assert_eq!(actual.status.code(), expected.status.code());
        assert_eq!(actual.stdout, expected.stdout, "{}", case.command);
        for (field, want) in Json::parse(case.expected).unwrap().object().unwrap() {
            assert_eq!(value.get(field).unwrap(), want, "{} {field}", case.command);
        }
    }
}

#[test]
fn mem_file_preserves_inline_byte_count_diagnostics() {
    let scratch = Scratch::new();
    let path = scratch.write("wrong-size.bin", &[0, 1, 2]);
    for case in cases() {
        let file = format!("{}={}", case.region, path.display());
        let inline = format!("{}=000102", case.region);
        let expected = cli(&case, &[("--mem", &inline)]);
        let actual = cli(&case, &[("--mem-file", &file)]);
        failure(&expected, &[case.region]);
        failure(&actual, &[case.region]);
        assert_eq!(actual.status.code(), expected.status.code());
        assert_eq!(actual.stderr, expected.stderr, "{}", case.command);
    }
}

#[test]
fn mem_file_names_missing_files_and_regions() {
    let scratch = Scratch::new();
    let path = scratch.0.join("missing region.bin");
    for case in cases() {
        let file = format!("{}={}", case.region, path.display());
        failure(
            &cli(&case, &[("--mem-file", &file)]),
            &["--mem-file", case.region, path.to_str().unwrap()],
        );
    }
}

#[test]
fn mem_and_mem_file_share_one_duplicate_name_check() {
    let scratch = Scratch::new();
    for case in cases() {
        let path = scratch.write("region.bin", case.bytes);
        let file = format!("{}={}", case.region, path.display());
        let inline = format!("{}={}", case.region, case.hex);
        for memory in [
            [("--mem-file", file.as_str()), ("--mem-file", file.as_str())],
            [("--mem", inline.as_str()), ("--mem-file", file.as_str())],
            [("--mem-file", file.as_str()), ("--mem", inline.as_str())],
            [("--mem", inline.as_str()), ("--mem", inline.as_str())],
        ] {
            failure(&cli(&case, &memory), &["duplicate", case.region]);
        }
    }
}

#[test]
fn mem_file_reports_its_assignment_syntax_and_missing_value() {
    for case in cases() {
        failure(
            &cli(&case, &[("--mem-file", "region.bin")]),
            &["--mem-file", "name=path"],
        );
        let output = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
            .arg(case.command)
            .arg(fixture(case.fixture))
            .arg(case.entry)
            .args(case.args)
            .arg("--mem-file")
            .output()
            .unwrap();
        failure(&output, &["missing value", "--mem-file"]);
    }
}

#[test]
fn mem_file_loads_a_65536_byte_region_without_an_oversized_argument() {
    let scratch = Scratch::new();
    // region[16,8] is the 65,536-byte backing region needed at N=2^10.
    let mut bytes = vec![0; 1 << 16];
    bytes[65535] = 0xa5;
    let path = scratch.write("large-region.bin", &bytes);
    let file = format!("db={}", path.display());
    let case = Case {
        command: "run-oracle",
        fixture: "cli_mem_file/large_region",
        entry: "read",
        args: &["--input", "65535", "--budget", "5"],
        region: "db",
        bytes: &[],
        hex: "",
        expected: r#"{"code":0,"payload":165,"steps":5}"#,
    };
    let value = success(&cli(&case, &[("--mem-file", &file)]));
    assert_eq!(value, Json::parse(case.expected).unwrap());

    #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
    {
        // Linux MAX_ARG_STRLEN includes the NUL; hex alone occupies 131,072 bytes.
        let inline = format!("db={}a5", "00".repeat(bytes.len() - 1));
        let error = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
            .arg(case.command)
            .arg(fixture(case.fixture))
            .arg(case.entry)
            .args(case.args)
            .args(["--mem", &inline])
            .output()
            .expect_err("the inline region must exceed Linux's argument limit");
        assert_eq!(error.raw_os_error(), Some(7), "expected E2BIG: {error}");
    }

    let sampled = Case {
        command: "step-check",
        args: &["--samples", "1", "--budget", "5"],
        ..case
    };
    let value = success(&cli(&sampled, &[("--mem-file", &file)]));
    assert_eq!(value.get("cases").unwrap().uint().unwrap(), 1);
    assert_eq!(
        value.get("fixed_regions").unwrap(),
        &Json::parse(r#"["db"]"#).unwrap()
    );
}
