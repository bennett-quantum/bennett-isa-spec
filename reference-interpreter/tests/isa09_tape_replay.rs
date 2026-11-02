//! Actual 0.9 CC/R programs and the unchanged saved-tape profile v1.
mod support;
use bennett_isa_ref::{json::Json, ClassicalMachine, Error, Object};
use std::{
    path::PathBuf,
    process::{Command, Output},
    sync::atomic::{AtomicU64, Ordering},
};
struct Scratch(PathBuf);
impl Scratch {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "isa09-tape-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }
    fn path(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().into()
    }
    fn write(&self, name: &str, bytes: impl AsRef<[u8]>) -> String {
        let p = self.path(name);
        std::fs::write(&p, bytes).unwrap();
        p
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}
fn object(code: Option<u64>, host: bool) -> Object {
    let mut text = include_str!("fixtures/classical.bisa")
        .replace("0.8", "0.9")
        .replace("\"heap_limit_max\":1024", "\"heap_limit_max\":2048");
    if let Some(code) = code {
        text = text.replace(
            "r = load.i64 p\nret r\nfn r @impure",
            &format!(
                "fault {code}\n^unreachable():\neffect console.write p len\nfault 99\nfn r @impure"
            ),
        );
    }
    let mut lines = text.lines().map(str::to_string).collect::<Vec<_>>();
    let index = lines.iter().position(|s| s == "section symbols").unwrap() + 1;
    let Json::Object(mut symbols) = support::json(&lines[index]) else {
        unreachable!()
    };
    symbols.insert("12".into(),support::json(r#"{"interface_hash":"0000000000000000000000000000000000000000000000000000000000000000","kind":"cc","name":"main","signature":{"abi":"0.9","kind":"cc","params":[],"results":["i64"],"tier":"cc"}}"#));
    symbols.insert("13".into(),support::json(r#"{"interface_hash":"0000000000000000000000000000000000000000000000000000000000000000","kind":"r","name":"marker","signature":{"abi":"0.9","kind":"r","params":[],"results":[],"tier":"r"}}"#));
    lines[index] = Json::Object(symbols).canonical();
    let host = if host { "n = call @host\n" } else { "" };
    let launch=format!("fn cc @main () (r:i64) () () ()\n^entry():\n{host}r = call @effects\ncall @marker\nret r\nfn r @marker () () () () ()\n^entry():\none = const.i16 1\np = alloc one\nmarker = const.i8 99\nstore.i8 p marker\neffect console.write p one\nret\n");
    let text = (lines.join("\n") + "\n").replace(
        "section functions\n",
        &format!("section functions\n{launch}"),
    );
    let object = support::rehash(&text);
    object.check_hashes().unwrap();
    object
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
fn load(path: &str) -> Json {
    Json::parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn reset(value: &Json) {
    assert_eq!(value.get("status").unwrap().string().unwrap(), "REVERSED");
    for field in ["equals_entry", "memory_equals_entry"] {
        assert_eq!(value.get(field).unwrap(), &Json::Bool(true));
    }
    assert_eq!(value.get("console_hex").unwrap().string().unwrap(), "");
    assert_eq!(value.get("effect_tape_length").unwrap().uint().unwrap(), 0);
    assert_eq!(value.get("first_difference").unwrap(), &Json::Null);
    assert_eq!(
        value.get("memory_digest").unwrap(),
        value.get("entry_memory_digest").unwrap()
    );
}
fn rejected(out: Output) {
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains('§'));
}

#[test]
fn successful_09_r_and_launch_runs_record_replay_and_reset_the_complete_state() {
    for host in [false, true] {
        let s = Scratch::new();
        let o = object(None, host);
        let source = s.write("program.bisa", &o.text);
        let replies = s.write(
            "replies.json",
            br#"[{"service":"org.bennett.test","payload":"a50000","reply":"112233"}]"#,
        );
        for (case, (command, reverse)) in [
            ("run-cc", false),
            ("reverse", true),
            ("run-program", false),
            ("run-program", true),
        ]
        .into_iter()
        .enumerate()
        {
            let launch = command == "run-program";
            let name = if launch { "main" } else { "effects" };
            let mut args = vec![command, source.as_str(), name, "--entropy", "0b1621"];
            if launch && host {
                args.extend(["--replies", replies.as_str()]);
            }
            if launch && reverse {
                args.push("--reverse");
            }
            let tape = s.path(&format!("record-{case}.json"));
            let copy = s.path(&format!("copy-{case}.json"));
            let mut record = args.clone();
            record.extend(["--tape-out", tape.as_str()]);
            let baseline = cli(&record);
            let value = success(&baseline);
            let mut replay = args.clone();
            replay.extend(["--replay-tape", tape.as_str(), "--tape-out", copy.as_str()]);
            let repeated = cli(&replay);
            success(&repeated);
            assert_eq!(baseline.stdout, repeated.stdout);
            assert_eq!(std::fs::read(&tape).unwrap(), std::fs::read(&copy).unwrap());
            assert_eq!(
                load(&tape).array().unwrap().len(),
                if launch { 6 } else { 5 }
            );
            assert_eq!(value.get("fault").unwrap(), &Json::Null);
            if reverse {
                reset(&value);
            } else {
                assert_eq!(value.get("status").unwrap().string().unwrap(), "COMPLETED");
                assert_eq!(
                    value.get("console_hex").unwrap().string().unwrap(),
                    if launch { "0b162163" } else { "0b1621" }
                );
            }
            if launch {
                assert_eq!(
                    value.get("host_requests").unwrap().array().unwrap().len(),
                    usize::from(host)
                );
            }
        }
    }
}

#[test]
fn exact_09_terminal_faults_replay_only_the_intact_effect_prefix_and_reverse_open_callers() {
    for code in [0, 1, 2, 3, u64::MAX] {
        let s = Scratch::new();
        let source = s.write("fault.bisa", object(Some(code), false).text);
        for (case, (command, reverse)) in [
            ("run-cc", false),
            ("reverse", true),
            ("run-program", false),
            ("run-program", true),
        ]
        .into_iter()
        .enumerate()
        {
            let launch = command == "run-program";
            let mut args = vec![
                command,
                source.as_str(),
                if launch { "main" } else { "effects" },
                "--entropy",
                "0b1621",
            ];
            if launch && reverse {
                args.push("--reverse");
            }
            let tape = s.path(&format!("fault-{case}.json"));
            let mut record = args.clone();
            record.extend(["--tape-out", tape.as_str()]);
            let baseline = cli(&record);
            let value = success(&baseline);
            let effective = if code == 0 { 4 } else { code as u128 };
            assert_eq!(value.get("fault").unwrap().uint().unwrap(), effective);
            let faults = value.get("faults").unwrap().array().unwrap();
            assert_eq!(faults.len(), 1);
            assert_eq!(faults[0].get("code").unwrap().uint().unwrap(), effective);
            assert!(value.get("results").unwrap().array().unwrap().is_empty());
            assert_eq!(load(&tape).array().unwrap().len(), 5);
            let mut replay = args.clone();
            replay.extend(["--replay-tape", tape.as_str()]);
            let repeated = cli(&replay);
            success(&repeated);
            assert_eq!(baseline.stdout, repeated.stdout);
            if reverse {
                reset(&value);
            } else {
                assert_eq!(value.get("status").unwrap().string().unwrap(), "FAILED");
                assert_eq!(
                    value.get("console_hex").unwrap().string().unwrap(),
                    "0b1621"
                );
            }
        }
    }
}

#[test]
fn every_09_successful_or_failed_prefix_has_exact_tape_consumption_and_reset() {
    for code in [None, Some(u64::MAX)] {
        let s = Scratch::new();
        let source = s.write("prefix.bisa", object(code, false).text);
        for steps in 0..=12 {
            let n = steps.to_string();
            let tape = s.path(&format!("prefix-{steps}.json"));
            let args = [
                "reverse",
                source.as_str(),
                "effects",
                "--steps",
                n.as_str(),
                "--entropy",
                "0b1621",
            ];
            let mut record = args.to_vec();
            record.extend(["--tape-out", tape.as_str()]);
            let baseline = cli(&record);
            let value = success(&baseline);
            reset(&value);
            let effects = [3, 4, 5, 7, 8].into_iter().filter(|n| steps > *n).count();
            assert_eq!(load(&tape).array().unwrap().len(), effects);
            let mut replay = args.to_vec();
            replay.extend(["--replay-tape", tape.as_str()]);
            let repeated = cli(&replay);
            success(&repeated);
            assert_eq!(baseline.stdout, repeated.stdout);
        }
    }
}

#[test]
fn profile_v1_rejects_every_field_order_exhaustion_and_schema_divergence_without_publication() {
    let s = Scratch::new();
    let source = s.write("failed.bisa", object(Some(u64::MAX), false).text);
    let tape = s.path("valid.json");
    success(&cli(&[
        "run-cc",
        &source,
        "effects",
        "--entropy",
        "0b1621",
        "--tape-out",
        &tape,
    ]));
    let entries = load(&tape).array().unwrap().to_vec();
    let mut mutations = Vec::new();
    for index in 0..entries.len() {
        for field in ["dev", "op", "args", "bytes"] {
            let mut rows = entries.clone();
            let Json::Object(row) = &mut rows[index] else {
                unreachable!()
            };
            let mut value = match field {
                "dev" => Json::String("clock".into()),
                "op" => Json::String("wrong".into()),
                "args" => support::json("[17]"),
                "bytes" => Json::String("ff".into()),
                _ => unreachable!(),
            };
            if row.get(field) == Some(&value) {
                value = Json::String("console".into());
            }
            row.insert(field.into(), value);
            mutations.push(Json::Array(rows).canonical());
        }
    }
    let mut rows = entries.clone();
    rows.swap(0, 1);
    mutations.push(Json::Array(rows).canonical());
    mutations.push(Json::Array(entries[..4].to_vec()).canonical());
    let mut rows = entries.clone();
    rows.push(entries[0].clone());
    mutations.push(Json::Array(rows).canonical());
    mutations.extend(
        [
            "{}",
            "[{}]",
            "[null]",
            "[",
            r#"[{"dev":"entropy","op":"read","args":[-1],"bytes":"00"}]"#,
            r#"[{"dev":"entropy","op":"read","args":["1"],"bytes":"00"}]"#,
            r#"[{"dev":"entropy","op":"read","args":[],"bytes":"0"}]"#,
            r#"[{"dev":"entropy","op":"read","args":[],"bytes":"0g"}]"#,
            r#"[{"dev":"entropy","op":"read","args":[],"bytes":"","extra":0}]"#,
        ]
        .into_iter()
        .map(str::to_string),
    );
    for (index, raw) in mutations.into_iter().enumerate() {
        let bad = s.write(&format!("bad-{index}.json"), raw);
        for (command, name) in [("run-cc", "effects"), ("run-program", "main")] {
            let output = s.write("keep.json", b"keep\n");
            rejected(cli(&[
                command,
                &source,
                name,
                "--entropy",
                "0b1621",
                "--replay-tape",
                &bad,
                "--tape-out",
                &output,
            ]));
            assert_eq!(std::fs::read(output).unwrap(), b"keep\n");
        }
    }
    for entropy in ["0b1622", ""] {
        rejected(cli(&[
            "run-program",
            &source,
            "main",
            "--entropy",
            entropy,
            "--replay-tape",
            &tape,
        ]));
    }
}

#[test]
fn actual_09_effect_observer_mismatch_is_retryable_without_advancing_memory_history_or_tape() {
    let o = object(Some(u64::MAX), false);
    let mut machine = ClassicalMachine::new(&o, "effects", &[], &[11, 22, 33]).unwrap();
    let entry = machine.snapshot();
    for _ in 0..3 {
        machine.step_once().unwrap();
    }
    let prefix = machine.snapshot();
    let frames = machine.outcome().steps;
    for _ in 0..2 {
        let error = machine
            .step_once_observed::<()>(|point, before, after| {
                assert_eq!(point.instruction.op, "effect");
                assert_eq!(before.effect_tape().len(), 0);
                assert_eq!(after.effect_tape().len(), 1);
                Err(Error::new("§7.3: replay mismatch before commit".into()))
            })
            .unwrap_err();
        assert!(error.to_string().contains("replay mismatch"));
        assert_eq!(machine.snapshot(), prefix);
        assert_eq!(machine.outcome().steps, frames);
    }
    let (_, effect) = machine
        .step_once_observed(|_, _, after| Ok(after.effect_tape().last().unwrap().clone()))
        .unwrap()
        .unwrap();
    assert_eq!(&*effect.bytes, &[11, 22, 33]);
    assert_eq!(machine.effect_tape_len(), 1);
    assert_eq!(machine.memory_bytes(16, 3).unwrap(), [11, 22, 33]);
    assert_eq!(machine.run(100).unwrap().fault, Some(u64::MAX as u128));
    machine.reverse().unwrap();
    assert_eq!(machine.snapshot(), entry);
}

#[test]
fn saved_09_journal_replays_effects_already_removed_by_nested_uncall_exactly_once() {
    let s = Scratch::new();
    let source = s.write("uncall.bisa", object(None, false).text);
    let tape = s.path("journal.json");
    let baseline = cli(&["run-cc", &source, "nested", "--tape-out", &tape]);
    let value = success(&baseline);
    assert!(value.get("tape").unwrap().array().unwrap().is_empty());
    assert_eq!(value.get("clock").unwrap().uint().unwrap(), 0);
    assert_eq!(load(&tape).array().unwrap().len(), 1);
    let replay = cli(&["run-cc", &source, "nested", "--replay-tape", &tape]);
    success(&replay);
    assert_eq!(baseline.stdout, replay.stdout);
    let reversed = success(&cli(&[
        "reverse",
        &source,
        "nested",
        "--replay-tape",
        &tape,
    ]));
    reset(&reversed);
    let empty = s.write("wrong-empty.json", b"[]\n");
    rejected(cli(&["run-cc", &source, "nested", "--replay-tape", &empty]));
}
