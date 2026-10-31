//! Council round 1: codex B17 (§9.3 linking composes certificates, §12.3).
mod support;
use bennett_isa_ref::{json::Json, Argument, Budget, Object, OracleInput};
use support::json;

const INC: &str = include_str!("fixtures/inc8.bisa");
const INC_HASH: &str = "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93";
const INC_SIG: &str =
    r#"{"abi":"0.8","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#;
const CLIENT_SIG: &str = r#"{"abi":"0.8","args":["token"],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#;

fn region(signature: &str, error: &str) -> String {
    format!(
        r#"{{"kind":"oracle","signature":{signature},"bounds":[],"frame":{{"interface":[{{"role":"in","width":8,"class":"any"}},{{"role":"tgt","width":8,"class":"any"}},{{"role":"fault","width":4,"class":"any"}}],"scratch":{{}},"retained_history":0,"borrow":{{}},"scheme_ancillae":{{}}}},"resources":{{}},"gates_used":{{}},"claims":[],"verification":{{"oracle":null,"borrow_frames":[],"inverse_pairs":[],"unitary":[],"channel":[],"fault_rate":{{"domain":{{"bundles":[],"borrowed":[],"regions":[]}},"faults_by_code":{{}},"undefined_excluded":0,"layout_dependent_excluded":0}}}},"cleanup":{{}},"error":{error},"definedness_assumptions":[]}}"#
    )
}
fn certify(text: &str, root: &str, regions: &str, multiplicities: &str, dialect: &str) -> Object {
    let o = Object::parse(text).unwrap();
    let id = o.identity().unwrap();
    let closure = id.closure_hashes[root].clone().unwrap();
    let cert = json(&format!(
        r#"{{"versions":{{"isa":"0.8","object":"0.8","launch":"0.8","sched":"0.8","cert":"0.8","coverage":"0.8","target":"0.8"}},"bindings":{{"object_hash":"{}","interface_hash":"{}","closure_hash":"{closure}","reference_program_hash":"{}","dialect_mapping_version":"{dialect}","target_hash":"{}"}},"producer":{{"name":"tests","version":"1","materialisation":{{"llvm_version":"none","pipeline":"none","normaliser_version":"1"}}}},"regions":{{{}}},"composition":{{"multiplicities":{{{multiplicities}}},"epsilon_total":{{"diamond":0}},"method":"sum_over_multiplicities"}}}}"#,
        id.object_hash,
        id.interface_hashes[root],
        id.interface_hashes[root],
        "3".repeat(64),
        regions.replace("ROOT", &closure)
    ));
    Object::parse(&format!(
        "{text}\nsection certificate\n{}\n",
        cert.canonical()
    ))
    .unwrap()
}
fn client_text() -> String {
    let imports =
        format!(r#"{{"inc8":{{"pinned_closure_hash":"{INC_HASH}","signature":{INC_SIG}}}}}"#);
    let body = "oracle @client (x:wire[8] in) (t:wire[8]) (f:wire[4]) () (tk:token)\nregs { r:reg[8], fr:reg[4] }\n^entry():\nlift.oracle_call @inc8 tk (x) r fr () ()\nlift.retval r";
    support::rehash(&support::object(
        &[("client", CLIENT_SIG, body)],
        &imports,
        "",
    ))
    .text
}
fn callee(dialect: &str) -> Object {
    certify(
        INC,
        "inc8",
        &format!(
            r#""ROOT":{}"#,
            region(
                INC_SIG,
                r#"{"forward:0":{"local":{"operator_norm":"0.25"},"inclusive":{"operator_norm":"0.25"}}}"#
            )
        ),
        "",
        dialect,
    )
}
fn client(dialect: &str) -> Object {
    let text = client_text();
    certify(
        &text,
        "client",
        &format!(
            r#""ROOT":{}"#,
            region(
                CLIENT_SIG,
                r#"{"forward:0":{"local":{"diamond":"0.1"},"inclusive":{"diamond":"1.6"}}}"#
            )
        ),
        &format!(r#""{INC_HASH}:forward:0":3"#),
        dialect,
    )
}

#[test]
fn b17_certified_client_and_callee_link_and_compose() {
    // The pinned import makes the client's closure hash known before linking.
    let c = client("test.1");
    assert!(Object::parse(&c.text)
        .unwrap()
        .identity()
        .unwrap()
        .closure_hashes["client"]
        .is_some());
    let linked = Object::link(&[c, callee("test.1")]).unwrap();
    linked.check_hashes().unwrap();
    let cert = linked.section("certificate").unwrap();
    let regions = cert.get("regions").unwrap().object().unwrap();
    assert_eq!(regions.len(), 2, "both region records are retained");
    let identity = linked.identity().unwrap();
    for name in ["client", "inc8"] {
        assert!(regions.contains_key(identity.closure_hashes[name].as_deref().unwrap()));
    }
    // epsilon_total = 3 x (0.25 x 2) + 0.1 = 1.6 (§12.3 sum over multiplicities).
    assert_eq!(
        cert.get("composition")
            .unwrap()
            .get("epsilon_total")
            .unwrap()
            .get("diamond")
            .unwrap(),
        &Json::String("1.6".into())
    );
    let mut input = OracleInput::bits(&[8]);
    input.args.push(Argument::Token(Budget::steps(3)));
    let r = linked
        .run_oracle("client", &input, Budget::steps(2))
        .unwrap();
    assert_eq!((r.payload, r.code, r.steps), (9, 0, 2));
}
#[test]
fn b17_incompatible_certificates_are_link_errors() {
    let e = Object::link(&[client("test.1"), callee("other.2")])
        .unwrap_err()
        .to_string();
    assert!(e.contains("§9.3"), "{e}");
}
#[test]
fn interface_region_names_cannot_alias_layout_globals() {
    // SPEC-QUESTIONS 32: only actual global/interface address ambiguity rejects.
    let text = include_str!("fixtures/sum_db.bisa")
        .replace("(db:region[4,8] in)", "(zero:region[4,8] in)")
        .replace("base = lift.addr @db", "base = lift.addr @zero")
        .replace("\"sum_db\":{\"db\":", "\"sum_db\":{\"zero\":");
    let e = Object::parse(&text).unwrap_err().to_string();
    assert!(e.contains("§9.2"), "{e}");
}

#[test]
fn r8_unsigned_multiplicity_composition() {
    let callee = certify(
        INC,
        "inc8",
        &format!(
            r#""ROOT":{}"#,
            region(
                INC_SIG,
                r#"{"forward:0":{"local":{"diamond":1},"inclusive":{"diamond":1}}}"#
            )
        ),
        "",
        "test.1",
    );
    let client = certify(
        &client_text(),
        "client",
        &format!(
            r#""ROOT":{}"#,
            region(
                CLIENT_SIG,
                r#"{"forward:0":{"local":{"diamond":1},"inclusive":{"diamond":"340282366920938463463374607431768211456"}}}"#
            )
        ),
        &format!(r#""{INC_HASH}:forward:0":"340282366920938463463374607431768211455""#),
        "test.1",
    );
    let linked = Object::link(&[client, callee]).unwrap();
    assert_eq!(
        linked
            .section("certificate")
            .unwrap()
            .get("composition")
            .unwrap()
            .get("epsilon_total")
            .unwrap()
            .get("diamond")
            .unwrap(),
        &Json::String("340282366920938463463374607431768211456".into())
    );
    linked.check_hashes().unwrap();
}

#[test]
fn r12_interface_region_may_share_function_name() {
    use bennett_isa_ref::{Budget, OracleInput};
    for region in ["o", "db"] {
        let o = Object::parse(&support::r12_address_text(region)).unwrap();
        o.check_hashes().unwrap();
        let mut input = OracleInput::bits(&[]);
        input.regions.insert(region.into(), vec![0]);
        for (steps, payload, code) in [(2, 16, 0), (1, 0, 1)] {
            let r = o.run_oracle("o", &input, Budget::steps(steps)).unwrap();
            assert_eq!((r.payload, r.code, r.steps), (payload, code, steps));
        }
    }
}

#[test]
fn r12_non_declaring_oracle_cannot_resolve_a_foreign_interface() {
    let raw = support::r12_address_text("o");
    let mut o = Object::parse(&raw).unwrap();
    let mut syms = o.section("symbols").unwrap().clone();
    let mut other = syms.get("0").unwrap().clone();
    if let Json::Object(s) = &mut other {
        s.insert("name".into(), Json::String("other".into()));
    }
    if let Json::Object(s) = &mut syms {
        s.insert("1".into(), other);
    }
    let mut layout = o.section("layout").unwrap().clone();
    if let Json::Object(l) = &mut layout {
        if let Json::Object(regions) = l.get_mut("interface_regions").unwrap() {
            regions.insert("other".into(), support::json(r#"{"db":{"base":16}}"#));
        }
    }
    let body = "oracle @other () (t:wire[32]) (f:wire[4]) (db:region[0,8] in) ()\nregs { p:reg[32] }\n^entry():\np = lift.addr @o\nlift.retval p\n";
    o.text = raw
        .replace(raw.lines().nth(3).unwrap(), &syms.canonical())
        .replace("\nsection layout\n", &format!("{body}\nsection layout\n"));
    o.text = o
        .text
        .replace(raw.lines().last().unwrap(), &layout.canonical());
    let e = Object::parse(&o.text).unwrap_err().to_string();
    assert!(e.contains("§4.2") && e.contains("in other"), "{e}");
}
