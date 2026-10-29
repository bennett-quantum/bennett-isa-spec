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
