//! Grammar-derived witnesses: no independent path/type union to go stale.
use super::*;
use crate::Object as IsaObject;
use std::collections::{BTreeMap, BTreeSet};
fn j(s: &str) -> Json {
    Json::parse(s).unwrap()
}
#[test]
fn r15_translation_frame_is_first_and_last_point_tuple() {
    // Literal §12.1 cardinality oracle, independent of TABLE/samples/baseline.
    let point =
        j(r#"["0000000000000000000000000000000000000000000000000000000000000000","entry",0]"#);
    let mut accepted = Vec::new();
    for len in 0..=3 {
        let mut witness = j(r#"{"B":[],"V":[],"W":[],"X":[],"frame":[]}"#);
        let Json::Object(fields) = &mut witness else {
            unreachable!()
        };
        fields.insert("frame".into(), Json::Array(vec![point.clone(); len]));
        if validate("translation_witness", &witness).is_ok() {
            accepted.push(len);
        }
    }
    assert_eq!(accepted, [2], "§12.1: frame is [first point, last point]");
    // V and W remain variable-length point arrays, including empty arrays.
    for len in 0..=3 {
        let points = Json::Array(vec![point.clone(); len]);
        validate("points", &points).unwrap();
    }
}
#[test]
fn r16_schedule_bindings_grammar_admits_omitted_expansion() {
    // Literal §17 item 7 control, independent of TABLE generation/samples.
    let bindings = j(
        r#"{"closure_hash":"0000000000000000000000000000000000000000000000000000000000000000","target_hash":"0000000000000000000000000000000000000000000000000000000000000000","region":"inc8","variant":"forward","inherited_controls":0,"budget":[3,0,0,0],"params_hash":"0000000000000000000000000000000000000000000000000000000000000000","epsilon":0,"scheduler":{"name":"test","version":"1"}}"#,
    );
    validate("schedule_bindings", &bindings).unwrap();
}
fn samples(name: &str) -> Vec<Json> {
    match entry(name).shape {
        String => vec![j(r#""""#), j(r#""vendor.2026""#)],
        Text | Name => vec![j(r#""a.b""#)],
        Hash => vec![Json::String("0".repeat(64))],
        Hex => vec![j(r#""""#), j(r#""00ff""#)],
        Uint => vec![
            j("0"),
            j("9007199254740991"),
            j(r#""9007199254740992""#),
            j(r#""18446744073709551615""#),
            j(r#""18446744073709551616""#),
            j(r#""340282366920938463463374607431768211456""#),
        ],
        U64String => vec![j(r#""0""#), j(r#""18446744073709551615""#)],
        Decimal => vec![
            j("0"),
            j("0.5"),
            j("1e-7"),
            j(r#""0""#),
            j(r#""0.0000000000000000000000000001""#),
        ],
        SignedDecimalString => vec![j(r#""0""#), j(r#""-0.5""#), j(r#""1.25""#)],
        Number => vec![j("0"), j("-0.5"), j("1e-7")],
        Null => vec![Json::Null],
        Enum(values) => values.iter().map(|v| Json::String((*v).into())).collect(),
        Union(types) => types
            .iter()
            .flat_map(|t| {
                // Recursive expressions terminate through their number alternative.
                vec![baseline(t, &mut BTreeSet::new())]
            })
            .collect(),
        Object(fields) => {
            let base = baseline(name, &mut BTreeSet::new());
            let mut out = vec![base.clone()];
            for (key, _) in fields.iter().filter(|(k, _)| k.ends_with('?')) {
                let mut absent = base.clone();
                let Json::Object(m) = &mut absent else {
                    unreachable!()
                };
                m.remove(key.trim_end_matches('?'));
                out.push(absent);
            }
            out
        }
        Array(_) | Map(..) => vec![
            baseline(name, &mut BTreeSet::new()),
            match entry(name).shape {
                Array(_) => Json::Array(vec![]),
                _ => Json::Object(BTreeMap::new()),
            },
        ],
        Tuple(_) => vec![baseline(name, &mut BTreeSet::new())],
    }
}
fn baseline(name: &str, active: &mut BTreeSet<&'static str>) -> Json {
    let e = entry(name);
    // The only cycle is expression -> args -> expression. Its finite base is 0.
    if !active.insert(e.name) {
        assert_eq!(name, "expression");
        return j("0");
    }
    let v = match e.shape {
        Object(fields) => Json::Object(
            fields
                .iter()
                .map(|(k, t)| (k.trim_end_matches('?').into(), baseline(t, active)))
                .collect(),
        ),
        Array(t) => Json::Array(vec![baseline(t, active)]),
        Map(k, t) => {
            let key = baseline(k, active).string().unwrap().to_string();
            Json::Object(BTreeMap::from([(key, baseline(t, active))]))
        }
        Tuple(types) => Json::Array(types.iter().map(|t| baseline(t, active)).collect()),
        Union(types) => baseline(types[0], active),
        _ => samples(name)[0].clone(),
    };
    active.remove(e.name);
    v
}
fn nodes(
    name: &'static str,
    v: &Json,
    path: &mut Vec<std::string::String>,
    out: &mut Vec<(Vec<std::string::String>, &'static str)>,
) {
    out.push((path.clone(), name));
    match entry(name).shape {
        Object(fields) => {
            for (k, t) in fields {
                let k = k.trim_end_matches('?');
                if let Some(child) = v.object().unwrap().get(k) {
                    path.push(k.into());
                    nodes(t, child, path, out);
                    path.pop();
                }
            }
        }
        Map(_, t) => {
            for (k, child) in v.object().unwrap() {
                path.push(k.clone());
                nodes(t, child, path, out);
                path.pop();
            }
        }
        Array(t) => {
            for (i, child) in v.array().unwrap().iter().enumerate() {
                path.push(i.to_string());
                nodes(t, child, path, out);
                path.pop();
            }
        }
        Tuple(types) => {
            for (i, (t, child)) in types.iter().zip(v.array().unwrap()).enumerate() {
                path.push(i.to_string());
                nodes(t, child, path, out);
                path.pop();
            }
        }
        Union(types) => {
            let t = types
                .iter()
                .find(|t| validate(t, v).is_ok())
                .expect("positive branch");
            let start = out.len();
            nodes(t, v, path, out);
            // The selected branch shares this path with its union. Keep the
            // union's alternatives when classifying mutations at that path.
            while out.get(start).is_some_and(|(p, _)| p == path) {
                out.remove(start);
            }
        }
        _ => (),
    }
}
fn replace(v: &mut Json, path: &[std::string::String], next: Json) {
    if path.is_empty() {
        *v = next;
        return;
    }
    let child = match v {
        Json::Object(m) => m.get_mut(&path[0]).unwrap(),
        Json::Array(a) => &mut a[path[0].parse::<usize>().unwrap()],
        _ => panic!("container"),
    };
    replace(child, &path[1..], next);
}
fn kinds(name: &str) -> u8 {
    match entry(name).shape {
        String | Text | Name | Hash | Hex | U64String | SignedDecimalString | Enum(_) => 2,
        Uint | Decimal => 1 | 2,
        Number => 1,
        Null => 8,
        Object(_) | Map(..) => 32,
        Array(_) | Tuple(_) => 16,
        Union(types) => types.iter().fold(0, |mask, t| mask | kinds(t)),
    }
}
fn kind(v: &Json) -> u8 {
    match v {
        Json::Number(_) => 1,
        Json::String(_) => 2,
        Json::Bool(_) => 4,
        Json::Null => 8,
        Json::Array(_) => 16,
        Json::Object(_) => 32,
    }
}
fn wrong(name: &str) -> Vec<Json> {
    // Expected incompatibility is derived from Shape, independently of the
    // production validator. A validator that skips a child makes this RED.
    let mut out = ["7", r#""wrong""#, "false", "null", "[]", "{}"]
        .map(j)
        .into_iter()
        .filter(|v| kind(v) & kinds(name) == 0)
        .collect::<Vec<_>>();
    let extra: &[&str] = match entry(name).shape {
        Uint => &["-1", "1.5", r#""1""#, r#""01""#, r#""-1""#, r#""wrong""#],
        U64String => &[r#""-1""#, r#""18446744073709551616""#, r#""wrong""#],
        Decimal => &[
            "-0.5",
            r#""-0.5""#,
            r#""NaN""#,
            r#""Infinity""#,
            r#""1e3""#,
            r#""1.""#,
            r#"".5""#,
            r#""+1""#,
            r#""""#,
        ],
        SignedDecimalString => &[
            r#""NaN""#,
            r#""Infinity""#,
            r#""1e3""#,
            r#""1.""#,
            r#"".5""#,
            r#""+1""#,
            r#""""#,
        ],
        Text => &[r#""""#],
        Name => &[r#""bad name""#, r#""""#],
        Hash => &[r#""wrong""#, r#""""#],
        Hex => &[r#""0""#, r#""gg""#],
        Enum(_) => &[r#""wrong""#],
        Object(_) => &[r#"{"unexpected":0}"#],
        _ => &[],
    };
    out.extend(extra.iter().map(|s| j(s)));
    if matches!(entry(name).shape, Uint) {
        out.extend(
            [
                "9007199254740992",
                "18446744073709551615",
                "18446744073709551616",
            ]
            .map(|s| Json::Number(s.into())),
        );
    }
    out
}
#[test]
fn r14_generated_metadata_schema_type_family() {
    let Object(root_fields) = entry("metadata").shape else {
        panic!("metadata root")
    };
    let roots = root_fields.iter().map(|(_, ty)| *ty).collect::<Vec<_>>();
    let inc = include_str!("../../tests/fixtures/inc8.bisa");
    let original = IsaObject::parse(inc).unwrap();
    let mut paths = BTreeSet::new();
    let mut negatives = 0;
    let mut positives = 0;
    let mut admissions = 0;
    let mut visited = BTreeSet::new();
    for e in TABLE {
        assert!(e.clause.contains('§'));
        assert!(visited.insert(e.name), "duplicate schema entry");
        for base in samples(e.name) {
            validate(e.name, &base).unwrap_or_else(|err| panic!("{} positive: {err}", e.name));
            positives += 1;
            let is_root = roots.contains(&e.name);
            let mut o = original.clone();
            if is_root {
                o.metadata.insert(e.name.into(), base.clone());
                o.validate_metadata_types().unwrap();
            }
            let mut ns = Vec::new();
            nodes(e.name, &base, &mut Vec::new(), &mut ns);
            for (path, ty) in ns {
                paths.insert(format!("{}:{path:?}:{ty}", e.name));
                for alternative in samples(ty) {
                    let mut good = base.clone();
                    replace(&mut good, &path, alternative);
                    validate(e.name, &good)
                        .unwrap_or_else(|err| panic!("{} {path:?} alternative: {err}", e.name));
                    positives += 1;
                    if is_root {
                        o.metadata.insert(e.name.into(), good);
                        o.validate_metadata_types().unwrap();
                    }
                }
                for next in wrong(ty) {
                    let mut bad = base.clone();
                    replace(&mut bad, &path, next);
                    assert!(
                        validate(e.name, &bad).is_err(),
                        "{} {path:?} must reject {}",
                        e.name,
                        bad.canonical()
                    );
                    negatives += 1;
                    if is_root {
                        o.metadata.insert(e.name.into(), bad.clone());
                        assert!(o.validate_metadata_types().is_err(), "{} {path:?}", e.name);
                        assert!(
                            o.validate_metadata().is_err(),
                            "second admission {} {path:?}",
                            e.name
                        );
                        if e.name == "certificate" {
                            assert!(o.validate_certificate_details(&bad).is_err());
                        }
                        admissions += 1;
                    }
                }
            }
        }
    }
    // Real pinned objects additionally exercise full parse before binding/hash
    // consumers, including populated tables, all signature kinds and payloads.
    let corpus = j(include_str!("../../tests/support/r13_schema_cases.json"));
    let mut parse_paths = 0;
    let mut parse_negatives = 0;
    for case in corpus.array().unwrap() {
        let base = case.get("object").unwrap().string().unwrap();
        let o = IsaObject::parse(base).unwrap();
        o.check_hashes().unwrap();
        for (section, v) in &o.metadata {
            let name = entry(section).name;
            let mut ns = Vec::new();
            nodes(name, v, &mut Vec::new(), &mut ns);
            for (path, ty) in ns {
                parse_paths += 1;
                for next in wrong(ty) {
                    let mut bad = v.clone();
                    replace(&mut bad, &path, next);
                    assert!(validate(name, &bad).is_err(), "{section} {path:?}");
                    let text = base.replace(
                        &format!("section {section}\n{}\n", v.canonical()),
                        &format!("section {section}\n{}\n", bad.canonical()),
                    );
                    assert_ne!(text, base);
                    let err = IsaObject::parse(&text).expect_err("malformed metadata admitted");
                    assert!(
                        err.to_string().contains("§9.1"),
                        "{section}:{path:?}: {err}"
                    );
                    parse_negatives += 1;
                }
            }
        }
    }
    eprintln!("R17 schema: {} entries; {positives} generated positives; {} unique typed paths; {negatives} grammar negatives, {admissions} through BOTH admission validators; {} pinned objects, {parse_paths} parse path visits, {parse_negatives} full-parse negatives",TABLE.len(),paths.len(),corpus.array().unwrap().len());
    assert_eq!(visited.len(), TABLE.len());
}

#[test]
fn r17_gate_payload_omission_is_literal_clause_control() {
    validate("gate_payload", &j(r#"{"realisation":"native"}"#)).unwrap();
    for bad in ["null", "{}", "-1", r#""invalid""#] {
        assert!(validate(
            "gate_payload",
            &j(&format!(
                r#"{{"realisation":"native","epsilon_share":{bad}}}"#
            ))
        )
        .is_err());
    }
    assert!(validate("gate_payload", &j(r#"{"epsilon_share":0}"#)).is_err());
}

#[test]
fn r17_sampler_count_default_is_optional() {
    validate(
        "sampler",
        &j(r#"{"algorithm":"xoshiro256**","seed":["1","2","3","4"]}"#),
    )
    .unwrap();
}

#[test]
fn r17_section17_defaults_are_optional_and_applied() {
    // This is clause-derived, NOT generated from TABLE. N/A is deliberate:
    // §17 includes nonsemantic decisions, hardware and staged target choices
    // which are not fields of any object metadata production. Do not invent
    // required fields for those choices or mistake a policy for implemented
    // target/enclosure execution. Each row records the consumer observation.
    struct DefaultCase {
        item: usize,
        field: &'static str,
        default: &'static str,
        grammar: Option<(&'static str, &'static str, &'static str)>,
        observation: &'static str,
    }
    let cases = [
        DefaultCase { item:1, field:"N/A: binary encoding", default:"none; §16 text", grammar:None, observation:"text parse and identity" },
        DefaultCase { item:2, field:"N/A: gate registry", default:"§4.4 names only", grammar:None, observation:"unknown gate is a load error" },
        DefaultCase { item:3, field:"N/A: device registry", default:"four §7.3 models", grammar:None, observation:"unknown device faults 8" },
        DefaultCase { item:4, field:"N/A: mechanisation", default:"none; interpreter oracle", grammar:None, observation:"reference oracle remains executable" },
        DefaultCase { item:5, field:"N/A: opaque C layout", default:"opaque", grammar:None, observation:"outside text metadata; no C layout field" },
        DefaultCase { item:6, field:"schedule.entries[].cleanup", default:"reverse; mbu=false", grammar:Some(("schedule_entry","cleanup",r#"{"path":[],"step":"*","phase":"*","generated":"*","kind":"gate","gate":{"realisation":"native"}}"#)), observation:"uncovered cleanup resolves reverse; target mbu staged" },
        DefaultCase { item:7, field:"schedule.bindings.expansion", default:"ref at ISA version", grammar:Some(("schedule_bindings","expansion",r#"{"closure_hash":"0000000000000000000000000000000000000000000000000000000000000000","target_hash":"0000000000000000000000000000000000000000000000000000000000000000","region":"inc8","variant":"forward","inherited_controls":0,"budget":[3,0,0,0],"params_hash":"0000000000000000000000000000000000000000000000000000000000000000","epsilon":0,"scheduler":{"name":"test","version":"1"}}"#)), observation:"schedule_expansion returns ref/0.8" },
        DefaultCase { item:8, field:"N/A: enclosure procedure", default:"Taylor/Lagrange < 2^-(precision_bits+2), reduced modulo enclosed 2π", grammar:None, observation:"interval verification staged: numerical claim is Unverified" },
        DefaultCase { item:9, field:"schedule.entries[].gate.epsilon_share", default:"region epsilon / approximate-operation count", grammar:Some(("gate_payload","epsilon_share",r#"{"realisation":"native"}"#)), observation:"schedule_epsilon_share returns exact rational recipe, exact operations stay exact" },
        DefaultCase { item:10, field:"N/A: target any-class resolution", default:"first class in JCS order with capacity", grammar:None, observation:"target capacity consumer staged; schedule alloc must be concrete" },
        DefaultCase { item:11, field:"N/A: target resources.*.rate.<class>", default:"1", grammar:None, observation:"target resource aggregation staged; no target production in TABLE" },
        DefaultCase { item:12, field:"schedule.entries[].start (uncovered operations)", default:"classical_step_time each, sequential", grammar:Some(("schedule_entry","start",r#"{"path":[],"step":"*","phase":"*","generated":"*","kind":"gate","gate":{"realisation":"native"}}"#)), observation:"target timing consumer staged; no synthetic start time at admission" },
        DefaultCase { item:13, field:"certificate.regions.*.claims[].sampler.count", default:"exhaustive ≤16 interface bits; 65536 samples above", grammar:Some(("sampler","count",r#"{"algorithm":"xoshiro256**","seed":["1","2","3","4"]}"#)), observation:"explicit sampled claim replays 65536 when count absent; suite selection is outside metadata" },
        DefaultCase { item:14, field:"N/A: hardware qreg.free", default:"FAILED / FAULT_UB_TRAP at instruction", grammar:None, observation:"unclean free faults 4" },
    ];
    let inc = IsaObject::parse(include_str!("../../tests/fixtures/inc8.bisa")).unwrap();
    let eps = IsaObject::parse(include_str!(
        "../../tests/support/r17-default-epsilon-share.bisa"
    ))
    .unwrap();
    let expansion = IsaObject::parse(include_str!(
        "../../tests/support/r16-default-expansion.bisa"
    ))
    .unwrap();
    let op = j(r#"{"path":[],"step":1,"phase":"fwd","generated":0}"#);
    for (index, case) in cases.iter().enumerate() {
        assert_eq!(case.item, index + 1);
        if let Some((production, key, literal)) = case.grammar {
            // A required-vs-default mismatch fails here, including a required
            // key added later. Literal independently supplies the omitted form.
            let Shape::Object(fields) = entry(production).shape else {
                panic!("object production")
            };
            assert!(
                fields.iter().any(|(k, _)| *k == format!("{key}?")),
                "§17.{} {} must be optional",
                case.item,
                case.field
            );
            let value = j(literal);
            assert!(!value.object().unwrap().contains_key(key));
            validate(production, &value).unwrap();
        }
        match case.item {
            1 => {
                inc.check_hashes().unwrap();
                assert!(inc.text.starts_with("bennett-object isa=0.8"));
            }
            2 => {
                let bad = inc.text.replace("lift.add x one", "lift.invented x one");
                assert!(IsaObject::parse(&bad).is_err());
            }
            3 | 14 => {
                let (source, name, code) = if case.item == 3 {
                    (
                        include_str!("../../tests/fixtures/classical.bisa"),
                        "unknown_effect",
                        8,
                    )
                } else {
                    (
                        include_str!("../../tests/fixtures/registers.bisa"),
                        "unclean",
                        4,
                    )
                };
                let o = IsaObject::parse(source).unwrap();
                let mut machine = crate::ClassicalMachine::new(&o, name, &[], &[]).unwrap();
                assert_eq!(machine.run(100).unwrap().fault, Some(code));
            }
            4 => {
                let result = inc
                    .run_oracle(
                        "inc8",
                        &crate::OracleInput::bits(&[255]),
                        crate::Budget {
                            steps: 3,
                            heap: 0,
                            stack: 0,
                            depth: 0,
                        },
                    )
                    .unwrap();
                assert_eq!((result.payload, result.code, result.steps), (0, 0, 3));
            }
            5 => {
                assert!(TABLE.iter().all(|e| e.name != "opaque_c_layout"));
            }
            6 => {
                assert_eq!(eps.schedule_cleanup(&op).unwrap(), "reverse");
            }
            7 => {
                let binding = expansion.schedule_expansion().unwrap().unwrap();
                assert_eq!(
                    (binding.name, binding.version, binding.hash),
                    ("ref", "0.8", None)
                );
            }
            8 => {
                let corpus = j(include_str!("../../tests/support/r13_schema_cases.json"));
                let source = corpus
                    .array()
                    .unwrap()
                    .iter()
                    .find(|c| c.get("name").unwrap().string().unwrap() == "certificate")
                    .unwrap()
                    .get("object")
                    .unwrap()
                    .string()
                    .unwrap();
                let o = IsaObject::parse(source).unwrap();
                assert!(matches!(
                    o.verify_claim(
                        "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93",
                        7
                    )
                    .unwrap(),
                    crate::ClaimVerdict::Unverified(_)
                ));
            }
            9 => {
                let epsilon = eps
                    .section("schedule")
                    .unwrap()
                    .get("bindings")
                    .unwrap()
                    .get("epsilon")
                    .unwrap();
                assert_eq!(
                    eps.schedule_epsilon_share(&op, false, 3).unwrap(),
                    Some(crate::ScheduleEpsilonShare::EqualShare {
                        epsilon,
                        operations: 3
                    })
                );
                assert_eq!(
                    eps.schedule_epsilon_share(&op, true, 0).unwrap(),
                    Some(crate::ScheduleEpsilonShare::Exact)
                );
            }
            10 => {
                validate("alloc_payload", &j(r#"{"class":"A"}"#)).unwrap(); /* capacity matching is staged, not a new required target field */
            }
            11 | 12 => {
                assert!(TABLE.iter().all(|e| e.name != "target")); /* no target/timing execution claimed */
            }
            13 => {
                assert_eq!(
                    crate::claims::sample_count(&j(case.grammar.unwrap().2)).unwrap(),
                    65536
                );
            }
            _ => unreachable!(),
        }
        eprintln!(
            "§17.{} | {} | {} | {}",
            case.item, case.field, case.default, case.observation
        );
    }
}
