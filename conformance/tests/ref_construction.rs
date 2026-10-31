use bennett_isa_conformance::{
    json::{self, Json},
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
fn parse(s: &str) -> Json {
    json::parse(s).unwrap()
}
#[test]
fn oracle_construction_complete_lifetimes_bounded_steps_commit_and_reverse() {
    let mut registry = Registry::new();
    let mut cases = Vec::new();
    for w in 1usize..=4 {
        for bound in [0usize, 1, 2, 3, 17] {
            for mode in ["success", "fault", "running"] {
                let p = 2;
                let n = 2;
                let q = 2 * p + w + 1;
                let mut names = vec![
                    ("pc".to_string(), p),
                    ("next".into(), p),
                    ("faulted".into(), 1),
                    ("code".into(), 4),
                    ("result".into(), w),
                    ("loc".into(), w),
                    ("imagecells".into(), 8),
                ];
                for i in 0..bound {
                    names.push((format!("frame{i}"), q));
                }
                let disp_end = p + w;
                let first = format!(
                    r#"{{"parameters":[{{"name":"f","type":"wire","width":{q}}}],"body":[{{"invoke":"lift.add","args":[{w},"x","y",{{"range":"f","from":{p},"to":{disp_end}}},"loc"]}}]}}"#
                );
                let last = match mode {
                    "success" => {
                        format!(r#"[{{"invoke":"lift.retval","args":[{w},"loc","result"]}}]"#)
                    }
                    "fault" => r#"[{"invoke":"fault","args":[4,7,"faulted","code"]}]"#.into(),
                    _ => "[]".into(),
                };
                let rows = format!(
                    r#"[{first},{{"parameters":[{{"name":"f","type":"wire","width":{q}}}],"body":{last}}}]"#
                );
                let targets = if mode == "running" { [1, 0] } else { [1, 2] };
                let next=targets.iter().map(|label|format!(r#"{{"parameters":[{{"name":"f","type":"wire","width":{q}}},{{"name":"s","type":"wire","width":{p}}}],"body":[{{"invoke":"value.literal","args":[{p},{label},"s"]}}]}}"#)).collect::<Vec<_>>().join(",");
                let state = names
                    .iter()
                    .map(|(name, width)| {
                        format!(
                            r#"{{"width":{width},"initial":0,"I":"{}","storage":"{name}"}}"#,
                            if name == "imagecells" { "81" } else { "" }
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let frames = (0..bound)
                    .map(|i| format!(r#"{{"index":{i},"frame":"frame{i}"}}"#))
                    .collect::<Vec<_>>()
                    .join(",");
                let call = format!(
                    r#"{{"invoke":"oracle.construct","args":[{p},{n},{q},{bound},4,{w},"pc","next","faulted","code","result","tgt","fr",[{state}],[{frames}],{rows},[{next}]]}}"#
                );
                let mut body = parse(&format!(r#"[{{"under":"control","body":[{call}]}}]"#));
                for (name, width) in names.iter().rev() {
                    body = parse(&format!(
                        r#"[{{"declare":"{name}","width":{width},"class":{{"class":"any"}},"state":"clean","body":{body}}}]"#
                    ));
                }
                let hash = format!("{:064x}", cases.len());
                registry.insert(
                    hash.clone(),
                    parse(&format!(r#"{{"parameters":[],"body":{body}}}"#)),
                );
                cases.push((hash, w, bound, mode));
            }
        }
    }
    let mut e = Expander::new(include_str!("../../spec/ref-0.9-draft.json"), registry).unwrap();
    let mut tuples = 0u128;
    for (hash, w, bound, mode) in &cases {
        let bundles = [("x", *w), ("y", *w), ("tgt", *w), ("fr", 4), ("control", 1)];
        let leaves = e.expand(hash, &[], &bundles).unwrap();
        assert!(
            leaves
                .iter()
                .any(|l| l.gate == "x" && l.controls.is_empty()),
            "unconditional image leaves remain present"
        );
        let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
        let mut b = Bdd::new();
        let mut state = m.symbolic_state();
        for (name, _) in bundles {
            for i in &m.inputs[name] {
                state[*i] = b.variable();
            }
        }
        let enabled = state[m.inputs["control"][0]];
        let old = state.clone();
        let mut want = old.clone();
        if *bound >= 2 && *mode == "success" {
            let x = m.inputs["x"].iter().map(|i| old[*i]).collect::<Vec<_>>();
            let y = m.inputs["y"].iter().map(|i| old[*i]).collect::<Vec<_>>();
            let sum = b.add(&x, &y);
            for (bit, value) in sum.iter().enumerate() {
                let delta = b.and(enabled, *value);
                let id = m.inputs["tgt"][bit];
                want[id] = b.xor(want[id], delta);
            }
        }
        let code = if *bound < 2 || *mode == "running" {
            1
        } else if *mode == "fault" {
            7
        } else {
            0
        };
        for bit in 0..4 {
            if code >> bit & 1 == 1 {
                let id = m.inputs["fr"][bit];
                want[id] = b.xor(want[id], enabled);
            }
        }
        // Effective controls were bound in the expander, so unconditional image
        // gates must execute even when the control input is false.
        m.run_symbolic(&mut b, &mut state, 1, false, 1).unwrap();
        assert_eq!(state, want, "W={w} B={bound} mode={mode}");
        m.run_symbolic(&mut b, &mut state, 1, true, 1).unwrap();
        assert_eq!(state, old);
        tuples += 1 << (3 * w + 5);
    }
    assert_eq!(cases.len(), 60);
    assert_eq!(tuples, 2246400);
    println!("construction: 60 shapes / {tuples} full-domain tuples at W1..4 B0/1/2/3/17");
}
