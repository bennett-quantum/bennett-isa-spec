//! Saved-tape consumer interface under ISA 0.8 §7.3/§7.4, dialect §19.2.
//! These device controls do not claim musl rung-3 acceptance or ISA 0.9 admission.
use bennett_isa_ref::{json::Json, Object};
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.tmp");
        let path = root.join(format!(
            "tape-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self(path)
    }
    fn path(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().into()
    }
    fn write(&self, name: &str, value: &Json) -> String {
        let path = self.path(name);
        std::fs::write(&path, value.canonical() + "\n").unwrap();
        path
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn fixture() -> String {
    format!(
        "{}/tests/fixtures/classical.bisa",
        env!("CARGO_MANIFEST_DIR")
    )
}
fn cli(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
        .args(args)
        .output()
        .unwrap()
}
fn success(out: &Output) -> Json {
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    Json::parse(std::str::from_utf8(&out.stdout).unwrap()).unwrap()
}
fn mismatch(out: Output) {
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stdout.is_empty());
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(
        err.contains('§') && err.contains("replay mismatch"),
        "{err}"
    );
}
fn load(path: &str) -> Json {
    Json::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn record(s: &Scratch) -> (String, Json) {
    let path = s.path("forward.json");
    let out = cli(&[
        "run-cc",
        &fixture(),
        "effects",
        "--entropy",
        "0b1621",
        "--tape-out",
        &path,
    ]);
    let value = success(&out);
    assert_eq!(&load(&path), value.get("tape").unwrap());
    (path, value)
}
#[test]
fn saved_five_device_effects_replay_byte_identically_and_reset() {
    let s = Scratch::new();
    let (path, value) = record(&s);
    assert_eq!(
        value.get("console_hex").unwrap().string().unwrap(),
        "0b1621"
    );
    assert_eq!(
        value.get("results").unwrap().array().unwrap()[0]
            .get("value")
            .unwrap()
            .string()
            .unwrap(),
        "0"
    );
    assert_eq!(load(&path).array().unwrap().len(), 5);
    let copy = s.path("copy.json");
    let baseline = cli(&["run-cc", &fixture(), "effects", "--entropy", "0b1621"]);
    success(&baseline);
    let replay = cli(&[
        "run-cc",
        &fixture(),
        "effects",
        "--entropy",
        "0b1621",
        "--replay-tape",
        &path,
        "--tape-out",
        &copy,
    ]);
    success(&replay);
    assert_eq!(baseline.stdout, replay.stdout);
    assert_eq!(std::fs::read(&path).unwrap(), std::fs::read(&copy).unwrap());
    let reverse_copy = s.path("reverse.json");
    let reversed = cli(&[
        "reverse",
        &fixture(),
        "effects",
        "--entropy",
        "0b1621",
        "--replay-tape",
        &path,
        "--tape-out",
        &reverse_copy,
    ]);
    let reset = success(&reversed);
    assert_eq!(reset.get("equals_entry").unwrap(), &Json::Bool(true));
    assert_eq!(reset.get("memory_equals_entry").unwrap(), &Json::Bool(true));
    assert_eq!(reset.get("console_hex").unwrap().string().unwrap(), "");
    assert_eq!(reset.get("effect_tape_length").unwrap().uint().unwrap(), 0);
    assert_eq!(reset.get("first_difference").unwrap(), &Json::Null);
    assert_eq!(
        reset.get("memory_digest").unwrap(),
        reset.get("entry_memory_digest").unwrap()
    );
    let fresh_reverse = cli(&["reverse", &fixture(), "effects", "--entropy", "0b1621"]);
    success(&fresh_reverse);
    assert_eq!(fresh_reverse.stdout, reversed.stdout);
    assert_eq!(load(&reverse_copy), load(&path));
}
#[test]
fn every_entry_field_order_and_exhaustion_are_checked_without_publishing() {
    let s = Scratch::new();
    let (path, _) = record(&s);
    let entries = load(&path).array().unwrap().to_vec();
    let mut mutations = Vec::new();
    for index in 0..entries.len() {
        for key in ["dev", "op", "args", "bytes"] {
            let mut changed = entries.clone();
            let Json::Object(row) = &mut changed[index] else {
                unreachable!()
            };
            let replacement = match key {
                "dev" => Json::String("clock".into()),
                "op" => Json::String("tampered".into()),
                "args" => Json::Array(vec![Json::Number("17".into())]),
                "bytes" => Json::String("ff".into()),
                _ => unreachable!(),
            };
            // Clock's existing dev is clock; choose a different one there.
            let replacement = if row.get(key) == Some(&replacement) {
                Json::String("console".into())
            } else {
                replacement
            };
            row.insert(key.into(), replacement);
            mutations.push(changed);
        }
    }
    let mut swapped = entries.clone();
    swapped.swap(0, 1);
    mutations.push(swapped);
    mutations.push(entries[..4].to_vec());
    let mut extra = entries.clone();
    extra.push(entries[0].clone());
    mutations.push(extra);
    for (i, rows) in mutations.into_iter().enumerate() {
        let bad = s.write(&format!("bad-{i}.json"), &Json::Array(rows));
        let output = s.path(&format!("never-{i}.json"));
        mismatch(cli(&[
            "run-cc",
            &fixture(),
            "effects",
            "--entropy",
            "0b1621",
            "--replay-tape",
            &bad,
            "--tape-out",
            &output,
        ]));
        assert!(!Path::new(&output).exists());
    }
    mismatch(cli(&[
        "run-cc",
        &fixture(),
        "effects",
        "--entropy",
        "0b1622",
        "--replay-tape",
        &path,
    ]));
    // This is a replay divergence even if entropy depletion turns the first
    // effect into fault 10 instead of appending an entry.
    mismatch(cli(&[
        "run-cc",
        &fixture(),
        "effects",
        "--entropy",
        "",
        "--replay-tape",
        &path,
    ]));
}
#[test]
fn tape_schema_is_strict_and_diagnostics_leave_existing_output_untouched() {
    let s = Scratch::new();
    let output = s.path("keep.json");
    std::fs::write(&output, b"keep\n").unwrap();
    for (i, raw) in [
        "{}",
        "[{}]",
        "[null]",
        "[",
        r#"[{"dev":"console","op":"write","args":[-1],"bytes":"00"}]"#,
        r#"[{"dev":"console","op":"write","args":["1"],"bytes":"00"}]"#,
        r#"[{"dev":"console","op":"write","args":[9007199254740992],"bytes":"00"}]"#,
        r#"[{"dev":"console","op":"write","args":[],"bytes":"0g"}]"#,
        r#"[{"dev":"console","op":"write","args":[],"bytes":"0"}]"#,
        r#"[{"dev":"console","op":"write","args":[],"bytes":"","extra":0}]"#,
    ]
    .iter()
    .enumerate()
    {
        let path = s.path(&format!("schema-{i}.json"));
        std::fs::write(&path, raw).unwrap();
        let out = cli(&[
            "run-cc",
            &fixture(),
            "effects",
            "--entropy",
            "0b1621",
            "--replay-tape",
            &path,
            "--tape-out",
            &output,
        ]);
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8(out.stderr).unwrap().contains('§'));
        assert_eq!(std::fs::read(&output).unwrap(), b"keep\n");
    }
}
#[test]
fn empty_and_prefix_tapes_require_exact_consumption() {
    let s = Scratch::new();
    let empty = s.write("empty.json", &Json::Array(vec![]));
    let copy = s.path("empty-copy.json");
    let out = cli(&[
        "run-cc",
        &fixture(),
        "sum",
        "--args",
        "3",
        "--replay-tape",
        &empty,
        "--tape-out",
        &copy,
    ]);
    success(&out);
    assert_eq!(load(&copy), load(&empty));
    mismatch(cli(&[
        "run-cc",
        &fixture(),
        "effects",
        "--entropy",
        "0b1621",
        "--replay-tape",
        &empty,
    ]));
    let (full, _) = record(&s);
    mismatch(cli(&[
        "run-cc",
        &fixture(),
        "sum",
        "--args",
        "3",
        "--replay-tape",
        &full,
    ]));
    let prefix = s.path("prefix.json");
    let args = [
        "reverse",
        &fixture(),
        "effects",
        "--steps",
        "4",
        "--entropy",
        "0b1621",
        "--tape-out",
        &prefix,
    ];
    let baseline = cli(&args);
    success(&baseline);
    assert_eq!(load(&prefix).array().unwrap().len(), 1);
    let replay = cli(&[
        "reverse",
        &fixture(),
        "effects",
        "--steps",
        "4",
        "--entropy",
        "0b1621",
        "--replay-tape",
        &prefix,
    ]);
    success(&replay);
    assert_eq!(baseline.stdout, replay.stdout);
    mismatch(cli(&[
        "reverse",
        &fixture(),
        "effects",
        "--steps",
        "4",
        "--entropy",
        "0b1621",
        "--replay-tape",
        &full,
    ]));
}
#[test]
fn saved_journal_keeps_effects_that_segment_uncall_removes_from_active_tape() {
    let s = Scratch::new();
    let path = s.path("uncall.json");
    let baseline = cli(&["run-cc", &fixture(), "nested", "--tape-out", &path]);
    let value = success(&baseline);
    assert!(value.get("tape").unwrap().array().unwrap().is_empty());
    assert_eq!(load(&path).array().unwrap().len(), 1);
    let replay = cli(&["run-cc", &fixture(), "nested", "--replay-tape", &path]);
    success(&replay);
    assert_eq!(baseline.stdout, replay.stdout);
    let reset = success(&cli(&[
        "reverse",
        &fixture(),
        "nested",
        "--replay-tape",
        &path,
    ]));
    assert_eq!(reset.get("equals_entry").unwrap(), &Json::Bool(true));
}

fn write_object(s: &Scratch, name: &str, text: &str) -> String {
    let hashes = Object::parse(text)
        .unwrap()
        .identity()
        .unwrap()
        .interface_hashes;
    let raw = text
        .split("section symbols\n")
        .nth(1)
        .unwrap()
        .lines()
        .next()
        .unwrap();
    let Json::Object(mut symbols) = Json::parse(raw).unwrap() else {
        unreachable!()
    };
    for row in symbols.values_mut() {
        let name = row.get("name").unwrap().string().unwrap().to_string();
        let Json::Object(row) = row else {
            unreachable!()
        };
        row.insert("interface_hash".into(), Json::String(hashes[&name].clone()));
    }
    let path = s.path(name);
    std::fs::write(&path, text.replace(raw, &Json::Object(symbols).canonical())).unwrap();
    path
}
// A CC launch stub with the same actual R effect body, optionally after a
// checked host request. Rehash after adding symbols; no published fixture edit.
fn launch(s: &Scratch) -> String {
    let text = std::fs::read_to_string(fixture())
        .unwrap()
        .replace("\"heap_limit_max\":1024", "\"heap_limit_max\":2048");
    let marker = "section functions\n";
    let syms = text
        .split("section symbols\n")
        .nth(1)
        .unwrap()
        .lines()
        .next()
        .unwrap();
    let Json::Object(mut symbols) = Json::parse(syms).unwrap() else {
        unreachable!()
    };
    symbols.insert("12".into(), Json::parse(r#"{"interface_hash":"0000000000000000000000000000000000000000000000000000000000000000","kind":"cc","name":"main","signature":{"abi":"0.8","kind":"cc","params":[],"results":["i64"],"tier":"cc"}}"#).unwrap());
    let text = text.replace(syms, &Json::Object(symbols).canonical()).replace(marker, &(marker.to_owned() + "fn cc @main () (r:i64) () () ()\n^entry():\nlen = call @host\nr = call @effects\nret r\n"));
    write_object(s, "launch.bisa", &text)
}
#[test]
fn launch_replay_preserves_host_replies_results_and_whole_run_reset() {
    let s = Scratch::new();
    let source = launch(&s);
    let replies = s.path("replies.json");
    std::fs::write(
        &replies,
        r#"[{"service":"org.bennett.test","payload":"a50000","reply":"112233"}]"#,
    )
    .unwrap();
    let tape = s.path("launch-tape.json");
    for reverse in [false, true] {
        let mut args = vec![
            "run-program",
            source.as_str(),
            "main",
            "--entropy",
            "0b1621",
            "--replies",
            replies.as_str(),
        ];
        if reverse {
            args.push("--reverse");
        }
        let mut record_args = args.clone();
        record_args.extend(["--tape-out", tape.as_str()]);
        let baseline = cli(&record_args);
        let value = success(&baseline);
        args.extend(["--replay-tape", tape.as_str()]);
        let replay = cli(&args);
        success(&replay);
        assert_eq!(baseline.stdout, replay.stdout);
        assert_eq!(
            value.get("host_requests").unwrap().array().unwrap().len(),
            1
        );
        assert_eq!(value.get("equals_entry").unwrap(), &Json::Bool(reverse));
        if reverse {
            assert_eq!(value.get("status").unwrap().string().unwrap(), "REVERSED");
        }
    }
}

#[test]
fn failed_run_replays_and_reverses_the_same_effect_prefix() {
    let s = Scratch::new();
    let text = std::fs::read_to_string(fixture()).unwrap();
    // Valid effect prefix followed by an actual memory fault (code 7),
    // not typed-fault source admission on ISA 0.8.
    let text = text.replace(
        "r = load.i64 p\nret r\nfn r @impure",
        "zero = const.i16 0\nbad = inttoptr zero\nr = load.i64 bad\nret r\nfn r @impure",
    );
    let source = write_object(&s, "fault.bisa", &text);
    let path = s.path("fault-tape.json");
    let baseline = cli(&[
        "run-cc",
        &source,
        "effects",
        "--entropy",
        "0b1621",
        "--tape-out",
        &path,
    ]);
    let value = success(&baseline);
    assert_eq!(value.get("fault").unwrap().uint().unwrap(), 7);
    assert!(value.get("results").unwrap().array().unwrap().is_empty());
    let replay = cli(&[
        "run-cc",
        &source,
        "effects",
        "--entropy",
        "0b1621",
        "--replay-tape",
        &path,
    ]);
    success(&replay);
    assert_eq!(baseline.stdout, replay.stdout);
    let reset = success(&cli(&[
        "reverse",
        &source,
        "effects",
        "--entropy",
        "0b1621",
        "--replay-tape",
        &path,
    ]));
    assert_eq!(reset.get("equals_entry").unwrap(), &Json::Bool(true));
    assert_eq!(reset.get("fault").unwrap().uint().unwrap(), 7);
}
#[test]
fn tape_options_reject_duplicate_and_missing_inputs_and_report_write_errors() {
    let s = Scratch::new();
    let empty = s.write("empty.json", &Json::Array(vec![]));
    for options in [
        vec!["--tape-out", empty.as_str(), "--tape-out", empty.as_str()],
        vec![
            "--replay-tape",
            empty.as_str(),
            "--replay-tape",
            empty.as_str(),
        ],
        vec!["--tape-out"],
        vec!["--replay-tape"],
        vec!["--replay-tape", "missing-no-such-tape.json"],
        vec!["--tape-out", s.0.to_str().unwrap()],
    ] {
        let source = fixture();
        let mut args = vec!["run-cc", source.as_str(), "sum", "--args", "3"];
        args.extend(options);
        let out = cli(&args);
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8(out.stderr).unwrap().contains('§'));
    }
    assert_eq!(load(&empty), Json::Array(vec![]));
}
