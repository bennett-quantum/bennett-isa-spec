//! CLI commands of bead b-isa-spec-xam (`run-cc`, `reverse`, `run-action`,
//! `run-program`) for the dialect 0.2 pre-flight checker. Every expected
//! value below is hand-derived from the fixture text and the ISA rules cited
//! in the comments; the fixtures' own `.comments` files carry the same
//! derivations for the library-level outcomes.
mod support;
use bennett_isa_ref::{json::Json, Object};
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
    assert_eq!(
        output.status.code(),
        Some(1),
        "stdout: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(needle), "{stderr} must contain {needle}");
    assert!(
        stderr.contains('§'),
        "diagnostic must cite the ISA: {stderr}"
    );
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
/// A `{"type":..,"value":..}` result object renders an iN as the unsigned
/// decimal string of its bit-vector value (the dialect checker's
/// INT(w):v form; bd note on b-isa-spec-xam records the reading).
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
    assert_eq!(array(&v, "budgets").len(), 0);
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

#[test]
fn reverse_cc_restores_the_entry_state_after_effects() {
    // Whole-run reversal (§7.4 LIFO history traversal, §10 bennett_reverse,
    // T4): the effects run of 11 instructions (const, alloc, const, the
    // sector const and 5 effects, load, ret) is undone frame by frame, so
    // the console is empty again, the clock returns to 0, the formerly
    // absent block sector is absent again, and the whole snapshot equals
    // the entry snapshot.
    let v = ok(&[
        "reverse",
        &fixture("classical"),
        "effects",
        "--args",
        "",
        "--entropy",
        "0b1621",
    ]);
    assert_eq!(text(&v, "status"), "REVERSED");
    assert_eq!(uint(&v, "steps"), 11);
    assert_eq!(
        array(&v, "results").iter().map(result).collect::<Vec<_>>(),
        [("i64".to_string(), "0".to_string())]
    );
    assert_eq!(field(&v, "equals_entry"), &Json::Bool(true));
    assert_eq!(field(&v, "memory_equals_entry"), &Json::Bool(true));
    assert_eq!(field(&v, "first_difference"), &Json::Null);
    assert_eq!(text(&v, "console"), "");
    assert_eq!(text(&v, "console_hex"), "");
    assert_eq!(uint(&v, "clock"), 0);
    assert_eq!(uint(&v, "effect_tape_length"), 0);
    assert_eq!(text(&v, "memory_digest"), text(&v, "entry_memory_digest"));
    // The forward run's memory digest differs from the entry digest, so the
    // equality above is a restoration, not an untouched run.
    let forward = ok(&[
        "run-cc",
        &fixture("classical"),
        "effects",
        "--args",
        "",
        "--entropy",
        "0b1621",
    ]);
    assert_ne!(text(&forward, "memory_digest"), text(&v, "memory_digest"));
}

#[test]
fn reverse_cc_prefix_stops_and_reverses_mid_run() {
    // --steps 5 runs the first five instructions of @sum(3) (const, const,
    // br, icmp, br -- still inside the loop), then reverses that prefix.
    let v = ok(&[
        "reverse",
        &fixture("classical"),
        "sum",
        "--args",
        "3",
        "--steps",
        "5",
    ]);
    assert_eq!(text(&v, "status"), "REVERSED");
    assert_eq!(uint(&v, "steps"), 5);
    assert_eq!(array(&v, "results").len(), 0);
    assert_eq!(field(&v, "fault"), &Json::Null);
    assert_eq!(field(&v, "equals_entry"), &Json::Bool(true));
}

#[test]
fn reverse_cc_measurement_is_not_reversible() {
    // registers.bisa @measurement measures a Bell pair; §10 reverse
    // eligibility excludes any run containing measure (E_NOT_REVERSIBLE).
    fails(
        &[
            "reverse",
            &fixture("registers"),
            "measurement",
            "--args",
            "",
            "--draws",
            "0.1",
        ],
        "E_NOT_REVERSIBLE",
    );
}

#[test]
fn reverse_oracle_prefix_reversal_restores_the_basis_tuple() {
    // §14.1/§3.6 prefix reversal of inc8 at B = 3: three forward steps give
    // F_3(8) = 9, code 0; three inverse steps restore the complete entry
    // tuple (pc 0, all §3.7 counters 0, x = 8, regs one = r = 0) with every
    // frame clean (T2).
    let v = ok(&[
        "reverse",
        &fixture("inc8"),
        "inc8",
        "--input",
        "8",
        "--budget",
        "3",
    ]);
    assert_eq!(text(&v, "status"), "REVERSED");
    assert_eq!(uint(&v, "payload"), 9);
    assert_eq!(uint(&v, "code"), 0);
    assert_eq!(uint(&v, "steps"), 3);
    assert_eq!(uint(&v, "reverse_steps"), 3);
    assert_eq!(field(&v, "equals_entry"), &Json::Bool(true));
    assert_eq!(field(&v, "frames_clean"), &Json::Bool(true));
    assert_eq!(field(&v, "first_difference"), &Json::Null);
    let state = field(&v, "state_after_reversal");
    assert_eq!(uint(state, "pc"), 0);
    assert_eq!(field(state, "faulted"), &Json::Bool(false));
    assert_eq!(
        array(state, "counters")
            .iter()
            .map(|c| c.uint().unwrap())
            .collect::<Vec<_>>(),
        [0, 0, 0, 0, 0]
    );
    let locations = field(state, "locations");
    assert_eq!(locations.get("inc8.x").unwrap().uint().unwrap(), 8);
    assert_eq!(locations.get("inc8.one").unwrap().uint().unwrap(), 0);
    assert_eq!(locations.get("inc8.r").unwrap().uint().unwrap(), 0);
    // At prefix 2 of the same budget the commit is not at HALT, so the code
    // is FAULT_BUDGET_STEPS (1) with payload 0, as run-oracle reports for
    // F_2; the two inverse steps still restore the entry tuple.
    let prefix = ok(&[
        "reverse",
        &fixture("inc8"),
        "inc8",
        "--input",
        "8",
        "--budget",
        "3",
        "--steps",
        "2",
    ]);
    assert_eq!(uint(&prefix, "payload"), 0);
    assert_eq!(uint(&prefix, "code"), 1);
    assert_eq!(uint(&prefix, "steps"), 2);
    assert_eq!(uint(&prefix, "reverse_steps"), 2);
    assert_eq!(field(&prefix, "equals_entry"), &Json::Bool(true));
}

#[test]
fn reverse_action_adjoint_returns_to_the_entry_state() {
    // actions.bisa @bell applies h t[0]; cx t[0] t[1] to the mut bundle t
    // (in bundle p untouched). From |0000> over the interface order
    // (t0, t1, p0, p1) the forward state is (|0000> + |1100>)/sqrt(2); the
    // adjoint then restores |0000> exactly (§10: top-level Q calls reverse
    // by adjoint).
    let v = ok(&[
        "reverse",
        &fixture("actions"),
        "bell",
        "--mut",
        "0",
        "--in",
        "0",
    ]);
    assert_eq!(text(&v, "status"), "REVERSED");
    assert_eq!(field(&v, "equals_entry"), &Json::Bool(true));
    assert_eq!(field(&v, "first_difference"), &Json::Null);
    let forward = field(&v, "forward_state").object().unwrap();
    assert_eq!(forward.len(), 2);
    for key in ["0000", "1100"] {
        let amp = forward[key].array().unwrap();
        let re: f64 = amp[0].canonical().parse().unwrap();
        assert!(
            (re - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12,
            "{key}: {re}"
        );
    }
    let state = field(&v, "state").object().unwrap();
    assert_eq!(state.len(), 1);
    let amp = state["0000"].array().unwrap();
    assert_eq!(amp[0].canonical(), "1");
    assert_eq!(amp[1].canonical(), "0");
}

#[test]
fn reverse_rejects_unknown_symbols_and_bad_prefixes() {
    fails(
        &["reverse", &fixture("classical"), "missing", "--args", ""],
        "§",
    );
    // A prefix longer than the oracle budget cannot run (B frames, §3.6).
    fails(
        &[
            "reverse",
            &fixture("inc8"),
            "inc8",
            "--input",
            "8",
            "--budget",
            "2",
            "--steps",
            "3",
        ],
        "§5.4",
    );
    // run-cc does not take --steps.
    fails(
        &[
            "run-cc",
            &fixture("classical"),
            "sum",
            "--args",
            "3",
            "--steps",
            "2",
        ],
        "--steps is a reverse option",
    );
}

#[test]
fn run_action_bell_from_basis_values() {
    // registers.bisa @bell (mut wire[2] only): h t[0]; cx t[0] t[1] maps
    // |00> to (|00> + |11>)/sqrt(2) in interface order (t0, t1). This is the
    // exact shape the dialect checker's state= examples consume: one JSON
    // `state` object keyed by bit string with [re, im] amplitudes.
    let v = ok(&["run-action", &fixture("registers"), "bell", "--mut", "0"]);
    assert_eq!(text(&v, "action"), "bell");
    assert_eq!(uint(&v, "wires"), 2);
    let state = field(&v, "state").object().unwrap();
    assert_eq!(state.len(), 2);
    for key in ["00", "11"] {
        let amp = state[key].array().unwrap();
        let re: f64 = amp[0].canonical().parse().unwrap();
        let im: f64 = amp[1].canonical().parse().unwrap();
        assert!(
            (re - std::f64::consts::FRAC_1_SQRT_2).abs() < 1e-12,
            "{key}: {re}"
        );
        assert_eq!(im, 0.0);
    }
    // The mut bundle has no definite basis value, reported as null (never a
    // guessed outcome).
    assert_eq!(array(&v, "mut"), [Json::Null]);
    assert_eq!(array(&v, "in").len(), 0);
    // registers.bisa @toggle flips its single wire: |0> -> |1> exactly.
    let v = ok(&["run-action", &fixture("registers"), "toggle", "--mut", "0"]);
    let state = field(&v, "state").object().unwrap();
    assert_eq!(state.len(), 1);
    assert_eq!(state["1"].canonical(), "[1,0]");
    assert_eq!(array(&v, "mut")[0].uint().unwrap(), 1);
}

#[test]
fn run_action_permutation_basis_and_adjoint() {
    // actions.bisa @flip computes t ^= p (fixture comment). With t = 1 and
    // p = 2 the result is t = 3, p = 2; in interface order (t0,t1,p0,p1)
    // that is |1101>. The same run spelled --basis starts from
    // (t0,t1,p0,p1) = 1001. flip is its own adjoint, so the adjoint run
    // from t = 3, p = 2 returns t = 1.
    let v = ok(&[
        "run-action",
        &fixture("actions"),
        "flip",
        "--mut",
        "1",
        "--in",
        "2",
    ]);
    assert_eq!(field(&v, "state").canonical(), "{\"1101\":[1,0]}");
    assert_eq!(array(&v, "mut")[0].uint().unwrap(), 3);
    assert_eq!(array(&v, "in")[0].uint().unwrap(), 2);
    let basis = ok(&["run-action", &fixture("actions"), "flip", "--basis", "1001"]);
    assert_eq!(field(&basis, "state"), field(&v, "state"));
    assert_eq!(array(&basis, "mut")[0].uint().unwrap(), 3);
    let adjoint = ok(&[
        "run-action",
        &fixture("actions"),
        "flip",
        "--mut",
        "3",
        "--in",
        "2",
        "--adjoint",
    ]);
    assert_eq!(field(&adjoint, "adjoint"), &Json::Bool(true));
    assert_eq!(array(&adjoint, "mut")[0].uint().unwrap(), 1);
    assert_eq!(field(&adjoint, "state").canonical(), "{\"1001\":[1,0]}");
}

#[test]
fn run_action_state_vector_input() {
    // The adjoint of Bell preparation maps (|00> + |11>)/sqrt(2) back to
    // |00>: cx gives (|00> + |10>)/sqrt(2), then h on t0 interferes to
    // |00>. The specified amplitude is a binary64 whose exact dyadic value
    // cancels exactly on the other component, so the support is a single
    // component of amplitude sqrt(2) * 0.7071067811865475 ~ 1.
    let v = ok(&[
        "run-action",
        &fixture("registers"),
        "bell",
        "--state",
        "00=0.7071067811865475,0;11=0.7071067811865475,0",
        "--adjoint",
    ]);
    let state = field(&v, "state").object().unwrap();
    assert_eq!(state.len(), 1);
    let amp = state["00"].array().unwrap();
    let re: f64 = amp[0].canonical().parse().unwrap();
    assert!((re - 1.0).abs() < 1e-12, "{re}");
    assert_eq!(array(&v, "mut")[0].uint().unwrap(), 0);
}

#[test]
fn run_action_mem_region_and_borrow_pool() {
    // control_flow.bisa @exchange swaps t with m[a] (fixture comment):
    // t = 2, a = 1, m = [1, 3] (one byte per 2-bit cell) gives t = 3 and
    // m = [1, 2]. All seven interface wires (t0,t1,a,m0.0,m0.1,m1.0,m1.1)
    // are then 1111001.
    let v = ok(&[
        "run-action",
        &fixture("control_flow"),
        "exchange",
        "--mut",
        "2",
        "--in",
        "1",
        "--mem",
        "m=0103",
    ]);
    assert_eq!(array(&v, "mut")[0].uint().unwrap(), 3);
    assert_eq!(
        field(field(&v, "mem"), "m")
            .array()
            .unwrap()
            .iter()
            .map(|c| c.uint().unwrap())
            .collect::<Vec<_>>(),
        [1, 2]
    );
    assert_eq!(field(&v, "state").canonical(), "{\"1111001\":[1,0]}");
    // Without --mem the region cells start zero: t swaps with m[1] = 0.
    let v = ok(&[
        "run-action",
        &fixture("control_flow"),
        "exchange",
        "--mut",
        "2",
        "--in",
        "1",
    ]);
    assert_eq!(array(&v, "mut")[0].uint().unwrap(), 0);
    assert_eq!(
        field(field(&v, "mem"), "m")
            .array()
            .unwrap()
            .iter()
            .map(|c| c.uint().unwrap())
            .collect::<Vec<_>>(),
        [0, 2]
    );
    // control_flow.bisa @borrowed borrows one pool wire and computes
    // t ^= x independently of it: --mut 0 --in 1 gives t = 1.
    let v = ok(&[
        "run-action",
        &fixture("control_flow"),
        "borrowed",
        "--mut",
        "0",
        "--in",
        "1",
    ]);
    assert_eq!(array(&v, "mut")[0].uint().unwrap(), 1);
    assert_eq!(field(&v, "state").canonical(), "{\"11\":[1,0]}");
}

#[test]
fn run_action_rejects_ambiguous_and_malformed_inputs() {
    let bell = fixture("registers");
    // Exactly one state mode.
    fails(
        &["run-action", &bell, "bell", "--mut", "0", "--basis", "00"],
        "exactly one of --mut, --basis, --state",
    );
    fails(
        &["run-action", &bell, "bell"],
        "exactly one of --mut, --basis, --state",
    );
    // --in only accompanies --mut.
    fails(
        &["run-action", &bell, "bell", "--basis", "00", "--in", "0"],
        "--in accompanies --mut only",
    );
    // Wrong bit-string length (2 interface wires).
    fails(&["run-action", &bell, "bell", "--basis", "000"], "§14.2");
    // A zero-amplitude state component is not a component.
    fails(&["run-action", &bell, "bell", "--state", "00=0,0"], "§14.2");
    // An oracle is not an action, and unknown names fail loudly.
    fails(
        &["run-action", &fixture("inc8"), "inc8", "--mut", "0"],
        "§5.3",
    );
    fails(&["run-action", &bell, "missing", "--mut", "0"], "§5.3");
}

#[test]
fn run_program_region_calls_complete_and_close() {
    // registers.bisa @views: qreg.new, slice, action_call @toggle, slice,
    // action_call @toggle, qreg.free, ret = 7 instructions with no result
    // values (fixture comment). The one-shot §10 launch walks
    // CREATED -> RUNNING -> COMPLETED and closes the run record.
    let v = ok(&["run-program", &fixture("registers"), "views", "--args", ""]);
    assert_eq!(text(&v, "status"), "COMPLETED");
    assert_eq!(text(&v, "run_state"), "CLOSED");
    assert_eq!(field(&v, "reversed"), &Json::Bool(false));
    assert_eq!(uint(&v, "steps"), 7);
    assert_eq!(array(&v, "results").len(), 0);
    assert_eq!(array(&v, "faults").len(), 0);
    assert_eq!(field(&v, "fault"), &Json::Null);
    assert_eq!(array(&v, "measurements").len(), 0);
    assert_eq!(array(&v, "budgets").len(), 0);
    // With --reverse the completed run is reversed (§10) before closing.
    let v = ok(&[
        "run-program",
        &fixture("registers"),
        "views",
        "--args",
        "",
        "--reverse",
    ]);
    assert_eq!(text(&v, "status"), "REVERSED");
    assert_eq!(field(&v, "reversed"), &Json::Bool(true));
    assert_eq!(field(&v, "memory_equals_entry"), &Json::Bool(true));
    assert_eq!(field(&v, "equals_entry"), &Json::Bool(true));
}

#[test]
fn run_program_measurements_use_fixed_draws() {
    // registers.bisa @measurement: qreg.new, action_call @bell, measure,
    // ret = 4 instructions. Draw 0.9 selects the |11> branch above 1/2, so
    // the measured value and the i2 result are 3 (fixture comment). The
    // first allocated register has id 1.
    let v = ok(&[
        "run-program",
        &fixture("registers"),
        "measurement",
        "--args",
        "",
        "--draws",
        "0.9",
    ]);
    assert_eq!(text(&v, "status"), "COMPLETED");
    assert_eq!(uint(&v, "steps"), 4);
    assert_eq!(
        array(&v, "results").iter().map(result).collect::<Vec<_>>(),
        [("i2".to_string(), "3".to_string())]
    );
    let measurements = array(&v, "measurements");
    assert_eq!(measurements.len(), 1);
    assert_eq!(uint(&measurements[0], "reg"), 1);
    assert_eq!(text(&measurements[0], "bits"), "11");
    assert_eq!(uint(&measurements[0], "value"), 3);
    // A run that measured is not reverse-eligible (§10).
    fails(
        &[
            "run-program",
            &fixture("registers"),
            "measurement",
            "--args",
            "",
            "--draws",
            "0.1",
            "--reverse",
        ],
        "E_NOT_REVERSIBLE",
    );
}

#[test]
fn run_program_failed_launch_exposes_no_values() {
    // registers.bisa @unclean: qreg.new, action_call @toggle, qreg.free
    // traps with FAULT_UB_TRAP (4) because the flipped register is not
    // clean (fixture comment). §10: a FAILED run exposes no values.
    let v = ok(&[
        "run-program",
        &fixture("registers"),
        "unclean",
        "--args",
        "",
    ]);
    assert_eq!(text(&v, "status"), "FAILED");
    assert_eq!(uint(&v, "steps"), 3);
    assert_eq!(uint(&v, "fault"), 4);
    assert_eq!(array(&v, "results").len(), 0);
    // bennett_reverse applies only to COMPLETED runs (§10).
    fails(
        &[
            "run-program",
            &fixture("registers"),
            "unclean",
            "--args",
            "",
            "--reverse",
        ],
        "E_INVALID_STATE",
    );
}

#[test]
fn run_program_oracle_call_with_qreg_and_token_arguments() {
    // A CC entry that calls the inc8 oracle as a region (§10 invocation
    // tree), linked from the inc8 fixture object the way tests/registers.rs
    // builds it. At token B = 3 the oracle delivers
    // t ^= (x + 1 mod 256): with x = 7 and entry t = 31 that is 31 ^ 8 =
    // 23; measuring t (a basis state, any draw) returns 23. Registers are
    // allocated in argument order, so t is register 2 and its bits in
    // wire-0-first order are 11101000.
    let inc = Object::parse(include_str!("fixtures/inc8.bisa")).unwrap();
    let oracle_sig = inc.symbols().unwrap()["0"]
        .get("signature")
        .unwrap()
        .canonical();
    let sig = r#"{"abi":"0.8","kind":"cc","params":["qreg[8]","qreg[8]","qreg[4]","token"],"results":["i8"],"tier":"cc"}"#;
    let body = "fn cc @invoke (x:qreg[8], t:qreg[8], f:qreg[4], tk:token) (r:i8) () () ()\n^entry():\noracle_call @inc8 tk (x) t f () ()\nr = measure t\nret r";
    let caller = support::rehash(&support::object(
        &[("invoke", sig, body)],
        &format!(r#"{{"inc8":{{"signature":{oracle_sig}}}}}"#),
        "",
    ));
    let linked = Object::link(&[caller, inc]).unwrap();
    let path = tmp_file("xam-invoke.bisa", &linked.text);
    let v = ok(&[
        "run-program",
        &path,
        "invoke",
        "--arg",
        "qreg:11100000",
        "--arg",
        "qreg:11111000",
        "--arg",
        "qreg:1100",
        "--arg",
        "token:3",
        "--draws",
        "0.5",
    ]);
    assert_eq!(text(&v, "status"), "COMPLETED");
    assert_eq!(uint(&v, "steps"), 3);
    assert_eq!(
        array(&v, "results").iter().map(result).collect::<Vec<_>>(),
        [("i8".to_string(), "23".to_string())]
    );
    let measurements = array(&v, "measurements");
    assert_eq!(measurements.len(), 1);
    assert_eq!(uint(&measurements[0], "reg"), 2);
    assert_eq!(text(&measurements[0], "bits"), "11101000");
    assert_eq!(uint(&measurements[0], "value"), 23);
    let budgets = array(&v, "budgets");
    assert_eq!(budgets.len(), 1);
    assert_eq!(uint(&budgets[0], "steps"), 3);
    assert_eq!(uint(&budgets[0], "heap"), 0);
}

#[test]
fn run_program_host_request_lifecycle() {
    // classical.bisa @host under the launch ABI: const, alloc, const,
    // store, hostcall, ret = 6 instructions and rlen 5 for the 5-byte
    // reply (§10; same hand derivation as the run-cc hostcall test).
    let replies = tmp_file(
        "xam-program-replies.json",
        r#"[{"payload":"a50000","reply":"1122334455","service":"org.bennett.test"}]"#,
    );
    let v = ok(&[
        "run-program",
        &fixture("classical"),
        "host",
        "--args",
        "",
        "--replies",
        &replies,
    ]);
    assert_eq!(text(&v, "status"), "COMPLETED");
    assert_eq!(uint(&v, "steps"), 6);
    assert_eq!(
        array(&v, "results").iter().map(result).collect::<Vec<_>>(),
        [("i16".to_string(), "5".to_string())]
    );
}

#[test]
fn run_program_rejects_non_cc_entries_and_staged_inputs() {
    // §10: a run executes a device CC entry; an R or oracle symbol is E_ARG.
    fails(
        &["run-program", &fixture("classical"), "sum", "--args", "3"],
        "§10",
    );
    fails(
        &["run-program", &fixture("inc8"), "inc8", "--args", ""],
        "§10",
    );
    // bennett_load's target_jcs needs §8 target descriptions: staged
    // deliverable 9, an explicit diagnostic, never a silent run.
    let target = tmp_file("xam-target.json", "{}");
    let output = cli(&[
        "run-program",
        &fixture("registers"),
        "views",
        "--args",
        "",
        "--target",
        &target,
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty(), "no partial JSON on a diagnostic");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("staged deliverable 9"), "{stderr}");
    assert!(stderr.contains("§8"), "{stderr}");
    // Even a missing target names the staged path rather than implying
    // that supplying an existing file could execute a §8 target.
    fails(
        &[
            "run-program",
            &fixture("registers"),
            "views",
            "--target",
            &format!("{target}/missing"),
        ],
        "staged deliverable 9",
    );
    // A schedule section needs verbatim scheduled execution: staged
    // deliverable 10 (§11 schedule consumer). The schedule JSON is the
    // valid inc8 schedule of tests/r1_certificates.rs.
    let schedule = Json::parse(&format!(
        r#"{{"bindings":{{"budget":[3,0,0,0],"closure_hash":"8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93","epsilon":0,"expansion":{{"hash":"{}","name":"ref","version":"0.8"}},"inherited_controls":0,"params_hash":"{}","region":"inc8","scheduler":{{"name":"test","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[{{"generated":"*","kind":"start","path":[{{"iteration":[],"point":"entry:0","symbol":"8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93"}}],"phase":"fwd","start":{{"time":0}},"step":1}}]}}"#,
        "1".repeat(64),
        "2".repeat(64),
        "3".repeat(64)
    ))
    .unwrap()
    .canonical();
    let scheduled = tmp_file(
        "xam-scheduled.bisa",
        &format!(
            "{}\nsection schedule\n{schedule}\n",
            include_str!("fixtures/inc8.bisa")
        ),
    );
    let output = cli(&["run-program", &scheduled, "inc8", "--args", ""]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty(), "no partial JSON on a diagnostic");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("staged deliverable 10"), "{stderr}");
    assert!(stderr.contains("§11"), "{stderr}");
}

#[test]
fn run_action_mem_cells_wider_than_128_bits() {
    // A mem region of 2 cells of wire[300]: cell hex is 38 little-endian
    // bytes per cell. Cell 0 is 2^299 + 1 (byte 0 = 0x01, byte 37 = 0x08)
    // and cell 1 is zero; qexchm with a = 0 swaps it with t = 5, so t ends
    // at 2^299 + 1 and the cells are [5, 0]. Widths above 128 must not be
    // narrowed through u128 anywhere in the --mem path.
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[{"A":1,"W":300,"role":"mut"}],"mut":[300],"tier":"q"}"#;
    let body = "perm_action @wex (t:wire[300] mut) (a:wire in) (m:region[1,300] mut) () ()\n^entry():\nqexchm m a t";
    let layout = "\nsection layout\n{\"A_tab\":0,\"H\":4,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";
    let text = support::rehash(&support::object(&[("wex", sig, body)], "{}", layout)).text;
    let path = tmp_file("xam-wide.bisa", &text);
    let mut cell0 = [0u8; 38];
    cell0[0] = 1;
    cell0[37] = 8;
    let mem: String = cell0
        .iter()
        .chain(&[0u8; 38])
        .map(|b| format!("{b:02x}"))
        .collect();
    let v = ok(&[
        "run-action",
        &path,
        "wex",
        "--mut",
        "5",
        "--in",
        "0",
        "--mem",
        &format!("m={mem}"),
    ]);
    // Decimal expansion of 2^299 + 1, independent of the interpreter.
    assert_eq!(array(&v, "mut")[0].string().unwrap(), "1018517988167243043134222844204689080525734196832968125318070224677190649881668353091698689");
    assert_eq!(
        field(field(&v, "mem"), "m")
            .array()
            .unwrap()
            .iter()
            .map(|c| c.uint().unwrap())
            .collect::<Vec<_>>(),
        [5, 0]
    );
}

#[test]
fn classical_commands_report_each_bound_budget_in_order() {
    // §10 positional binding: tk0 and tk1 are separated by an i8 argument.
    // The sole row returns x = 255; tokens consume no oracle steps because
    // this entry calls no oracle. §5.4 memoryless tokens have h=s=d=0.
    let sig =
        r#"{"abi":"0.8","kind":"cc","params":["token","i8","token"],"results":["i8"],"tier":"cc"}"#;
    let body = "fn cc @tokens (tk0:token, x:i8, tk1:token) (r:i8) () () ()\n^entry():\nret x";
    let object = support::rehash(&support::object(&[("tokens", sig, body)], "{}", ""));
    let path = tmp_file("xam-tokens.bisa", &object.text);
    let expected = Json::parse(
        r#"[{"depth":0,"heap":0,"stack":0,"steps":3},{"depth":0,"heap":0,"stack":0,"steps":7}]"#,
    )
    .unwrap();
    for command in ["run-cc", "reverse", "run-program"] {
        let v = ok(&[
            command, &path, "tokens", "--arg", "token:3", "--arg", "-1", "--arg", "token:7",
        ]);
        assert_eq!(field(&v, "budgets"), &expected);
        assert_eq!(
            field(&v, "results").canonical(),
            r#"[{"type":"i8","value":"255"}]"#
        );
        assert_eq!(uint(&v, "steps"), 1);
        if command == "reverse" {
            assert_eq!(field(&v, "equals_entry"), &Json::Bool(true));
        }
    }
}

#[test]
fn measurement_json_preserves_the_high_bit_of_i128() {
    // §3.3/§14.2: measurement of a definite 128-wire register with only
    // wire 127 set returns 2^127 exactly. §9.1 encodes it as a decimal
    // string, both as an i128 result and as the measurement's value.
    let sig = r#"{"abi":"0.8","kind":"cc","params":["qreg[128]"],"results":["i128"],"tier":"cc"}"#;
    let body = "fn cc @wide (q:qreg[128]) (r:i128) () () ()\n^entry():\nr = measure q\nret r";
    let object = support::rehash(&support::object(&[("wide", sig, body)], "{}", ""));
    let path = tmp_file("xam-measure128.bisa", &object.text);
    let bits = format!("{}1", "0".repeat(127));
    for command in ["run-cc", "run-program"] {
        let v = ok(&[command, &path, "wide", "--arg", &format!("qreg:{bits}")]);
        assert_eq!(
            field(&v, "results").canonical(),
            r#"[{"type":"i128","value":"170141183460469231731687303715884105728"}]"#
        );
        let measurements = array(&v, "measurements");
        assert_eq!(measurements.len(), 1);
        assert_eq!(text(&measurements[0], "bits"), bits);
        assert_eq!(
            text(&measurements[0], "value"),
            "170141183460469231731687303715884105728"
        );
    }
}

#[test]
fn malformed_cli_inputs_are_cited_diagnostics_without_json() {
    // A malformed CLI input is an error, never a partial execution result.
    for command in ["run-cc", "reverse", "run-program"] {
        fails(
            &[command, &fixture("classical"), "sum", "--entropy", "z"],
            "HEX",
        );
    }
    fails(
        &["run-action", &fixture("registers"), "bell", "--state"],
        "missing value",
    );
    let replies = tmp_file("xam-malformed-replies.json", "{}");
    fails(
        &[
            "run-program",
            &fixture("classical"),
            "host",
            "--replies",
            &replies,
        ],
        "expected JSON array",
    );
}

#[test]
fn r2_finite_rotations_render_when_the_combined_angle_exceeds_binary64() {
    for literal in [true, false] {
        let sig = if literal {
            r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#
        } else {
            r#"{"abi":"0.8","args":["f64"],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#
        };
        let angle = if literal { "1e+308" } else { "theta" };
        let params = if literal { "" } else { "theta:f64" };
        let body = format!("action @a (t:wire mut) () () ({params}) ()\n^entry():\nrz {angle} t\nrz {angle} t\nrz {angle} t\nrz {angle} t");
        let o = support::rehash(&support::object(&[("a", sig, &body)], "{}", ""));
        let path = tmp_file(
            if literal {
                "r2-large-literal.bisa"
            } else {
                "r2-large-arg.bisa"
            },
            &o.text,
        );
        for t in ["0", "1"] {
            let mut args = vec!["run-action", &path, "a", "--mut", t];
            if !literal {
                args.extend(["--arg", "f64:1e308"]);
            }
            let v = ok(&args);
            let state = field(&v, "state").object().unwrap();
            assert_eq!(state.len(), 1);
            let amp = state[t].array().unwrap();
            let re: f64 = amp[0].canonical().parse().unwrap();
            let im: f64 = amp[1].canonical().parse().unwrap();
            // theta is the EXACT binary64 1e308. Four single-gate phases
            // multiply to exp(-i*2*theta) on |0>, its conjugate on |1>.
            assert!((re - 0.588863244801576).abs() < 1e-12, "{re}");
            let sign = if t == "0" { 1.0 } else { -1.0 };
            assert!((im - sign * 0.8082326886001081).abs() < 1e-12, "{im}");
            assert!((re * re + im * im - 1.0).abs() < 1e-12);
            args[0] = "reverse";
            let v = ok(&args);
            assert_eq!(field(&v, "equals_entry"), &Json::Bool(true));
            assert_eq!(field(&v, "state").canonical(), format!("{{\"{t}\":[1,0]}}"));
        }
    }
}

#[test]
fn r2_qinplace_identity_workspace_cleanup_runs_and_reverses() {
    let os =
        r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let ac = r#"{"abi":"0.8","args":["token"],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let text = support::object(&[("g", os, "oracle @g (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x"), ("a", ac, "action @a (t:wire mut) () () (tk:token) ()\n^entry():\nd = qalloc any 1\nqinplace @g @g tk d\nqinplace @g @g tk d\nqfree d")], "{}", "");
    let path = tmp_file("r2-qinplace-cli.bisa", &support::rehash(&text).text);
    for t in ["0", "1"] {
        for adj in [false, true] {
            let mut args = vec![
                "run-action",
                &path,
                "a",
                "--mut",
                t,
                "--arg",
                "token:1,0,0,0",
            ];
            if adj {
                args.push("--adjoint");
            }
            assert_eq!(
                field(&ok(&args), "state").canonical(),
                format!("{{\"{t}\":[1,0]}}")
            );
        }
        let v = ok(&["reverse", &path, "a", "--mut", t, "--arg", "token:1,0,0,0"]);
        assert_eq!(field(&v, "equals_entry"), &Json::Bool(true));
    }
    // Published sum_db's n=0 path: wrapper 4 + callee 6 = 10 rows;
    // 13 saved locations + 3 record cells = stack 16, payload and code zero.
    let v = ok(&[
        "run-oracle",
        &fixture("sum_db"),
        "sum_db",
        "--input",
        "0",
        "--budget",
        "10,4080,16,64",
    ]);
    assert_eq!(
        (uint(&v, "payload"), uint(&v, "code"), uint(&v, "steps")),
        (0, 0, 10)
    );
}

#[test]
fn r4_dynamic_region_aliases_report_failed_json_and_reverse_the_prefix() {
    for tier in ["cc", "r"] {
        for kind in ["interface", "control", "readonly-control", "oracle"] {
            let o = support::dynamic_region_alias_object(kind, tier);
            let path = tmp_file(&format!("r4-alias-{tier}-{kind}.bisa"), &o.text);
            for c in ["0", "1"] {
                for command in ["run-cc", "reverse", "run-program"] {
                    if command == "run-program" && tier == "r" {
                        continue;
                    }
                    let mut args = vec![
                        command, &path, "p", "--arg", c, "--arg", "qreg:0", "--arg", "qreg:1",
                    ];
                    if kind == "oracle" {
                        args.extend(["--arg", "qreg:0000", "--arg", "token:1"]);
                    }
                    let v = ok(&args);
                    // The faulting call is the second executed row. No ret or
                    // output delivery executes. §7.4 reversal undoes both frames.
                    assert_eq!(
                        text(&v, "status"),
                        if command == "reverse" {
                            "REVERSED"
                        } else if c == "1" {
                            "FAILED"
                        } else {
                            "COMPLETED"
                        }
                    );
                    assert_eq!(uint(&v, "steps"), if c == "1" { 2 } else { 3 });
                    assert_eq!(
                        field(&v, "fault"),
                        &if c == "1" {
                            Json::Number("4".into())
                        } else {
                            Json::Null
                        }
                    );
                    assert!(array(&v, "results").is_empty());
                    if c == "1" {
                        let faults = array(&v, "faults");
                        assert_eq!(faults.len(), 1);
                        assert_eq!(uint(&faults[0], "code"), 4);
                        assert_eq!(text(&faults[0], "name"), "FAULT_UB_TRAP");
                    }
                    if command == "reverse" {
                        assert_eq!(field(&v, "equals_entry"), &Json::Bool(true));
                        assert_eq!(field(&v, "first_difference"), &Json::Null);
                    }
                }
            }
        }
    }
}

#[test]
fn sol_r4_b01_tiny_born_branch_is_selectable() {
    // Rx(theta)|1> = -i sin(theta/2)|0> + cos(theta/2)|1>.
    // For nonzero theta, draw=0 selects the first supported outcome, even
    // when sin^2(theta/2) is below binary64's least positive number.
    let a = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let p = r#"{"abi":"0.8","kind":"cc","params":["qreg[1]"],"results":["i1"],"tier":"cc"}"#;
    for angle in ["1e-100", "1e-161", "1e-162", "1e-200", "5e-324", "0.0"] {
        let action = format!("action @a (t:wire mut) () () () ()\n^entry():\nrx {angle} t");
        let parent = "fn cc @p (q:qreg[1]) (r:i1) () () ()\n^entry():\naction_call @a (q) () () ()\nr = measure q\nret r";
        let o = support::rehash(&support::object(
            &[("a", a, &action), ("p", p, parent)],
            "{}",
            "",
        ));
        let path = tmp_file(&format!("sol-r4-b01-{angle}.bisa"), &o.text);
        ok(&["check", &path]);
        for draw in ["0", "0.5"] {
            let v = ok(&["run-cc", &path, "p", "--arg", "qreg:1", "--draws", draw]);
            let expected = if draw == "0" && angle != "0.0" {
                "0"
            } else {
                "1"
            };
            assert_eq!(text(&v, "status"), "COMPLETED");
            assert_eq!(uint(&v, "steps"), 3);
            assert_eq!(field(&v, "fault"), &Json::Null);
            assert_eq!(
                result(&array(&v, "results")[0]).1,
                expected,
                "angle={angle}, draw={draw}"
            );
            assert_eq!(text(&array(&v, "measurements")[0], "bits"), expected);
        }
        fails(
            &["reverse", &path, "p", "--arg", "qreg:1", "--draws", "0"],
            "measure",
        );
    }
}
