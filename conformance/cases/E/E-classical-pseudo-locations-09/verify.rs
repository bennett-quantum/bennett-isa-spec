//! Run with repo-local CARGO_TARGET_DIR and this directory's Cargo.toml.
use bennett_isa_conformance::json::{self, Json};
use bennett_isa_ref::{ref_library::OracleInvocation, Argument, Budget, Object, OracleInput};
use std::collections::BTreeMap;

fn expand(
    source: &str,
    name: &str,
    values: &str,
    target: usize,
) -> Vec<bennett_isa_ref::ref_library::Leaf> {
    let object = Object::parse(source).unwrap();
    let values = json::parse(values).unwrap();
    let context = json::parse(r#"{"token":["0","0","0","0"]}"#).unwrap();
    let call = OracleInvocation::capture(
        &object,
        name,
        Some(&context),
        Budget::steps(0),
        values.as_arr().unwrap(),
    )
    .unwrap();
    call.expand_source(
        &object.bound_ref_library().unwrap(),
        &[("t", target), ("f", 4)],
    )
    .unwrap()
}

#[test]
fn publication_pins_the_reference_order() {
    let draft = include_str!("../../../../spec/Bennett-ISA-Spec-0.9.md");
    let marker = "**CLASSICAL-PSEUDO-LOCATION witness**\n\n```json\n";
    let raw = draft
        .split_once(marker)
        .expect("missing classical pseudo-location publication witness")
        .1
        .split_once("\n```")
        .unwrap()
        .0;
    assert_eq!(
        json::parse(raw).unwrap(),
        json::parse(include_str!("trace.json")).unwrap()
    );
}

#[test]
fn i2_three_and_zero_pin_support_ordinals_and_allocation_lifetimes() {
    let expected = json::parse(include_str!("trace.json")).unwrap();
    let leaves = expand(
        include_str!("argument.bisa"),
        "argument",
        r#"["t","f","3"]"#,
        2,
    );
    let storage = [
        "argument",
        "pc",
        "pc-prime",
        "faulted",
        "code",
        "result_val",
    ];
    let allocations = leaves
        .iter()
        .filter_map(|leaf| {
            (leaf.gate == "alloc" && leaf.source.as_ref().unwrap().terminal.point == "$construct")
                .then(|| leaf.allocation.as_ref().unwrap().0.clone())
        })
        .collect::<Vec<_>>();
    assert_eq!(allocations.len(), storage.len());
    let names = allocations
        .iter()
        .cloned()
        .zip(storage)
        .collect::<BTreeMap<_, _>>();
    for (point, field) in [("$construct", "construct"), ("$release", "release")] {
        let actual = leaves
            .iter()
            .filter(|leaf| leaf.source.as_ref().unwrap().terminal.point == point)
            .map(|leaf| {
                let location = leaf.source.as_ref().unwrap();
                assert_eq!(
                    location.terminal.symbol,
                    expected.get("symbol").unwrap().as_str().unwrap()
                );
                assert!(
                    location.terminal.instance.is_empty() && location.terminal.iteration.is_empty()
                );
                assert!(location.path.is_empty() && location.steps.is_empty());
                assert!(leaf.controls.is_empty());
                let (name, width_or_bit) = if let Some((id, width, class, state)) = &leaf.allocation
                {
                    assert_eq!((class.as_str(), state.as_str()), ("any", "clean"));
                    (names[id], *width)
                } else {
                    assert_eq!(leaf.gate, "x");
                    assert_eq!(leaf.wires.len(), 1);
                    (names[&leaf.wires[0].0], leaf.wires[0].1)
                };
                Json::Arr(vec![
                    Json::Str(leaf.gate.clone()),
                    Json::Str(name.into()),
                    Json::Num(width_or_bit as f64),
                    Json::Num(leaf.ordinal as f64),
                    Json::Bool(leaf.scratch),
                ])
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, expected.get(field).unwrap().as_arr().unwrap());
    }
    let zero = expand(
        include_str!("argument.bisa"),
        "argument",
        r#"["t","f","0"]"#,
        2,
    );
    let first = &zero[0];
    assert_eq!(first.gate, "alloc");
    assert_eq!(first.allocation.as_ref().unwrap().1, 2);
    assert!(!first.scratch);
    assert_eq!(
        (zero[1].gate.as_str(), zero[1].ordinal, zero[1].scratch),
        ("alloc", 1, true)
    );
    assert_eq!(zero.last().unwrap().allocation, first.allocation);
    let object = Object::parse(include_str!("argument.bisa")).unwrap();
    let mut input = OracleInput::bits(&[]);
    input.args.push(Argument::Bits(3));
    let outcome = object
        .run_oracle("argument", &input, Budget::steps(1))
        .unwrap();
    assert_eq!((outcome.payload, outcome.code, outcome.steps), (3, 0, 1));
}

#[test]
fn unused_pointer_binary64_and_token_values_keep_their_exact_storage_rules() {
    for (bits, set_bits) in [
        ("0x8000000000000000", vec![63]),
        (
            "0x7ff8000000000001",
            (0..64)
                .filter(|bit| (0x7ff8000000000001u64 >> bit) & 1 != 0)
                .collect(),
        ),
    ] {
        let values = format!(r#"["t","f","3","2147483649","{bits}",["0","0","0","0"]]"#);
        // The source has a primary token and one argument token. Give the
        // tuple-only source call the explicit distinct partition K91 requires.
        let object = Object::parse(include_str!("typed.bisa")).unwrap();
        let context = json::parse(r#"{"token":["0","0","0","0"]}"#).unwrap();
        let args = json::parse(&values).unwrap();
        let call = OracleInvocation::capture_with_token_aliases(
            &object,
            "typed",
            Some(&context),
            Budget::steps(0),
            args.as_arr().unwrap(),
            &[0, 1],
        )
        .unwrap();
        let leaves = call
            .expand_source(&object.bound_ref_library().unwrap(), &[("t", 1), ("f", 4)])
            .unwrap();
        let declarations = leaves
            .iter()
            .filter(|leaf| {
                leaf.gate == "alloc"
                    && !leaf.scratch
                    && leaf.source.as_ref().unwrap().terminal.point == "$construct"
            })
            .collect::<Vec<_>>();
        assert_eq!(declarations.len(), 3, "token adds no fourth allocation");
        for (declaration, (width, wanted)) in
            declarations
                .iter()
                .zip([(2, vec![0, 1]), (32, vec![0, 31]), (64, set_bits)])
        {
            let metadata = declaration.allocation.as_ref().unwrap();
            assert_eq!(metadata.1, width);
            let prepared = leaves
                .iter()
                .filter(|leaf| {
                    leaf.gate == "x"
                        && leaf.source.as_ref().unwrap().terminal.point == "$construct"
                        && leaf.wires[0].0 == metadata.0
                })
                .map(|leaf| leaf.wires[0].1)
                .collect::<Vec<_>>();
            assert_eq!(prepared, wanted);
            let cleared = leaves
                .iter()
                .filter(|leaf| {
                    leaf.gate == "x"
                        && leaf.source.as_ref().unwrap().terminal.point == "$release"
                        && leaf.wires[0].0 == metadata.0
                })
                .map(|leaf| leaf.wires[0].1)
                .collect::<Vec<_>>();
            assert_eq!(cleared, wanted.into_iter().rev().collect::<Vec<_>>());
        }
        assert!(leaves
            .iter()
            .filter(|leaf| leaf.gate == "alloc" || leaf.gate == "release")
            .all(|leaf| leaf.controls.is_empty()));
    }
}

#[test]
fn inherited_controls_apply_to_argument_x_gates_and_leave_acquisitions_unconditional() {
    let source = include_str!("argument.bisa");
    let (header, rest) = source.split_once("section symbols\n").unwrap();
    let (symbols, body) = rest.split_once("\n\nsection functions\n").unwrap();
    let Json::Obj(mut symbols) = json::parse(symbols).unwrap() else {
        panic!("symbols")
    };
    symbols.insert("1".into(), json::parse(&format!(r#"{{"name":"controlled","kind":"perm_action","interface_hash":"{}","signature":{{"abi":"0.9","args":["token","i2"],"bounds":[],"in":[1],"kind":"perm_action","mem":[],"mut":[2,4],"tier":"q"}}}}"#, "0".repeat(64))).unwrap());
    let functions = format!("{body}perm_action @controlled (t:wire[2] mut, f:wire[4] mut) (c:wire in) () (tk:token, a:i2) ()\n^entry():\nctl c oracle_call @argument tk () t f () (a)\n");
    let text = format!(
        "{header}section symbols\n{}\n\nsection functions\n{functions}",
        Json::Obj(symbols.clone())
    );
    let object = rehash(&text);
    let args = json::parse(r#"["t","f","c",["0","0","0","0"],"3"]"#).unwrap();
    let leaves = object
        .expand_action_source_with_token_aliases(
            "controlled",
            args.as_arr().unwrap(),
            &[("t", 2), ("f", 4), ("c", 1)],
            false,
            &[0],
        )
        .unwrap();
    let hash = object.identity().unwrap().closure_hashes["argument"]
        .clone()
        .unwrap();
    let support = leaves
        .iter()
        .filter(|leaf| {
            let location = leaf.source.as_ref().unwrap();
            location.terminal.symbol == hash && location.terminal.point == "$construct"
        })
        .collect::<Vec<_>>();
    assert_eq!(support[0].gate, "alloc");
    assert!(!support[0].scratch && support[0].controls.is_empty());
    for (ordinal, leaf) in support[1..3].iter().enumerate() {
        assert_eq!(
            (leaf.gate.as_str(), leaf.wires[0].1, leaf.ordinal),
            ("x", ordinal, ordinal + 1)
        );
        assert_eq!(leaf.controls, vec![("c".into(), 0)]);
    }
    assert!(leaves
        .iter()
        .filter(|leaf| leaf.gate == "alloc" || leaf.gate == "release")
        .all(|leaf| leaf.controls.is_empty()));
}

// Compute fixture identities using the public object API; expansion still
// checks every supplied source/closure binding against the linked functions.
fn rehash(text: &str) -> Object {
    let hashes = Object::parse(text)
        .unwrap()
        .identity()
        .unwrap()
        .interface_hashes;
    let (header, rest) = text.split_once("section symbols\n").unwrap();
    let (raw, functions) = rest.split_once("\n\nsection functions\n").unwrap();
    let Json::Obj(mut symbols) = json::parse(raw).unwrap() else {
        panic!("symbols")
    };
    for symbol in symbols.values_mut() {
        let Json::Obj(fields) = symbol else {
            panic!("symbol")
        };
        let name = fields["name"].as_str().unwrap();
        fields.insert("interface_hash".into(), Json::Str(hashes[name].clone()));
    }
    Object::parse(&format!(
        "{header}section symbols\n{}\n\nsection functions\n{functions}",
        Json::Obj(symbols)
    ))
    .unwrap()
}

#[test]
fn fixed_binding_instances_prepare_in_source_index_space_order_even_at_zero_steps() {
    let root = r#"{"abi":"0.9","args":["i2","i2"],"fault":4,"in":[],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let worker = r#"{"abi":"0.9","args":["i2"],"kind":"lproc","mem":[],"params":[],"result":1,"tier":"q","tokens":[]}"#;
    let declaration = |name: &str, kind: &str, signature: &str| {
        Json::Obj(BTreeMap::from([
            ("name".into(), Json::Str(name.into())),
            ("kind".into(), Json::Str(kind.into())),
            ("interface_hash".into(), Json::Str("0".repeat(64))),
            ("signature".into(), json::parse(signature).unwrap()),
        ]))
    };
    let symbols = Json::Obj(BTreeMap::from([
        ("0".into(), declaration("root", "oracle", root)),
        ("1".into(), declaration("worker", "lproc", worker)),
    ]));
    let object = rehash(&format!("bennett-object isa=0.9 object=0.9 launch=0.9 sched=0.9 cert=0.9 coverage=0.9 target=0.9 ptr_width=32\n\nsection symbols\n{symbols}\n\nsection functions\noracle @root () (t:wire) (f:wire[4]) () (a:i2, b:i2)\nregs {{ r:reg[1] }}\n^entry():\nr = lift.call @worker () () (a) ()\nr = lift.call @worker () () (b) ()\nlift.retval r\nlproc @worker () (v:reg[1]) () (x:i2) ()\n^entry():\nv = lift.const.i1 1\nlift.ret v\n\nsection layout\n{{\"A_tab\":0,\"H\":1,\"S_cells\":3,\"globals\":{{}},\"heap_base\":1,\"heap_limit_max\":1,\"interface_regions\":{{}},\"null_size\":1,\"ptr_width\":32}}\n"));
    for (second, wanted) in [
        ("2", vec![vec![0], vec![1], vec![0], vec![1]]),
        ("1", vec![vec![0]; 4]),
    ] {
        let budget = Budget {
            steps: 0,
            heap: 1,
            stack: 8,
            depth: 1,
        };
        let context = json::parse(r#"{"token":["0","1","8","1"]}"#).unwrap();
        let args = json::parse(&format!(r#"["t","f","1","{second}"]"#)).unwrap();
        let call = OracleInvocation::capture(
            &object,
            "root",
            Some(&context),
            budget,
            args.as_arr().unwrap(),
        )
        .unwrap();
        let leaves = call
            .expand_source(&object.bound_ref_library().unwrap(), &[("t", 1), ("f", 4)])
            .unwrap();
        let parameters = leaves
            .iter()
            .filter(|leaf| {
                leaf.gate == "alloc"
                    && !leaf.scratch
                    && leaf.source.as_ref().unwrap().terminal.point == "$construct"
            })
            .collect::<Vec<_>>();
        assert_eq!(
            parameters.len(),
            4,
            "two root parameters and two lproc instance parameters"
        );
        let mut identities = std::collections::BTreeSet::new();
        for (leaf, expected) in parameters.iter().zip(wanted) {
            let id = &leaf.allocation.as_ref().unwrap().0;
            assert!(identities.insert(id));
            let bits = leaves
                .iter()
                .filter(|leaf| {
                    leaf.gate == "x"
                        && leaf.source.as_ref().unwrap().terminal.point == "$construct"
                        && leaf.wires[0].0 == *id
                })
                .map(|leaf| leaf.wires[0].1)
                .collect::<Vec<_>>();
            assert_eq!(bits, expected);
        }
    }
}
