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
    let bounds = format!("{:?}", o.functions["a"].bounds);
    let cert = json(&format!(
        r#"{{"versions":{{"isa":"0.8","object":"0.8","launch":"0.8","sched":"0.8","cert":"0.8","coverage":"0.8","target":"0.8"}},"bindings":{{"object_hash":"{}","interface_hash":"{}","closure_hash":"{a}","reference_program_hash":"{}","dialect_mapping_version":"test.1","target_hash":"{}"}},"producer":{{"name":"tests","version":"1","materialisation":{{"llvm_version":"none","pipeline":"none","normaliser_version":"1"}}}},"regions":{{"{a}":{{"kind":"perm_action","signature":{signature},"bounds":{bounds},"frame":{{"interface":[{{"role":"mut","width":{n},"class":"any"}},{{"role":"in","width":1,"class":"any"}}],"scratch":{{}},"retained_history":0,"borrow":{{"any":1}},"scheme_ancillae":{{}}}},"resources":{{}},"gates_used":{{}},"claims":[{claims}],"verification":{{"oracle":null,"borrow_frames":[{borrow_frames}],"inverse_pairs":[],"unitary":[],"channel":[],"fault_rate":{{"domain":{{"bundles":[],"borrowed":[],"regions":[]}},"faults_by_code":{{}},"undefined_excluded":0,"layout_dependent_excluded":0}}}},"cleanup":{{}},"error":{{}},"definedness_assumptions":[]}}}},"composition":{{"multiplicities":{{}},"epsilon_total":{{"diamond":0}},"method":"sum_over_multiplicities"}}}}"#,
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

#[test]
fn r2_interval_claim_diagnostics_name_deliverable_eleven() {
    let text = swap_frame(1);
    let o = Object::parse(&text).unwrap();
    let closure = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
    for kind in ["numerical", "channel"] {
        let metric = if kind == "channel" {
            "diamond"
        } else {
            "operator_norm"
        };
        let dual = if kind == "channel" {
            ",\"dual\":[]"
        } else {
            ""
        };
        let claim = format!(
            r#"{{"kind":"{kind}","subject":{{"closure_hash":"{closure}","variant":"fwd","inherited_controls":0,"schedule_hash":"{}"}},"metric":"{metric}","precision_bits":128,"bound":0{dual}}}"#,
            "4".repeat(64)
        );
        let o = Object::parse(&certificate(&text, 1, &claim, "")).unwrap();
        match o.verify_claim(&closure, 0).unwrap() {
            ClaimVerdict::Unverified(why) => assert!(why.contains("deliverable 11"), "{why}"),
            other => panic!("staged interval checker cannot prove a claim: {other:?}"),
        }
    }
}

fn sol_translation_object(gates: &str, nested: bool) -> String {
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let asig = if nested {
        sig.replace("\"bounds\":[]", "\"bounds\":[2]")
    } else {
        sig.into()
    };
    let regs = if nested { "regs {  } bounds {2}\n" } else { "" };
    let body = format!("perm_action @a (t:wire mut) (p:wire in) () () ()\n{regs}^entry():\nb = qborrow any 1\n{}\nqreturn b", if nested { format!("qloop 2 {{\n{gates}\n}}") } else { gates.into() });
    support::rehash(&support::object(
        &[
            ("a", &asig, &body),
            (
                "id",
                sig,
                "perm_action @id (u:wire mut) (v:wire in) () () ()\n^entry():\nx u\nx u",
            ),
        ],
        "{}",
        "",
    ))
    .text
}
fn sol_translation_claim(text: &str, start: u32, end: u32, v: u32, w: u32, b: &str) -> String {
    let o = Object::parse(text).unwrap();
    let h = &o.identity().unwrap().interface_hashes["a"];
    format!(
        r#"{{"kind":"structural","rule":"translation_lemma","subclaims":[],"witness":{{"V":[["{h}","entry",{v}]],"W":[["{h}","entry",{w}]],"X":["p"],"B":["{b}"],"frame":[["{h}","entry",{start}],["{h}","entry",{end}]]}}}}"#
    )
}
fn sol_assert_translation(text: &str, claim: &str, valid: bool) {
    let o = Object::parse(&certificate(text, 1, claim, "")).unwrap();
    let h = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
    for verdict in [
        o.verify_claim(&h, 0),
        o.verify_certificate().map(|r| r[0].verdict.clone()),
    ] {
        if valid {
            assert_eq!(verdict.unwrap(), ClaimVerdict::Proof);
        } else {
            let e = verdict
                .expect_err("false structural frame proved")
                .to_string();
            assert!(e.contains("§5.6") || e.contains("§12.1"), "{e}");
        }
    }
}
#[test]
fn sol_r4_b02_translation_claim_must_prove_its_complete_named_frame() {
    // Full frame: b ^= p; t ^= b; b ^= p; t ^= b => (t ^= p, b unchanged).
    // Truncated frame: b ^= p; t ^= b => b dirty and t dependent on entry b.
    let text = sol_translation_object("cx p b\ncx b t\ncx p b\ncx b t", false);
    for (start, end, v, w, valid) in [
        (1, 2, 1, 2, false),
        (1, 4, 1, 2, true),
        (99, 100, 1, 2, false),
        (4, 1, 1, 2, false),
        (1, 4, 99, 2, false),
        (1, 4, 1, 3, false),
    ] {
        sol_assert_translation(
            &text,
            &sol_translation_claim(&text, start, end, v, w, "b"),
            valid,
        );
    }
    sol_assert_translation(&text, &sol_translation_claim(&text, 1, 4, 1, 2, "p"), false);
    // The object itself is still valid, but its first four gates have a
    // changed second W; later gates complete the actual independent circuit.
    let changed = sol_translation_object("cx p b\ncx b t\ncx p b\nx t\ncx b t\nx t", false);
    sol_assert_translation(
        &changed,
        &sol_translation_claim(&changed, 1, 4, 1, 2, "b"),
        false,
    );
}
#[test]
fn sol_r4_b02_translation_claim_uses_nested_line_ordinals() {
    // qborrow=0, qloop=1, nested gates=2..5, qreturn=6. Prove the
    // complete body frame, uniformly for both loop iterations.
    let text = sol_translation_object("cx p b\ncx b t\ncx p b\ncx b t", true);
    sol_assert_translation(&text, &sol_translation_claim(&text, 2, 5, 2, 3, "b"), true);
    sol_assert_translation(&text, &sol_translation_claim(&text, 2, 3, 2, 3, "b"), false);
}

#[test]
fn sol_r4_b02_translation_frame_rejects_omissions_and_checks_reverse_order() {
    let gates = "x b\ncx p b\ncx b t\ncx p b\nx b\ncx b t";
    let text = sol_translation_object(gates, false);
    let mut claim = json(&sol_translation_claim(&text, 1, 6, 1, 3, "b"));
    let h = Object::parse(&text)
        .unwrap()
        .identity()
        .unwrap()
        .interface_hashes["a"]
        .clone();
    let Json::Object(c) = &mut claim else {
        panic!()
    };
    let Json::Object(w) = c.get_mut("witness").unwrap() else {
        panic!()
    };
    w.insert(
        "V".into(),
        json(&format!(r#"[["{h}","entry",1],["{h}","entry",2]]"#)),
    );
    sol_assert_translation(&text, &claim.canonical(), true);
    // Skip x b between the named V and W; the remaining four gates happen
    // to form a valid independent circuit, but the named frame contains six.
    sol_assert_translation(&text, &sol_translation_claim(&text, 1, 6, 2, 3, "b"), false);
    let twice = sol_translation_object(
        "cx p b\ncx b t\ncx p b\ncx b t\ncx p b\ncx b t\ncx p b\ncx b t",
        false,
    );
    sol_assert_translation(
        &twice,
        &sol_translation_claim(&twice, 1, 4, 1, 2, "b"),
        false,
    );
}

fn sol_positional_object(region: bool, reference: &str) -> String {
    let mem = if region {
        r#"[{"A":1,"W":1,"role":"mut"}]"#
    } else {
        "[]"
    };
    let sig = format!(
        r#"{{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":{mem},"mut":[1],"tier":"q"}}"#
    );
    let a = format!(
        "perm_action @a (t:wire mut) (p:wire in) ({}) () ()\n^entry():\nx t\nx t",
        if region { "m:region[1,1] mut" } else { "" }
    );
    let id = format!(
        "perm_action @id (u:wire mut) (v:wire in) ({}) () ()\n^entry():\n{reference}",
        if region { "n:region[1,1] mut" } else { "" }
    );
    support::rehash(&support::object(&[("a",&sig,&a),("id",&sig,&id)],"{}",if region { "\nsection layout\n{\"A_tab\":0,\"H\":0,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n" } else {""})).text
}
fn sol_positional_claim(regions: &str, borrowed: &str, bundles: &str) -> String {
    format!(
        r#"{{"kind":"exhaustive","label":"proof","domain":{{"bundles":[{bundles}],"borrowed":[{borrowed}],"regions":[{regions}]}},"expected":{{"program":"ID"}}}}"#
    )
}
const SOL_BUNDLES: &str =
    r#"{"name":"t","width":1,"lo":0,"hi":1},{"name":"p","width":1,"lo":0,"hi":1}"#;
fn sol_positional_verdict(text: &str, claim: &str) -> Result<ClaimVerdict, bennett_isa_ref::Error> {
    let o = Object::parse(&certificate(text, 1, claim, "")).unwrap();
    let h = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
    let v = o.verify_claim(&h, 0)?;
    assert_eq!(o.verify_certificate().unwrap()[0].verdict, v);
    Ok(v)
}
#[test]
fn sol_r4_b03_expected_program_uses_signature_positions_for_renamed_bundles() {
    // Independently: both bodies are identity on 00,01,10,11. Local names
    // t,p versus u,v have no ABI significance (§9.4).
    let text = sol_positional_object(false, "x u\nx u");
    let claim = sol_positional_claim("", "", SOL_BUNDLES);
    assert_eq!(
        sol_positional_verdict(&text, &claim).unwrap(),
        ClaimVerdict::Proof
    );
    // Permuting domain enumeration order must not permute interface slots.
    let reverse = sol_positional_claim(
        "",
        "",
        r#"{"name":"p","width":1,"lo":0,"hi":1},{"name":"t","width":1,"lo":0,"hi":1}"#,
    );
    assert_eq!(
        sol_positional_verdict(&text, &reverse).unwrap(),
        ClaimVerdict::Proof
    );
    // Borrowed pool entries follow their domain order, independently of the
    // expected program's local names. A truly omitted input stays zero.
    let borrowed = r#"{"name":"z","width":1,"lo":0,"hi":1},{"name":"b","width":2,"lo":0,"hi":3}"#;
    assert_eq!(
        sol_positional_verdict(&text, &sol_positional_claim("", borrowed, SOL_BUNDLES)).unwrap(),
        ClaimVerdict::Proof
    );
    let partial = sol_positional_claim("", "", r#"{"name":"t","width":1,"lo":0,"hi":1}"#);
    assert_eq!(
        sol_positional_verdict(&text, &partial).unwrap(),
        ClaimVerdict::Evidence { elements: 2 }
    );
    let wrong = sol_positional_object(false, "x u");
    let e = sol_positional_verdict(&wrong, &claim)
        .unwrap_err()
        .to_string();
    assert!(
        e.contains("§12.1") && e.contains("differs from the expected program"),
        "{e}"
    );
}
#[test]
fn sol_r4_b03_expected_program_maps_renamed_region_cells_positionally() {
    // 2 interface bits and 2 independent region cells => 16 identical final
    // tuples. Changing only reference memory must still be detected.
    let text = sol_positional_object(true, "x u\nx u");
    let claim = sol_positional_claim(r#"{"name":"m","cells":"all"}"#, "", SOL_BUNDLES);
    assert_eq!(
        sol_positional_verdict(&text, &claim).unwrap(),
        ClaimVerdict::Proof
    );
    let partial = sol_positional_claim(
        r#"{"name":"m","cells":[{"index":1,"lo":0,"hi":1}]}"#,
        "",
        SOL_BUNDLES,
    );
    assert_eq!(
        sol_positional_verdict(&text, &partial).unwrap(),
        ClaimVerdict::Evidence { elements: 8 }
    );
    let wrong = sol_positional_object(true, "qstore n v u");
    assert!(sol_positional_verdict(&wrong, &claim)
        .unwrap_err()
        .to_string()
        .contains("§12.1"));
}

#[test]
fn sol_r4_b03_positions_distinguish_same_width_slots_and_crossed_names() {
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[1,1],"tier":"q"}"#;
    // Rename by crossing existing local names. Both maps are (t,u,p) ->
    // (t XOR p,u,p); a name-based correspondence would exchange mut and in.
    let a = "perm_action @a (t:wire mut, u:wire mut) (p:wire in) () () ()\n^entry():\ncx p t";
    let id = "perm_action @id (p:wire mut, t:wire mut) (u:wire in) () () ()\n^entry():\ncx u p";
    let text = support::rehash(&support::object(
        &[("a", sig, a), ("id", sig, id)],
        "{}",
        "",
    ))
    .text;
    let claim = sol_positional_claim(
        "",
        "",
        r#"{"name":"u","width":1,"lo":0,"hi":1},{"name":"p","width":1,"lo":0,"hi":1},{"name":"t","width":1,"lo":0,"hi":1}"#,
    );
    assert_eq!(
        sol_positional_verdict(&text, &claim).unwrap(),
        ClaimVerdict::Proof
    );
    // Writing the second mutable slot instead must fail the full-tuple check.
    let wrong = support::rehash(&support::object(
        &[("a", sig, a), ("id", sig, &id.replace("cx u p", "cx u t"))],
        "{}",
        "",
    ))
    .text;
    assert!(sol_positional_verdict(&wrong, &claim)
        .unwrap_err()
        .to_string()
        .contains("§12.1"));
}

/// The hashes are derived without parsing the invalid uncertified subject.
fn r5_certified_lifetime(gate: &str) -> String {
    let (h, object_hash) = match gate {
        "x b" => (
            "3bc76b4398502416eed66400092c8ec2e5e273d97283b5199065a88a3ed9f1d7",
            "9b10654c083129047b847830d3582ff3dbf15cf7c8759f4586f836c02e5b6266",
        ),
        "cx b t" => (
            "1e0d1484447af491bd512ef5cc3cbe4f1b9532863aa56ae2149f61247973f343",
            "4aaf72716633ad6037e3b542f000eb31372accd743ddb97aa97d2960447ac43b",
        ),
        "cx t b" => (
            "020372af2df49bc56df0f37f08dfc7b58af2e258df9fb58da7010a7a8d3eb24a",
            "cb7ef9243fc8e1144a6863142a46e2969941940894e993bd2abddb685725f7cd",
        ),
        _ => panic!("test gate"),
    };
    let body = format!("qloop 2 {{\nb = qborrow any 1\n{gate}\nqreturn b\n}}");
    let text = support::borrow_lifetime_object(&body, &[2]).replace(&"0".repeat(64), h);
    let certified = support::certified_borrow_identity(&body, &[2]);
    assert_eq!(
        certified.split("\nsection certificate").next().unwrap(),
        text
    );
    let cert = json(certified.lines().last().unwrap());
    assert_eq!(
        cert.get("bindings")
            .unwrap()
            .get("object_hash")
            .unwrap()
            .string()
            .unwrap(),
        object_hash
    );
    certified
}

#[test]
fn sol_r5_b01_certificates_cannot_waive_completed_lifetimes() {
    for (i, gate) in ["x b", "cx b t"].iter().enumerate() {
        // Each beta_i is independent: X fails restoration, CX fails output
        // independence. Whole-loop parity is irrelevant to either obligation.
        let certified = r5_certified_lifetime(gate);
        for (j, text) in [
            certified.split("\nsection certificate").next().unwrap(),
            &certified,
        ]
        .iter()
        .enumerate()
        {
            support::reject_cli(text, "§5.6", &format!("r5-certified-{i}-{j}"));
        }
    }
}

#[test]
fn sol_r5_b01_library_replay_checks_boundaries_even_when_region_map_is_identity() {
    for gate in ["x b", "cx b t"] {
        let invalid = r5_certified_lifetime(gate);
        // Object's fields are public. Isolate verify_certificate from parse's
        // definite-failure guard by constructing the same compiled subject from
        // a valid paired-gate object, then installing the independently bound
        // invalid source/metadata. This tests the second admission defense.
        let valid = support::borrow_lifetime_object(
            &format!("qloop 2 {{\nb = qborrow any 1\n{gate}\n{gate}\nqreturn b\n}}"),
            &[2],
        );
        let mut o = Object::parse(&valid).unwrap();
        let f = o.functions.get_mut("a").unwrap();
        f.blocks[0].instructions[0].bodies[0].remove(2);
        f.lines = invalid
            .split("section functions\n")
            .nth(1)
            .unwrap()
            .split("\n\nsection certificate")
            .next()
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect();
        o.metadata.insert(
            "symbols".into(),
            json(
                invalid
                    .split("section symbols\n")
                    .nth(1)
                    .unwrap()
                    .lines()
                    .next()
                    .unwrap(),
            ),
        );
        o.metadata
            .insert("certificate".into(), json(invalid.lines().last().unwrap()));
        o.text = invalid;
        let closure = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
        assert_eq!(o.verify_claim(&closure, 0).unwrap(), ClaimVerdict::Proof);
        let error = o.verify_certificate().unwrap_err().to_string();
        assert!(
            error.contains("§5.6") && error.contains("qreturn"),
            "{error}"
        );
    }
    for body in [
        "qloop 2 {\nb = qborrow any 1\nx b\nx b\nqreturn b\n}",
        "qloop 2 {\nb = qborrow any 1\ncx b t\ncx b t\nqreturn b\n}",
        "b = qborrow any 1\nqloop 2 {\ncx b t\n}\nqreturn b",
    ] {
        let o = Object::parse(&support::certified_borrow_identity(body, &[2])).unwrap();
        assert_eq!(
            o.verify_certificate().unwrap()[0].verdict,
            ClaimVerdict::Proof
        );
        support::borrow_identity(&o);
    }
}

#[test]
fn sol_r5_b01_certificate_return_checks_keep_quantum_surrounding_state() {
    // Each borrowed frame is identity even though t is a superposition at
    // qreturn. Auditing the classical values of all other wires would narrow
    // this valid action; compare their exact quantum state instead.
    for body in [
        "h t\nb = qborrow any 1\nx b\nx b\nqreturn b\nh t",
        "b = qborrow any 1\nx b\nx b\nh t\nh t\nx b\nx b\nqreturn b",
    ] {
        let o = Object::parse(&support::certified_borrow_identity_kind(
            body,
            &[],
            "action",
        ))
        .unwrap();
        assert_eq!(
            o.verify_certificate().unwrap()[0].verdict,
            ClaimVerdict::Proof
        );
        support::borrow_identity(&o);
    }
}

#[test]
fn sol_r5_b01_library_replay_checks_frames_before_nonpermutation_boundaries() {
    for gate in ["x b", "cx b t"] {
        let body = format!("b = qborrow any 1\n{gate}\nh t\nh t\n{gate}\nqreturn b");
        let invalid = support::certified_borrow_identity_kind(&body, &[], "action");
        let valid = support::borrow_lifetime_object(
            &format!("b = qborrow any 1\n{gate}\n{gate}\nh t\nh t\n{gate}\n{gate}\nqreturn b"),
            &[],
        )
        .replace("perm_action", "action");
        let mut o = Object::parse(&valid).unwrap();
        let f = o.functions.get_mut("a").unwrap();
        f.blocks[0].instructions.remove(6);
        f.blocks[0].instructions.remove(2);
        f.lines = invalid
            .split("section functions\n")
            .nth(1)
            .unwrap()
            .split("\n\nsection certificate")
            .next()
            .unwrap()
            .lines()
            .map(str::to_string)
            .collect();
        o.metadata.insert(
            "symbols".into(),
            json(
                invalid
                    .split("section symbols\n")
                    .nth(1)
                    .unwrap()
                    .lines()
                    .next()
                    .unwrap(),
            ),
        );
        o.metadata
            .insert("certificate".into(), json(invalid.lines().last().unwrap()));
        o.text = invalid;
        let closure = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
        assert_eq!(o.verify_claim(&closure, 0).unwrap(), ClaimVerdict::Proof);
        assert!(o
            .verify_certificate()
            .unwrap_err()
            .to_string()
            .contains("§5.6"));
    }
}

#[test]
fn codex_r5_b01_exact_three_certified_lifetime_witnesses() {
    for (i, gate) in ["x b", "cx b t", "cx t b"].iter().enumerate() {
        let exact = r5_certified_lifetime(gate);
        for (j, text) in [exact.split("\nsection certificate").next().unwrap(), &exact]
            .iter()
            .enumerate()
        {
            support::reject_cli(text, "§5.6", &format!("codex-r5-b01-{i}-{j}"));
        }
        // No lifetime may borrow a future lifetime's parity cancellation.
        for (j, (body, bounds)) in [
            (format!("b = qborrow any 1\n{gate}\nqreturn b"), vec![]),
            (
                format!("qloop 2 {{\nqloop 2 {{\nb = qborrow any 1\n{gate}\nqreturn b\n}}\n}}"),
                vec![2, 2],
            ),
        ]
        .iter()
        .enumerate()
        {
            support::reject_cli(
                &support::certified_borrow_identity(body, bounds),
                "§5.6",
                &format!("codex-r5-b01-more-{i}-{j}"),
            );
        }
    }
}

fn codex_r5_install_subject(invalid: &str, body: &str) -> Object {
    // Parse a legal shape, then reconstruct public fields to isolate the
    // certificate API's checks from load-time admission.
    let valid = support::borrow_lifetime_object(body, &[]).replace("perm_action", "action");
    let mut o = Object::parse(&valid.replace("cz b ", "x ")).unwrap();
    for i in &mut o.functions.get_mut("a").unwrap().blocks[0].instructions {
        if i.op == "x" {
            i.op = "cz".into();
            i.operands.insert(0, "b".into());
        }
    }
    o.functions.get_mut("a").unwrap().lines = invalid
        .split("section functions\n")
        .nth(1)
        .unwrap()
        .split("\n\nsection certificate")
        .next()
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect();
    o.metadata.insert(
        "symbols".into(),
        json(
            invalid
                .split("section symbols\n")
                .nth(1)
                .unwrap()
                .lines()
                .next()
                .unwrap(),
        ),
    );
    o.metadata
        .insert("certificate".into(), json(invalid.lines().last().unwrap()));
    o.text = invalid.into();
    o
}

#[test]
fn codex_r5_b02_exact_cz_certificate_cannot_admit_a_nonpermutation_frame() {
    let body = "b = qborrow any 1\ncz b t\nqreturn b";
    let exact = support::certified_borrow_identity_kind(body, &[], "action");
    let cert = json(exact.lines().last().unwrap());
    let bindings = cert.get("bindings").unwrap();
    assert_eq!(
        bindings.get("interface_hash").unwrap().string().unwrap(),
        "26374c7b2da2747a9a755d48da06130bfa53e8ba67102e4749fa1cb8adc2f1a0"
    );
    assert_eq!(
        bindings.get("object_hash").unwrap().string().unwrap(),
        "2340cad379662fdcec0e7664aa161a667d77294eee1da295bced60916553158d"
    );
    for (i, text) in [
        exact.split("\nsection certificate").next().unwrap(),
        &exact,
        &exact.replace("\"borrow_frames\":[0]", "\"borrow_frames\":[]"),
    ]
    .iter()
    .enumerate()
    {
        support::reject_cli(text, "§5.6", &format!("codex-r5-b02-{i}"));
    }
    // Public API: a full tuple claim alone is not a T3 proof. Even a pair of
    // illegal phase touches which cancels in the final unitary must fail the
    // mandatory permutation-only gate-boundary premise, independently of replay.
    for body in [
        body,
        "b = qborrow any 1\ncz b t\ncz b t\nqreturn b",
        "c = qalloc any 1\nb = qborrow any 1\ncz b c\ncz b c\nqfree c\nqreturn b",
    ] {
        let invalid = support::certified_borrow_identity_kind(body, &[], "action");
        let o = codex_r5_install_subject(&invalid, body);
        let error = o
            .verify_certificate()
            .expect_err("nonpermutation borrow frame certified");
        assert!(error.to_string().contains("§5.6"), "{error}");
    }
    // Quantum phase witness, not integer basis labels: CZ|++> has -1/2 at 11.
    let o = codex_r5_install_subject(&exact, body);
    let mut q = bennett_isa_ref::QuantumState::new();
    let w = q.allocate(2).unwrap();
    for wire in &w {
        q.gate("h", &[*wire], None, &[]).unwrap();
    }
    let mut b = bennett_isa_ref::QuantumBindings::default();
    b.bundles.insert("t".into(), vec![w[0]]);
    b.borrowed.push(vec![w[1]]);
    o.execute_action("a", &mut q, &b, false, &[]).unwrap();
    for (bits, (re, im)) in q.exact_support() {
        assert_eq!(re, if bits == [true, true] { -0.5 } else { 0.5 });
        assert_eq!(im, 0.0);
    }
    // Non-permutation gates on unrelated wires remain legal, including when
    // t is superposed at the frame's entry or return.
    for body in [
        "h t\nb = qborrow any 1\nx b\nx b\nqreturn b\nh t",
        "b = qborrow any 1\nh t\nx b\nx b\nh t\nqreturn b",
    ] {
        let o = Object::parse(&support::certified_borrow_identity_kind(
            body,
            &[],
            "action",
        ))
        .unwrap();
        assert_eq!(
            o.verify_certificate().unwrap()[0].verdict,
            ClaimVerdict::Proof
        );
        support::borrow_identity(&o);
    }
}
