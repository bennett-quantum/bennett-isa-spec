mod support;
use bennett_isa_ref::{ClassicalMachine, ClassicalValue, ErrorKind, Object};
use std::process::Command;

#[test]
fn r23_store_cow_retry() {
    for limit in [128, 256] {
        for seat in ["sol", "codex"] {
            let out = Command::new("python3")
                .args(["-c", "import os,resource,sys; n=int(sys.argv[1])*1024*1024; resource.setrlimit(resource.RLIMIT_AS,(n,256*1024*1024)); resource.setrlimit(resource.RLIMIT_CORE,(0,0)); os.execv(sys.argv[2],sys.argv[2:])"])
                .arg(limit.to_string()).arg(std::env::current_exe().unwrap())
                .args(["--exact", "r23_store_child", "--nocapture"])
                .env("R23_SEAT", seat).env("R23_LIMIT", limit.to_string())
                .env("MALLOC_ARENA_MAX", "1").output().unwrap();
            assert!(out.status.success(), "{seat}/{limit}: {out:?}");
            eprintln!("{}", String::from_utf8_lossy(&out.stderr));
        }
    }
}
#[test]
fn r23_store_child() {
    let Ok(seat) = std::env::var("R23_SEAT") else {
        return;
    };
    let text = if seat == "sol" {
        include_str!("fixtures/r23/sol-store-cow.bisa")
    } else {
        include_str!("fixtures/r23/codex-store-retry.bisa")
    };
    for small in [true, false] {
        let o = if small {
            support::rehash(&text.replace(if seat == "sol" { "1572864" } else { "1048576" }, "8"))
        } else {
            Object::parse(text).unwrap()
        };
        o.check_hashes().unwrap();
        let mut m = ClassicalMachine::new(&o, "f", &[], &[]).unwrap();
        let entry = m.snapshot();
        m.run(3).unwrap();
        let n = m.outcome().request.unwrap().capacity as usize;
        m.reply(&vec![if seat == "sol" { 0xa5 } else { 1 }; n])
            .unwrap();
        m.step_once().unwrap();
        let before = m.snapshot();
        let frames = m.outcome().steps;
        assert_eq!(frames, 4);
        match m.step_once() {
            Err(e) => {
                assert_eq!(e.kind(), ErrorKind::HostResource);
                assert_eq!(
                    m.snapshot(),
                    before,
                    "{}",
                    before.first_difference(&m.snapshot()).unwrap_or_default()
                );
                assert_eq!(m.outcome().steps, frames);
                assert_eq!(m.step_once().unwrap_err(), e);
                assert_eq!(m.snapshot(), before);
                assert_eq!(m.outcome().steps, frames);
                // Raise only the soft limit in this same process, preserving the rejected prefix.
                unsafe {
                    assert_eq!(
                        setrlimit(
                            9,
                            &Limit {
                                soft: 256 * 1024 * 1024,
                                hard: 256 * 1024 * 1024
                            }
                        ),
                        0
                    );
                }
                assert_eq!(m.step_once().unwrap().unwrap().instruction.op, "store.i8");
            }
            Ok(p) => assert_eq!(p.unwrap().instruction.op, "store.i8"),
        }
        assert_eq!(m.outcome().steps, frames + 1);
        assert_eq!(
            m.memory_bytes(1, 1).unwrap(),
            [if seat == "sol" { 0 } else { 2 }]
        );
        let result = m.run(4).unwrap();
        if seat == "codex" {
            assert_eq!(result.values[0], ClassicalValue::Int(64, 2));
        }
        m.reverse().unwrap();
        assert_eq!(m.snapshot(), entry);
        eprintln!("{seat}/{n}: exact prefix on denial, same-store retry, write/frame/result and reversal pass");
    }
}
#[repr(C)]
struct Limit {
    soft: u64,
    hard: u64,
}
unsafe extern "C" {
    fn setrlimit(resource: i32, limit: *const Limit) -> i32;
}
#[test]
fn r23_literal_envelopes() {
    for (text, envelope) in [
        (
            include_str!("fixtures/r23/sol-store-cow.bisa"),
            include_str!("fixtures/r23/sol-store-cow.envelope"),
        ),
        (
            include_str!("fixtures/r23/codex-store-retry.bisa"),
            include_str!("fixtures/r23/codex-store-retry.envelope"),
        ),
    ] {
        let o = Object::parse(text).unwrap();
        o.check_hashes().unwrap();
        let id = o.identity().unwrap();
        assert_eq!(id.envelopes["f"], envelope);
        assert_eq!(
            id.interface_hashes["f"],
            bennett_isa_ref::sha256_hex(envelope.as_bytes())
        );
    }
}
