use bennett_isa_conformance::{
    json,
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
fn set_if(b: &mut Bdd, d: &mut usize, old: usize, new: usize, condition: usize) {
    let delta = b.xor(old, new);
    let delta = b.and(condition, delta);
    *d = b.xor(*d, delta);
}
fn swap_if(b: &mut Bdd, s: &mut [usize], i: usize, j: usize, c: usize) {
    let delta = b.xor(s[i], s[j]);
    let delta = b.and(c, delta);
    s[i] = b.xor(s[i], delta);
    s[j] = b.xor(s[j], delta);
}
fn get(m: &Machine, s: &[usize], n: &str) -> Vec<usize> {
    m.inputs[n].iter().map(|i| s[*i]).collect()
}
#[test]
fn heterogeneous_recursive_activation_full_symbolic_domains() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut totals = [0u128; 2];
    let mut configs = 0;
    for w in 1usize..=4 {
        for dest in [0usize, 1, 2] {
            for op in ["call", "ret"] {
                if op == "ret" && dest != 0 {
                    continue;
                }
                let narrow = w.saturating_sub(1).max(1);
                let widths = [w, w, narrow, w];
                let names = ["l0", "l1", "l2", "other"];
                let mut bundles = vec![
                    ("depth", 2),
                    ("sp", 4),
                    ("fp", 4),
                    ("stack", 32),
                    ("dfp", 4),
                    ("dsp", 4),
                    ("ddepth", 2),
                    ("flag", 1),
                    ("faulted", 1),
                    ("code", 4),
                    ("cap", w),
                    ("delivery", w),
                    ("succ", 3),
                ];
                bundles.extend(names.iter().copied().zip(widths));
                // Oracle location id0 precedes the selected lproc's
                // parameter/result/local ids1..3 in §3.7 source order.
                let locations = (0..3)
                    .map(|slot| {
                        format!(
                            r#"{{"id":{},"width":{},"slot":{slot},"location":"{}"}}"#,
                            slot + 1,
                            widths[slot],
                            names[slot]
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let locations = format!("[{locations}]");
                let global_indices = [3usize, 0, 1, 2];
                let globals = global_indices
                    .iter()
                    .enumerate()
                    .map(|(id, j)| {
                        format!(
                            r#"{{"id":{id},"width":{},"slot":{},"location":"{}"}}"#,
                            widths[*j],
                            if *j == 3 { 0 } else { *j },
                            names[*j]
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                let globals = format!("[{globals}]");
                let argument = if dest == 0 { "other" } else { "l0" };
                let return_index = if dest == 0 { 1 } else { 3 };
                let common = format!(
                    r#"2,4,5,6,8,4,3,2,4,2,8,{return_index},{dest},"depth","sp","fp","stack","dfp","dsp","ddepth","flag","faulted","code""#
                );
                let actuals = if op == "call" {
                    format!(
                        r#"[{common},{locations},[{{"position":0,"width":{w},"source":"{argument}","parameter":"l0","capture":"cap"}}]]"#
                    )
                } else {
                    format!(
                        r#"[2,4,8,4,3,2,4,{w},{w},"depth","fp","stack","dfp","ddepth","flag","faulted","code","l1","delivery","succ",{locations},{globals}]"#
                    )
                };
                let args = json::parse(&actuals).unwrap();
                let leaves = e
                    .expand(&format!("lift.{op}"), args.as_arr().unwrap(), &bundles)
                    .unwrap();
                let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
                let mut b = Bdd::new();
                let mut state = m.symbolic_state();
                // Decision bits precede data bits, retaining every stack coordinate.
                for name in [
                    "depth", "sp", "fp", "stack", "l0", "l1", "l2", "other", "faulted", "code",
                ] {
                    for id in &m.inputs[name] {
                        state[*id] = b.variable();
                    }
                }
                let ambient = b.variable();
                let old = state.clone();
                let mut want = old.clone();
                let depth = get(&m, &old, "depth");
                let sp = get(&m, &old, "sp");
                let fp = get(&m, &old, "fp");
                let (flag, success, mut domain) = if op == "call" {
                    let mut dx = depth.clone();
                    dx.push(0);
                    let dplus = b.add(&dx, &Bdd::constant(1, 3));
                    let d_bad = b.less(&Bdd::constant(2, 3), &dplus);
                    let mut sx = sp.clone();
                    sx.push(0);
                    let splus = b.add(&sx, &Bdd::constant(6, 5));
                    let s_bad = b.less(&Bdd::constant(8, 5), &splus);
                    let flag = b.or(d_bad, s_bad);
                    let ok = b.not(flag);
                    let active = b.and(ambient, ok);
                    let sp_bad = b.less(&Bdd::constant(8, 4), &sp);
                    let depth_bad = b.less(&Bdd::constant(2, 2), &depth);
                    let invalid = b.or(sp_bad, depth_bad);
                    let domain = b.not(invalid);
                    (flag, active, domain)
                } else {
                    let flag = b.equal(&depth, &Bdd::constant(0, 2));
                    let ok = b.not(flag);
                    let active = b.and(ambient, ok);
                    let invalid = b.less(&Bdd::constant(2, 2), &depth);
                    let domain = b.not(invalid);
                    (flag, active, domain)
                };
                let bad = b.and(ambient, flag);
                want[m.inputs["flag"][0]] = bad;
                let id = m.inputs["faulted"][0];
                want[id] = b.xor(want[id], bad);
                for bit in 0..4 {
                    if (if op == "call" { 3 } else { 4 }) >> bit & 1 == 1 {
                        let id = m.inputs["code"][bit];
                        want[id] = b.xor(want[id], bad);
                    }
                }
                if op == "ret" {
                    for bit in 0..w {
                        want[m.inputs["delivery"][bit]] = b.and(success, old[m.inputs["l1"][bit]]);
                    }
                }
                let mut fp_valid = 0;
                for base in 0usize..=2 {
                    let address = if op == "call" { &sp } else { &fp };
                    let equal = b.equal(address, &Bdd::constant(base, 4));
                    let at = b.and(success, equal);
                    fp_valid = b.or(fp_valid, equal);
                    if op == "call" {
                        // Record cleanliness is a domain precondition, not inferred from a
                        // chosen zero basis sample. Everything outside that record is arbitrary.
                        for cell in base..base + 6 {
                            for bit in 0..4 {
                                let both = b.and(at, old[m.inputs["stack"][4 * cell + bit]]);
                                let valid = b.not(both);
                                domain = b.and(domain, valid);
                            }
                        }
                        for j in 0..3 {
                            for bit in 0..widths[j] {
                                swap_if(
                                    &mut b,
                                    &mut want,
                                    m.inputs[names[j]][bit],
                                    m.inputs["stack"][4 * (base + 3 + j) + bit],
                                    at,
                                );
                            }
                        }
                        for (off, bits) in [
                            (0, Bdd::constant(return_index, 3)),
                            (1, fp.clone()),
                            (2, Bdd::constant(dest, 2)),
                        ] {
                            for (bit, value) in bits.iter().enumerate() {
                                let id = m.inputs["stack"][4 * (base + off) + bit];
                                let d = b.and(at, *value);
                                want[id] = b.xor(want[id], d);
                            }
                        }
                    } else {
                        for bit in 0..3 {
                            let d = b.and(at, old[m.inputs["stack"][4 * base + bit]]);
                            let id = m.inputs["succ"][bit];
                            want[id] = b.xor(want[id], d);
                        }
                        // R2 captures before any callee locations move.
                        for j in 0..3 {
                            for bit in 0..widths[j] {
                                swap_if(
                                    &mut b,
                                    &mut want,
                                    m.inputs[names[j]][bit],
                                    m.inputs["stack"][4 * (base + 3 + j) + bit],
                                    at,
                                );
                            }
                        }
                        let selector = (0..2)
                            .map(|i| old[m.inputs["stack"][4 * (base + 2) + i]])
                            .collect::<Vec<_>>();
                        let mut valid_id = 0;
                        for (id, j) in global_indices.iter().enumerate() {
                            let select = b.equal(&selector, &Bdd::constant(id, 2));
                            if widths[*j] == w {
                                valid_id = b.or(valid_id, select);
                            }
                            let selected = b.and(at, select);
                            for bit in 0..widths[*j] {
                                swap_if(
                                    &mut b,
                                    &mut want,
                                    m.inputs["delivery"][bit],
                                    m.inputs[names[*j]][bit],
                                    selected,
                                );
                            }
                        }
                        let wrong = b.not(valid_id);
                        let wrong = b.and(at, wrong);
                        let legal = b.not(wrong);
                        domain = b.and(domain, legal);
                        for bit in 0..4 {
                            let new = old[m.inputs["stack"][4 * (base + 1) + bit]];
                            let i = m.inputs["dfp"][bit];
                            let d = b.and(at, old[m.inputs["fp"][bit]]);
                            want[i] = b.xor(want[i], d);
                            let i = m.inputs["fp"][bit];
                            set_if(&mut b, &mut want[i], old[i], new, at);
                        }
                    }
                }
                if op == "call" {
                    for bit in 0..w {
                        let d = b.and(success, old[m.inputs[argument][bit]]);
                        want[m.inputs["cap"][bit]] = d;
                        let id = m.inputs["l0"][bit];
                        want[id] = b.xor(want[id], d);
                    }
                    let newsp = b.add(&sp, &Bdd::constant(6, 4));
                    let newdepth = b.add(&depth, &Bdd::constant(1, 2));
                    for (name, slot, bits, new) in [
                        ("fp", "dfp", fp.clone(), sp.clone()),
                        ("sp", "dsp", sp.clone(), newsp),
                        ("depth", "ddepth", depth.clone(), newdepth),
                    ] {
                        for bit in 0..bits.len() {
                            let id = m.inputs[name][bit];
                            set_if(&mut b, &mut want[id], old[id], new[bit], success);
                            want[m.inputs[slot][bit]] = b.and(success, bits[bit]);
                        }
                    }
                } else {
                    let invalid = b.not(fp_valid);
                    let invalid = b.and(success, invalid);
                    let legal = b.not(invalid);
                    domain = b.and(domain, legal);
                    let newdepth = b.add(&depth, &Bdd::constant(3, 2));
                    for bit in 0..2 {
                        let id = m.inputs["depth"][bit];
                        set_if(&mut b, &mut want[id], old[id], newdepth[bit], success);
                        want[m.inputs["ddepth"][bit]] = b.and(success, old[id]);
                    }
                }
                m.run_symbolic(&mut b, &mut state, ambient, false, domain)
                    .unwrap();
                for (i, v) in state.iter().enumerate() {
                    assert!(
                        b.equivalent(*v, want[i], domain),
                        "{op} W={w} dest={dest} bit={i}"
                    );
                }
                m.run_symbolic(&mut b, &mut state, ambient, true, domain)
                    .unwrap();
                for (i, v) in state.iter().enumerate() {
                    assert!(
                        b.equivalent(*v, old[i], domain),
                        "inverse {op} W={w} bit={i}"
                    );
                }
                let count = b.satisfying_count(domain).unwrap();
                totals[usize::from(op == "ret")] += count;
                configs += 1;
                println!("activation {op}: W={w}, destination={dest}, {count} full-domain tuples");
            }
        }
    }
    assert_eq!(configs, 16);
    println!("activation totals call={} ret={}", totals[0], totals[1]);
}
