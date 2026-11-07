mod support;
use bennett_isa_ref::{json::Json, ClaimVerdict, Object};
use std::collections::BTreeMap;
const SOURCE: &str = include_str!("support/adjoint-table/arith.bij.add.w1.bisa");
const CLAIM: &str = r#"{"kind":"structural","rule":"adjoint_table","subclaims":[],"witness":{},"invocation":{"args":"all","budget":"all"},"label":"proof"}"#;

fn fields(j: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(m) = j else { panic!("object") };
    m
}
fn claims<'a>(c: &'a mut Json, root: &str) -> &'a mut Vec<Json> {
    let record = fields(fields(c).get_mut("regions").unwrap())
        .get_mut(root)
        .unwrap();
    let Json::Array(a) = fields(record).get_mut("claims").unwrap() else {
        panic!("claims")
    };
    a
}
fn attach(base: &Object, list: Vec<Json>) -> (String, String) {
    let id = base.identity().unwrap();
    let (name, f) = base.functions.first_key_value().unwrap();
    let root = id.closure_hashes[name].clone().unwrap();
    let mut c = Object::parse(SOURCE)
        .unwrap()
        .section("certificate")
        .unwrap()
        .clone();
    let bindings = fields(fields(&mut c).get_mut("bindings").unwrap());
    for (key, value) in [
        ("object_hash", &id.object_hash),
        ("interface_hash", &id.interface_hashes[name]),
        ("closure_hash", &root),
        ("reference_program_hash", &root),
    ] {
        bindings.insert(key.into(), Json::String(value.clone()));
    }
    bindings.remove("invocation");
    let regions = fields(fields(&mut c).get_mut("regions").unwrap());
    let mut rec = regions.values().next().unwrap().clone();
    fields(&mut rec).insert("kind".into(), Json::String(f.kind.clone()));
    fields(&mut rec).insert("signature".into(), f.signature_for(base.isa_version));
    fields(&mut rec).insert("bounds".into(), support::json(&format!("{:?}", f.bounds)));
    let slots = if matches!(f.kind.as_str(), "action" | "perm_action") {
        r#"[{"id":"mut:0","role":"mut","class":"any","width":1}]"#
    } else {
        "[]"
    };
    fields(&mut rec).insert(
        "frame".into(),
        support::json(&format!(
            r#"{{"slots":{slots},"scratch":[],"history":[],"borrow":[],"ancillae":[]}}"#
        )),
    );
    fields(&mut rec).insert("claims".into(), Json::Array(list));
    regions.clear();
    regions.insert(root.clone(), rec);
    (
        format!("{}\nsection certificate\n{}\n", base.text, c.canonical()),
        root,
    )
}
fn action(kind: &str, body: &str) -> Object {
    let signature = format!(
        r#"{{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"{kind}","mem":[],"mut":[1],"tier":"q"}}"#
    );
    let text = support::object(
        &[(
            "a",
            &signature,
            &format!("{kind} @a (x:wire mut) () () () ()\n^entry():\n{body}"),
        )],
        "{}",
        "",
    )
    .replace("0.8", "0.9");
    support::rehash(&text)
}
fn verify(text: &str, root: &str, index: usize) -> bennett_isa_ref::Result<ClaimVerdict> {
    Object::parse(text)?.verify_claim(root, index)
}
#[test]
fn published_arithmetic_adjoint_claims_establish() {
    let o = Object::parse(SOURCE).unwrap();
    o.check_hashes().unwrap();
    let id = o.identity().unwrap();
    let sub = id.closure_hashes["arith.bij.sub.w1"].as_ref().unwrap();
    assert_eq!(o.verify_claim(sub, 1).unwrap(), ClaimVerdict::Proof);
    let add = id.closure_hashes["arith.bij.add.w1"].as_ref().unwrap();
    let mut c = o.section("certificate").unwrap().clone();
    claims(&mut c, add).push(support::json(CLAIM));
    let text = format!(
        "{}section certificate\n{}\n",
        SOURCE.split_once("section certificate\n").unwrap().0,
        c.canonical()
    );
    let o = Object::parse(&text).unwrap();
    o.check_hashes().unwrap();
    assert_eq!(o.verify_claim(add, 1).unwrap(), ClaimVerdict::Proof);
    assert!(o
        .verify_certificate()
        .unwrap()
        .claims
        .iter()
        .all(|r| r.verdict == ClaimVerdict::Proof));
}
#[test]
fn minimal_non_involution_and_quantum_action_establish() {
    for (kind, body) in [("perm_action", "x x[0]"), ("action", "h x[0]\ns x[0]")] {
        let (text, root) = attach(&action(kind, body), vec![support::json(CLAIM)]);
        assert_eq!(verify(&text, &root, 0).unwrap(), ClaimVerdict::Proof);
    }
}
#[test]
fn non_action_subject_rejects() {
    let base = support::rehash(
        &support::object(
            &[(
                "a",
                r#"{"abi":"0.9","kind":"cc","tier":"cc","params":[],"results":[]}"#,
                "fn cc @a () () () () ()\n^entry():\nret",
            )],
            "{}",
            "",
        )
        .replace("0.8", "0.9"),
    );
    let (text, root) = attach(&base, vec![support::json(CLAIM)]);
    assert!(
        verify(&text, &root, 0).is_err(),
        "non-action adjoint admitted"
    );
}
#[test]
fn malformed_witness_and_indices_reject() {
    for bad in [
        CLAIM.replace(r#""witness":{}"#, r#""witness":{"other":0}"#),
        CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0]"),
        CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[8]"),
        CLAIM.replace(
            r#""args":"all","budget":"all""#,
            r#""args":[],"budget":"all""#,
        ),
    ] {
        let (text, root) = attach(&action("perm_action", "x x[0]"), vec![support::json(&bad)]);
        assert!(
            verify(&text, &root, 0).is_err(),
            "malformed claim admitted: {bad}"
        );
    }
}
fn replay(root: &str, kind: &str, hi: u32) -> Json {
    let sampler = if kind == "sampled" {
        r#", "sampler":{"algorithm":"xoshiro256**","seed":["1","0","0","0"],"count":1}"#
    } else {
        ""
    };
    support::json(&format!(
        r#"{{"kind":"{kind}","domain":{{"bundles":[{{"name":"x","width":1,"lo":0,"hi":{hi}}}],"regions":[],"borrowed":[]}},"expected":{{"program":"{root}"}},"subject":{{"closure_hash":"{root}","variant":"open_reference_name","inherited_controls":0}},"invocation":{{"args":[],"budget":["0","0","0","0"]}}{sampler}}}"#
    ))
}
#[test]
fn subclaims_are_checked_without_promoting_evidence_or_partial_domains() {
    let base = action("perm_action", "x x[0]");
    let root = base.identity().unwrap().closure_hashes["a"]
        .clone()
        .unwrap();
    for (kind, hi, proof) in [
        ("exhaustive", 1, true),
        ("exhaustive", 0, false),
        ("sampled", 1, false),
    ] {
        let structural = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0]"));
        let (text, _) = attach(&base, vec![replay(&root, kind, hi), structural]);
        let verdict = verify(&text, &root, 1).unwrap();
        if proof {
            assert_eq!(verdict, ClaimVerdict::Proof);
        } else {
            assert!(
                matches!(verdict, ClaimVerdict::Unverified(ref s) if s.contains("adjoint_table subclaim 0")),
                "{verdict:?}"
            );
        }
    }
}
#[test]
fn a_refuted_later_premise_is_not_hidden_by_an_unverified_one() {
    let base = action("perm_action", "x x[0]");
    let root = base.identity().unwrap().closure_hashes["a"]
        .clone()
        .unwrap();
    let mut false_claim = replay(&root, "exhaustive", 1);
    fields(&mut false_claim).insert("expected".into(), support::json(r#"{"table":[{"before":{"bundles":["0"],"borrowed":[],"regions":[]},"after":{"bundles":["0"],"borrowed":[],"regions":[]}},{"before":{"bundles":["1"],"borrowed":[],"regions":[]},"after":{"bundles":["1"],"borrowed":[],"regions":[]}}]}"#));
    let claim = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0,1]"));
    let (text, _) = attach(&base, vec![replay(&root, "sampled", 1), false_claim, claim]);
    assert!(verify(&text, &root, 2).is_err(), "refuted premise hidden");
}

#[test]
fn unavailable_linked_body_and_unverified_subclaim_stay_unverified() {
    let sig = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let imports = format!(
        r#"{{"b":{{"signature":{sig},"pinned_closure_hash":"{}"}}}}"#,
        "ab".repeat(32)
    );
    let base = support::rehash(
        &support::object(
            &[(
                "a",
                sig,
                "perm_action @a (x:wire mut) () () () ()\n^entry():\naction_call @b (x) () () ()",
            )],
            &imports,
            "",
        )
        .replace("0.8", "0.9"),
    );
    let dependent = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0]"));
    let (text, root) = attach(&base, vec![support::json(CLAIM), dependent]);
    let o = Object::parse(&text).unwrap();
    assert!(
        matches!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Unverified(ref s) if s.contains("linked definition unavailable: @b"))
    );
    assert!(
        matches!(o.verify_claim(&root, 1).unwrap(), ClaimVerdict::Unverified(ref s) if s.contains("adjoint_table subclaim 0"))
    );
}

#[test]
fn checked_source_discipline_is_required_even_through_public_fields() {
    let (text, root) = attach(&action("perm_action", "x x[0]"), vec![support::json(CLAIM)]);
    for op in ["unmapped_instruction", "h"] {
        let mut o = Object::parse(&text).unwrap();
        o.functions.get_mut("a").unwrap().blocks[0].instructions[0].op = op.into();
        assert!(
            o.verify_claim(&root, 0).is_err(),
            "unchecked source {op} proved"
        );
    }
}

#[test]
fn repeated_unverified_premises_do_not_become_a_cycle() {
    let base = action("perm_action", "x x[0]");
    let root = base.identity().unwrap().closure_hashes["a"]
        .clone()
        .unwrap();
    let dependency = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[0]"));
    let claim = support::json(&CLAIM.replace("\"subclaims\":[]", "\"subclaims\":[1,1]"));
    let (text, _) = attach(&base, vec![replay(&root, "sampled", 1), dependency, claim]);
    assert!(
        matches!(verify(&text, &root, 2).unwrap(), ClaimVerdict::Unverified(ref s) if s.contains("adjoint_table subclaim 1"))
    );
}

#[test]
fn binding_and_source_obligations_reject_instead_of_establishing() {
    let base = action("perm_action", "x x[0]");
    for bad in [
        CLAIM.replace(r#""budget":"all""#, r#""budget":["0","0","0","0"]"#),
        CLAIM.replace(r#""witness":{}"#, r#""witness":{},"subject":{"closure_hash":"INVALID","variant":"adj","inherited_controls":0}"#),
        CLAIM.replace(r#""witness":{}"#, r#""witness":{},"domain":{"bundles":[],"borrowed":[],"regions":[]}"#),
    ] {
        let (text, root) = attach(&base, vec![support::json(&bad)]);
        assert!(verify(&text, &root, 0).is_err(), "invalid binding/field admitted: {bad}");
    }
    let (text, root) = attach(&base, vec![support::json(CLAIM)]);
    let mut o = Object::parse(&text).unwrap();
    o.functions.get_mut("a").unwrap().bounds.push(1);
    assert!(
        o.verify_claim(&root, 0).is_err(),
        "wrong source bounds established"
    );
}

const TOKEN_ACTION: &str = r#"{"abi":"0.9","args":["token"],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
const EMPTY_ORACLE: &str =
    r#"{"abi":"0.9","args":[],"fault":4,"in":[],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
const EMPTY_LPROC: &str = r#"{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[],"result":1,"tier":"q","tokens":[]}"#;
const LAYOUT: &str = "\nsection layout\n{\"A_tab\":0,\"H\":8,\"S_cells\":4,\"globals\":{},\"heap_base\":16,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";

fn with_claim(base: &Object, tokens: bool) -> (Object, String) {
    let claim = if tokens {
        CLAIM.replace(
            r#""budget":"all""#,
            r#""budget":"all","token_aliases":"all""#,
        )
    } else {
        CLAIM.into()
    };
    let (text, root) = attach(base, vec![support::json(&claim)]);
    (Object::parse(&text).unwrap(), root)
}
fn lifted_dependencies(lproc: bool) -> (Object, String) {
    let result = if lproc {
        "r = lift.call @worker ()"
    } else {
        "r = lift.const.i1 1"
    };
    let oracle = format!("oracle @z () (t:wire) (f:wire[4]) () ()\nregs {{ r:reg[1] }}\n^entry():\n{result}\nlift.retval r");
    let base = support::rehash(&support::object(&[
        ("a", TOKEN_ACTION, "perm_action @a (x:wire mut) () () (tk:token) ()\n^entry():\nf = qalloc any 4\noracle_call @z tk () x f () ()\noracle_call @z tk () x f () ()\nqfree f"),
        ("z", EMPTY_ORACLE, &oracle),
        ("worker", EMPTY_LPROC, "lproc @worker () (r:reg[1]) () () ()\n^entry():\nr = lift.const.i1 1\nlift.ret r"),
    ], "{}", LAYOUT).replace("0.8", "0.9"));
    with_claim(&base, true)
}
fn rejects_current_definition(o: &Object, root: &str, name: &str) {
    let verdict = o.verify_claim(root, 0);
    assert!(
        verdict.is_err(),
        "invalid linked {name} verdict: {verdict:?}"
    );
    let diagnostic = verdict.unwrap_err().to_string();
    assert!(
        diagnostic.contains("adjoint_table") && diagnostic.contains(name),
        "{diagnostic}"
    );
    assert!(
        o.verify_certificate().is_err(),
        "certificate proved invalid linked {name}"
    );
}

#[test]
fn invalid_current_linked_oracle_rejects_after_an_earlier_proof() {
    let (mut o, root) = lifted_dependencies(false);
    assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
    o.functions.get_mut("z").unwrap().blocks[0].instructions[0].op = "lift.magic".into();
    rejects_current_definition(&o, &root, "z");
    o.functions.get_mut("z").unwrap().blocks[0].instructions[0].op = "lift.const.i1".into();
    assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
}

#[test]
fn invalid_current_transitive_lproc_rejects_after_an_earlier_proof() {
    let (mut o, root) = lifted_dependencies(true);
    assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
    o.functions.get_mut("worker").unwrap().blocks[0].instructions[0].op = "lift.magic".into();
    rejects_current_definition(&o, &root, "worker");
}

#[test]
fn unreached_invalid_lproc_is_outside_the_premise_two_dependency_closure() {
    // K94 premise 2 explicitly names the dependency closure; worker is linked
    // here but z does not call it. This is not a whole-object validity verdict.
    let (mut o, root) = lifted_dependencies(false);
    o.functions.get_mut("worker").unwrap().blocks[0].instructions[0].op = "lift.magic".into();
    assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
    assert_eq!(
        o.verify_certificate().unwrap().claims[0].verdict,
        ClaimVerdict::Proof
    );
}

fn action_dependency(kind: &str) -> (Object, String) {
    let sig = format!(
        r#"{{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"{kind}","mem":[],"mut":[1],"tier":"q"}}"#
    );
    let root =
        format!("{kind} @a (x:wire mut) () () () ()\n^entry():\naction_call @z (x) () () ()");
    let child = format!("{kind} @z (x:wire mut) () () () ()\n^entry():\nx x[0]");
    let base = support::rehash(
        &support::object(&[("a", &sig, &root), ("z", &sig, &child)], "{}", "")
            .replace("0.8", "0.9"),
    );
    with_claim(&base, false)
}

#[test]
fn both_linked_action_kinds_revalidate_current_instructions() {
    for kind in ["action", "perm_action"] {
        let (mut o, root) = action_dependency(kind);
        assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
        o.functions.get_mut("z").unwrap().blocks[0].instructions[0].op = "qmagic".into();
        rejects_current_definition(&o, &root, "z");
    }
}

#[test]
fn both_linked_action_kinds_revalidate_current_signatures() {
    for kind in ["action", "perm_action"] {
        let (mut o, root) = action_dependency(kind);
        o.functions.get_mut("z").unwrap().params[0][0].role = Some("in".into());
        rejects_current_definition(&o, &root, "z");
    }
}

#[test]
fn linked_data_definition_matches_its_current_declared_signature() {
    let sig = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let base = support::rehash(
        &support::object_with_data(
            &[(
                "a",
                sig,
                "perm_action @a (x:wire mut) () () () ()\n^entry():\nqregion r 0 8 @image",
            )],
            &[("image", "00")],
            "",
        )
        .replace("0.8", "0.9"),
    );
    let (mut o, root) = with_claim(&base, false);
    assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
    let symbols = fields(o.metadata.get_mut("symbols").unwrap());
    let image = symbols
        .values_mut()
        .find(|v| v.get("name").unwrap().string().unwrap() == "image")
        .unwrap();
    fields(fields(image).get_mut("signature").unwrap()).insert("size".into(), support::json("2"));
    // The declaration changes the closure identity. Rebind the same claim to
    // that identity so lookup cannot mask the source-validity obligation.
    let new_root = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
    let certificate = o.metadata.get_mut("certificate").unwrap();
    let regions = fields(fields(certificate).get_mut("regions").unwrap());
    let record = regions.remove(&root).unwrap();
    regions.insert(new_root.clone(), record);
    let bindings = fields(fields(certificate).get_mut("bindings").unwrap());
    for key in ["closure_hash", "reference_program_hash"] {
        bindings.insert(key.into(), Json::String(new_root.clone()));
    }
    rejects_current_definition(&o, &new_root, "image");
}

#[test]
fn classical_function_kinds_cannot_be_quantum_dependencies() {
    // All function kinds are covered: action, perm_action, oracle and lproc
    // above; cc/r cannot be reached by a well-typed Q-region call.
    for kind in ["cc", "r"] {
        let (mut o, root) = action_dependency("perm_action");
        o.functions.get_mut("z").unwrap().kind = kind.into();
        assert!(
            o.verify_claim(&root, 0).is_err(),
            "Q call admitted {kind} callee"
        );
        assert!(
            o.verify_certificate().is_err(),
            "certificate admitted {kind} callee"
        );
    }
}

#[test]
fn renamed_definition_cannot_escape_dependency_closure_validation() {
    let (mut o, root) = action_dependency("perm_action");
    o.functions.get_mut("z").unwrap().name = "renamed".into();
    rejects_current_definition(&o, &root, "renamed");
}

#[test]
fn empty_current_lifted_bodies_cannot_inherit_parse_time_validity() {
    for lproc in [false, true] {
        let name = if lproc { "worker" } else { "z" };
        for remove_blocks in [false, true] {
            let (mut o, root) = lifted_dependencies(lproc);
            let f = o.functions.get_mut(name).unwrap();
            if remove_blocks {
                f.blocks.clear();
            } else {
                f.blocks[0].instructions.clear();
            }
            rejects_current_definition(&o, &root, name);
        }
    }
}

#[test]
fn loader_empty_body_diagnostics_remain_byte_identical() {
    for (suffix, diagnostic) in [
        ("\n^entry():", "§16.2: empty block"),
        ("", "§16.2: function requires blocks"),
    ] {
        let body = format!("oracle @z () (t:wire) (f:wire[4]) () ()\nregs {{ r:reg[1] }}{suffix}");
        let text = support::object(&[("z", EMPTY_ORACLE, &body)], "{}", "").replace("0.8", "0.9");
        assert_eq!(Object::parse(&text).unwrap_err().to_string(), diagnostic);
    }
}

fn zero_trip_source(body: &str) -> String {
    let sig = r#"{"abi":"0.9","args":[],"bounds":[0],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    support::object(&[("a", sig, &format!("perm_action @a (x:wire mut) () () () ()\nregs {{  }} bounds {{0}}\n^entry():\nqloop 0 {{\n{body}\n}}"))], "{}", "").replace("0.8", "0.9")
}
fn rejects_zero_trip_body(body: &str, tag: &str) {
    let text = zero_trip_source(body);
    match Object::parse(&text) {
        Err(e) => assert!(
            e.to_string().contains("§4.6") || e.to_string().contains("§3.5"),
            "{e}"
        ),
        Ok(_) => {
            let base = support::rehash(&text);
            base.check_hashes().unwrap();
            let path = std::path::PathBuf::from(std::env::var("TMPDIR").unwrap())
                .join(format!("zero-trip-{tag}.bisa"));
            std::fs::write(path, &base.text).unwrap();
            let (o, root) = with_claim(&base, false);
            let verdict = o.verify_claim(&root, 0);
            assert!(verdict.is_err(), "malformed zero-trip {tag}: {verdict:?}");
            assert!(o.verify_certificate().is_err());
        }
    }
}
#[test]
fn zero_trip_unmatched_allocation_rejects() {
    rejects_zero_trip_body("d = qalloc any 1", "allocation");
}
#[test]
fn zero_trip_unmatched_predicate_rejects() {
    rejects_zero_trip_body("qpred x", "predicate");
}
#[test]
fn zero_trip_balanced_lifetimes_and_version_changes_prove() {
    for body in ["d = qalloc any 1\nqfree d", "qpred x\nqunpred x", "x x[0]"] {
        let base = support::rehash(&zero_trip_source(body));
        let (o, root) = with_claim(&base, false);
        assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
        assert_eq!(
            o.verify_certificate().unwrap().claims[0].verdict,
            ClaimVerdict::Proof
        );
    }
}

#[test]
fn zero_trip_same_name_same_type_rebinding_proves() {
    let mut text = zero_trip_source("qfree d\nd = qalloc any 1")
        .replace("^entry():\n", "^entry():\nd = qalloc any 1\n");
    text.push_str("qfree d\n");
    let base = support::rehash(&text);
    let (o, root) = with_claim(&base, false);
    assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
    assert_eq!(
        o.verify_certificate().unwrap().claims[0].verdict,
        ClaimVerdict::Proof
    );
}

#[test]
fn legacy08_zero_trip_bindings_keep_published_admission() {
    for (text, _, _) in [
        support::r6_witness("zero-trip"),
        support::r8_witness("zero-wide"),
    ] {
        let o = Object::parse(&text).unwrap();
        o.check_hashes().unwrap();
    }
}

#[test]
fn current_zero_trip_exit_premises_reject_through_both_apis() {
    for body in ["d = qalloc any 1\nqfree d", "qpred x\nqunpred x"] {
        let base = support::rehash(&zero_trip_source(body));
        let (mut o, root) = with_claim(&base, false);
        o.functions.get_mut("a").unwrap().blocks[0].instructions[0].bodies[0].pop();
        rejects_current_definition(&o, &root, "a");
    }
}

fn angle_object(forward: bool, used: bool) -> Object {
    let sig = r#"{"abi":"0.9","args":["f64"],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let root_body = if forward {
        "action_call @z (x) () () (theta)"
    } else if used {
        "rx theta x[0]"
    } else {
        "x x[0]"
    };
    let root = format!("action @a (x:wire mut) () () (theta:f64) ()\n^entry():\n{root_body}");
    let child = "action @z (x:wire mut) () () (phi:f64) ()\n^entry():\nrz phi x[0]";
    let mut functions = vec![("a", sig, root.as_str())];
    if forward {
        functions.push(("z", sig, child));
    }
    support::rehash(&support::object(&functions, "{}", "").replace("0.8", "0.9"))
}
fn angle_claim(bits: u64) -> Json {
    support::json(&CLAIM.replace(
        r#""args":"all","budget":"all""#,
        &format!(r#""args":["0x{bits:016x}"],"budget":["0","0","0","0"]"#),
    ))
}
#[test]
fn nonfinite_concrete_rotation_angles_reject() {
    for bits in [0x7ff0000000000000, 0xfff0000000000000, 0x7ff8000000000001] {
        let (text, root) = attach(&angle_object(false, true), vec![angle_claim(bits)]);
        let o = Object::parse(&text).unwrap();
        o.check_hashes().unwrap();
        rejects_current_definition(&o, &root, "a");
    }
}
#[test]
fn nonfinite_forwarded_rotation_angles_reject() {
    for bits in [0x7ff0000000000000, 0x7ff8000000000001] {
        let (text, root) = attach(&angle_object(true, true), vec![angle_claim(bits)]);
        let o = Object::parse(&text).unwrap();
        o.check_hashes().unwrap();
        rejects_current_definition(&o, &root, "z");
    }
}
#[test]
fn finite_symbolic_and_unused_nonfinite_f64_arguments_prove() {
    for forward in [false, true] {
        let base = angle_object(forward, true);
        for claim in [
            angle_claim(0),
            angle_claim(0x3ff0000000000000),
            support::json(CLAIM),
        ] {
            let (text, root) = attach(&base, vec![claim]);
            let o = Object::parse(&text).unwrap();
            assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
            assert_eq!(
                o.verify_certificate().unwrap().claims[0].verdict,
                ClaimVerdict::Proof
            );
        }
    }
    let (text, root) = attach(
        &angle_object(false, false),
        vec![angle_claim(0x7ff8000000000001)],
    );
    assert_eq!(verify(&text, &root, 0).unwrap(), ClaimVerdict::Proof);
}

#[test]
fn reached_zero_width_register_rejects_current_type() {
    use bennett_isa_ref::{Instruction, Param, Type};
    let (mut o, root) = lifted_dependencies(true);
    assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
    let worker = o.functions.get_mut("worker").unwrap();
    worker.regs.push(Param {
        name: "zero".into(),
        ty: Type::Reg(0),
        role: None,
    });
    worker.blocks[0].instructions.insert(
        0,
        Instruction {
            lhs: vec!["zero".into()],
            op: "lift.mov".into(),
            operands: vec!["zero".into()],
            bodies: vec![],
            line: 0,
        },
    );
    rejects_current_definition(&o, &root, "worker");
}
#[test]
fn implicit_oracle_layout_data_signature_rejects() {
    let layout = "\nsection layout\n{\"A_tab\":1,\"H\":8,\"S_cells\":0,\"globals\":{\"g\":{\"base\":1,\"image\":\"g\",\"size\":1}},\"heap_base\":16,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";
    let base = support::rehash(&support::object_with_data(&[
        ("a", TOKEN_ACTION, "perm_action @a (x:wire mut) () () (tk:token) ()\n^entry():\nf = qalloc any 4\noracle_call @z tk () x f () ()\noracle_call @z tk () x f () ()\nqfree f"),
        ("z", EMPTY_ORACLE, "oracle @z () (t:wire) (f:wire[4]) () ()\nregs { r:reg[1] }\n^entry():\nr = lift.const.i1 1\nlift.retval r"),
    ], &[("g", "00")], layout).replace("0.8", "0.9"));
    let (mut o, root) = with_claim(&base, true);
    assert_eq!(o.verify_claim(&root, 0).unwrap(), ClaimVerdict::Proof);
    let symbols = fields(o.metadata.get_mut("symbols").unwrap());
    let g = symbols
        .values_mut()
        .find(|s| s.get("name").unwrap().string().unwrap() == "g")
        .unwrap();
    fields(fields(g).get_mut("signature").unwrap()).insert("size".into(), support::json("2"));
    let new_root = o.identity().unwrap().closure_hashes["a"].clone().unwrap();
    let certificate = o.metadata.get_mut("certificate").unwrap();
    let regions = fields(fields(certificate).get_mut("regions").unwrap());
    let record = regions.remove(&root).unwrap();
    regions.insert(new_root.clone(), record);
    rejects_current_definition(&o, &new_root, "g");
}
