//! Retention stages must run after a feasible transfer, in an isolated AS limit.
mod support;
use bennett_isa_ref::{ClassicalMachine, Object};
use std::process::Command;
const WITNESS: &str = include_str!("fixtures/r22/hostcall-64MiB.bisa");

fn guarded(family: &str) {
    let out = Command::new("python3")
        .args(["-c", "import os,resource,sys; resource.setrlimit(resource.RLIMIT_AS,(128*1024*1024,)*2); resource.setrlimit(resource.RLIMIT_CORE,(0,0)); os.execv(sys.argv[1],sys.argv[1:])"])
        .arg(std::env::current_exe().unwrap())
        .args(["--exact", "r22_retention_child", "--nocapture"])
        .env("R22_FAMILY", family).env("MALLOC_ARENA_MAX", "1").output().unwrap();
    assert!(out.status.success(), "{family}: {out:?}");
    eprintln!("{}", String::from_utf8_lossy(&out.stderr));
}
#[test]
fn r22_b01_request_retention_and_observers() {
    guarded("host");
}
#[test]
fn r22_later_reply_retention_is_atomic() {
    guarded("reply");
}
#[test]
fn r22_console_later_reservation_is_atomic() {
    guarded("console");
}
#[test]
fn r22_retention_child() {
    let Ok(family) = std::env::var("R22_FAMILY") else {
        return;
    };
    std::panic::catch_unwind(|| {
        if family == "json" || family == "cow" {
            for n in [0, 1, 8, 1 << 20, 64 << 20] {
                let mut bytes = Vec::new(); bytes.try_reserve_exact(n).unwrap(); bytes.resize(n, b'a');
                eprintln!("{family}/{n}: first reservation and initialization succeeded");
                if family == "json" {
                    let value = bennett_isa_ref::json::Json::String(String::from_utf8(bytes).unwrap());
                    let result = value.try_canonical();
                    if n == 64 << 20 {
                        let e = result.unwrap_err();
                        assert_eq!(e.kind(), bennett_isa_ref::ErrorKind::HostResource);
                        assert!(e.to_string().contains("JSON output bytes") && e.to_string().contains("§10"));
                    } else { assert_eq!(result.unwrap().len(), n+2); }
                    assert_eq!(value.string().unwrap().len(), n);
                } else {
                    // Public snapshots/outcomes share this same buffer type.
                    let original = bennett_isa_ref::Retained::from(bytes);
                    let copy = original.clone();
                    assert_eq!(original.as_ptr(), copy.as_ptr());
                    assert_eq!(original, copy);
                }
            }
            return;
        }
        if family == "nonzero" {
            let o = support::rehash(&WITNESS.replace("const.i64 67108864", "const.i64 262144"));
            let mut m = ClassicalMachine::new(&o, "f", &[], &[]).unwrap();
            let entry = m.snapshot(); m.run(3).unwrap();
            let reply = vec![0xa5; 262144]; m.reply(&reply).unwrap();
            let after = m.snapshot();
            let request = m.host_transcript().unwrap();
            let transcript = m.host_transcript().unwrap();
            assert_eq!(request, transcript);
            assert_eq!(request[0].reply.as_ptr(), transcript[0].reply.as_ptr());
            assert_eq!(m.memory_bytes(1, reply.len()).unwrap(), reply);
            assert_eq!(m.memory_digest().unwrap().len(), 64);
            m.reverse().unwrap(); assert_eq!(m.snapshot(), entry);
            assert_ne!(after, entry); // observer retained throughout reversal
            eprintln!("nonzero/262144: reply/memory/undo/snapshot/transcript/reversal succeeded");
            return;
        }
        for n in [0, 1, 8, 1 << 20, 64 << 20, 1u64 << 63] {
            let mut text = WITNESS.replace("const.i64 67108864", &format!("const.i64 {n}"));
            if family == "console" { text = text.replace("\"models\":{}", "\"models\":{\"console\":{\"version\":\"0.8\"}}").replace("cc @f", "r @f").replace("\"cc\"", "\"r\"").replace("r = hostcall org.bennett.test p n", "effect console.write p n\nr = const.i64 0"); }
            let o = support::rehash(&text);
            o.check_hashes().unwrap();
            let mut m = ClassicalMachine::new(&o, "f", &[], &[]).unwrap();
            let entry = m.snapshot();
            m.step_once().unwrap(); m.step_once().unwrap();
            let prefix = m.snapshot();
            let result = m.step_once();
            if n == 1 << 63 || (family == "console" && n == 64 << 20) {
                let error = result.unwrap_err();
                assert_eq!(error.kind(), bennett_isa_ref::ErrorKind::HostResource);
                let e = error.to_string();
                assert!(e.contains("host limit") && e.contains(if family == "console" { "§7.3" } else { "§10" }), "{e}");
                assert!(e.contains(if n == 1 << 63 { "transfer bytes" } else { "console bytes" }), "wrong reservation stage: {e}");
                assert_eq!(m.snapshot(), prefix);
                assert!(m.outcome().request.is_none()); assert!(m.host_transcript().unwrap().is_empty());
                assert!(m.console().is_empty()); assert_eq!(m.effect_tape_len(), 0);
                eprintln!("{family}/{n}: resource at {}", if n == 1 << 63 { "first reservation" } else { "console retention after transfer" });
            } else {
                result.unwrap();
                eprintln!("{family}/{n}: transfer and history retained");
                if family != "console" {
                    let outcome = m.outcome();
                    let request = outcome.request.unwrap();
                    assert_eq!(request.payload.len(), n as usize);
                    assert!(request.payload.iter().all(|b| *b == 0));
                    let suspended = m.snapshot();
                    let copied = suspended.clone();
                    assert_eq!(copied, suspended);
                    let transcript = m.host_transcript().unwrap();
                    assert_eq!(transcript.len(), 1);
                    assert_eq!(transcript[0].request, request);
                    if family == "reply" && n == 64 << 20 {
                        let reply = vec![0; 8 << 20];
                        let error = m.reply(&reply).unwrap_err();
                        assert_eq!(error.kind(), bennett_isa_ref::ErrorKind::HostResource);
                        let e = error.to_string();
                        assert!(e.contains("host limit") && e.contains("retained memory bytes"), "wrong later stage: {e}");
                        assert_eq!(m.snapshot(), suspended);
                        assert!(m.host_transcript().unwrap()[0].reply.is_empty());
                        eprintln!("reply/{n}: reply copy succeeded; overwritten-byte retention denied; unchanged suspended prefix");
                    }
                    m.reply(&[]).unwrap();
                } else {
                    assert_eq!(m.console().len(), n as usize);
                    assert_eq!(m.effect_tape()[0].bytes.len(), n as usize);
                    assert_eq!(m.snapshot(), m.snapshot().clone());
                }
            }
            m.reverse().unwrap(); assert_eq!(m.snapshot(), entry);
        }
    }).expect("no retention panic");
}
#[test]
fn r22_witness_pin() {
    assert_eq!(
        bennett_isa_ref::sha256_hex(WITNESS.as_bytes()),
        "df7f95dd72496eb446c39319d5ef1e6287ebed8fdd6b9a7f92eba31ba1431071"
    );
    let o = Object::parse(WITNESS).unwrap();
    o.check_hashes().unwrap();
    let id = o.identity().unwrap();
    assert_eq!(
        id.interface_hashes["f"],
        "655e72211b66a9e4e3f8dcaff74c5203da6ec1d834fc43570833f0ab1387a577"
    );
    assert_eq!(
        id.envelopes["f"],
        include_str!("fixtures/r22/hostcall-64MiB.envelope")
    );
}

#[test]
fn r22_json_later_serialization_is_fallible() {
    guarded("json");
}
#[test]
fn r22_shared_payload_clones() {
    guarded("cow");
}
#[test]
fn r22_nonzero_reply_snapshot_and_reversal() {
    guarded("nonzero");
}
#[test]
fn r22_cli_exact_witness_suspends_without_abort() {
    let out = Command::new("python3")
        .args(["-c", "import os,resource,sys; resource.setrlimit(resource.RLIMIT_AS,(128*1024*1024,)*2); resource.setrlimit(resource.RLIMIT_CORE,(0,0)); os.execv(sys.argv[1],sys.argv[1:])"])
        .env("MALLOC_ARENA_MAX", "1").arg(env!("CARGO_BIN_EXE_bennett-isa-ref"))
        .arg("run-cc").arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/r22/hostcall-64MiB.bisa"))
        .arg("f").output().unwrap();
    assert_eq!(out.status.code(), Some(1), "{out:?}");
    assert!(out.stdout.is_empty());
    assert!(
        String::from_utf8_lossy(&out.stderr).contains("§10: HOST_REQUEST"),
        "{out:?}"
    );
}
#[test]
fn r22_json_exact_escaping_and_order() {
    let value = bennett_isa_ref::json::Json::parse(
        r#"{"é":"\u0001\b\t\n\f\r\"\\","a":[null,true,12],"😀":"𝄞"}"#,
    )
    .unwrap();
    assert_eq!(
        value.try_canonical().unwrap(),
        "{\"a\":[null,true,12],\"é\":\"\\u0001\\b\\t\\n\\f\\r\\\"\\\\\",\"😀\":\"𝄞\"}"
    );
    assert_eq!(value.try_canonical().unwrap(), value.canonical());
}
