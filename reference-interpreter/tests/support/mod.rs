#![allow(dead_code)]
use bennett_isa_ref::{json::Json, Object};
use std::collections::BTreeMap;

pub fn json(s: &str) -> Json {
    Json::parse(s).unwrap()
}
pub fn object(functions: &[(&str, &str, &str)], imports: &str, extra: &str) -> String {
    let symbols = functions
        .iter()
        .enumerate()
        .map(|(i, (name, signature, _))| {
            let sig = json(signature);
            let value = Json::Object(BTreeMap::from([
                ("name".into(), Json::String((*name).into())),
                ("kind".into(), sig.get("kind").unwrap().clone()),
                ("signature".into(), sig),
                ("interface_hash".into(), Json::String("0".repeat(64))),
            ]));
            (i.to_string(), value)
        })
        .collect();
    let mut text=format!("bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{}\n",Json::Object(symbols).canonical());
    if imports != "{}" {
        text.push_str(&format!(
            "\nsection imports\n{}\n",
            json(imports).canonical()
        ));
    }
    text.push_str("\nsection functions\n");
    for (_, _, body) in functions {
        text.push_str(body.trim_end_matches('\n'));
        text.push('\n');
    }
    text.push_str(extra);
    text
}
pub fn rehash(text: &str) -> Object {
    let parsed = Object::parse(text).unwrap();
    let hashes = parsed.identity().unwrap().interface_hashes;
    let mut lines: Vec<_> = text.lines().map(str::to_string).collect();
    let i = lines.iter().position(|s| s == "section symbols").unwrap() + 1;
    let Json::Object(mut symbols) = json(&lines[i]) else {
        unreachable!()
    };
    for symbol in symbols.values_mut() {
        let name = symbol.get("name").unwrap().string().unwrap().to_string();
        let Json::Object(value) = symbol else {
            unreachable!()
        };
        value.insert("interface_hash".into(), Json::String(hashes[&name].clone()));
    }
    lines[i] = Json::Object(symbols).canonical();
    Object::parse(&(lines.join("\n") + "\n")).unwrap()
}
/// An object with functions, data symbols (name, hex) and trailing sections
/// (layout and later, each starting with "\nsection ..."). Hashes are zero;
/// pass the text through `rehash` to obtain canonical identities.
pub fn object_with_data(
    functions: &[(&str, &str, &str)],
    data: &[(&str, &str)],
    tail: &str,
) -> String {
    let mut symbols = BTreeMap::new();
    for (i, (name, signature, _)) in functions.iter().enumerate() {
        let sig = json(signature);
        symbols.insert(
            i.to_string(),
            Json::Object(BTreeMap::from([
                ("name".into(), Json::String((*name).into())),
                ("kind".into(), sig.get("kind").unwrap().clone()),
                ("signature".into(), sig),
                ("interface_hash".into(), Json::String("0".repeat(64))),
            ])),
        );
    }
    for (j, (name, hex)) in data.iter().enumerate() {
        let sig = json(&format!(
            r#"{{"abi":"0.8","kind":"data","size":{},"tier":"-"}}"#,
            hex.len() / 2
        ));
        symbols.insert(
            (functions.len() + j).to_string(),
            Json::Object(BTreeMap::from([
                ("name".into(), Json::String((*name).into())),
                ("kind".into(), Json::String("data".into())),
                ("signature".into(), sig),
                ("interface_hash".into(), Json::String("0".repeat(64))),
            ])),
        );
    }
    let mut text = format!(
        "bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{}\n\nsection functions\n",
        Json::Object(symbols).canonical()
    );
    for (_, _, body) in functions {
        text.push_str(body.trim_end_matches('\n'));
        text.push('\n');
    }
    if !data.is_empty() {
        text.push_str("\nsection data\n");
        for (name, hex) in data {
            text.push_str(&format!("{name} {hex}\n"));
        }
    }
    text.push_str(tail);
    text
}
pub const ACTION_1_1: &str =
    r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;

/// Invalid objects must fail before producing any CLI data.
pub fn reject_cli(text: &str, rule: &str, tag: &str) {
    let e = Object::parse(text).expect_err("malformed object accepted");
    assert!(e.to_string().contains(rule), "{e}");
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".tmp");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join(format!("r3-{tag}.bisa"));
    std::fs::write(&path, text).unwrap();
    for cmd in ["hash", "check"] {
        let out = std::process::Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
            .arg(cmd)
            .arg(&path)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1), "{out:?}");
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains(rule));
    }
}

/// Memoryless round-4 borrow fixtures, with exactly the advertised loop bounds.
pub fn borrow_lifetime_object(body: &str, bounds: &[u64]) -> String {
    let sig = format!(
        r#"{{"abi":"0.8","args":[],"bounds":{bounds:?},"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}}"#
    );
    let regs = if bounds.is_empty() {
        String::new()
    } else {
        format!(
            "regs {{  }} bounds {{{}}}\n",
            bounds
                .iter()
                .map(u64::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    object(
        &[(
            "a",
            &sig,
            &format!("perm_action @a (t:wire mut) () () () ()\n{regs}^entry():\n{body}"),
        )],
        "{}",
        "",
    )
}

/// Identity on all t,b basis entries, a superposition, and b entangled with an
/// untouched reference proves the controls do not merely hide a pool mutation.
pub fn borrow_identity(o: &Object) {
    use bennett_isa_ref::{QuantumBindings, QuantumState};
    for input in 0..6u32 {
        let mut q = QuantumState::new();
        let w = q.allocate(3).unwrap();
        if input == 5 {
            q.gate("h", &[w[1]], None, &[]).unwrap();
            q.gate("cx", &[w[1], w[2]], None, &[]).unwrap();
            q.gate("h", &[w[0]], None, &[]).unwrap();
        } else {
            for (bit, wire) in w[..2].iter().enumerate() {
                if input == 4 {
                    q.gate("h", &[*wire], None, &[]).unwrap();
                } else if input >> bit & 1 == 1 {
                    q.gate("x", &[*wire], None, &[]).unwrap();
                }
            }
        }
        let entry = q.clone();
        let mut b = QuantumBindings::default();
        b.bundles.insert("t".into(), vec![w[0]]);
        b.borrowed.push(vec![w[1]]);
        for adjoint in [false, true] {
            o.execute_action("a", &mut q, &b, adjoint, &[]).unwrap();
            assert_eq!(q, entry, "input={input}, adjoint={adjoint}");
        }
    }
}

/// R4-B02: select dynamically chooses q itself (c=1) or distinct r (c=0).
/// Both handles have the same static type; the precondition is physical.
pub fn dynamic_region_alias_object(kind: &str, tier: &str) -> Object {
    let ac = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let xc = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let os =
        r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let (sig, child, call, extra_types, extra_params) = match kind {
        "interface" => (
            ac,
            "action @a (t:wire mut) (s:wire in) () () ()\n^entry():\ncx s t",
            "action_call @a (q) (s) () ()",
            "",
            "",
        ),
        "control" => (
            xc,
            "action @a (t:wire mut) () () () ()\n^entry():\nx t",
            "ctl s action_call @a (q) () () ()",
            "",
            "",
        ),
        "readonly-control" => (
            ac,
            "action @a (t:wire mut) (s:wire in) () () ()\n^entry():\ncx s t",
            "ctl s action_call @a (q) (s) () ()",
            "",
            "",
        ),
        "oracle" => (
            os,
            "oracle @a (s:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval s",
            "oracle_call @a tk (s) q f () ()",
            ",\"qreg[4]\",\"token\"",
            ", f:qreg[4], tk:token",
        ),
        _ => panic!("unknown fixture family {kind}"),
    };
    let ps = format!(
        r#"{{"abi":"0.8","kind":"{tier}","params":["i1","qreg[1]","qreg[1]"{extra_types}],"results":[],"tier":"{tier}"}}"#
    );
    let parent = format!("fn {tier} @p (c:i1, q:qreg[1], r:qreg[1]{extra_params}) () () () ()\n^entry():\ns = select c q r\n{call}\nret");
    rehash(&object(&[("a", sig, child), ("p", &ps, &parent)], "{}", ""))
}

/// Independently hash a memoryless subject and attach its complete four-row
/// interface/pool identity claim. Never parse an uncertified subject here.
pub fn certified_borrow_identity(body: &str, bounds: &[u64]) -> String {
    certified_borrow_identity_kind(body, bounds, "perm_action")
}
pub fn certified_borrow_identity_kind(body: &str, bounds: &[u64], kind: &str) -> String {
    let sig = format!(
        r#"{{"abi":"0.8","args":[],"bounds":{bounds:?},"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}}"#
    );
    let sig = json(&sig.replace("perm_action", kind)).canonical();
    let raw = borrow_lifetime_object(body, bounds).replace("perm_action", kind);
    let function = raw.split("section functions\n").nth(1).unwrap();
    let envelope = format!("isa=0.8\nptr_width=32\n{sig}\n{function}");
    let h = bennett_isa_ref::sha256_hex(envelope.as_bytes());
    let text = raw.replace(&"0".repeat(64), &h);
    let object_hash = bennett_isa_ref::sha256_hex(text.as_bytes());
    let cert = json(&format!(
        r#"{{"versions":{{"isa":"0.8","object":"0.8","launch":"0.8","sched":"0.8","cert":"0.8","coverage":"0.8","target":"0.8"}},"bindings":{{"object_hash":"{object_hash}","interface_hash":"{h}","closure_hash":"{h}","reference_program_hash":"{h}","dialect_mapping_version":"test.1","target_hash":"{}"}},"producer":{{"name":"tests","version":"1","materialisation":{{"llvm_version":"none","pipeline":"none","normaliser_version":"1"}}}},"regions":{{"{h}":{{"kind":"{kind}","signature":{sig},"bounds":{bounds:?},"frame":{{"interface":[{{"role":"mut","width":1,"class":"any"}}],"scratch":{{}},"retained_history":0,"borrow":{{"any":1}},"scheme_ancillae":{{}}}},"resources":{{}},"gates_used":{{}},"claims":[{{"kind":"exhaustive","label":"proof","domain":{{"bundles":[{{"name":"t","width":1,"lo":0,"hi":1}}],"borrowed":[{{"name":"b","width":1,"lo":0,"hi":1}}],"regions":[]}},"expected":{{"table":[[0,0],[0,1],[1,0],[1,1]]}}}}],"verification":{{"oracle":null,"borrow_frames":[0],"inverse_pairs":[],"unitary":[],"channel":[],"fault_rate":{{"domain":{{"bundles":[],"borrowed":[],"regions":[]}},"faults_by_code":{{}},"undefined_excluded":0,"layout_dependent_excluded":0}}}},"cleanup":{{}},"error":{{}},"definedness_assumptions":[]}}}},"composition":{{"multiplicities":{{}},"epsilon_total":{{"diamond":0}},"method":"sum_over_multiplicities"}}}}"#,
        "3".repeat(64)
    ));
    format!("{text}\nsection certificate\n{}\n", cert.canonical())
}

/// Literal round-6 subjects and independently published interface/object pins.
pub fn r6_witness(which: &str) -> (String, &'static str, &'static str) {
    let (body, bounds, kind, h, oh) = match which {
        "sol-boundary" => (
            "c = qalloc any 1\nh t\nb = qborrow any 1\ncx c b\nqreturn b\nh t\nqfree c",
            vec![],
            "action",
            "ba813954a842962eecd96263d6dbe2f549f7f8d4c99241c660f0a3c7626cf10a",
            "db747f53ae732bd2cef0eeeaed48b8bb2a523dc74f53b38d2876601a3760445f",
        ),
        "codex-boundary" => (
            "c = qalloc any 1\nh t\nb = qborrow any 1\nqreturn b\nqfree c",
            vec![],
            "action",
            "6c7101e365eccc827653aa5dc596d68fefae112b9e4b6df00b5078f5639bf7d3",
            "e5517c39c82f32f9c006828c6b9535d899dad9c0a7b59339ba9fabea3672fc3f",
        ),
        "zero-trip" => (
            "d = qalloc any 1\nqloop 0 {\nqfree d\n}\nqfree d",
            vec![0],
            "perm_action",
            "9a453b6720abda445235294c98c6288e616dc5bc1907c78af1755104b3a2e48a",
            "1e9a003077d2afd78e6f14384902f031c6aa140bebaf14702889de13d4cd814a",
        ),
        "changing-width" => (
            "d = qalloc any 1\nqloop 2 {\nqfree d\nd = qalloc any 2\nx d[1]\nx d[1]\n}\nqfree d",
            vec![2],
            "perm_action",
            "de76a9295cf8233d97909660d48c9f6ed82abe593735257858457d35a0c0557e",
            "128fa2a938dfbdcccdd255aafd128d125737a4a00fb63e9aef791a1f405bfb1a",
        ),
        _ => panic!("unknown witness"),
    };
    (
        borrow_lifetime_object(body, &bounds)
            .replace("perm_action", kind)
            .replace(&"0".repeat(64), h),
        h,
        oh,
    )
}

/// Full H_t tensor I_bR map, including basis, superposed and entangled inputs.
/// Each direction starts independently at entry; workspace cannot escape.
pub fn r6_map(o: &Object, hadamard: bool) {
    use bennett_isa_ref::{QuantumBindings, QuantumState};
    for input in 0..18 {
        let mut q = QuantumState::new();
        let w = q.allocate(5).unwrap();
        for (bit, wire) in w[..4].iter().enumerate() {
            if input >= 16 {
                q.gate("h", &[*wire], None, &[]).unwrap();
            } else if input >> bit & 1 == 1 {
                q.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        if input == 17 {
            q.gate("cx", &[w[1], w[4]], None, &[]).unwrap();
        }
        let entry = q.clone();
        let mut expected = q.clone();
        if hadamard {
            expected.gate("h", &[w[0]], None, &[]).unwrap();
        }
        let mut b = QuantumBindings::default();
        b.bundles.insert("t".into(), vec![w[0]]);
        b.borrowed.push(w[1..4].to_vec());
        for adjoint in [false, true] {
            let mut actual = entry.clone();
            o.execute_action("a", &mut actual, &b, adjoint, &[])
                .unwrap();
            assert_eq!(actual, expected, "input={input}, adjoint={adjoint}");
            o.execute_action("a", &mut actual, &b, !adjoint, &[])
                .unwrap();
            assert_eq!(actual, entry);
        }
    }
}

/// Literal round-7 member witnesses; hashes computed independently by the members.
pub fn r7_witness(case: &str) -> (String, &'static str, &'static str) {
    let (body, bounds, h, oh): (&str, &[u64], &str, &str) = match case {
        "restored" => ("c = qalloc any 1\nh c\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c", &[], "12efae749cb84eec0918c9aa49adb348957eba9e81e0eaa8f2514481a0d9dba0", "b2534762bf5eb795efd7ba74195198f0b67e2012b9f3243700e58ee92ee9fc53"),
        "loop-bad" => ("c = qalloc any 1\nx c\nqloop 4 {\nh t\nx c\n}\nb = qborrow any 1\ncx c b\nqreturn b\nh t\nx c\nqfree c", &[4], "232f157c4fe6c4abdb3ebc384d1b19b01751b68df4949b86d25996e87c300515", "3e8f134a472e3e31545e6bd3ec22058da463927ea24af3929bf3357a36ca3926"),
        "loop-good" => ("c = qalloc any 1\nqloop 4 {\nh t\nx c\n}\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c", &[4], "fe26a45f4d75fde7d13f0e915a13d15a6ab9bb6b280dcc302996df1391f899d9", "7f1312cd66fc88f5819be157a1a5964c27363f8f1bc728a9b21cdb342685b198"),
        "diagonal" => ("c = qalloc any 1\ns c\nb = qborrow any 1\ncx c b\nqreturn b\nsdg c\nqfree c", &[], "0d7a3eccbc3cdd7440ec3c1ad66a12e8650203486b2736f6fe5ea2ddb87a56f6", "5b407065c615539c15c7f9381b8fccfebc8e109cae3e8966c05fbd4acd687205"),
        "control" => ("c = qalloc any 1\nqpred c\nh t\nqunpred c\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c", &[], "06b70e4ca0799aac622c280b3f1de5b825888d21c19da5ec3f015c88b4fc1fe6", ""),
        "float" => ("qloop 5000 {\nh t\nt t\n}", &[5000], "2b29d4983fd4149572528e684e31ed7802cd7727f628c50058f7d62329ca22e9", "8af0b609daca8eb239a88ad3f47fddec0e6e43e9a456bca951887ccd89af0080"),
        "rotate-4" => ("c = qalloc any 4\nx c[0]\nqloop 4 {\nh t\nb = qborrow any 1\ncx c[3] b\nqreturn b\nh t\nswap c[3] c[2]\nswap c[2] c[1]\nswap c[1] c[0]\n}\nqloop 4 {\nh t\nswap c[1] c[0]\nswap c[2] c[1]\nswap c[3] c[2]\nh t\n}\nx c[0]\nqfree c", &[4, 4], "5dcd11b58f335ac7f301caf4e2773ffbfb172a01330fdb6c7c990c15036b340f", "d58420852a90bad7d351d52072827bff5c6a8b592ae25072cffdf3b793e88de3"),
        "rotate-8" => ("c = qalloc any 4\nx c[0]\nqloop 8 {\nh t\nb = qborrow any 1\ncx c[3] b\nqreturn b\nh t\nswap c[3] c[2]\nswap c[2] c[1]\nswap c[1] c[0]\n}\nqloop 8 {\nh t\nswap c[1] c[0]\nswap c[2] c[1]\nswap c[3] c[2]\nh t\n}\nx c[0]\nqfree c", &[8, 8], "4036a594dfa32ab65353367719970549a2cbe1237e471ff6f43b169e09844112", "bc9ac8d642fe4c4569d81021da302108919d500fda82cde3577a2916b681068c"),
        _ => panic!("unknown round-7 witness"),
    };
    let text = borrow_lifetime_object(body, bounds)
        .replace("perm_action", "action")
        .replace(&"0".repeat(64), h);
    // Bind even rejected witnesses to their exact published stored-H bytes.
    if !oh.is_empty() {
        assert_eq!(bennett_isa_ref::sha256_hex(text.as_bytes()), oh);
    }
    (text, h, oh)
}

/// Exact round-8 member objects. Pins are independently supplied council hashes.
pub fn r8_witness(case: &str) -> (String, &'static str, &'static str) {
    let (body, bounds, h, oh): (&str, &[u64], &str, &str) = match case {
        "nested-x" => ("c = qalloc any 1\nh c\nx c\nx c\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c", &[], "0fd6d6eadcb219457ce3d9aa091a2176d38ce9f81551235799dae7cabe6e0e2f", "5eff1283a8f71dcce612e1802e510fdcae30c609395686311962bd630f01461c"),
        "nested-s" => ("c = qalloc any 1\nh c\ns c\nsdg c\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c", &[], "a8a4f90c95a1995df9a28afe4369f3455239ae54b6c22b5c8133df7a60253401", "535f274d3c7328e13cfd99e8785751ec3874e4d3090da3544e781855ca35439f"),
        "zero-wide" => ("c = qalloc any 1\nqloop 0 {\nqfree c\nc = qalloc any 2\n}\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c", &[0], "c9cbc01c190e4454e37b553e1d7e8e9eeea856631a6ab397029577447b053b87", "708531c08cb63447ea2bbaa966c69562300ab1bc89e07efb9c3278799c54922d"),
        "zero-same" => ("c = qalloc any 1\nqloop 0 {\nqfree c\nc = qalloc any 1\n}\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c", &[0], "d607c1df25e3a4d8f4e730501cf13a04a62cfa6adbeae5c36133a4f140f406bd", "568f2a4ee6c4cb927cafe4bd54414def5e91257186832704e6be7abfe99fd1bc"),
        "zero-h" => ("b = qborrow any 1\nqloop 0 {\nqreturn b\nb = qalloc any 1\n}\nh b\nqreturn b", &[0], "90cbbcb904bff3ae413e3aaa3ec398173654dc1690354777bcb3837881098080", "60865de88a18adc53e2ecb23ab54d86f44836e4895cb7fb54e3ac78c0c879280"),
        "zero-s" => ("b = qborrow any 1\nqloop 0 {\nqreturn b\nb = qalloc any 1\n}\ns b\nqreturn b", &[0], "1c0599e2f06674ccbc34f0017d5d158af1588d7d54982aa1adfebddc4262fa58", "fd5167fefabbbc72d9cd486e6cf036119f2ba5088558cccb155f898eaa7285ec"),
        "call" => ("c = qalloc any 1\nctl c action_call @u (t) () () ()\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c", &[], "63ae90bc9af75ebe3e958956282567f52129b0097651a445c08f7c5f2845ac79", "c4a3c1adad250d856a334556766f0cc398a71b6c65df3ce5f81adf86d07f968e"),
        _ => panic!("unknown round-8 witness"),
    };
    let text = if case == "call" {
        let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
        object(
            &[
                (
                    "u",
                    sig,
                    "action @u (v:wire mut) () () () ()\n^entry():\nh v",
                ),
                (
                    "a",
                    sig,
                    &format!("action @a (t:wire mut) () () () ()\n^entry():\n{body}"),
                ),
            ],
            "{}",
            "",
        )
        .replacen(
            &"0".repeat(64),
            "c0c51a4bf1e9713e237601daaf342134f91eae5fd7143f8638595157d33db5b6",
            1,
        )
    } else {
        borrow_lifetime_object(body, bounds).replace("perm_action", "action")
    };
    let text = text.replace(&"0".repeat(64), h);
    assert_eq!(bennett_isa_ref::sha256_hex(text.as_bytes()), oh);
    (text, h, oh)
}

/// Independent I_t tensor I_bR expectation; each direction starts at entry.
pub fn r9_identity(o: &Object, args: &[bennett_isa_ref::Argument]) {
    use bennett_isa_ref::{QuantumBindings, QuantumState};
    for input in 0..10 {
        let mut q = QuantumState::new();
        let w = q.allocate(3).unwrap();
        for (bit, wire) in w.iter().enumerate() {
            if input < 8 && input >> bit & 1 == 1 {
                q.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        if input >= 8 {
            q.gate("h", &[w[0]], None, &[]).unwrap();
            q.gate("h", &[w[1]], None, &[]).unwrap();
            if input == 9 {
                q.gate("cx", &[w[1], w[2]], None, &[]).unwrap();
            }
        }
        let entry = q;
        let mut bindings = QuantumBindings::default();
        bindings.bundles.insert("t".into(), vec![w[0]]);
        bindings.borrowed.push(vec![w[1]]);
        if let Some(arg) = args.first() {
            bindings.args.insert("tk".into(), arg.clone());
        }
        for adjoint in [false, true] {
            let mut actual = entry.clone();
            o.execute_action("a", &mut actual, &bindings, adjoint, &[])
                .unwrap();
            assert_eq!(actual, entry, "input={input} adjoint={adjoint}");
            o.execute_action("a", &mut actual, &bindings, !adjoint, &[])
                .unwrap();
            assert_eq!(actual, entry);
        }
    }
}
pub fn r9_fixture(case: &str) -> &'static str {
    let (text, hash) = match case {
        "sol-oracle" => (
            include_str!("../fixtures/r9-sol-oracle.bisa"),
            "ef010633dbc984abbaa9fbfb97c6233c5390fa99a17bc38e09725cddec936832",
        ),
        "sol-qinplace" => (
            include_str!("../fixtures/r9-sol-qinplace.bisa"),
            "77664ac9392aec50b0415027771ae7ef49dcea7762b70025f0cd918d53a6ee1e",
        ),
        "codex-oracle" => (
            include_str!("../fixtures/r9-codex-oracle.bisa"),
            "49ad3ba6c411ed73bebacdebe6728ebff5995dbf8bbc03c016d1cb0c5c3e8625",
        ),
        "loop" => (
            include_str!("../fixtures/r9-loop.bisa"),
            "80478acc73d52bbf64c395816470293b5b4b4a1ee36a5c550bf00436f06fc192",
        ),
        _ => panic!("unknown R9 fixture"),
    };
    assert_eq!(bennett_isa_ref::sha256_hex(text.as_bytes()), hash);
    text
}

pub fn r9_opaque_object(body: &str, bounds: &[u64], child: Option<&str>) -> String {
    let os =
        r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let ac = format!(
        r#"{{"abi":"0.8","args":["token"],"bounds":{:?},"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}}"#,
        bounds
    );
    let bound_text = bounds
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let regs = if bounds.is_empty() {
        String::new()
    } else {
        format!("regs {{  }} bounds {{{bound_text}}}\n")
    };
    let action = format!("action @a (t:wire mut) () () (tk:token) ()\n{regs}^entry():\n{body}");
    let mut functions = vec![(
        "o",
        os,
        "oracle @o (p:wire in) (v:wire) (fr:wire[4]) () ()\n^entry():\nlift.retval p",
    )];
    let child_sig = r#"{"abi":"0.8","args":["token"],"bounds":[],"in":[1],"kind":"action","mem":[],"mut":[1,4],"tier":"q"}"#;
    if let Some(child) = child {
        functions.push(("child", child_sig, child));
    }
    functions.push(("a", &ac, &action));
    object(&functions, "{}", "")
}

pub fn r10_case(tag: &str) -> Json {
    json(include_str!("r10_cases.json"))
        .get(tag)
        .unwrap()
        .clone()
}
pub fn r10_text(tag: &str) -> String {
    r10_case(tag).get("text").unwrap().string().unwrap().into()
}
pub fn r10_pin(tag: &str) -> Object {
    let c = r10_case(tag);
    let text = c.get("text").unwrap().string().unwrap();
    let o = Object::parse(text).unwrap();
    o.check_hashes().unwrap();
    let id = o.identity().unwrap();
    assert_eq!(
        id.object_hash,
        c.get("object_hash").unwrap().string().unwrap()
    );
    for (name, h) in c.get("hashes").unwrap().object().unwrap() {
        assert_eq!(id.interface_hashes[name], h.string().unwrap());
        assert_eq!(
            id.closure_hashes[name].as_deref(),
            Some(h.string().unwrap())
        );
    }
    o
}
/// Every forbidden prefix at both definition positions, including aliases of
/// otherwise valid minimal function bodies. No hash normalization is involved.
pub fn r10_bad_headers() -> Vec<String> {
    let mut cases = vec![];
    let cc = r10_text("cc");
    for kind in ["cc", "r", "oracle", "action", "perm_action", "lproc"] {
        let (sig, body, bad) = match kind {
            "cc" | "r" => {
                let text = r10_text(if kind == "cc" { "cc" } else { "r" });
                let o = Object::parse(&text).unwrap();
                (o.functions["a"].signature().canonical(),
                 text.split("section functions\n").nth(1).unwrap().to_string(),
                 kind.to_string())
            }
            "oracle" => (r#"{"abi":"0.8","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#.into(),
                include_str!("../fixtures/inc8.bisa").split("section functions\n").nth(1).unwrap().replace("@inc8", "@a"),
                "fn oracle".into()),
            "lproc" => (r#"{"abi":"0.8","kind":"lproc","params":[8],"result":8,"tier":"q"}"#.into(),
                "lproc @a (x:reg[8]) (r:reg[8]) () () ()\n^entry():\nlift.ret x\n".into(), "fn lproc".into()),
            _ => (format!(r#"{{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"{kind}","mem":[],"mut":[1],"tier":"q"}}"#),
                format!("{kind} @a (t:wire mut) () () () ()\n^entry():\nx t\n"), format!("fn {kind}")),
        };
        let first_space = body.find(" @").unwrap();
        let mutated = format!("{bad}{}", &body[first_space..]);
        cases.push(object(&[("a", &sig, &mutated)], "{}", ""));
        let cc_o = Object::parse(&cc).unwrap();
        let cc_sig = cc_o.functions["a"].signature().canonical();
        let cc_body = cc
            .split("section functions\n")
            .nth(1)
            .unwrap()
            .replace("@a", "@first");
        cases.push(object(
            &[("first", &cc_sig, &cc_body), ("a", &sig, &mutated)],
            "{}",
            "",
        ));
    }
    cases
}

/// Independent identity on t,p,q and the arbitrary borrowed b/reference R.
/// Each direction starts from the original state; equality includes allocator.
pub fn r10_identity(o: &Object, steps: u64) {
    use bennett_isa_ref::{Argument, Budget, QuantumBindings, QuantumState};
    for input in 0..34 {
        let mut q = QuantumState::new();
        let w = q.allocate(5).unwrap();
        if input < 32 {
            for (bit, wire) in w.iter().enumerate() {
                if input >> bit & 1 == 1 {
                    q.gate("x", &[*wire], None, &[]).unwrap();
                }
            }
        } else {
            for wire in &w[..4] {
                q.gate("h", &[*wire], None, &[]).unwrap();
            }
            if input == 33 {
                q.gate("cx", &[w[3], w[4]], None, &[]).unwrap();
            }
        }
        let mut bindings = QuantumBindings::default();
        for (name, wire) in [("t", w[0]), ("p", w[1]), ("q", w[2])] {
            bindings.bundles.insert(name.into(), vec![wire]);
        }
        bindings.borrowed.push(vec![w[3]]);
        bindings
            .args
            .insert("tk".into(), Argument::Token(Budget::steps(steps)));
        for adjoint in [false, true] {
            let mut actual = q.clone();
            o.execute_action("a", &mut actual, &bindings, adjoint, &[])
                .unwrap();
            assert_eq!(actual, q, "input={input} adjoint={adjoint} B={steps}");
            o.execute_action("a", &mut actual, &bindings, !adjoint, &[])
                .unwrap();
            assert_eq!(actual, q);
        }
    }
}

pub fn r10_context(stack: &[&str], op: &str) -> String {
    let push: String = stack.iter().map(|p| format!("qpred {p}\n")).collect();
    let pop: String = stack
        .iter()
        .rev()
        .map(|p| format!("qunpred {p}\n"))
        .collect();
    format!("{push}{op}\n{pop}")
}
pub const R10_ORACLE: &str = "oracle_call @o tk (c) d f () ()";
pub fn r10_object(middle: &str, bounds: &[u64], child: bool, inverse: bool) -> String {
    let pinned = r10_text("controls");
    let sigs = json(pinned.lines().nth(3).unwrap());
    let os = sigs.get("0").unwrap().get("signature").unwrap().canonical();
    let mut sig = sigs.get("1").unwrap().get("signature").unwrap().clone();
    if let Json::Object(map) = &mut sig {
        map.insert("bounds".into(), json(&format!("{bounds:?}")));
    }
    let oracle = "oracle @o (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x";
    let bound_text = bounds
        .iter()
        .map(u64::to_string)
        .collect::<Vec<_>>()
        .join(", ");
    let regs = if bounds.is_empty() {
        String::new()
    } else {
        format!("regs {{  }} bounds {{{bound_text}}}\n")
    };
    let action = format!("action @a (t:wire mut) (p:wire in, q:wire in) () (tk:token) ()\n{regs}^entry():\nc = qalloc any 1\nd = qalloc any 1\nf = qalloc any 4\nh c\n{middle}h c\nb = qborrow any 1\ncx c b\nqreturn b\nqfree f\nqfree d\nqfree c");
    let child_sig = r#"{"abi":"0.8","args":["token"],"bounds":[],"in":[1,1],"kind":"action","mem":[],"mut":[1,1,4],"tier":"q"}"#;
    let child_body = "action @child (c:wire mut, d:wire mut, f:wire[4] mut) (p:wire in, q:wire in) () (tk:token) ()\n^entry():\nqpred q\nqpred p\noracle_call @o tk (c) d f () ()\nqunpred p\nqunpred q";
    let oi = oracle.replace("@o ", "@oi ");
    let a_sig = sig.canonical();
    let mut funcs = vec![("o", os.as_str(), oracle)];
    if child {
        funcs.push(("child", child_sig, child_body));
    }
    if inverse {
        funcs.push(("oi", os.as_str(), oi.as_str()));
    }
    funcs.push(("a", a_sig.as_str(), action.as_str()));
    object(&funcs, "{}", "")
}

pub fn r11_case(tag: &str) -> Json {
    json(include_str!("r11_cases.json"))
        .get(tag)
        .unwrap()
        .clone()
}
pub fn r11_text(tag: &str) -> String {
    r11_case(tag).get("text").unwrap().string().unwrap().into()
}
pub fn r11_pin(tag: &str) -> Object {
    let c = r11_case(tag);
    let o = Object::parse(c.get("text").unwrap().string().unwrap()).unwrap();
    let id = o.identity().unwrap();
    assert_eq!(
        id.object_hash,
        c.get("object_hash").unwrap().string().unwrap()
    );
    for (name, hash) in c.get("hashes").unwrap().object().unwrap() {
        assert_eq!(id.interface_hashes[name], hash.string().unwrap());
        assert_eq!(
            id.closure_hashes[name].as_deref(),
            Some(hash.string().unwrap())
        );
        assert_eq!(
            id.envelopes[name],
            c.get("envelopes")
                .unwrap()
                .get(name)
                .unwrap()
                .string()
                .unwrap()
        );
    }
    o.check_hashes().unwrap();
    o
}

pub fn r12_case(tag: &str) -> Json {
    json(include_str!("r12_cases.json"))
        .get(tag)
        .unwrap()
        .clone()
}
pub fn r12_text(tag: &str) -> String {
    r12_case(tag).get("text").unwrap().string().unwrap().into()
}
pub fn r12_pin(tag: &str) -> Object {
    let c = r12_case(tag);
    let o = Object::parse(c.get("text").unwrap().string().unwrap()).unwrap();
    let id = o.identity().unwrap();
    assert_eq!(
        id.object_hash,
        c.get("object_hash").unwrap().string().unwrap()
    );
    for (name, hash) in c.get("hashes").unwrap().object().unwrap() {
        assert_eq!(
            id.envelopes[name],
            c.get("envelopes")
                .unwrap()
                .get(name)
                .unwrap()
                .string()
                .unwrap()
        );
        assert_eq!(id.interface_hashes[name], hash.string().unwrap());
        assert_eq!(
            id.closure_hashes[name].as_deref(),
            Some(hash.string().unwrap())
        );
    }
    o.check_hashes().unwrap();
    o
}

pub fn r12_address_case(region: &str) -> Json {
    json(include_str!("r12_address_cases.json"))
        .get(region)
        .unwrap()
        .clone()
}
pub fn r12_address_text(region: &str) -> String {
    r12_address_case(region)
        .get("text")
        .unwrap()
        .string()
        .unwrap()
        .into()
}

/// Literal R13 member witnesses and independently assembled §9.2 pins.
pub fn r13_case(name: &str) -> Json {
    json(include_str!("r13_cases.json"))
        .array()
        .unwrap()
        .iter()
        .find(|c| c.get("name").unwrap().string().unwrap() == name)
        .unwrap()
        .clone()
}
pub fn r13_text(name: &str) -> String {
    r13_case(name)
        .get("object")
        .unwrap()
        .string()
        .unwrap()
        .into()
}
pub fn r13_check(name: &str, root: &str) -> Object {
    let c = r13_case(name);
    let o = Object::parse(c.get("object").unwrap().string().unwrap()).unwrap();
    o.check_hashes().unwrap();
    let i = o.identity().unwrap();
    let pin = c.get("interface_hash").unwrap().string().unwrap();
    assert_eq!(i.interface_hashes[root], pin);
    assert_eq!(i.closure_hashes[root].as_deref(), Some(pin));
    assert_eq!(
        i.envelopes[root],
        c.get("envelope").unwrap().string().unwrap()
    );
    assert_eq!(
        i.object_hash,
        c.get("object_hash").unwrap().string().unwrap()
    );
    o
}

pub mod r14;
pub mod r15;
