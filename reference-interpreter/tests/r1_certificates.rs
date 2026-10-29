//! Council round 1: codex B14 (claim route), B16, muse B7 (typed metadata and
//! checked claims).
mod support;
use bennett_isa_ref::{json::Json, ClaimVerdict, Object};
use support::json;

const INC: &str = include_str!("fixtures/inc8.bisa");

fn rejects(text: &str, rule: &str) {
    match Object::parse(text) {
        Err(e) => assert!(e.to_string().contains(rule), "{e} must cite {rule}"),
        Ok(_) => panic!("malformed metadata accepted"),
    }
}
/// A perm_action `a` borrowing one wire around a swap frame over t[0..n),
/// and an identity reference `id` of the same signature.
fn swap_frame(n: u64) -> String {
    let sig = format!(
        r#"{{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[{n}],"tier":"q"}}"#
    );
    let forward: Vec<String> = (0..n).map(|i| format!("swap b t[{i}]")).collect();
    let back: Vec<String> = (0..n).rev().map(|i| format!("swap b t[{i}]")).collect();
    let a = format!(
        "perm_action @a (t:wire[{n}] mut) (x:wire in) () () ()\n^entry():\nb = qborrow any 1\n{}\nqreturn b",
        [forward, back].concat().join("\n")
    );
    let id = format!(
        "perm_action @id (t:wire[{n}] mut) (x:wire in) () () ()\n^entry():\nx t[0]\nx t[0]"
    );
    support::rehash(&support::object(
        &[("a", &sig, &a), ("id", &sig, &id)],
        "{}",
        "",
    ))
    .text
}
fn certificate(text: &str, n: u64, claims: &str, borrow_frames: &str) -> String {
    let o = Object::parse(text).unwrap();
    let identity = o.identity().unwrap();
    let a = identity.closure_hashes["a"].clone().unwrap();
    let id = identity.closure_hashes["id"].clone().unwrap();
    let signature = o.functions["a"].signature().canonical();
    let claims = claims.replace("ID", &id);
    let cert = json(&format!(
        r#"{{"versions":{{"isa":"0.8","object":"0.8","launch":"0.8","sched":"0.8","cert":"0.8","coverage":"0.8","target":"0.8"}},"bindings":{{"object_hash":"{}","interface_hash":"{}","closure_hash":"{a}","reference_program_hash":"{}","dialect_mapping_version":"test.1","target_hash":"{}"}},"producer":{{"name":"tests","version":"1","materialisation":{{"llvm_version":"none","pipeline":"none","normaliser_version":"1"}}}},"regions":{{"{a}":{{"kind":"perm_action","signature":{signature},"bounds":[],"frame":{{"interface":[{{"role":"mut","width":{n},"class":"any"}},{{"role":"in","width":1,"class":"any"}}],"scratch":{{}},"retained_history":0,"borrow":{{"any":1}},"scheme_ancillae":{{}}}},"resources":{{}},"gates_used":{{}},"claims":[{claims}],"verification":{{"oracle":null,"borrow_frames":[{borrow_frames}],"inverse_pairs":[],"unitary":[],"channel":[],"fault_rate":{{"domain":{{"bundles":[],"borrowed":[],"regions":[]}},"faults_by_code":{{}},"undefined_excluded":0,"layout_dependent_excluded":0}}}},"cleanup":{{}},"error":{{}},"definedness_assumptions":[]}}}},"composition":{{"multiplicities":{{}},"epsilon_total":{{"diamond":0}},"method":"sum_over_multiplicities"}}}}"#,
        identity.object_hash,
        identity.interface_hashes["a"],
        identity.interface_hashes["a"],
        "3".repeat(64)
    ));
    format!("{text}\nsection certificate\n{}\n", cert.canonical())
}
fn claim(n: u64, t_hi: &str, b_hi: u32, expected: &str) -> String {
    format!(
        r#"{{"kind":"exhaustive","label":"proof","domain":{{"bundles":[{{"name":"t","width":{n},"lo":0,"hi":{t_hi}}},{{"name":"x","width":1,"lo":0,"hi":1}}],"borrowed":[{{"name":"b","width":1,"lo":0,"hi":{b_hi}}}],"regions":[]}},"expected":{expected}}}"#
    )
}

#[test]
fn b14_checked_borrow_claim_and_altered_or_incomplete_claims() {
    let n = 4;
    let text = swap_frame(n);
    let good = certificate(&text, n, &claim(n, "15", 1, r#"{"program":"ID"}"#), "0");
    let o = Object::parse(&good).unwrap();
    let reports = o.verify_certificate().unwrap();
    assert_eq!(reports.len(), 1);
    assert_eq!(reports[0].verdict, ClaimVerdict::Proof);
    // Altered: the expected program is the frame itself... replaced by a
    // table with one wrong row (2^6 rows: t fastest-last order).
    let mut rows = Vec::new();
    for t in 0..16u32 {
        for x in 0..2u32 {
            for b in 0..2u32 {
                rows.push(format!("[{t},{x},{b}]"));
            }
        }
    }
    let table = format!(r#"{{"table":[{}]}}"#, rows.join(","));
    let exact = certificate(&text, n, &claim(n, "15", 1, &table), "0");
    assert_eq!(
        Object::parse(&exact).unwrap().verify_certificate().unwrap()[0].verdict,
        ClaimVerdict::Proof
    );
    let wrong = table.replacen("[0,0,0]", "[1,0,0]", 1);
    let altered = certificate(&text, n, &claim(n, "15", 1, &wrong), "0");
    let e = Object::parse(&altered)
        .unwrap()
        .verify_certificate()
        .unwrap_err()
        .to_string();
    assert!(e.contains("§12.1"), "{e}");
    // Incomplete: t only up to 7, so the domain is not D: evidence, not proof,
    // and therefore not a borrow_frames proof (§5.6, §12.2).
    let incomplete = certificate(&text, n, &claim(n, "7", 1, r#"{"program":"ID"}"#), "0");
    let e = Object::parse(&incomplete)
        .unwrap()
        .verify_certificate()
        .unwrap_err()
        .to_string();
    assert!(e.contains("§5.6"), "{e}");
    // A borrow claim must range over the borrowed bundle.
    let no_borrowed = claim(n, "15", 1, r#"{"program":"ID"}"#)
        .replace(r#"[{"name":"b","width":1,"lo":0,"hi":1}]"#, "[]");
    let e = Object::parse(&certificate(&text, n, &no_borrowed, "0"))
        .unwrap()
        .verify_certificate()
        .unwrap_err()
        .to_string();
    assert!(e.contains("§5.6"), "{e}");
}
#[test]
#[ignore = "the codex B14 17-wire frame with its 2^18-element claim; run with --ignored"]
fn b14_seventeen_wire_frame_with_checked_claim() {
    let n = 16;
    let text = swap_frame(n);
    let good = certificate(&text, n, &claim(n, "65535", 1, r#"{"program":"ID"}"#), "0");
    let reports = Object::parse(&good).unwrap().verify_certificate().unwrap();
    assert_eq!(reports[0].verdict, ClaimVerdict::Proof);
}

fn schedule(field: &str, value: &str) -> Json {
    let base = format!(
        r#"{{"bindings":{{"budget":[3,0,0,0],"closure_hash":"8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93","epsilon":0,"expansion":{{"hash":"{}","name":"ref","version":"0.8"}},"inherited_controls":0,"params_hash":"{}","region":"inc8","scheduler":{{"name":"test","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[{{"generated":"*","kind":"start","path":[{{"iteration":[],"point":"entry:0","symbol":"8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93"}}],"phase":"fwd","start":{{"time":0}},"step":1}}]}}"#,
        "1".repeat(64),
        "2".repeat(64),
        "3".repeat(64)
    );
    let mut s = json(&base);
    let Json::Object(top) = &mut s else { panic!() };
    match field {
        "" => {}
        "budget" | "variant" => {
            let Json::Object(b) = top.get_mut("bindings").unwrap() else {
                panic!()
            };
            b.insert(field.into(), json(value));
        }
        _ => {
            let Json::Array(entries) = top.get_mut("entries").unwrap() else {
                panic!()
            };
            let Json::Object(e) = &mut entries[0] else {
                panic!()
            };
            if field == "time" {
                e.insert("start".into(), json(&format!(r#"{{"time":{value}}}"#)));
            } else if field == "path" {
                e.insert("path".into(), json(value));
            } else {
                e.insert(field.into(), json(value));
            }
        }
    }
    s
}
#[test]
fn b16_schedule_values_are_typed() {
    let with = |s: &Json| format!("{INC}\nsection schedule\n{}\n", s.canonical());
    Object::parse(&with(&schedule("", ""))).unwrap();
    for (field, value) in [
        ("budget", "[3,true,0,0]"),
        ("budget", "[3,null,0,0]"),
        ("budget", "[3,-1,0,0]"),
        ("phase", "\"garbage\""),
        (
            "path",
            r#"[{"iteration":[],"point":7,"symbol":"8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93"}]"#,
        ),
        (
            "path",
            r#"[{"iteration":"all","point":"entry:0","symbol":"8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93"}]"#,
        ),
        ("time", "-1"),
        ("variant", "\"a:b\""),
    ] {
        rejects(&with(&schedule(field, value)), "§");
    }
}
#[test]
fn b16_muse_b7_certificate_values_and_claims_are_typed() {
    let text = swap_frame(2);
    let good = certificate(&text, 2, &claim(2, "3", 1, r#"{"program":"ID"}"#), "0");
    Object::parse(&good).unwrap();
    for (from, to) in [
        (
            r#""versions":{"cert":"0.8""#,
            r#""versions":false,"x":{"cert":"0.8""#,
        ),
        (
            r#""dialect_mapping_version":"test.1""#,
            r#""dialect_mapping_version":false"#,
        ),
        (r#""multiplicities":{}"#, r#""multiplicities":false"#),
        (
            r#""method":"sum_over_multiplicities""#,
            r#""method":"nonsense""#,
        ),
        (
            r#""epsilon_total":{"diamond":0}"#,
            r#""epsilon_total":false"#,
        ),
        (r#""claims":[{"#, r#""claims":[{"bogus":1},{"#),
        (r#""scratch":{}"#, r#""scratch":{},"unknown":1"#),
        (r#""gates_used":{}"#, r#""gates_used":{"x":{"count":-1}}"#),
    ] {
        let bad = good.replacen(from, to, 1);
        assert_ne!(bad, good, "{from}");
        // Re-canonicalise so the test reaches the schema, not the JCS check.
        let line = bad.lines().last().unwrap();
        let fixed = match Json::parse(line) {
            Ok(j) => bad.replace(line, &j.canonical()),
            Err(_) => bad,
        };
        rejects(&fixed, "§");
    }
}
