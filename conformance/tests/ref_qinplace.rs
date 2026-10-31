use bennett_isa_conformance::{
    json::{self, Json},
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Leaf, Registry},
};
fn parse(s: &str) -> Json {
    json::parse(s).unwrap()
}
fn gate(g: &str, wires: Vec<String>) -> Json {
    parse(&format!(
        r#"{{"gate":"{g}","wires":{}}}"#,
        Json::Arr(wires.into_iter().map(Json::Str).collect())
    ))
}
fn xor_context(w: usize) -> Vec<Json> {
    let mut out = Vec::new();
    for bit in 0..w {
        for source in [
            format!("y[{bit}]"),
            format!("r[{bit}]"),
            format!("r[{}]", w + bit),
        ] {
            out.push(gate("cx", vec![source, format!("work[{bit}]")]));
        }
    }
    // The low N bits of the exact i128 argument 18446744073709551617.
    out.push(gate("x", vec!["work[0]".into()]));
    out
}
fn oracle(w: usize, bound: usize, full: bool, inverse: bool) -> Json {
    let mut parameters = vec![parse(&format!(
        r#"{{"name":"x","type":"wire","width":{w}}}"#
    ))];
    if full {
        parameters.push(parse(&format!(
            r#"{{"name":"y","type":"wire","width":{w}}}"#
        )));
    }
    parameters.extend([
        parse(&format!(r#"{{"name":"t","type":"wire","width":{w}}}"#)),
        parse(r#"{"name":"f","type":"wire","width":4}"#),
    ]);
    if full {
        parameters.extend([
            parse(&format!(
                r#"{{"name":"r","type":"region","address":1,"cell":{w},"role":"in"}}"#
            )),
            parse(r#"{"name":"c","type":"value","isa_type":"i128"}"#),
            parse(r#"{"name":"d","type":"value","isa_type":"f64"}"#),
        ]);
    }
    parameters.push(parse(r#"{"name":"tk","type":"value","isa_type":"token"}"#));
    let copy = (0..w)
        .map(|bit| gate("cx", vec![format!("x[{bit}]"), format!("work[{bit}]")]))
        .collect::<Vec<_>>();
    let mut compute = copy;
    if inverse && full {
        compute.extend(xor_context(w));
    }
    // Increment high bits first so the low-bit controls see the old value.
    // The inverse is decrement. This is a source fixture, not a catalog helper.
    let increment = (0..w)
        .rev()
        .map(|bit| gate("mcx", (0..=bit).map(|i| format!("work[{i}]")).collect()))
        .collect::<Vec<_>>();
    let step = if inverse {
        parse(&format!(
            r#"[{{"inverse":true,"body":{}}}]"#,
            Json::Arr(increment)
        ))
    } else {
        Json::Arr(increment)
    };
    compute.push(parse(&format!(
        r#"{{"for":"i","from":0,"to":{bound},"body":{step}}}"#
    )));
    if !inverse && full {
        compute.extend(xor_context(w));
    }
    let mut body = vec![parse(&format!(
        r#"{{"alloc":"work","width":{w},"class":"any","state":"clean"}}"#
    ))];
    body.extend(compute.clone());
    body.extend((0..w).map(|bit| gate("cx", vec![format!("work[{bit}]"), format!("t[{bit}]")])));
    body.push(parse(&format!(
        r#"{{"inverse":true,"body":{}}}"#,
        Json::Arr(compute)
    )));
    body.push(parse(r#"{"release":"work"}"#));
    parse(&format!(
        r#"{{"parameters":{},"body":{},"scratch":["work"]}}"#,
        Json::Arr(parameters),
        Json::Arr(body)
    ))
}
fn fixture(
    registry: &mut Registry,
    id: usize,
    w: usize,
    bound: usize,
    full: bool,
    adjoint: bool,
) -> (String, Json) {
    let g = format!("{:064x}", 3 * id + 1);
    let h = format!("{:064x}", 3 * id + 2);
    let root = format!("{:064x}", 3 * id + 3);
    registry.insert(g.clone(), oracle(w, bound, full, false));
    registry.insert(h.clone(), oracle(w, bound, full, true));
    let parameters = format!(
        r#"[{{"name":"G","type":"symbol"}},{{"name":"H","type":"symbol"}},{{"name":"x","type":"wire","width":{w}}},{{"name":"y","type":"wire","width":{w}}},{{"name":"r","type":"wire","width":{rw}}},{{"name":"control","type":"wire","width":1}}]"#,
        rw = 2 * w
    );
    let template = |symbol: &str| {
        let args = if full {
            format!(
                r#"["u","y","v","f",{{"address":1,"cell":{w},"role":"in","origin":"local:db","cells":"r"}},"18446744073709551617","0x7ff8000000000001",["64","2","3","4"]]"#
            )
        } else {
            r#"["u","v","f",["64","2","3","4"]]"#.to_string()
        };
        format!(
            r#"[{{"parameters":[{{"name":"u","type":"wire","width":{w}}},{{"name":"v","type":"wire","width":{w}}},{{"name":"f","type":"wire","width":4}}],"body":[{{"call":"{symbol}","args":{args}}}]}}]"#
        )
    };
    let (first, last) = if adjoint { ("H", "G") } else { ("G", "H") };
    let body = format!(
        r#"[{{"invoke":"qinplace","args":[{w},4,"x",{},{},"control"]}}]"#,
        template(first),
        template(last)
    );
    registry.insert(
        root.clone(),
        parse(&format!(r#"{{"parameters":{parameters},"body":{body}}}"#)),
    );
    let args = parse(&format!(r#"["{g}","{h}","x","y","r","control"]"#));
    (root, args)
}
fn check_order(leaves: &[Leaf], w: usize) {
    let row = leaves
        .iter()
        .filter(|l| l.invocation.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(row.len(), w + 4);
    assert_eq!(
        row.iter().map(|l| l.ordinal).collect::<Vec<_>>(),
        (0..w + 4).collect::<Vec<_>>()
    );
    assert_eq!(
        (row[0].gate.as_str(), row[1].gate.as_str()),
        ("alloc", "alloc")
    );
    assert!(row[2..w + 2]
        .iter()
        .all(|l| l.gate == "swap" && l.controls == [("control".into(), 0)]));
    assert_eq!(
        (row[w + 2].gate.as_str(), row[w + 3].gate.as_str()),
        ("release", "release")
    );
    assert_eq!(row[0].allocation, row[w + 2].allocation);
    assert_eq!(row[1].allocation, row[w + 3].allocation);
    for l in [row[0], row[1], row[w + 2], row[w + 3]] {
        assert!(!l.scratch && l.controls.is_empty());
        assert_eq!(l.allocation.as_ref().unwrap().2, "any");
        assert_eq!(l.allocation.as_ref().unwrap().3, "clean");
    }
    let calls = leaves
        .iter()
        .filter(|l| !l.invocation.is_empty())
        .map(|l| l.invocation.clone())
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(calls.len(), 2);
    for path in calls {
        let call = leaves
            .iter()
            .filter(|l| l.invocation == path)
            .collect::<Vec<_>>();
        assert_eq!(
            call.iter().map(|l| l.ordinal).collect::<Vec<_>>(),
            (0..call.len()).collect::<Vec<_>>()
        );
        assert!(call
            .iter()
            .filter(|l| l.allocation.is_some())
            .all(|l| l.scratch && l.controls.is_empty()));
        assert!(call
            .iter()
            .filter(|l| l.allocation.is_none())
            .all(|l| l.controls == [("control".into(), 0)]));
    }
}
// Build the expectation from an integer truth table, independently of the
// fixture's increment gates and the catalog's recipe composition.
fn integer_map(b: &mut Bdd, x: &[usize], bound: usize, subtract: bool) -> Vec<usize> {
    let n = 1usize << x.len();
    let mut out = vec![0; x.len()];
    for value in 0..n {
        let mapped = if subtract {
            value.wrapping_sub(bound)
        } else {
            value + bound
        } & (n - 1);
        let eq = b.equal(x, &Bdd::constant(value, x.len()));
        for (bit, result) in out.iter_mut().enumerate() {
            if mapped >> bit & 1 != 0 {
                *result = b.or(*result, eq);
            }
        }
    }
    out
}
#[test]
fn qinplace_full_domains_short_and_complete_interfaces_both_orientations() {
    let mut tuples = [0u128; 2];
    let mut shapes = 0;
    let mut registry = Registry::new();
    let mut cases = Vec::new();
    for full in [false, true] {
        for w in 1..=4 {
            for bound in [0, 1, 2, 3, 17] {
                for adjoint in [false, true] {
                    let (hash, args) = fixture(&mut registry, cases.len(), w, bound, full, adjoint);
                    cases.push((hash, args, w, bound, full, adjoint));
                }
            }
        }
    }
    let mut expander =
        Expander::new(include_str!("../../spec/ref-0.9-draft.json"), registry).unwrap();
    for (hash, args, w, bound, full, adjoint) in cases {
        let bundles = [
            ("x", w),
            ("y", w),
            ("r", 2 * w),
            ("control", 1),
            ("spectator", 2),
        ];
        let leaves = expander
            .expand(&hash, args.as_arr().unwrap(), &bundles)
            .unwrap();
        check_order(&leaves, w);
        let bundles = [
            ("x", w),
            ("y", w),
            ("r", 2 * w),
            ("control", 1),
            ("spectator", 2),
        ];
        let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
        let mut b = Bdd::new();
        let mut state = m.symbolic_state();
        for (name, _) in bundles {
            for id in &m.inputs[name] {
                state[*id] = b.variable();
            }
        }
        let old = state.clone();
        let mut expected = old.clone();
        let mut x = m.inputs["x"].iter().map(|i| old[*i]).collect::<Vec<_>>();
        let context = (0..w)
            .map(|bit| {
                let yr = b.xor(old[m.inputs["y"][bit]], old[m.inputs["r"][bit]]);
                let yr = b.xor(yr, old[m.inputs["r"][w + bit]]);
                if bit == 0 {
                    b.not(yr)
                } else {
                    yr
                }
            })
            .collect::<Vec<_>>();
        if full && adjoint {
            for bit in 0..w {
                x[bit] = b.xor(x[bit], context[bit]);
            }
        }
        let mut value = integer_map(&mut b, &x, bound, adjoint);
        if full && !adjoint {
            for bit in 0..w {
                value[bit] = b.xor(value[bit], context[bit]);
            }
        }
        for (bit, mapped) in value.iter().enumerate() {
            let id = m.inputs["x"][bit];
            let delta = b.xor(old[id], *mapped);
            let delta = b.and(old[m.inputs["control"][0]], delta);
            expected[id] = b.xor(old[id], delta);
        }
        m.run_symbolic(&mut b, &mut state, 1, false, 1).unwrap();
        assert_eq!(
            state, expected,
            "w={w}, B={bound}, full={full}, adjoint={adjoint}"
        );
        m.run_symbolic(&mut b, &mut state, 1, true, 1).unwrap();
        assert_eq!(state, old);
        // All declared interface coordinates, including unused y/r in
        // the short form and arbitrary spectators, are in the domain.
        tuples[usize::from(full)] += 1u128 << (4 * w + 3);
        shapes += 1;
    }
    assert_eq!(shapes, 80);
    assert_eq!(tuples, [5592320, 5592320]);
    println!("qinplace: {shapes} shapes; short {} / complete {} full-domain tuples; W1..4, B0/1/2/3/17, both orientations",tuples[0],tuples[1]);
}

#[test]
fn exact_section_16_3_example_7_addya_and_adjoint() {
    let mut tuples = 0;
    for a in 0..4 {
        let mut registry = Registry::new();
        let g = "d".repeat(64);
        let h = "e".repeat(64);
        for (hash, op) in [(&g, "add"), (&h, "sub")] {
            let compute = format!(
                r#"[{{"invoke":"bind.value.{op}","args":[2,"x","y","s"]}},{{"invoke":"bind.value.{op}","args":[2,"s","constant","result"]}}]"#
            );
            registry.insert(hash.clone(),parse(&format!(r#"{{"parameters":[{{"name":"x","type":"wire","width":2}},{{"name":"y","type":"wire","width":2}},{{"name":"t","type":"wire","width":2}},{{"name":"f","type":"wire","width":4}},{{"name":"a","type":"value","isa_type":"i2"}},{{"name":"tk","type":"value","isa_type":"token"}}],"scratch":["s","result","constant"],"body":[{{"alloc":"constant","width":2,"class":"any","state":"clean"}},{{"invoke":"value.literal","args":[2,{a},"constant"]}},{{"alloc":"s","width":2,"class":"any","state":"clean"}},{{"alloc":"result","width":2,"class":"any","state":"clean"}},{compute_body},{{"invoke":"bind.qxor","args":[2,"result","t"]}},{{"inverse":true,"body":{compute}}},{{"release":"result"}},{{"release":"s"}},{{"inverse":true,"body":[{{"invoke":"value.literal","args":[2,{a},"constant"]}}]}},{{"release":"constant"}}]}}"#,compute_body=&compute[1..compute.len()-1])));
        }
        for adjoint in [false, true] {
            let root = "f".repeat(64);
            let template = |name: &str| {
                format!(
                    r#"[{{"parameters":[{{"name":"u","type":"wire","width":2}},{{"name":"v","type":"wire","width":2}},{{"name":"f","type":"wire","width":4}}],"body":[{{"call":"{name}","args":["u","y","v","f","{a}",["3","0","0","0"]]}}]}}]"#
                )
            };
            let (first, last) = if adjoint { ("H", "G") } else { ("G", "H") };
            registry.insert(root.clone(),parse(&format!(r#"{{"parameters":[{{"name":"G","type":"symbol"}},{{"name":"H","type":"symbol"}},{{"name":"x","type":"wire","width":2}},{{"name":"y","type":"wire","width":2}},{{"name":"control","type":"wire","width":1}}],"body":[{{"invoke":"qinplace","args":[2,4,"x",{},{},"control"]}}]}}"#,template(first),template(last))));
            let args = parse(&format!(r#"["{g}","{h}","x","y","control"]"#));
            let bundles = [("x", 2), ("y", 2), ("control", 1)];
            let leaves = Expander::new(
                include_str!("../../spec/ref-0.9-draft.json"),
                registry.clone(),
            )
            .unwrap()
            .expand(&root, args.as_arr().unwrap(), &bundles)
            .unwrap();
            check_order(&leaves, 2);
            let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
            for x in 0..4u64 {
                for y in 0..4u64 {
                    for enabled in [false, true] {
                        let mut state = m.state();
                        m.set(&mut state, "x", x);
                        m.set(&mut state, "y", y);
                        m.set(&mut state, "control", u64::from(enabled));
                        let old = state.clone();
                        m.run(&mut state, true, false).unwrap();
                        let expected = if !enabled {
                            x
                        } else if adjoint {
                            x.wrapping_sub(y).wrapping_sub(a) & 3
                        } else {
                            (x + y + a) & 3
                        };
                        assert_eq!(m.get(&state, "x"), expected);
                        assert_eq!(m.get(&state, "y"), y);
                        assert_eq!(m.get(&state, "control"), u64::from(enabled));
                        m.run(&mut state, true, true).unwrap();
                        assert_eq!(state, old);
                        tuples += 1;
                    }
                }
            }
        }
    }
    assert_eq!(tuples, 256);
    println!("§16.3 example7: 256 exact full-domain tuples (64 claim elements times 2 controls times 2 orientations)");
}
