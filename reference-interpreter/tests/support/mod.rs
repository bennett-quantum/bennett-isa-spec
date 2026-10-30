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
    for input in 0..6 {
        let mut q = QuantumState::new();
        let w = q.allocate(4).unwrap();
        for (bit, wire) in w[..3].iter().enumerate() {
            if input >= 4 {
                q.gate("h", &[*wire], None, &[]).unwrap();
            } else if input >> bit & 1 == 1 {
                q.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        if input == 5 {
            q.gate("cx", &[w[1], w[3]], None, &[]).unwrap();
        }
        let entry = q.clone();
        let mut expected = q.clone();
        if hadamard {
            expected.gate("h", &[w[0]], None, &[]).unwrap();
        }
        let mut b = QuantumBindings::default();
        b.bundles.insert("t".into(), vec![w[0]]);
        b.borrowed.push(w[1..3].to_vec());
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
