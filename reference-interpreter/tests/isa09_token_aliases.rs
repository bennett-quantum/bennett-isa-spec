mod support;
use bennett_isa_ref::{json::Json, ClaimVerdict, Object};
use std::collections::BTreeMap;
const BASE: &str = include_str!("../../conformance/fixtures/accepted-k84-09/baseline.bisa");
fn fields(value: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(fields) = value else {
        panic!("object")
    };
    fields
}
fn certificate_object(aliases: Option<&str>, second: u64) -> String {
    let (source, certificate) = BASE.split_once("section certificate\n").unwrap();
    let source = source.trim_end().to_string() + "\n";
    let mut symbols = Object::parse(&source)
        .unwrap()
        .section("symbols")
        .unwrap()
        .clone();
    let symbol = fields(&mut symbols).get_mut("0").unwrap();
    fields(fields(symbol).get_mut("signature").unwrap())
        .insert("args".into(), support::json(r#"["token","token"]"#));
    let mut lines = source.lines().map(str::to_string).collect::<Vec<_>>();
    let section = lines.iter().position(|s| s == "section symbols").unwrap();
    lines[section + 1] = symbols.canonical();
    let source = (lines.join("\n") + "\n").replace(
        "(t:wire mut) () () () ()",
        "(t:wire mut) () () (a:token, b:token) ()",
    );
    let object = support::rehash(&source);
    let identity = object.identity().unwrap();
    let closure = identity.closure_hashes["flip"].as_ref().unwrap();
    let mut c = Json::parse(certificate.trim()).unwrap();
    let bindings = fields(fields(&mut c).get_mut("bindings").unwrap());
    for (key, value) in [
        ("object_hash", &identity.object_hash),
        ("interface_hash", &identity.interface_hashes["flip"]),
        ("closure_hash", closure),
        ("reference_program_hash", closure),
    ] {
        bindings.insert(key.into(), Json::String(value.clone()));
    }
    let mut invocation = support::json(&format!(
        r#"{{"args":[["1","0","0","0"],["{second}","0","0","0"]],"budget":["0","0","0","0"]}}"#
    ));
    if let Some(aliases) = aliases {
        fields(&mut invocation).insert("token_aliases".into(), support::json(aliases));
    }
    bindings.insert("invocation".into(), invocation.clone());
    let regions = fields(fields(&mut c).get_mut("regions").unwrap());
    let mut region = regions.values().next().unwrap().clone();
    fields(&mut region).insert(
        "signature".into(),
        object.functions["flip"].signature_for(object.isa_version),
    );
    let Json::Array(claims) = fields(&mut region).get_mut("claims").unwrap() else {
        panic!("claims")
    };
    fields(&mut claims[0]).insert("invocation".into(), invocation);
    fields(fields(&mut claims[0]).get_mut("subject").unwrap())
        .insert("closure_hash".into(), Json::String(closure.clone()));
    regions.clear();
    regions.insert(closure.clone(), region);
    fields(fields(&mut c).get_mut("composition").unwrap()).insert(
        "multiplicities".into(),
        support::json(&format!(r#"{{"{closure}:forward:0":1}}"#)),
    );
    format!("{}\nsection certificate\n{}\n", object.text, c.canonical())
}

#[test]
fn certificate_invocations_admit_both_equal_tuple_identity_partitions() {
    let mut hashes = Vec::new();
    for labels in ["[0,0]", "[0,1]"] {
        let object = Object::parse(&certificate_object(Some(labels), 1)).unwrap();
        object.check_hashes().unwrap();
        hashes.push(object.identity().unwrap().object_hash);
        let certificate = object.section("certificate").unwrap();
        let closure = certificate
            .get("bindings")
            .unwrap()
            .get("closure_hash")
            .unwrap()
            .string()
            .unwrap();
        assert_eq!(
            object.verify_claim(closure, 0).unwrap(),
            ClaimVerdict::Proof
        );
        assert_eq!(
            object.verify_certificate().unwrap()[0].verdict,
            ClaimVerdict::Proof
        );
    }
    assert_eq!(
        hashes[0], hashes[1],
        "partition metadata does not change object identity"
    );
}

#[test]
fn missing_malformed_and_tuple_inconsistent_certificate_partitions_reject() {
    for (labels, second) in [
        (None, 1),
        (Some("[1,1]"), 1),
        (Some("[0,2]"), 1),
        (Some("[0]"), 1),
        (Some("[0,0]"), 2),
        (Some("[-1,0]"), 1),
    ] {
        let result = Object::parse(&certificate_object(labels, second));
        assert!(result.is_err(), "admitted {labels:?}/{second}");
        assert!(result.unwrap_err().to_string().contains("K91"));
    }
}

#[test]
fn uniform_token_claims_quantify_over_every_consistent_partition() {
    let original = certificate_object(Some("[0,1]"), 1);
    let (source, c) = original.split_once("section certificate\n").unwrap();
    for aliases in [false, true] {
        let mut certificate = Json::parse(c.trim()).unwrap();
        let regions = fields(fields(&mut certificate).get_mut("regions").unwrap());
        let region = regions.values_mut().next().unwrap();
        let mut claim = support::json(
            r#"{"kind":"structural","rule":"adjoint_table","subclaims":[],"witness":{},"invocation":{"args":"all","budget":"all"}}"#,
        );
        if aliases {
            fields(fields(&mut claim).get_mut("invocation").unwrap())
                .insert("token_aliases".into(), Json::String("all".into()));
        }
        fields(region).insert("claims".into(), Json::Array(vec![claim]));
        let result = Object::parse(&format!(
            "{source}section certificate\n{}\n",
            certificate.canonical()
        ));
        assert_eq!(result.is_ok(), aliases);
        if !aliases {
            assert!(result.unwrap_err().to_string().contains("K91"));
        }
    }
}

fn oracle_source(token: bool) -> Object {
    let signature = format!(
        r#"{{"abi":"0.8","args":{},"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}}"#,
        if token { r#"["token"]"# } else { "[]" }
    );
    let body = format!(
        "oracle @id (x:wire in) (t:wire) (f:wire[4]) () {}\n^entry():\nlift.retval x",
        if token { "(a:token)" } else { "()" }
    );
    support::rehash(
        &support::object(&[("id", &signature, &body)], "{}", "")
            .replace("=0.8", "=0.9")
            .replace("\"0.8\"", "\"0.9\""),
    )
}
fn scheduled(source: &Object, aliases: Option<&str>) -> String {
    let name = &source.function_order[0];
    let closure = source.identity().unwrap().closure_hashes[name]
        .clone()
        .unwrap();
    let mut schedule = support::json(&format!(
        r#"{{"bindings":{{"budget":[1,0,0,0],"closure_hash":"{closure}","epsilon":0,"inherited_controls":0,"params_hash":"{}","region":"{name}","scheduler":{{"name":"k91","version":"1"}},"target_hash":"{}","variant":"forward"}},"entries":[]}}"#,
        bennett_isa_ref::sha256_hex(b"[]"),
        "0".repeat(64)
    ));
    if let Some(aliases) = aliases {
        fields(fields(&mut schedule).get_mut("bindings").unwrap())
            .insert("token_aliases".into(), support::json(aliases));
    }
    format!(
        "{}\nsection schedule\n{}\n",
        source.text,
        schedule.canonical()
    )
}

#[test]
fn schedule_alias_slots_prepend_primary_only_for_oracles() {
    let action = Object::parse(
        &(BASE
            .split_once("section certificate\n")
            .unwrap()
            .0
            .trim_end()
            .to_string()
            + "\n"),
    )
    .unwrap();
    for (source, valid, invalid) in [(&action, "[]", "[0]"), (&oracle_source(false), "[0]", "[]")] {
        Object::parse(&scheduled(source, Some(valid)))
            .unwrap()
            .check_hashes()
            .unwrap();
        Object::parse(&scheduled(source, None)).unwrap(); // legacy token-free binding
        assert!(Object::parse(&scheduled(source, Some(invalid)))
            .unwrap_err()
            .to_string()
            .contains("K91"));
    }
    let source = oracle_source(true);
    for aliases in ["[0,0]", "[0,1]"] {
        Object::parse(&scheduled(&source, Some(aliases))).unwrap();
    }
    let error = Object::parse(&scheduled(&source, None)).unwrap_err();
    assert!(error.to_string().contains("K91"));
}

#[test]
fn a_schedule_certificate_cannot_reuse_equal_params_under_another_partition() {
    let original = certificate_object(Some("[0,0]"), 1);
    let object = Object::parse(&original).unwrap();
    let (source, raw) = original.split_once("section certificate\n").unwrap();
    let mut certificate = Json::parse(raw.trim()).unwrap();
    let binding = certificate.get("bindings").unwrap();
    let hash = binding.get("closure_hash").unwrap().string().unwrap();
    let params = bennett_isa_ref::sha256_hex(
        binding
            .get("invocation")
            .unwrap()
            .get("args")
            .unwrap()
            .canonical()
            .as_bytes(),
    );
    let schedule = support::json(&format!(
        r#"{{"bindings":{{"budget":[0,0,0,0],"closure_hash":"{hash}","epsilon":0,"inherited_controls":0,"params_hash":"{params}","region":"flip","scheduler":{{"name":"k91","version":"1"}},"target_hash":"{}","token_aliases":[0,0],"variant":"forward"}},"entries":[]}}"#,
        "0".repeat(64)
    ));
    fields(fields(&mut certificate).get_mut("bindings").unwrap()).insert(
        "schedule_hash".into(),
        Json::String(bennett_isa_ref::sha256_hex(schedule.canonical().as_bytes())),
    );
    let render = |c: &Json| {
        format!(
            "{}\n\nsection schedule\n{}\n\nsection certificate\n{}\n",
            source.trim_end(),
            schedule.canonical(),
            c.canonical()
        )
    };
    let scheduled = Object::parse(&render(&certificate)).unwrap();
    assert_eq!(
        scheduled.identity().unwrap().object_hash,
        object.identity().unwrap().object_hash
    );
    let invocation = fields(fields(&mut certificate).get_mut("bindings").unwrap())
        .get_mut("invocation")
        .unwrap();
    fields(invocation).insert("token_aliases".into(), support::json("[0,1]"));
    let error = Object::parse(&render(&certificate)).unwrap_err();
    assert!(
        error
            .to_string()
            .contains("invocation and schedule token_aliases differ"),
        "{error}"
    );
}

fn identity_pair_source() -> Object {
    let action = r#"{"abi":"0.9","args":["token","token"],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[1,4],"tier":"q"}"#;
    let oracle =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    support::rehash(&support::object(&[
        ("pair",action,"perm_action @pair (t:wire mut, f:wire[4] mut) (x:wire in) () (a:token, b:token) ()\n^entry():\noracle_call @id a (x) t f () ()\noracle_call @id b (x) t f () ()"),
        ("id",oracle,"oracle @id (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x")
    ],"{}", "").replace("=0.8","=0.9").replace("ptr_width=32","ptr_width=2"))
}
fn pair_schedule(source: &Object, aliases: &str, entries: &[Json]) -> String {
    let closure = source.identity().unwrap().closure_hashes["pair"]
        .clone()
        .unwrap();
    let params = bennett_isa_ref::sha256_hex(br#"[["1","0","0","0"],["1","0","0","0"]]"#);
    let schedule = support::json(&format!(
        r#"{{"bindings":{{"budget":[0,0,0,0],"closure_hash":"{closure}","epsilon":0,"inherited_controls":0,"params_hash":"{params}","region":"pair","scheduler":{{"name":"k91","version":"1"}},"target_hash":"{}","token_aliases":{aliases},"variant":"forward"}},"entries":{}}}"#,
        "0".repeat(64),
        Json::Array(entries.to_vec()).canonical()
    ));
    format!(
        "{}\nsection schedule\n{}\n",
        source.text,
        schedule.canonical()
    )
}
fn pair_cleanup(source: &Object) -> Json {
    let symbol = &source.identity().unwrap().interface_hashes["pair"];
    support::json(&format!(
        r#"{{"cleanup":{{"realisation":"recompute"}},"generated":0,"kind":"cleanup","path":[{{"instance":[],"iteration":[],"point":"entry:1","symbol":"{symbol}"}}],"phase":"*","step":"*"}}"#
    ))
}
#[test]
fn equal_identity_has_the_published_carrier_but_equal_tuples_alone_do_not() {
    let source = identity_pair_source();
    let entry = pair_cleanup(&source);
    let entries = [entry.clone(), entry];
    // A tie exists exactly on a real K65 carrier. Neither call's unitary
    // truth table nor equality of the two tuples determines this partition.
    for (aliases, carrier) in [("[0,0]", true), ("[0,1]", false)] {
        let result = Object::parse(&pair_schedule(&source, aliases, &entries));
        assert_eq!(result.is_err(), carrier, "{aliases}: {result:?}");
        if carrier {
            assert!(result.unwrap_err().to_string().contains("multiple maximal"));
        }
    }
}

#[test]
fn tuple_only_source_token_arguments_require_an_identity_binding() {
    let source = identity_pair_source();
    let args = bennett_isa_conformance::json::parse(
        r#"["t","f","x",["1","0","0","0"],["1","0","0","0"]]"#,
    )
    .unwrap();
    let result = source.expand_action_source(
        "pair",
        args.as_arr().unwrap(),
        &[("t", 1), ("f", 4), ("x", 1)],
        false,
    );
    assert!(
        result.is_err(),
        "source adapter admitted an unspecified partition"
    );
    assert!(result.unwrap_err().to_string().contains("K91"));
}

fn expand_pair(
    source: &Object,
    aliases: &[u64],
    adjoint: bool,
) -> Vec<bennett_isa_conformance::ref_language::Leaf> {
    let args = bennett_isa_conformance::json::parse(
        r#"["t","f","x",["1","0","0","0"],["1","0","0","0"]]"#,
    )
    .unwrap();
    source
        .expand_action_source_with_token_aliases(
            "pair",
            args.as_arr().unwrap(),
            &[("t", 1), ("f", 4), ("x", 1)],
            adjoint,
            aliases,
        )
        .unwrap()
}
#[test]
fn source_cleanup_decisions_require_the_actual_root_partition() {
    let source = identity_pair_source();
    let entry = pair_cleanup(&source);
    let default = expand_pair(&source, &[0, 0], false);
    let wrong = Object::parse(&pair_schedule(&source, "[0,1]", &[entry.clone()])).unwrap();
    let leaves = expand_pair(&wrong, &[0, 0], false);
    assert_eq!(
        leaves.len(),
        default.len(),
        "a different partition uses defaults"
    );
    let matched = Object::parse(&pair_schedule(&source, "[0,0]", &[entry])).unwrap();
    let leaves = expand_pair(&matched, &[0, 0], false);
    assert!(
        leaves.len() > default.len(),
        "same identity selects real recomputation"
    );
}

#[test]
fn source_captures_keep_identity_and_restore_all_sixty_four_basis_tuples() {
    use bennett_isa_conformance::{
        ref_catalog::Machine,
        ref_language::{invert_trace, permutation},
    };
    let source = identity_pair_source();
    for (labels, partition) in [("[0,0]", vec![0, 0]), ("[0,1]", vec![0, 1])] {
        let scheduled =
            Object::parse(&pair_schedule(&source, labels, &[pair_cleanup(&source)])).unwrap();
        for adjoint in [false, true] {
            let leaves = expand_pair(&scheduled, &partition, adjoint);
            for leaf in &leaves {
                let location = leaf.source.as_ref().unwrap();
                let root = &location.invocations[0];
                let tokens = root.tokens.as_ref().unwrap();
                assert!(
                    tokens.primary.is_none(),
                    "an action has no primary identity"
                );
                assert_eq!(
                    tokens.arguments["a"] == tokens.arguments["b"],
                    partition[0] == partition[1]
                );
                assert_eq!(root.binding.as_ref().unwrap().token_aliases, partition);
                if let Some(child) = location.invocations.get(1) {
                    assert_eq!(child.binding.as_ref().unwrap().token_aliases, [0]);
                    let token = child.tokens.as_ref().unwrap().primary.unwrap();
                    assert!(tokens.arguments.values().any(|id| *id == token));
                }
            }
            let mut inverse = leaves.clone();
            invert_trace(&mut inverse).unwrap();
            for (a, b) in inverse.iter().zip(leaves.iter().rev()) {
                assert_eq!(a.source, b.source);
            }
            let bundles = [("x", 1), ("t", 1), ("f", 4)];
            let machine = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
            let inv = Machine::compile(&permutation(&inverse).unwrap(), &bundles).unwrap();
            for m in [&machine, &inv] {
                let mut state = m.packed_state();
                for lane in 0..64 {
                    m.set_lane(&mut state, "x", (lane & 1) as u64, lane);
                    m.set_lane(&mut state, "t", ((lane >> 1) & 1) as u64, lane);
                    m.set_lane(&mut state, "f", (lane >> 2) as u64, lane);
                }
                let entry = state.clone();
                m.run_packed(&mut state, u64::MAX, false).unwrap();
                assert_eq!(
                    state, entry,
                    "{labels}, adjoint={adjoint}: complete identity"
                );
                m.run_packed(&mut state, u64::MAX, true).unwrap();
                assert_eq!(state, entry);
            }
        }
    }
}

#[test]
fn source_tuple_adapters_reject_malformed_bindings_before_construction() {
    let source = identity_pair_source();
    for token in [
        r#"[]"#,
        r#"["1","0","0"]"#,
        r#"[1,"0","0","0"]"#,
        r#"["01","0","0","0"]"#,
    ] {
        let args = bennett_isa_conformance::json::parse(&format!(
            r#"["t","f","x",{token},["1","0","0","0"]]"#
        ))
        .unwrap();
        assert!(source
            .expand_action_source_with_token_aliases(
                "pair",
                args.as_arr().unwrap(),
                &[("t", 1), ("f", 4), ("x", 1)],
                false,
                &[0, 0]
            )
            .is_err());
    }
    let args = bennett_isa_conformance::json::parse(
        r#"["t","f","x",["1","0","0","0"],["1","0","0","0"]]"#,
    )
    .unwrap();
    for labels in [vec![1, 1], vec![0, 2], vec![0], vec![]] {
        assert!(source
            .expand_action_source_with_token_aliases(
                "pair",
                args.as_arr().unwrap(),
                &[("t", 1), ("f", 4), ("x", 1)],
                false,
                &labels
            )
            .is_err());
    }
}

#[test]
fn a_token_free_oracle_uses_defaults_for_a_different_primary_budget() {
    use bennett_isa_ref::{Budget, OracleInput, OracleMachine};
    let source = oracle_source(false);
    let raw = scheduled(&source, Some("[0]"));
    let (body, raw) = raw.split_once("section schedule\n").unwrap();
    let mut schedule = Json::parse(raw.trim()).unwrap();
    fields(fields(&mut schedule).get_mut("bindings").unwrap())
        .insert("budget".into(), support::json("[2,0,0,0]"));
    let symbol = &source.identity().unwrap().interface_hashes["id"];
    let entry = support::json(&format!(
        r#"{{"gate":{{"epsilon_share":0,"realisation":"native"}},"generated":"*","kind":"gate","path":[{{"instance":[],"iteration":[],"point":"entry:0","symbol":"{symbol}"}}],"phase":"*","step":3}}"#
    ));
    fields(&mut schedule).insert("entries".into(), Json::Array(vec![entry.clone(), entry]));
    let object = Object::parse(&format!(
        "{body}section schedule\n{}\n",
        schedule.canonical()
    ))
    .unwrap();
    for x in 0..2 {
        let input = OracleInput::bits(&[x]);
        assert_eq!(
            object
                .run_oracle("id", &input, Budget::steps(3))
                .unwrap()
                .payload,
            x
        );
        let mut machine = OracleMachine::new(&object, "id", &input, Budget::steps(3)).unwrap();
        let before = machine.snapshot();
        for _ in 0..3 {
            machine.forward_step().unwrap();
        }
        assert_eq!(machine.outcome().unwrap().payload, x);
        for _ in 0..3 {
            machine.inverse_step().unwrap();
        }
        assert_eq!(machine.snapshot(), before);
    }
}

#[test]
fn tuple_only_native_oracle_arguments_require_an_identity_binding() {
    use bennett_isa_ref::{Argument, Budget, OracleInput, OracleMachine};
    let object = oracle_source(true);
    let mut input = OracleInput::bits(&[1]);
    input.args.push(Argument::Token(Budget::steps(1)));
    let result = object.run_oracle("id", &input, Budget::steps(1));
    assert!(
        result.is_err(),
        "native tuple adapter admitted unspecified primary/argument equality"
    );
    assert!(result.unwrap_err().to_string().contains("K91"));
    assert!(OracleMachine::new(&object, "id", &input, Budget::steps(1)).is_err());
}

#[test]
fn native_oracle_partitions_bind_the_primary_and_preserve_actual_handles() {
    use bennett_isa_ref::{Argument, Budget, OracleInput, OracleMachine};
    let object = oracle_source(true);
    for aliases in [&[0, 0][..], &[0, 1][..]] {
        for x in 0..2 {
            let mut input = OracleInput::bits(&[x]);
            input.args.push(Argument::Token(Budget::steps(1)));
            assert_eq!(
                object
                    .run_oracle_with_token_aliases("id", &input, Budget::steps(1), aliases)
                    .unwrap()
                    .payload,
                x
            );
            let mut machine = OracleMachine::new_with_token_aliases(
                &object,
                "id",
                &input,
                Budget::steps(1),
                aliases,
            )
            .unwrap();
            let before = machine.snapshot();
            machine.forward_step().unwrap();
            assert_eq!(machine.outcome().unwrap().payload, x);
            machine.inverse_step().unwrap();
            assert_eq!(machine.snapshot(), before);
        }
    }
    let primary = object.budget_token("id", Budget::steps(1)).unwrap();
    let distinct = object.budget_token("id", Budget::steps(1)).unwrap();
    for token in [&primary, &distinct] {
        let mut input = OracleInput::bits(&[1]);
        input.args.push(Argument::DeviceToken(token.clone()));
        assert_eq!(
            object
                .run_oracle_with_token("id", &input, &primary)
                .unwrap()
                .payload,
            1
        );
        assert!(
            object
                .run_oracle_with_token_aliases("id", &input, Budget::steps(1), &[0, 0])
                .is_ok(),
            "an unspecified primary can bind to the supplied handle"
        );
    }
    for labels in [vec![1, 1], vec![0, 2], vec![], vec![0]] {
        let mut input = OracleInput::bits(&[1]);
        input.args.push(Argument::Token(Budget::steps(1)));
        assert!(object
            .run_oracle_with_token_aliases("id", &input, Budget::steps(1), &labels)
            .is_err());
    }
    let mut unequal = OracleInput::bits(&[1]);
    unequal.args.push(Argument::Token(Budget::steps(2)));
    assert!(object
        .run_oracle_with_token_aliases("id", &unequal, Budget::steps(1), &[0, 0])
        .is_err());
}

fn scheduled_nested_oracle() -> Object {
    let outer = r#"{"abi":"0.9","args":["token"],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let inner =
        r#"{"abi":"0.9","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let source=support::rehash(&support::object(&[
        ("outer",outer,"oracle @outer (x:wire in) (t:wire) (f:wire[4]) () (a:token)\nregs { r:reg[1], fr:reg[4] }\n^entry():\nlift.oracle_call @inner a (x) r fr () ()\nlift.retval r"),
        ("inner",inner,"oracle @inner (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x")
    ],"{}", "").replace("=0.8","=0.9"));
    let id = source.identity().unwrap();
    let root = id.closure_hashes["outer"].as_ref().unwrap();
    let child = &id.interface_hashes["inner"];
    let params = bennett_isa_ref::sha256_hex(br#"[["3","0","0","0"]]"#);
    let entry = support::json(&format!(
        r#"{{"gate":{{"epsilon_share":0,"realisation":"native"}},"generated":"*","kind":"gate","path":[{{"call_index":0,"instance":[],"iteration":[],"point":"entry:0","symbol":"{root}"}},{{"instance":[],"iteration":[],"point":"entry:0","symbol":"{child}"}}],"phase":"*","step":3}}"#
    ));
    let schedule = support::json(&format!(
        r#"{{"bindings":{{"budget":[3,0,0,0],"closure_hash":"{root}","epsilon":0,"inherited_controls":0,"params_hash":"{params}","region":"outer","scheduler":{{"name":"k91","version":"1"}},"target_hash":"{}","token_aliases":[0,0],"variant":"forward"}},"entries":{}}}"#,
        "0".repeat(64),
        Json::Array(vec![entry.clone(), entry]).canonical()
    ));
    Object::parse(&format!(
        "{}\nsection schedule\n{}\n",
        source.text,
        schedule.canonical()
    ))
    .unwrap()
}
#[test]
fn native_schedule_dispatch_compares_the_actual_primary_argument_partition() {
    use bennett_isa_ref::{Argument, Budget, OracleInput, OracleMachine};
    let object = scheduled_nested_oracle();
    for x in 0..2 {
        let mut input = OracleInput::bits(&[x]);
        input.args.push(Argument::Token(Budget::steps(3)));
        let error = object
            .run_oracle_with_token_aliases("outer", &input, Budget::steps(3), &[0, 0])
            .unwrap_err();
        assert!(error.to_string().contains("multiple maximal"));
        assert!(OracleMachine::new_with_token_aliases(
            &object,
            "outer",
            &input,
            Budget::steps(3),
            &[0, 0]
        )
        .is_err());
        assert_eq!(
            object
                .run_oracle_with_token_aliases("outer", &input, Budget::steps(3), &[0, 1])
                .unwrap()
                .payload,
            x
        );
        let mut machine = OracleMachine::new_with_token_aliases(
            &object,
            "outer",
            &input,
            Budget::steps(3),
            &[0, 1],
        )
        .unwrap();
        let before = machine.snapshot();
        for _ in 0..3 {
            machine.forward_step().unwrap();
        }
        assert_eq!(machine.outcome().unwrap().payload, x);
        for _ in 0..3 {
            machine.inverse_step().unwrap();
        }
        assert_eq!(machine.snapshot(), before);
    }
    let primary = object.budget_token("outer", Budget::steps(3)).unwrap();
    let distinct = object.budget_token("inner", Budget::steps(3)).unwrap();
    let mut input = OracleInput::bits(&[1]);
    input.args.push(Argument::DeviceToken(primary.clone()));
    assert!(object
        .run_oracle_with_token("outer", &input, &primary)
        .is_err());
    input.args[0] = Argument::DeviceToken(distinct);
    assert_eq!(
        object
            .run_oracle_with_token("outer", &input, &primary)
            .unwrap()
            .payload,
        1
    );
}

#[test]
fn public_cli_q_invocations_bind_aliases_for_run_reverse_and_step_check() {
    use std::{path::PathBuf, process::Command};
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(".tmp");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("k91-native-cli.bisa");
    std::fs::write(&path, oracle_source(true).text).unwrap();
    for aliases in ["[0,0]", "[0,1]"] {
        for command in ["run-oracle", "reverse", "step-check"] {
            let mut cmd = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"));
            cmd.arg(command).arg(&path).arg("id").args([
                "--budget",
                "1,0,0,0",
                "--arg",
                "token:1,0,0,0",
                "--token-aliases",
                aliases,
            ]);
            if command == "step-check" {
                cmd.args(["--exhaustive", "--max-bits", "1"]);
            } else {
                cmd.args(["--input", "1"]);
            }
            let output = cmd.output().unwrap();
            assert!(
                output.status.success(),
                "{command}/{aliases}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let json = Json::parse(std::str::from_utf8(&output.stdout).unwrap().trim()).unwrap();
            if command == "run-oracle" {
                assert_eq!(json.get("payload").unwrap().uint().unwrap(), 1);
            }
            if command == "reverse" {
                assert_eq!(json.get("equals_entry").unwrap(), &Json::Bool(true));
            }
        }
    }
    for aliases in [
        None,
        Some("[1,1]"),
        Some("[0,2]"),
        Some("[0]"),
        Some("[-1,0]"),
    ] {
        let mut cmd = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"));
        cmd.arg("run-oracle").arg(&path).arg("id").args([
            "--input",
            "1",
            "--budget",
            "1,0,0,0",
            "--arg",
            "token:1,0,0,0",
        ]);
        if let Some(labels) = aliases {
            cmd.args(["--token-aliases", labels]);
        }
        let output = cmd.output().unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("E_ARG"));
    }
    std::fs::write(&path, identity_pair_source().text).unwrap();
    for aliases in ["[0,0]", "[0,1]"] {
        for command in ["run-action", "reverse"] {
            let output = Command::new(env!("CARGO_BIN_EXE_bennett-isa-ref"))
                .arg(command)
                .arg(&path)
                .arg("pair")
                .args([
                    "--mut",
                    "1,7",
                    "--in",
                    "1",
                    "--arg",
                    "token:1,0,0,0",
                    "--arg",
                    "token:1,0,0,0",
                    "--token-aliases",
                    aliases,
                ])
                .output()
                .unwrap();
            assert!(
                output.status.success(),
                "{command}/{aliases}: {}",
                String::from_utf8_lossy(&output.stderr)
            );
            let json = Json::parse(std::str::from_utf8(&output.stdout).unwrap().trim()).unwrap();
            if command == "reverse" {
                assert_eq!(json.get("equals_entry").unwrap(), &Json::Bool(true));
            }
        }
    }
}

#[test]
fn controlled_native_oracles_match_the_actual_inherited_control_count() {
    use bennett_isa_ref::{Argument, Budget, QuantumBindings, QuantumState};
    let source = scheduled_nested_oracle();
    let (body, schedule) = source.text.split_once("section schedule\n").unwrap();
    let mut symbols = source.section("symbols").unwrap().clone();
    fields(&mut symbols).insert("2".into(),support::json(r#"{"interface_hash":"0000000000000000000000000000000000000000000000000000000000000000","kind":"perm_action","name":"wrap","signature":{"abi":"0.9","args":["token","token"],"bounds":[],"in":[1,1],"kind":"perm_action","mem":[],"mut":[1,4],"tier":"q"}}"#));
    let mut lines = body.lines().map(str::to_string).collect::<Vec<_>>();
    let index = lines.iter().position(|s| s == "section symbols").unwrap() + 1;
    lines[index] = symbols.canonical();
    let text=format!("{}\nperm_action @wrap (t:wire mut, f:wire[4] mut) (x:wire in, c:wire in) () (p:token, a:token) ()\n^entry():\nctl c oracle_call @outer p (x) t f () (a)\n\nsection schedule\n{schedule}",lines.join("\n").trim_end());
    let object = support::rehash(&text);
    for bits in 0..128u128 {
        let mut state = QuantumState::new();
        let wires = state.allocate(7).unwrap();
        for (i, w) in wires.iter().enumerate() {
            if bits >> i & 1 != 0 {
                state.gate("x", &[*w], None, &[]).unwrap();
            }
        }
        let mut binding = QuantumBindings::default();
        binding.bundles.insert("t".into(), vec![wires[0]]);
        binding.bundles.insert("f".into(), wires[1..5].to_vec());
        binding.bundles.insert("x".into(), vec![wires[5]]);
        binding.bundles.insert("c".into(), vec![wires[6]]);
        for n in ["p", "a"] {
            binding
                .args
                .insert(n.into(), Argument::Token(Budget::steps(3)));
        }
        let entry = state.clone();
        object
            .execute_action_with_token_aliases("wrap", &mut state, &binding, false, &[], &[0, 0])
            .unwrap();
        assert_eq!(
            state.read_basis(&wires).unwrap(),
            bits ^ ((bits >> 5 & 1) & (bits >> 6 & 1))
        );
        object
            .execute_action_with_token_aliases("wrap", &mut state, &binding, true, &[], &[0, 0])
            .unwrap();
        assert_eq!(state, entry);
    }
}

#[test]
fn schedule_reader_checks_canonical_aliases_and_keeps_the_closed_schema() {
    let object = oracle_source(true);
    let original = scheduled(&object, Some("[0,1]"));
    let parsed = Object::parse(&original).unwrap();
    assert_eq!(
        parsed
            .section("schedule")
            .unwrap()
            .get("bindings")
            .unwrap()
            .get("token_aliases")
            .unwrap(),
        &support::json("[0,1]")
    );
    for labels in ["[1,1]", "[0,2]", "[0]", "[]", "[-1,0]", "[0,0.5]"] {
        let text = scheduled(&object, Some(labels));
        let error = Object::parse(&text).unwrap_err();
        assert!(error.to_string().contains("K91"), "{labels}: {error}");
    }
    let mut schedule = parsed.section("schedule").unwrap().clone();
    fields(fields(&mut schedule).get_mut("bindings").unwrap())
        .insert("token_alias".into(), support::json("[0,1]"));
    let error = Object::parse(&format!(
        "{}\nsection schedule\n{}\n",
        object.text,
        schedule.canonical()
    ))
    .unwrap_err();
    assert!(error.to_string().contains("unknown"), "{error}");
}

#[test]
fn alias_metadata_is_versioned_and_does_not_widen_08_readers() {
    let original = certificate_object(Some("[0,1]"), 1);
    let source = original.split_once("section certificate\n").unwrap().0;
    let legacy = support::rehash(
        &(source.trim_end().to_string() + "\n")
            .replace("=0.9", "=0.8")
            .replace("\"0.9\"", "\"0.8\""),
    );
    let with_aliases = scheduled(&legacy, Some("[0,1]"));
    let error = Object::parse(&with_aliases).unwrap_err();
    assert!(error.to_string().contains("token_aliases"), "{error}");
    Object::parse(&scheduled(&legacy, None))
        .unwrap()
        .check_hashes()
        .unwrap();
}
