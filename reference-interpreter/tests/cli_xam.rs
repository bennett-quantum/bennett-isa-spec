//! CLI commands of bead b-isa-spec-xam (`run-cc`, `reverse`, `run-action`,
//! `run-program`) for the dialect 0.2 pre-flight checker. Every expected
//! value below is hand-derived from the fixture text and the ISA rules cited
//! in the comments; the fixtures' own `.comments` files carry the same
//! derivations for the library-level outcomes.
use bennett_isa_ref::json::Json;
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
fn ok(args: &[&str]) -> Json {
    let output = cli(args);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    Json::parse(std::str::from_utf8(&output.stdout).unwrap()).unwrap()
}
fn fails(args: &[&str], needle: &str) {
    let output = cli(args);
    assert!(
        !output.status.success(),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(needle), "{stderr} must contain {needle}");
    assert!(output.stdout.is_empty(), "no partial JSON on a diagnostic");
}
fn field<'a>(v: &'a Json, key: &str) -> &'a Json {
    v.get(key)
        .unwrap_or_else(|_| panic!("missing field {key} in {}", v.canonical()))
}
fn text<'a>(v: &'a Json, key: &str) -> &'a str {
    field(v, key).string().unwrap()
}
fn uint(v: &Json, key: &str) -> u128 {
    field(v, key).uint().unwrap()
}
fn array<'a>(v: &'a Json, key: &str) -> &'a [Json] {
    field(v, key).array().unwrap()
}
/// A `{"type":..,"value":..}` result object renders an iN as its signed
/// decimal string (§10 INT binding; the dialect checker's INT(w):v form).
fn result(v: &Json) -> (String, String) {
    (
        field(v, "type").string().unwrap().to_string(),
        field(v, "value").string().unwrap().to_string(),
    )
}
fn tmp_file(name: &str, contents: &str) -> String {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(name);
    std::fs::write(&path, contents).unwrap();
    path.to_str().unwrap().to_string()
}

#[test]
fn run_cc_sum_reports_every_step_with_register_deltas() {
    // classical.bisa @sum(n) = n+(n-1)+...+1 (fixture comment). For n = 3 the
    // value is 6 and the executed rows are, by hand:
    //   entry 3 (const, const, br) + 3 loop passes x (icmp, br, sub, add, br)
    //   + final pass (icmp, br) + ret = 3 + 15 + 2 + 1 = 21 steps.
    let v = ok(&["run-cc", &fixture("classical"), "sum", "--args", "3"]);
    assert_eq!(text(&v, "status"), "COMPLETED");
    assert_eq!(uint(&v, "steps"), 21);
    assert_eq!(field(&v, "fault"), &Json::Null);
    assert_eq!(array(&v, "faults").len(), 0);
    assert_eq!(
        array(&v, "results").iter().map(result).collect::<Vec<_>>(),
        [("i8".to_string(), "6".to_string())]
    );
    let trace = array(&v, "trace");
    assert_eq!(trace.len(), 21);
    // Step 0 is `zero = const.i8 0` in ^entry(): defines `zero` from nothing.
    let t0 = &trace[0];
    assert_eq!(text(t0, "function"), "sum");
    assert_eq!(text(t0, "block"), "entry");
    assert_eq!(uint(t0, "index"), 0);
    assert_eq!(text(t0, "op"), "const.i8");
    let registers = array(t0, "registers");
    assert_eq!(registers.len(), 1);
    assert_eq!(text(&registers[0], "name"), "zero");
    assert_eq!(field(&registers[0], "pre"), &Json::Null);
    assert_eq!(
        result(field(&registers[0], "post")),
        ("i8".into(), "0".into())
    );
    assert_eq!(array(t0, "memory").len(), 0);
    assert_eq!(text(&trace[20], "op"), "ret");
    // --input is the same positional argument list under the run-oracle name.
    let alias = ok(&["run-cc", &fixture("classical"), "sum", "--input", "3"]);
    assert_eq!(alias, v);
}

#[test]
fn run_cc_effects_entropy_tape_console_and_memory_deltas() {
    // classical.bisa @effects with entropy [11, 22, 33]: entropy.read copies
    // the three bytes to the fresh 1024-byte allocation at heap_base 16,
    // console.write emits them, clock.read writes c = 0 (then c = 1),
    // block.write/read move 512 zero bytes, and the i64 load is 0 (fixture
    // comment; §7.3 device rows).
    let v = ok(&[
        "run-cc",
        &fixture("classical"),
        "effects",
        "--args",
        "",
        "--entropy",
        "0b1621",
    ]);
    assert_eq!(text(&v, "status"), "COMPLETED");
    assert_eq!(
        array(&v, "results").iter().map(result).collect::<Vec<_>>(),
        [("i64".to_string(), "0".to_string())]
    );
    assert_eq!(text(&v, "console_hex"), "0b1621");
    assert_eq!(uint(&v, "clock"), 1);
    assert_eq!(uint(&v, "effect_tape_length"), 5);
    // Tape entries are (dev, op, args) of §7.3 plus the transferred bytes:
    // entropy bytes land verbatim; the clock writes eight little-endian
    // bytes of 0.
    let tape = array(&v, "tape");
    assert_eq!(tape.len(), 5);
    assert_eq!(text(&tape[0], "dev"), "entropy");
    assert_eq!(text(&tape[0], "op"), "read");
    assert_eq!(text(&tape[0], "bytes"), "0b1621");
    assert_eq!(
        array(&tape[0], "args")
            .iter()
            .map(|a| a.uint().unwrap())
            .collect::<Vec<_>>(),
        [16, 3]
    );
    assert_eq!(text(&tape[1], "dev"), "console");
    assert_eq!(text(&tape[1], "bytes"), "0b1621");
    assert_eq!(text(&tape[2], "dev"), "clock");
    assert_eq!(text(&tape[2], "bytes"), "0000000000000000");
    assert_eq!(text(&tape[3], "dev"), "block");
    assert_eq!(text(&tape[3], "bytes").len(), 1024);
    // The entropy.read is step 3 (const, alloc, const, effect): three memory
    // bytes change 0 -> 11, 22, 33 at the allocation base 16.
    let t3 = &array(&v, "trace")[3];
    assert_eq!(text(t3, "op"), "effect");
    let memory = array(t3, "memory");
    assert_eq!(memory.len(), 3);
    for (i, want) in [11u128, 22, 33].iter().enumerate() {
        assert_eq!(uint(&memory[i], "address"), 16 + i as u128);
        assert_eq!(uint(&memory[i], "pre"), 0);
        assert_eq!(uint(&memory[i], "post"), *want);
    }
}

#[test]
fn run_cc_hostcall_consumes_a_checked_replies_file() {
    // classical.bisa @host stores a5 in a 3-byte allocation at 16 and issues
    // hostcall org.bennett.test p n; the request payload is a5 00 00. The
    // reply 11 22 33 44 55 writes its first three bytes and rlen is the full
    // reply length 5 (§10 hostcall; fixture comment).
    let replies = tmp_file(
        "xam-host-replies.json",
        r#"[{"payload":"a50000","reply":"1122334455","service":"org.bennett.test"}]"#,
    );
    let v = ok(&[
        "run-cc",
        &fixture("classical"),
        "host",
        "--args",
        "",
        "--replies",
        &replies,
    ]);
    assert_eq!(
        array(&v, "results").iter().map(result).collect::<Vec<_>>(),
        [("i16".to_string(), "5".to_string())]
    );
    let requests = array(&v, "host_requests");
    assert_eq!(requests.len(), 1);
    assert_eq!(text(&requests[0], "service"), "org.bennett.test");
    assert_eq!(text(&requests[0], "payload"), "a50000");
    assert_eq!(text(&requests[0], "reply"), "1122334455");
    assert_eq!(uint(&requests[0], "address"), 16);
    assert_eq!(uint(&requests[0], "capacity"), 3);
    // A reply that names the wrong payload is a §10 mismatch, not a result;
    // an unanswered HOST_REQUEST is likewise a diagnostic.
    let wrong = tmp_file(
        "xam-host-wrong.json",
        r#"[{"payload":"00","reply":"","service":"org.bennett.test"}]"#,
    );
    fails(
        &[
            "run-cc",
            &fixture("classical"),
            "host",
            "--args",
            "",
            "--replies",
            &wrong,
        ],
        "§10",
    );
    fails(
        &["run-cc", &fixture("classical"), "host", "--args", ""],
        "HOST_REQUEST",
    );
}

#[test]
fn run_cc_fault_is_a_failed_result_not_a_diagnostic() {
    // classical.bisa @divzero computes 3/0: FAULT_UB_DIV0 = 5 (§5.4), no
    // values (§10: a FAILED run exposes no values).
    let v = ok(&["run-cc", &fixture("classical"), "divzero", "--args", ""]);
    assert_eq!(text(&v, "status"), "FAILED");
    assert_eq!(uint(&v, "fault"), 5);
    assert_eq!(array(&v, "results").len(), 0);
    let faults = array(&v, "faults");
    assert_eq!(faults.len(), 1);
    assert_eq!(uint(&faults[0], "code"), 5);
    assert_eq!(text(&faults[0], "name"), "FAULT_UB_DIV0");
    // Steps: const 3, const 0, udiv (the faulting row executes and halts).
    assert_eq!(uint(&v, "steps"), 3);
}

#[test]
fn run_cc_rejects_bad_entries_and_arguments_loudly() {
    // 256 does not fit the i8 parameter (§10 positional INT binding: E_ARG).
    fails(
        &["run-cc", &fixture("classical"), "sum", "--args", "256"],
        "E_ARG",
    );
    // @sum takes one argument, not zero.
    fails(
        &["run-cc", &fixture("classical"), "sum", "--args", ""],
        "E_ARG",
    );
    // An action is not a CC/R entry (§3.4).
    fails(
        &["run-cc", &fixture("actions"), "bell", "--args", ""],
        "§3.4",
    );
    fails(
        &["run-cc", &fixture("classical"), "missing", "--args", ""],
        "§3.4",
    );
    // Both spellings of the argument list at once is ambiguous.
    fails(
        &[
            "run-cc",
            &fixture("classical"),
            "sum",
            "--args",
            "3",
            "--input",
            "3",
        ],
        "--args",
    );
}
