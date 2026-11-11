mod support;
use bennett_isa_ref::json::Json;
use std::{
    io::Write,
    path::PathBuf,
    process::{Command, Stdio},
    sync::atomic::{AtomicUsize, Ordering},
};

struct Source(PathBuf);
impl Source {
    fn new(name: &str, width: usize, body: &str) -> Self {
        let signature = format!(
            r#"{{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[{width}],"tier":"q"}}"#
        );
        let text = support::object(
            &[(
                name,
                &signature,
                &format!("action @{name} (t:wire[{width}] mut) () () () ()\n^entry():\n{body}"),
            )],
            "{}",
            "",
        );
        Self::text(&text)
    }
    fn text(text: &str) -> Self {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let object = support::rehash(text);
        let path = std::env::temp_dir().join(format!(
            "batch-{}-{}.bisa",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::write(&path, object.text).unwrap();
        Self(path)
    }
}

#[test]
fn dirty_borrow_pool_and_private_lifetimes_are_fresh_for_every_tuple() {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let source=Source::text(&support::object(&[("a",signature,"perm_action @a (t:wire mut) (x:wire in) () () ()\n^entry():\nc = qcopy x\nqxor c t\nquncopy x c\nb = qborrow any 1\ncx b t\ncx b t\nqreturn b")],"{}",""));
    let output = call(&source, "a", &["--range", "0..8", "--format", "jsonl"], "");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = lines(&output);
    for rank in 0usize..8 {
        let (x, t, b) = (rank >> 2, (rank >> 1) & 1, rank & 1);
        let row = rows[rank + 1].array().unwrap();
        assert_eq!(row[2].uint().unwrap(), 127);
        assert_eq!(
            row[3].string().unwrap(),
            format!("{:02x}", (t ^ x) | (x << 1) | (b << 2))
        );
        assert_eq!(
            row[4].string().unwrap(),
            format!("{:02x}", t | (x << 1) | (b << 2))
        );
    }
}

#[test]
fn mutable_regions_are_in_both_full_tuples_and_nonbasis_outputs_fail_explicitly() {
    let source = Source::text(include_str!("fixtures/control_flow.bisa"));
    let output = call(
        &source,
        "exchange",
        &["--format", "jsonl"],
        "{\"mut\":[2],\"in\":[1],\"mem\":{\"m\":[1,3]},\"borrowed\":[]}\n",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = lines(&output);
    let row = rows[1].array().unwrap();
    assert_eq!(row[2].uint().unwrap(), 127);
    assert_eq!(row[3].string().unwrap(), "4f");
    assert_eq!(row[4].string().unwrap(), "6e");
    let bell = Source::new("bell", 2, "h t[0]\ncx t[0] t[1]");
    let output = call(&bell, "bell", &["--range", "0..1", "--format", "jsonl"], "");
    assert!(!output.status.success());
    let rows = lines(&output);
    assert_eq!(rows[1].array().unwrap()[2].uint().unwrap(), 124);
    assert_eq!(rows[1].array().unwrap()[3], Json::Null);
    assert_eq!(rows[1].array().unwrap()[4].string().unwrap(), "00");
    assert_eq!(
        rows.last().unwrap().get("complete").unwrap(),
        &Json::Bool(true)
    );
}
impl Drop for Source {
    fn drop(&mut self) {
        std::fs::remove_file(&self.0).unwrap();
    }
}
fn call(source: &Source, action: &str, options: &[&str], input: &str) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
        .args(["reverse-action-batch", source.0.to_str().unwrap(), action])
        .args(options)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(input.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
fn lines(output: &std::process::Output) -> Vec<Json> {
    std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .map(|s| Json::parse(s).unwrap())
        .collect()
}

#[test]
fn range_shards_match_scalar_reverse_and_preserve_the_ordered_full_tuple() {
    let source = Source::new("toggle", 2, "x t[0]");
    let output = call(
        &source,
        "toggle",
        &["--range", "0..4", "--format", "jsonl"],
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = lines(&output);
    assert_eq!(
        rows[0].get("format").unwrap().string().unwrap(),
        "bennett.reverse-action-batch/1"
    );
    assert_eq!(rows[0].get("domain_bits").unwrap().uint().unwrap(), 2);
    for i in 0..4 {
        let row = rows[i + 1].array().unwrap();
        assert_eq!(row[0].string().unwrap(), "case");
        assert_eq!(row[1].string().unwrap(), i.to_string());
        assert_eq!(row[2].uint().unwrap(), 127);
        assert_eq!(row[3].string().unwrap(), format!("{:02x}", i ^ 1));
        assert_eq!(row[4].string().unwrap(), format!("{i:02x}"));
        let scalar = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
            .args([
                "reverse",
                source.0.to_str().unwrap(),
                "toggle",
                "--mut",
                &i.to_string(),
            ])
            .output()
            .unwrap();
        assert!(scalar.status.success());
        assert_eq!(
            Json::parse(std::str::from_utf8(&scalar.stdout).unwrap())
                .unwrap()
                .get("equals_entry")
                .unwrap(),
            &Json::Bool(true)
        );
    }
    assert_eq!(
        rows.last().unwrap().get("cases").unwrap().string().unwrap(),
        "4"
    );
    let shard = call(
        &source,
        "toggle",
        &["--range", "2..4", "--format", "jsonl"],
        "",
    );
    let shard_rows = lines(&shard);
    assert_eq!(&shard_rows[1..3], &rows[3..5]);
    for range in ["4..5", "2..1", "-1..2", "0..01"] {
        let bad = call(&source, "toggle", &["--range", range], "");
        assert!(!bad.status.success(), "{range}");
        assert!(bad.stdout.is_empty(), "invalid range published a header");
    }
}

#[test]
fn jsonl_cases_are_fresh_and_incomplete_streams_have_no_complete_footer() {
    let source = Source::new("toggle", 1, "x t[0]");
    let input="{\"mut\":[0],\"in\":[],\"mem\":{},\"borrowed\":[]}\n{\"mut\":[1],\"in\":[],\"mem\":{},\"borrowed\":[]}\n{\"mut\":[0],\"in\":[],\"mem\":{},\"borrowed\":[]}\n";
    let output = call(&source, "toggle", &["--format", "jsonl"], input);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = lines(&output);
    assert_eq!(rows[1].array().unwrap()[3].string().unwrap(), "01");
    assert_eq!(rows[2].array().unwrap()[3].string().unwrap(), "00");
    assert_eq!(rows[3].array().unwrap()[3].string().unwrap(), "01");
    let bad = call(
        &source,
        "toggle",
        &["--format", "jsonl"],
        &(input.lines().next().unwrap().to_owned() + "\n{bad}\n"),
    );
    assert!(!bad.status.success());
    assert_eq!(lines(&bad).len(), 2);
    assert!(String::from_utf8_lossy(&bad.stderr).contains("tuple 1"));
}

#[test]
fn binary_records_are_fixed_width_and_phase_failures_are_not_accepted() {
    let source = Source::new("phase", 1, "s t[0]\ns t[0]");
    let output = call(&source, "phase", &["--range", "0..2"], "");
    assert!(!output.status.success());
    let split = output.stdout.iter().position(|b| *b == b'\n').unwrap();
    let header = Json::parse(std::str::from_utf8(&output.stdout[..split]).unwrap()).unwrap();
    assert_eq!(header.get("record_bytes").unwrap().uint().unwrap(), 4);
    let records = &output.stdout[split + 1..];
    assert_eq!(&records[..8], &[1, 127, 0, 0, 1, 125, 1, 1]);
    assert_eq!(records[8], 0);
    let footer = Json::parse(std::str::from_utf8(&records[9..]).unwrap().trim()).unwrap();
    assert_eq!(footer.get("failed").unwrap().string().unwrap(), "1");
    assert_eq!(footer.get("first_failed").unwrap().string().unwrap(), "1");
}

#[test]
fn range_addresses_above_u128_are_not_truncated() {
    let source = Source::new("wide", 130, "x t[129]\nx t[129]");
    let output = call(
        &source,
        "wide",
        &[
            "--range",
            "680564733841876926926749214863536422912..680564733841876926926749214863536422914",
            "--format",
            "jsonl",
        ],
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = lines(&output);
    assert_eq!(
        rows[1].array().unwrap()[3].string().unwrap(),
        "0000000000000000000000000000000002"
    );
    assert_eq!(
        rows[2].array().unwrap()[3].string().unwrap(),
        "0100000000000000000000000000000002"
    );
}

#[test]
fn interactive_jsonl_returns_each_case_before_stdin_eof() {
    use std::{
        io::{BufRead, BufReader},
        sync::mpsc,
        time::Duration,
    };
    let source = Source::new("toggle", 1, "x t[0]");
    let mut child = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
        .args([
            "reverse-action-batch",
            source.0.to_str().unwrap(),
            "toggle",
            "--format",
            "jsonl",
            "--flush-each",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    let stdout = child.stdout.take().unwrap();
    let (send, receive) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        for line in BufReader::new(stdout).lines() {
            if send.send(line.unwrap()).is_err() {
                break;
            }
        }
    });
    let next = |child: &mut std::process::Child| match receive.recv_timeout(Duration::from_secs(5))
    {
        Ok(line) => Json::parse(&line).unwrap(),
        Err(error) => {
            child.kill().ok();
            child.wait().ok();
            panic!("stream did not respond before EOF: {error}");
        }
    };
    assert_eq!(
        next(&mut child).get("type").unwrap().string().unwrap(),
        "header"
    );
    for value in [0, 1] {
        writeln!(
            child.stdin.as_mut().unwrap(),
            "{{\"mut\":[{value}],\"in\":[],\"mem\":{{}},\"borrowed\":[]}}"
        )
        .unwrap();
        child.stdin.as_mut().unwrap().flush().unwrap();
        let row = next(&mut child);
        assert_eq!(
            row.array().unwrap()[3].string().unwrap(),
            format!("{:02x}", value ^ 1)
        );
    }
    drop(child.stdin.take());
    assert_eq!(
        next(&mut child).get("cases").unwrap().string().unwrap(),
        "2"
    );
    assert!(child.wait().unwrap().success());
    reader.join().unwrap();
}

#[test]
fn fixed_09_token_arguments_and_aliases_resolve_fresh_per_tuple() {
    let oracle =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let action = r#"{"abi":"0.9","args":["token"],"bounds":[],"in":[1],"kind":"action","mem":[],"mut":[1,4],"tier":"q"}"#;
    let source=Source::text(&support::object(&[("o",oracle,"oracle @o (x:wire in) (t:wire) (e:wire[4]) () ()\n^entry():\nlift.retval x"),("a",action,"action @a (t:wire mut, e:wire[4] mut) (x:wire in) () (tk:token) ()\n^entry():\noracle_call @o tk (x) t e () ()")],"{}","").replace("=0.8","=0.9"));
    let output = call(
        &source,
        "a",
        &[
            "--arg",
            "token:1,0,0,0",
            "--token-aliases",
            "[0]",
            "--range",
            "0..64",
            "--format",
            "jsonl",
        ],
        "",
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let rows = lines(&output);
    for rank in 0usize..64 {
        let (x, t, e) = (rank >> 5, (rank >> 4) & 1, rank & 15);
        let row = rows[rank + 1].array().unwrap();
        assert_eq!(row[2].uint().unwrap(), 127);
        assert_eq!(
            row[3].string().unwrap(),
            format!("{:02x}", (t ^ x) | (e << 1) | (x << 5))
        );
        assert_eq!(
            row[4].string().unwrap(),
            format!("{:02x}", t | (e << 1) | (x << 5))
        );
    }
}

#[test]
fn adjoint_selection_and_empty_shards_are_explicit() {
    let source = Source::new("cycle", 2, "cx t[0] t[1]\nx t[0]");
    let output = call(
        &source,
        "cycle",
        &["--adjoint", "--range", "0..1", "--format", "jsonl"],
        "",
    );
    assert!(output.status.success());
    let rows = lines(&output);
    assert_eq!(rows[1].array().unwrap()[3].string().unwrap(), "03");
    assert_eq!(rows[1].array().unwrap()[4].string().unwrap(), "00");
    let output = call(
        &source,
        "cycle",
        &["--range", "4..4", "--format", "jsonl"],
        "",
    );
    assert!(output.status.success());
    let rows = lines(&output);
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[1].get("cases").unwrap().string().unwrap(), "0");
}
