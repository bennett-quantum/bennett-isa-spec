//! Resource guards run in a fresh process: catch_unwind cannot catch allocator aborts.
mod support;
use bennett_isa_ref::{ClaimVerdict, ClassicalMachine, Object, QuantumRegisters, QuantumState};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::process::{Command, Output};

const CLAIM: &str = include_str!("fixtures/r21/domain-all.bisa");
const CC: &str = include_str!("fixtures/r21/cc-length.bisa");
const R: &str = include_str!("fixtures/r21/r-length.bisa");

fn guarded(command: &mut Command) -> Output {
    // exec, rather than fork a Rust multithreaded test runner. No unsafe/libc dependency.
    let mut child = Command::new("python3");
    child.args(["-c", "import os,resource,sys; resource.setrlimit(resource.RLIMIT_AS,(256*1024*1024,)*2); resource.setrlimit(resource.RLIMIT_CORE,(0,0)); os.execv(sys.argv[1],sys.argv[1:])"]);
    child.arg(command.get_program()).args(command.get_args());
    for (k, v) in command.get_envs() {
        if let Some(v) = v {
            child.env(k, v);
        }
    }
    child.output().unwrap()
}
fn family(name: &str) {
    let out = guarded(
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "r21_resource_child", "--nocapture"])
            .env("R21_FAMILY", name),
    );
    assert!(out.status.success(), "{name}: {out:?}");
    eprintln!("{}", String::from_utf8_lossy(&out.stderr));
}
fn resource<T: std::fmt::Debug>(r: bennett_isa_ref::Result<T>, section: &str) {
    let e = r.expect_err("host-resource error required").to_string();
    assert!(e.contains(section) && e.contains("host limit"), "{e}");
}
fn claim(text: &str, small: bool) {
    let o = Object::parse(text).unwrap();
    o.check_hashes().unwrap();
    let h = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
    let r = catch_unwind(|| o.verify_claim(&h, 0)).expect("no claim panic");
    if small {
        assert_eq!(r.unwrap(), ClaimVerdict::Evidence { elements: 1 });
    } else {
        resource(r, "§12.2");
    }
    let r = catch_unwind(|| o.verify_certificate()).expect("no certificate panic");
    if small {
        assert_eq!(
            r.unwrap()[0].verdict,
            ClaimVerdict::Evidence { elements: 1 }
        );
    } else {
        resource(r, "§12.2");
    }
}
fn region_claim(a: u64, explicit: bool) -> String {
    let old = Object::parse(CLAIM).unwrap().identity().unwrap();
    let (base, cert) = CLAIM.split_once("\nsection certificate\n").unwrap();
    let base = base
        .replace("\"A\":63", &format!("\"A\":{a}"))
        .replace("region[63,1]", &format!("region[{a},1]"));
    let o = support::rehash(&base);
    let id = o.identity().unwrap();
    let mut cert = cert
        .replace("\"A\":63", &format!("\"A\":{a}"))
        .replace(&old.interface_hashes["a"], &id.interface_hashes["a"])
        .replace(&old.object_hash, &id.object_hash);
    if explicit {
        cert = cert.replace(
            "\"cells\":\"all\"",
            "\"cells\":[{\"hi\":1,\"index\":0,\"lo\":0}]",
        );
    }
    format!("{}\nsection certificate\n{cert}", o.text)
}
fn transfer(text: &str, len: u64, host: bool) {
    let heap = len.max(1);
    let meta = if heap <= 9_007_199_254_740_991 {
        heap.to_string()
    } else {
        format!("\"{heap}\"")
    };
    let text = text
        .replace("\"9223372036854775808\"", &meta)
        .replace("const.i64 9223372036854775808", &format!("const.i64 {len}"));
    let o = if len == 1 << 63 {
        Object::parse(&text).unwrap()
    } else {
        support::rehash(&text)
    };
    o.check_hashes().unwrap();
    let mut m = ClassicalMachine::new(&o, "f", &[], &[]).unwrap();
    let entry = m.snapshot();
    m.step_once().unwrap();
    m.step_once().unwrap(); // alloc stays sparse and legal at every tested length.
    let prefix = m.snapshot();
    let digest = m.memory_digest();
    assert_eq!(digest.len(), 64); // digest iterates resident bytes, never allocation size.
    let result = catch_unwind(AssertUnwindSafe(|| m.step_once())).expect("no transfer panic");
    if len > 8 {
        resource(result, if host { "§10" } else { "§7.3" });
        assert_eq!(
            m.snapshot(),
            prefix,
            "failed transfer must restore the entire prefix"
        );
        assert!(m.host_transcript().is_empty());
        assert!(m.console().is_empty());
        assert_eq!(m.effect_tape_len(), 0);
        resource(m.memory_bytes(1, len as usize), "§7.1");
    } else {
        result.unwrap();
        assert_eq!(
            m.memory_bytes(1, len as usize).unwrap(),
            vec![0; len as usize]
        );
        if host {
            assert_eq!(m.outcome().request.unwrap().payload, vec![0; len as usize]);
            m.reply(&[]).unwrap();
        } else {
            assert_eq!(m.console(), vec![0; len as usize]);
            assert_eq!(m.effect_tape_len(), 1);
        }
    }
    m.reverse().unwrap();
    assert_eq!(m.snapshot(), entry);
}
#[test]
fn r21_resource_child() {
    let Ok(name) = std::env::var("R21_FAMILY") else {
        return;
    };
    match name.as_str() {
        "claims" => {
            for (text, small) in [
                (CLAIM, false),
                (include_str!("fixtures/r21/domain-all-A0.bisa"), true),
                (include_str!("fixtures/r21/domain-all-A1.bisa"), true),
                (include_str!("fixtures/r21/domain-all-A64.bisa"), false),
                (
                    include_str!("fixtures/r21/sol-claim-region-all-sampled.bisa"),
                    false,
                ),
                (
                    include_str!("fixtures/r21/sol-claim-region-all-sampled-small.bisa"),
                    true,
                ),
            ] {
                claim(text, small);
            }
            for a in [0, 1, 3, 62, 63, 64, 127, 128] {
                for explicit in [false, true] {
                    claim(&region_claim(a, explicit), a < 4);
                }
            }
            eprintln!("R21 executable region domains: 2 paths x 8 exponents x 2 public consumers = 32 cases; 13 pinned controls");
            let o = Object::parse(include_str!("fixtures/r21/sol-claim-region-all-small.bisa"))
                .unwrap();
            o.check_hashes().unwrap();
            assert_eq!(
                o.verify_certificate().unwrap()[0].verdict,
                ClaimVerdict::Proof
            );
        }
        "hostcall" | "console" => {
            let host = name == "hostcall";
            for len in [0, 1, 8, (1 << 63) - 1, 1 << 63, u64::MAX] {
                transfer(if host { CC } else { R }, len, host);
            }
            eprintln!("R21 {name}: 6 transfer, 6 memory-query, 6 sparse digest/trace, 6 reversal controls");
        }
        "entropy" => {
            for len in [0, 1, 8, (1u64 << 63) - 1, 1 << 63, u64::MAX] {
                let text = R
                    .replace("console", "entropy")
                    .replace("entropy.write", "entropy.read")
                    .replace("const.i64 9223372036854775808", &format!("const.i64 {len}"))
                    .replace(
                        "\"heap_limit_max\":\"9223372036854775808\"",
                        "\"heap_limit_max\":\"18446744073709551615\"",
                    );
                let o = support::rehash(&text);
                o.check_hashes().unwrap();
                let mut m = ClassicalMachine::new(&o, "f", &[], &[1; 8]).unwrap();
                let entry = m.snapshot();
                let out = catch_unwind(AssertUnwindSafe(|| m.run(10)))
                    .expect("no entropy length panic")
                    .unwrap();
                assert_eq!(out.fault, if len <= 8 { None } else { Some(10) });
                assert_eq!(m.effect_tape_len(), if len <= 8 { 1 } else { 0 });
                m.reverse().unwrap();
                assert_eq!(m.snapshot(), entry);
            }
        }
        "quantum" => {
            for n in [0, 1, 8, (1usize << 63) - 1, 1 << 63, usize::MAX] {
                let mut q = QuantumRegisters::new();
                let r = catch_unwind(AssertUnwindSafe(|| q.allocate(n))).expect("no qreg panic");
                if n <= 8 {
                    assert_eq!(q.bundle(r.unwrap()).unwrap().len(), n);
                } else {
                    resource(r, "§4.3");
                }
                let mut q = QuantumRegisters::new();
                let id = q.allocate(8).unwrap();
                let slice =
                    catch_unwind(AssertUnwindSafe(|| q.slice(id, 0, n))).expect("no slice panic");
                if n <= 8 {
                    assert_eq!(q.bundle(slice.unwrap()).unwrap().len(), n);
                } else {
                    let e = slice.unwrap_err().to_string();
                    assert!(e.contains("§3.3") && !e.contains("host limit"), "{e}");
                }
            }
            let mut q = QuantumRegisters::new();
            let a = q.allocate(3).unwrap();
            let b = q.allocate(5).unwrap();
            let c = q.concat(a, b).unwrap();
            assert_eq!(q.bundle(c).unwrap().len(), 8);
            eprintln!(
                "R21 qreg allocation/slice/dense amplitudes: 3 paths / 17 cases; concat control"
            );
            for exponent in [1, 3, 62, 63, 64] {
                let mut q = QuantumState::new();
                q.allocate(exponent).unwrap();
                let r = catch_unwind(|| q.amplitudes()).expect("no dense panic");
                if exponent < 4 {
                    assert_eq!(r.unwrap().len(), 1 << exponent);
                } else {
                    resource(r, "§4.3");
                }
            }
        }
        _ => panic!("unknown family {name}"),
    }
}

#[test]
fn r21_cli_cardinality_and_sparse_output() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
    std::fs::create_dir_all(&dir).unwrap();
    for a in [0, 1, 3, 62, 63, 64, 127, 128] {
        let text = region_claim(a, false);
        let path = dir.join(format!("r21-cli-region-{a}.bisa"));
        std::fs::write(&path, text).unwrap();
        let out = guarded(
            Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                .arg("run-action")
                .arg(&path)
                .args(["a", "--mut", "0"]),
        );
        if a < 4 {
            assert!(out.status.success(), "{out:?}");
        } else {
            assert_eq!(out.status.code(), Some(1), "{out:?}");
            assert!(out.stdout.is_empty());
            assert!(
                String::from_utf8_lossy(&out.stderr).contains("host limit"),
                "{out:?}"
            );
        }
    }
    for len in [0, 1, 8, (1u64 << 63) - 1, 1 << 63, u64::MAX] {
        let text = CC
            .replace("r = hostcall org.bennett.test p n\nret r", "ret n")
            .replace("const.i64 9223372036854775808", &format!("const.i64 {len}"))
            // Full pointer range remains available independent of the chosen n.
            .replace(
                "\"heap_limit_max\":\"9223372036854775808\"",
                "\"heap_limit_max\":\"18446744073709551615\"",
            );
        let o = support::rehash(&text);
        let path = dir.join(format!("r21-cli-sparse-{len}.bisa"));
        std::fs::write(&path, o.text).unwrap();
        let out = guarded(
            Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                .arg("run-cc")
                .arg(&path)
                .arg("f"),
        );
        assert!(out.status.success(), "{out:?}");
        assert!(
            out.stdout.len() < 8192,
            "sparse memory trace must stay sparse"
        );
    }
    eprintln!("R21 CLI region cells / sparse digest+trace+output: 2 paths / 14 cases");
    let inc = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/inc8.bisa");
    for count in [0, 1, 8, (1u64 << 63) - 1, 1 << 63, u64::MAX] {
        // The replay odometer stays lazy. A downstream frame reservation stops
        // the first element; this does not claim completion of a huge replay.
        let out = guarded(
            Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                .arg("step-check")
                .arg(&inc)
                .args([
                    "inc8",
                    "--samples",
                    &count.to_string(),
                    "--budget",
                    "18446744073709551615",
                ]),
        );
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert!(out.stdout.is_empty());
        let e = String::from_utf8_lossy(&out.stderr);
        assert!(e.contains('§'), "{e}");
        assert_eq!(e.contains("host limit"), count != 0, "{e}");
    }
    for count in [1, 8] {
        let out = guarded(
            Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                .arg("step-check")
                .arg(&inc)
                .args(["inc8", "--samples", &count.to_string(), "--budget", "3"]),
        );
        assert!(out.status.success(), "{out:?}");
    }
    eprintln!("R21 CLI lazy sample count: 1 path / 6 cases; 2 completed small controls");
}

#[test]
fn r21_independent_witness_pins() {
    for (text, hash, envelope) in [
        (
            CC,
            "23802c541101025a57a13ffb5f00ca72a7ad5ff95f78e9a4fd565c59f566752d",
            include_str!("fixtures/r21/cc-length.envelope"),
        ),
        (
            R,
            "2751a25e6e906ea512bc075da07cda7463161576c56cebf06455ba2d9dfab704",
            include_str!("fixtures/r21/r-length.envelope"),
        ),
    ] {
        let o = Object::parse(text).unwrap();
        o.check_hashes().unwrap();
        let id = o.identity().unwrap();
        assert_eq!(bennett_isa_ref::sha256_hex(envelope.as_bytes()), hash);
        assert_eq!(id.envelopes["f"], envelope);
        assert_eq!(id.interface_hashes["f"], hash);
        assert_eq!(id.closure_hashes["f"].as_deref(), Some(hash));
    }
    let id = Object::parse(CLAIM).unwrap().identity().unwrap();
    assert_eq!(
        id.interface_hashes["a"],
        "f30725fa31cc35b68090b82cfd046681c4342821591cd2d6dd5300ea522499af"
    );
    assert_eq!(
        id.object_hash,
        "5fa6df63e007ad2bca8ab69c5a0c82e7671c585982048cf99919e59cc48a1e43"
    );
    assert_eq!(
        bennett_isa_ref::sha256_hex(include_bytes!(
            "fixtures/r21/sol-claim-region-all-sampled.bisa"
        )),
        "dfca89aa2073705841586a1c4a76e27be92618417476e9cb3b9016c0e09ab3c4"
    );
}
#[test]
fn r21_b01_claims_public_consumers() {
    family("claims");
}
#[test]
fn r21_b02_hostcall_public_consumer() {
    family("hostcall");
}
#[test]
fn r21_b02_console_public_consumer() {
    family("console");
}
#[test]
fn r21_entropy_transfer_length_fault_separation() {
    family("entropy");
}
#[test]
fn r21_quantum_cardinality_and_lengths() {
    family("quantum");
}

#[test]
fn r21_b02_cli_pinned_transfers() {
    for file in ["cc-length.bisa", "r-length.bisa"] {
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/r21")
            .join(file);
        let out = guarded(
            Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                .arg("run-cc")
                .arg(path)
                .arg("f"),
        );
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert!(out.stdout.is_empty());
        let e = String::from_utf8_lossy(&out.stderr);
        assert!(
            e.contains("host limit")
                && e.contains(if file.starts_with("cc") {
                    "§10"
                } else {
                    "§7.3"
                }),
            "{e}"
        );
    }
}
