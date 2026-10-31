use bennett_isa_conformance::{
    json::{self, Json},
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
fn template(body: Json, parameters: Json) -> Json {
    json::parse(&format!(r#"{{"parameters":{parameters},"body":{body}}}"#)).unwrap()
}
fn v(s: &str) -> Json {
    json::parse(s).unwrap()
}
#[test]
fn heterogeneous_step_full_domains_and_idle_frames() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut count = 0u64;
    for w in 1..=4 {
        for n in [0usize, 1, 2, 3, 17] {
            let p = (usize::BITS - n.leading_zeros()).max(1) as usize;
            let q = p + w + 1;
            let frame = v(&format!(r#"[{{"name":"f","type":"wire","width":{q}}}]"#));
            let frame_next = v(&format!(
                r#"[{{"name":"f","type":"wire","width":{q}}},{{"name":"out","type":"wire","width":{p}}}]"#
            ));
            let mut rows = vec![];
            let mut next = vec![];
            for k in 0..n {
                let body = match k {
                    0 => format!(
                        r#"[{{"invoke":"lift.mov","args":[{w},"src",{{"range":"f","from":{p},"to":{end}}},"w"]}}]"#,
                        end = p + w
                    ),
                    1 => format!(
                        r#"[{{"invoke":"lift.faultif","args":[4,7,"cond","f[{flag}]","faulted","code"]}}]"#,
                        flag = p + w
                    ),
                    _ => format!(r#"[{{"gate":"x","wires":["w[{bit}]"]}}]"#, bit = k % w),
                };
                rows.push(template(v(&body), frame.clone()));
                let body = if k == 1 {
                    format!(
                        r#"[{{"invoke":"successor.flagged","args":[{p},{good},{n},"f[{flag}]","out"]}}]"#,
                        good = k + 1,
                        flag = p + w
                    )
                } else {
                    format!(
                        r#"[{{"invoke":"value.literal","args":[{p},{good},"out"]}}]"#,
                        good = k + 1
                    )
                };
                next.push(template(v(&body), frame_next.clone()));
            }
            let args = v(&format!(
                r#"[{p},{n},{q},"pc","next","frame",{},{}]"#,
                Json::Arr(rows),
                Json::Arr(next)
            ));
            let bundles = [
                ("pc", p),
                ("next", p),
                ("frame", q),
                ("src", w),
                ("w", w),
                ("cond", 1),
                ("faulted", 1),
                ("code", 4),
            ];
            let leaves = e.expand("step", args.as_arr().unwrap(), &bundles).unwrap();
            let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
            for pc in 0..=n {
                let mut b = Bdd::new();
                let mut old = m.symbolic_state();
                for name in ["src", "w", "cond", "faulted", "code"] {
                    for id in &m.inputs[name] {
                        old[*id] = b.variable();
                    }
                }
                for (bit, id) in m.inputs["pc"].iter().enumerate() {
                    old[*id] = (pc >> bit) & 1;
                }
                let en = b.variable();
                let mut expected = old.clone();
                let enabled_next = if pc == 1 && pc < n {
                    let cond = old[m.inputs["cond"][0]];
                    (0..p)
                        .map(|bit| {
                            let yes = (n >> bit) & 1;
                            let no = ((pc + 1) >> bit) & 1;
                            if yes == no {
                                yes
                            } else if yes == 1 {
                                cond
                            } else {
                                b.not(cond)
                            }
                        })
                        .collect::<Vec<_>>()
                } else {
                    Bdd::constant(if pc == n { n } else { pc + 1 }, p)
                };
                for (bit, successor) in enabled_next.iter().enumerate() {
                    let id = m.inputs["pc"][bit];
                    let diff = b.xor(old[id], *successor);
                    let change = b.and(en, diff);
                    expected[id] = b.xor(old[id], change);
                    expected[m.inputs["frame"][bit]] = b.and(en, old[id]);
                }
                if pc == 0 && pc < n {
                    for bit in 0..w {
                        let src = old[m.inputs["src"][bit]];
                        let dest = m.inputs["w"][bit];
                        let diff = b.xor(old[dest], src);
                        let flip = b.and(en, diff);
                        expected[dest] = b.xor(old[dest], flip);
                        expected[m.inputs["frame"][p + bit]] = b.and(en, old[dest]);
                    }
                } else if pc == 1 && pc < n {
                    let cond = old[m.inputs["cond"][0]];
                    let failed = b.and(en, cond);
                    expected[m.inputs["frame"][p + w]] = failed;
                    let f = m.inputs["faulted"][0];
                    expected[f] = b.xor(old[f], failed);
                    for id in &m.inputs["code"][..3] {
                        expected[*id] = b.xor(old[*id], failed);
                    }
                } else if pc < n {
                    let id = m.inputs["w"][pc % w];
                    expected[id] = b.xor(old[id], en);
                }
                let mut state = old.clone();
                m.run_symbolic(&mut b, &mut state, en, false, 1).unwrap();
                assert_eq!(state, expected, "w={w} n={n} pc={pc}");
                m.run_symbolic(&mut b, &mut state, en, true, 1).unwrap();
                assert_eq!(state, old);
                count += 1u64 << (2 * w + 7);
            }
        }
    }
    assert_eq!(count, 1218560);
    println!("step: {count} full-domain tuples, widths1..4, row counts0/1/2/3/17; idle frames retain HALT; inverse restores every bit");
}

#[test]
fn repeated_step_templates_use_distinct_frames_through_static_fixpoint() {
    let mut d = json::parse(include_str!("../../spec/ref-0.9-draft.json")).unwrap();
    let root = v(
        r#"{"kind":"zzz.step.bounds","parameters":["B","W",{"name":"pc","type":"wire","width":1},{"name":"next","type":"wire","width":1},{"name":"frames","type":"wire","width":"B"},{"name":"Rows","type":"templates","length":1},{"name":"Next","type":"templates","length":1}],"body":[{"for":"t","from":0,"to":"B","body":[{"invoke":"step","args":[1,1,1,"pc","next",{"range":"frames","from":"t","to":["+","t",1]},"Rows","Next"]}]}],"scratch":[],"removable":[],"source":"§3.6 repeated frame-binding witness"}"#,
    );
    if let Json::Obj(m) = &mut d {
        if let Some(Json::Arr(rs)) = m.get_mut("recipes") {
            rs.push(root);
        }
    }
    let mut e = Expander::new(&d.to_string(), Registry::new()).unwrap();
    let mut count = 0;
    for width in 1..=4 {
        for bound in [0, 1, 2, 3, 17] {
            let args = v(&format!(
                r#"[{bound},{width},"pc","next","frames",[{{"parameters":[{{"name":"f","type":"wire","width":1}}],"body":[{{"for":"i","from":0,"to":{width},"body":[{{"gate":"cx","wires":["src[i]","w[i]"]}}]}}]}}],[{{"parameters":[{{"name":"f","type":"wire","width":1}},{{"name":"out","type":"wire","width":1}}],"body":[{{"gate":"x","wires":["out"]}}]}}]]"#
            ));
            let bundles = [
                ("pc", 1),
                ("next", 1),
                ("frames", bound),
                ("src", width),
                ("w", width),
            ];
            let leaves = e
                .expand("zzz.step.bounds", args.as_arr().unwrap(), &bundles)
                .unwrap();
            let names = leaves
                .iter()
                .filter(|l| l.gate == "alloc")
                .map(|l| l.allocation.as_ref().unwrap().0.clone())
                .collect::<Vec<_>>();
            assert_eq!(
                names
                    .iter()
                    .collect::<std::collections::BTreeSet<_>>()
                    .len(),
                names.len()
            );
            let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
            let mut b = Bdd::new();
            let mut old = m.symbolic_state();
            for name in ["src", "w"] {
                for id in &m.inputs[name] {
                    old[*id] = b.variable();
                }
            }
            let en = b.variable();
            let mut expected = old.clone();
            if bound > 0 {
                expected[m.inputs["pc"][0]] = en;
                for bit in 0..width {
                    let id = m.inputs["w"][bit];
                    let flip = b.and(en, old[m.inputs["src"][bit]]);
                    expected[id] = b.xor(old[id], flip);
                }
                for id in &m.inputs["frames"][1..] {
                    expected[*id] = en;
                }
            }
            let mut state = old.clone();
            m.run_symbolic(&mut b, &mut state, en, false, 1).unwrap();
            assert_eq!(state, expected);
            m.run_symbolic(&mut b, &mut state, en, true, 1).unwrap();
            assert_eq!(state, old);
            count += 1u64 << (2 * width + 1);
        }
    }
    assert_eq!(count, 3400);
    println!(
        "repeated frame slices: {count} full-domain tuples at widths1..4 and bounds0/1/2/3/17"
    );
}
