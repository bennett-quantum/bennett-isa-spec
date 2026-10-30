//! Differential soundness check of the §6 static discipline against the exact
//! §14.2 simulator: every randomly generated action that `Object::parse`
//! accepts must, on every basis input of its interface and on a superposed
//! input, release its workspace exactly clean (§4.3 qfree, §6.3 T5) and
//! restore its local state. A failure prints the witness program.
mod support;
use bennett_isa_ref::{Object, QuantumBindings, QuantumState};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        // xorshift64*
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
    fn chance(&mut self, percent: u64) -> bool {
        self.below(100) < percent
    }
}
#[derive(Clone)]
struct Op {
    line: String,
    adjoint: String,
}
fn op(line: &str, adjoint: &str) -> Op {
    Op {
        line: line.into(),
        adjoint: adjoint.into(),
    }
}
/// A gate on workspace wire `d`, controlled from `controls`.
fn compute_op(r: &mut Rng, d: &str, controls: &[String]) -> Op {
    let c = |r: &mut Rng| controls[r.below(controls.len() as u64) as usize].clone();
    match r.below(16) {
        0 => op(&format!("x {d}"), &format!("x {d}")),
        1 | 2 => {
            let c1 = c(r);
            op(&format!("cx {c1} {d}"), &format!("cx {c1} {d}"))
        }
        3 => {
            let (c1, c2) = (c(r), c(r));
            if c1 == c2 {
                op(&format!("cx {c1} {d}"), &format!("cx {c1} {d}"))
            } else {
                op(&format!("ccx {c1} {c2} {d}"), &format!("ccx {c1} {c2} {d}"))
            }
        }
        4 => op(&format!("h {d}"), &format!("h {d}")),
        5 => op(&format!("t {d}"), &format!("tdg {d}")),
        6 => op(&format!("s {d}"), &format!("sdg {d}")),
        7 => op(&format!("rz 0.5 {d}"), &format!("rz -0.5 {d}")),
        8 => op(&format!("ry 0.75 {d}"), &format!("ry -0.75 {d}")),
        9 => {
            let c1 = c(r);
            let (a, b) = (compute_op(r, d, controls), compute_op(r, d, controls));
            op(
                &format!("qif {c1} {{\n{}\n}} {{\n{}\n}}", a.line, b.line),
                &format!("qif {c1} {{\n{}\n}} {{\n{}\n}}", a.adjoint, b.adjoint),
            )
        }
        10 => {
            let n = r.below(4);
            let a = compute_op(r, d, controls);
            op(
                &format!("qloop {n} {{\n{}\n}}", a.line),
                &format!("qloop {n} {{\n{}\n}}", a.adjoint),
            )
        }
        11 => {
            let (c1, c2) = (c(r), c(r));
            let a = compute_op(r, d, controls);
            if c1 == c2 {
                return a;
            }
            op(
                &format!(
                    "qpred {c1}\nqpred {c2}\n{}\nqunpred {c2}\nqunpred {c1}",
                    a.line
                ),
                &format!(
                    "qpred {c1}\nqpred {c2}\n{}\nqunpred {c2}\nqunpred {c1}",
                    a.adjoint
                ),
            )
        }
        13 | 14 => {
            let c1 = c(r);
            let gate = if r.chance(50) { "swap" } else { "qexch" };
            let line = format!("{gate} {d} {c1}");
            op(&line, &line)
        }
        15 => {
            let line = format!("qexchm r p {d}");
            op(&line, &line)
        }
        _ => {
            let c1 = c(r);
            op(
                &format!("qpred {c1}\nx {d}\nqunpred {c1}"),
                &format!("qpred {c1}\nx {d}\nqunpred {c1}"),
            )
        }
    }
}
/// A middle operation: reads workspace into the interface, or disturbs it.
fn middle_op(r: &mut Rng, work: &[String]) -> String {
    let d = &work[r.below(work.len() as u64) as usize];
    match r.below(23) {
        0..=2 => format!("cx {d} t"),
        3 => format!("s {d}"),
        4 => "x t".into(),
        5 => "h t".into(),
        6 => format!("qpred {d}\nx t\nqunpred {d}"),
        7 => "s p".into(),
        8 => format!("qif {d} {{\nx t\n}} {{\ns t\n}}"),
        9 => "x m".into(),
        10 => "h m".into(),
        11 => format!("cx {d} m"),
        12 => "qxor t m".into(),
        13 => "qexch t m".into(),
        14 => format!("ry 0.25 {d}"),
        15 => "qstore r p t".into(),
        16 => format!("qstore r {d} t\nqstore r {d} t"),
        17 => format!("qstore r p {d}\nqstore r p {d}"),
        18 => format!("qloop 2 {{\ncx {d} t\n}}"),
        20 => format!("swap {d} m"),
        21 => format!("qexch {d} m"),
        22 => format!("qexchm r p {d}"),
        _ => format!("cz {d} t"),
    }
}
fn mutate(r: &mut Rng, uncompute: &mut Vec<String>) {
    if uncompute.is_empty() {
        return;
    }
    let i = r.below(uncompute.len() as u64) as usize;
    match r.below(4) {
        0 => {
            uncompute.remove(i);
        }
        1 if uncompute.len() > 1 => {
            let j = (i + 1) % uncompute.len();
            uncompute.swap(i, j);
        }
        2 => {
            uncompute[i] = uncompute[i]
                .replace(" t\n", " p\n")
                .replace("cx t ", "cx p ")
        }
        _ => uncompute.insert(i, "x t".into()),
    }
}
fn program(r: &mut Rng) -> (String, Vec<u64>) {
    let mut lines = Vec::new();
    let mut work: Vec<String> = Vec::new();
    let mut releases = Vec::new();
    let mut compute = Vec::new();
    let count = 1 + r.below(2) as usize;
    for k in 0..count {
        let d = format!("d{k}");
        match r.below(4) {
            0 => {
                let s = ["t", "m", "p"][r.below(3) as usize];
                lines.push(format!("{d} = qcopy {s}"));
                releases.push(format!("quncopy {s} {d}"));
            }
            1 => {
                let a = ["t", "m", "p"][r.below(3) as usize];
                lines.push(format!("{d} = qload r {a}"));
                releases.push(format!("qunload r {a} {d}"));
            }
            _ => {
                lines.push(format!("{d} = qalloc any 1"));
                releases.push(format!("qfree {d}"));
            }
        }
        work.push(d);
    }
    let mut controls: Vec<String> = vec!["t".into(), "m".into(), "p".into()];
    controls.extend(work.iter().cloned());
    for _ in 0..1 + r.below(4) {
        let d = work[r.below(work.len() as u64) as usize].clone();
        let others: Vec<String> = controls.iter().filter(|c| **c != d).cloned().collect();
        compute.push(compute_op(r, &d, &others));
    }
    for c in &compute {
        lines.push(c.line.clone());
    }
    for _ in 0..r.below(3) {
        lines.push(middle_op(r, &work));
    }
    let mut uncompute: Vec<String> = compute.iter().rev().map(|c| c.adjoint.clone()).collect();
    if r.chance(35) {
        mutate(r, &mut uncompute);
    }
    lines.extend(uncompute);
    lines.extend(releases.into_iter().rev());
    let mut body = format!("qregion r 1 1\n{}", lines.join("\n"));
    if r.chance(25) {
        let n = r.below(4);
        body = format!("qregion r 1 1\nqloop {n} {{\n{}\n}}", lines.join("\n"));
    }
    // §5.3: the bounds list holds every qloop trip count in text order.
    let bounds = body
        .lines()
        .filter_map(|l| l.strip_prefix("qloop "))
        .map(|l| l.split(' ').next().unwrap().parse().unwrap())
        .collect();
    (body, bounds)
}
fn object(body: &str, bounds: &[u64]) -> String {
    let values: Vec<String> = bounds.iter().map(u64::to_string).collect();
    let signature = format!(
        r#"{{"abi":"0.8","args":[],"bounds":[{}],"in":[1],"kind":"action","mem":[],"mut":[1,1],"tier":"q"}}"#,
        values.join(",")
    );
    let regs = if bounds.is_empty() {
        String::new()
    } else {
        format!("regs {{  }} bounds {{{}}}\n", values.join(", "))
    };
    support::object(
        &[(
            "a",
            &signature,
            &format!(
                "action @a (t:wire mut, m:wire mut) (p:wire in) () () ()\n{regs}^entry():\n{body}"
            ),
        )],
        "{}",
        "",
    )
}
/// Runs @a on |t m p> for all basis values and on H|0> H|0> H|0>.
fn runs_clean(o: &Object) -> Result<(), String> {
    for input in 0..9u32 {
        let mut q = QuantumState::new();
        let wires = q.allocate(3).map_err(|e| e.to_string())?;
        for (bit, wire) in wires.iter().enumerate() {
            if input == 8 {
                q.gate("h", &[*wire], None, &[])
                    .map_err(|e| e.to_string())?;
            } else if input >> bit & 1 == 1 {
                q.gate("x", &[*wire], None, &[])
                    .map_err(|e| e.to_string())?;
            }
        }
        let mut b = QuantumBindings::default();
        for (name, wire) in ["t", "m", "p"].iter().zip(&wires) {
            b.bundles.insert((*name).into(), vec![*wire]);
        }
        let before = q.clone();
        o.execute_action("a", &mut q, &b, false, &[])
            .map_err(|e| format!("input {input}: {e}"))?;
        if q.wire_count() != 3 {
            return Err(format!("input {input}: workspace survived"));
        }
        o.execute_action("a", &mut q, &b, true, &[])
            .map_err(|e| format!("input {input} adjoint: {e}"))?;
        if q != before {
            return Err(format!("input {input}: adjoint is not the inverse"));
        }
    }
    Ok(())
}
fn sweep(seed: u64, count: usize) -> (usize, usize) {
    let mut r = Rng(seed);
    let (mut accepted, mut rejected) = (0, 0);
    for _ in 0..count {
        let (body, bounds) = program(&mut r);
        let text = object(&body, &bounds);
        match Object::parse(&text) {
            Ok(o) => {
                accepted += 1;
                if let Err(e) = runs_clean(&o) {
                    panic!("§6 accepted an ill-formed action ({e}):\n{body}");
                }
            }
            Err(_) => rejected += 1,
        }
    }
    (accepted, rejected)
}
#[test]
fn accepted_random_actions_run_exactly_clean() {
    let (accepted, rejected) = sweep(0x5eed_0001, 400);
    assert!(
        accepted > 40,
        "generator accepts too rarely: {accepted}/{rejected}"
    );
}
#[test]
#[ignore = "long differential sweep; run with --ignored and record in PROGRESS.md"]
fn accepted_random_actions_run_exactly_clean_sweep() {
    let mut total = (0, 0);
    for seed in 2..12u64 {
        let (a, r) = sweep(0x5eed_0000 + seed, 20_000);
        total = (total.0 + a, total.1 + r);
    }
    let (accepted, rejected) = total;
    println!("§6 differential sweep: {accepted} accepted (all clean), {rejected} rejected");
}

#[test]
fn generated_borrow_frames_keep_differing_classical_bindings_distinct() {
    use bennett_isa_ref::{Argument, Budget};
    let os = r#"{"abi":"0.8","args":["i1"],"fault":4,"in":[],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    let ac = r#"{"abi":"0.8","args":["token","i1","i1"],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[4],"tier":"q"}"#;
    let mut r = Rng(0x0b07_7002);
    for _ in 0..80 {
        let mut parity = [false; 2];
        let mut body = "perm_action @a (f:wire[4] mut) () () (tk:token, x:i1, y:i1) ()\n^entry():\nd = qborrow any 1\n".to_string();
        for _ in 0..2 * (1 + r.below(3)) {
            let j = r.below(2) as usize;
            parity[j] ^= true;
            body.push_str(&format!(
                "oracle_call @o tk () d f () ({})\n",
                ["x", "y"][j]
            ));
        }
        body.push_str("qreturn d");
        let text = support::object(
            &[
                (
                    "o",
                    os,
                    "oracle @o () (t:wire) (f:wire[4]) () (n:i1)\n^entry():\nlift.retval n",
                ),
                ("a", ac, &body),
            ],
            "{}",
            "",
        );
        let parsed = Object::parse(&text);
        // d' = d XOR parity_x*x XOR parity_y*y. This is identity for ALL
        // classical bindings iff both formal-argument occurrence counts are even.
        if parity.iter().any(|v| *v) {
            assert!(parsed.unwrap_err().to_string().contains("§5.6"));
            continue;
        }
        let o = parsed.unwrap();
        for input in 0..16u32 {
            let mut q = QuantumState::new();
            let wires = q.allocate(5).unwrap();
            for (bit, w) in wires.iter().enumerate() {
                if (input & 3) >> (bit % 2) & 1 == 1 {
                    q.gate("x", &[*w], None, &[]).unwrap();
                }
            }
            let before = q.clone();
            let mut b = QuantumBindings::default();
            b.bundles.insert("f".into(), wires[..4].to_vec());
            b.borrowed.push(vec![wires[4]]);
            b.args
                .insert("tk".into(), Argument::Token(Budget::steps(1)));
            b.args
                .insert("x".into(), Argument::Bits((input >> 2 & 1) as u128));
            b.args
                .insert("y".into(), Argument::Bits((input >> 3 & 1) as u128));
            o.execute_action("a", &mut q, &b, false, &[]).unwrap();
            assert_eq!(q, before, "{body}");
            o.execute_action("a", &mut q, &b, true, &[]).unwrap();
            assert_eq!(q, before, "{body}");
        }
    }
}

#[test]
fn generated_child_call_wrapping_preserves_bell_cleanup_obligations() {
    let cs = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1,1],"tier":"q"}"#;
    let ps = cs
        .replace("\"in\":[]", "\"in\":[1]")
        .replace("perm_action", "action");
    for kind in ["swap", "cx", "region"] {
        for count in 0..6 {
            for wrapped in [false, true] {
                let (child_sig, child, decl, direct, forward, inverse, middle) = if kind == "region"
                {
                    (
                        r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[{"A":1,"W":1,"role":"mut"}],"mut":[1],"tier":"q"}"#,
                        "perm_action @child (d:wire mut) (p:wire in) (r:region[1,1] mut) () ()\n^entry():\nqexchm r p d".to_string(),
                        "qregion r 1 1\n",
                        "qexchm r p d".to_string(),
                        "action_call @child (d) (p) (r) ()",
                        "adj action_call @child (d) (p) (r) ()",
                        format!("z = qload r p\n{}qunload r p z\n", "cx z t\n".repeat(count)),
                    )
                } else {
                    (cs, format!("perm_action @child (a:wire mut, b:wire mut) () () () ()\n^entry():\n{kind} a b"), "", format!("{kind} d m"),
                     "action_call @child (d, m) () () ()", "adj action_call @child (d, m) () () ()", "cx m t\n".repeat(count))
                };
                let (compute, uncompute) = if wrapped {
                    (forward, inverse)
                } else {
                    (direct.as_str(), direct.as_str())
                };
                let body = format!("action @a (t:wire mut, m:wire mut) (p:wire in) () () ()\n^entry():\n{decl}d = qalloc any 1\nh d\n{compute}\n{middle}{uncompute}\nh d\nqfree d");
                let text = support::object(&[("child",child_sig,&child),("a",&ps,&body)],"{}", "\nsection layout\n{\"A_tab\":0,\"H\":0,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n");
                // For each operation, odd CX copies the H-created bit into t;
                // even CX is identity. Calling must preserve this obligation.
                if count % 2 == 1 {
                    assert!(
                        Object::parse(&text)
                            .unwrap_err()
                            .to_string()
                            .contains("§6.2"),
                        "{kind}, wrapped={wrapped}, count={count}"
                    );
                } else {
                    let o = Object::parse(&text).unwrap();
                    runs_clean(&o).unwrap();
                }
            }
        }
    }
}

#[test]
fn generated_changing_input_opaque_loops_match_independent_parity() {
    use bennett_isa_ref::{Argument, Budget};
    let os =
        r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
    for flip in [false, true] {
        for n in 0..8 {
            for nested in [false, true] {
                let count = if nested { 2 * n } else { n };
                let bounds = if nested {
                    format!("[2,{n}]")
                } else {
                    format!("[{n}]")
                };
                let regs = if nested {
                    format!("2, {n}")
                } else {
                    n.to_string()
                };
                let ps = format!(
                    r#"{{"abi":"0.8","args":["token"],"bounds":{bounds},"in":[],"kind":"perm_action","mem":[],"mut":[1,4],"tier":"q"}}"#
                );
                let body = format!(
                    "qloop {n} {{\noracle_call @o tk (t) d f () ()\n{}\n}}",
                    if flip { "x t" } else { "" }
                );
                let body = if nested {
                    format!("qloop 2 {{\n{body}\n}}")
                } else {
                    body
                };
                let body = body.replace("\n\n", "\n");
                let text = support::object(&[("o",os,"oracle @o (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x"),("a",&ps,&format!("perm_action @a (t:wire mut, f:wire[4] mut) () () (tk:token) ()\nregs {{  }} bounds {{{regs}}}\n^entry():\nd = qborrow any 1\n{body}\nqreturn d"))],"{}", "");
                // Identity oracle: without flips, d' = d XOR (count%2)*t;
                // with flips, pairs contribute t XOR !t = 1. Identity on d
                // for ALL t iff count is even (no flips) or a multiple of 4.
                let valid = count % (if flip { 4 } else { 2 }) == 0;
                let parsed = Object::parse(&text);
                if !valid {
                    assert!(parsed.unwrap_err().to_string().contains("§5.6"), "{body}");
                    continue;
                }
                let o = parsed.unwrap_or_else(|e| panic!("{e}: {body}"));
                for input in 0..65u32 {
                    let mut q = QuantumState::new();
                    let w = q.allocate(6).unwrap();
                    for (bit, wire) in w.iter().enumerate() {
                        if input == 64 {
                            q.gate("h", &[*wire], None, &[]).unwrap();
                        } else if input >> bit & 1 == 1 {
                            q.gate("x", &[*wire], None, &[]).unwrap();
                        }
                    }
                    let before = q.clone();
                    let mut b = QuantumBindings::default();
                    b.bundles.insert("t".into(), vec![w[0]]);
                    b.bundles.insert("f".into(), w[1..5].to_vec());
                    b.borrowed.push(vec![w[5]]);
                    b.args
                        .insert("tk".into(), Argument::Token(Budget::steps(1)));
                    o.execute_action("a", &mut q, &b, false, &[]).unwrap();
                    assert_eq!(q, before, "{body}");
                    o.execute_action("a", &mut q, &b, true, &[]).unwrap();
                    assert_eq!(q, before, "{body}");
                }
            }
        }
    }
}

#[test]
fn generated_loop_local_borrow_returns_match_alpha_renamed_unrollings() {
    // Independent expectation: each lifetime's X parity must be even, not
    // merely the product of trip count and parity. No existing RNG is changed.
    let mut counts = (0, 0);
    for trips in 1..=4u64 {
        for flips in 1..=2usize {
            for nested in [false, true] {
                for unrolled in [false, true] {
                    let lifetime = |name: &str| {
                        format!(
                            "{name} = qborrow any 1\n{}qreturn {name}\n",
                            format!("x {name}\n").repeat(flips)
                        )
                    };
                    let (mut body, mut bounds) = if unrolled {
                        (
                            (0..trips)
                                .map(|j| lifetime(&format!("b{j}")))
                                .collect::<String>(),
                            vec![],
                        )
                    } else {
                        (
                            format!("qloop {trips} {{\n{}}}", lifetime("b")),
                            vec![trips],
                        )
                    };
                    if nested {
                        body = format!("qloop 2 {{\n{}\n}}", body.trim_end());
                        bounds.insert(0, 2);
                    }
                    let text = support::borrow_lifetime_object(&body, &bounds);
                    if flips == 1 {
                        let e = Object::parse(&text).expect_err("one-X lifetime accepted");
                        assert!(e.to_string().contains("§5.6"), "{e}");
                        counts.1 += 1;
                    } else {
                        let o = Object::parse(&text).unwrap();
                        support::borrow_identity(&o);
                        counts.0 += 1;
                    }
                }
            }
        }
    }
    assert_eq!(counts, (16, 16));
}

#[test]
fn generated_fresh_borrow_cx_loops_match_per_lifetime_maps() {
    // Fresh beta_i is universally quantified for EVERY return: neither an
    // even number of iterations nor alpha-renaming can cancel its influence.
    let mut counts = (0, 0);
    for trips in 1..=4u64 {
        for gate in ["cx d t", "cx t d"] {
            for copies in 1..=2 {
                for nested in [false, true] {
                    for unrolled in [false, true] {
                        let lifetime = |name: &str| {
                            format!(
                                "{name} = qborrow any 1\n{}qreturn {name}\n",
                                format!("{gate}\n").replace('d', name).repeat(copies)
                            )
                        };
                        let (mut body, mut bounds) = if unrolled {
                            (
                                (0..trips)
                                    .map(|i| lifetime(&format!("b{i}")))
                                    .collect::<String>(),
                                vec![],
                            )
                        } else {
                            (
                                format!("qloop {trips} {{\n{}}}", lifetime("d")),
                                vec![trips],
                            )
                        };
                        if nested {
                            body = format!("qloop 2 {{\n{}\n}}", body.trim_end());
                            bounds.insert(0, 2);
                        }
                        let text = support::borrow_lifetime_object(&body, &bounds);
                        if copies == 1 {
                            assert!(
                                Object::parse(&text)
                                    .unwrap_err()
                                    .to_string()
                                    .contains("§5.6"),
                                "{body}"
                            );
                            counts.1 += 1;
                        } else {
                            support::borrow_identity(&Object::parse(&text).unwrap());
                            counts.0 += 1;
                        }
                    }
                }
            }
        }
    }
    assert_eq!(counts, (32, 32));
}

#[test]
fn generated_nested_calls_with_regions_and_local_borrows_match_boolean_maps() {
    // V: b ^= p; W: t ^= b, r[p] ^= b. V;W;V;W is exactly
    // (t,r[0],r[1],p,b) -> (t XOR p,r[0],r[1] XOR p,p,b).
    // Dropping the second W leaves beta in both outputs iff W's count is odd.
    // Expected states are built from this independent map, not another execution.
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[1],"kind":"perm_action","mem":[{"A":1,"W":1,"role":"mut"}],"mut":[1],"tier":"q"}"#;
    let mut counts = (0, 0);
    for trips in 1..=4u64 {
        for copies in 0..=3 {
            for complete in [false, true] {
                for wrapped in [false, true] {
                    for nested in [false, true] {
                        let w = "cx b t\nqstore r p b\n".repeat(copies);
                        let leaf = format!("perm_action @leaf (t:wire mut) (p:wire in) (r:region[1,1] mut) () ()\n^entry():\nb = qborrow any 1\ncx p b\n{w}cx p b\n{}qreturn b", if complete { &w } else { "" });
                        let middle = "perm_action @middle (u:wire mut) (v:wire in) (s:region[1,1] mut) () ()\n^entry():\naction_call @leaf (u) (v) (s) ()";
                        let call = format!(
                            "action_call @{} (t) (p) (r) ()",
                            if wrapped { "middle" } else { "leaf" }
                        );
                        let (body, bounds) = if nested {
                            (
                                format!("qloop 2 {{\nqloop {trips} {{\n{call}\n}}\n}}"),
                                vec![2, trips],
                            )
                        } else {
                            (format!("qloop {trips} {{\n{call}\n}}"), vec![trips])
                        };
                        let ps = sig.replace("\"bounds\":[]", &format!("\"bounds\":{bounds:?}"));
                        let regs = bounds
                            .iter()
                            .map(u64::to_string)
                            .collect::<Vec<_>>()
                            .join(", ");
                        let parent = format!("perm_action @a (t:wire mut) (p:wire in) (r:region[1,1] mut) () ()\nregs {{  }} bounds {{{regs}}}\n^entry():\n{body}");
                        let text = support::object(&[("leaf", sig, &leaf), ("middle", sig, middle), ("a", &ps, &parent)], "{}", "\nsection layout\n{\"A_tab\":0,\"H\":0,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n");
                        if !complete && copies % 2 == 1 {
                            assert!(
                                Object::parse(&text)
                                    .unwrap_err()
                                    .to_string()
                                    .contains("§5.6"),
                                "{leaf}\n{body}"
                            );
                            counts.1 += 1;
                            continue;
                        }
                        let o = Object::parse(&text).unwrap();
                        counts.0 += 1;
                        for input in 0..34u32 {
                            let mut q = QuantumState::new();
                            let wires = q.allocate(6).unwrap();
                            for (bit, wire) in wires[..5].iter().enumerate() {
                                if input == 32 {
                                    q.gate("h", &[*wire], None, &[]).unwrap();
                                } else if input == 33 {
                                    if bit == 0 || bit == 1 || bit == 4 {
                                        q.gate("h", &[*wire], None, &[]).unwrap();
                                    }
                                } else if input >> bit & 1 == 1 {
                                    q.gate("x", &[*wire], None, &[]).unwrap();
                                }
                            }
                            if input == 33 {
                                q.gate("cx", &[wires[4], wires[5]], None, &[]).unwrap();
                            }
                            let entry = q.clone();
                            let mut expected = entry.clone();
                            if complete && copies % 2 == 1 && trips % 2 == 1 && !nested {
                                expected
                                    .gate("cx", &[wires[1], wires[0]], None, &[])
                                    .unwrap();
                                expected
                                    .gate("cx", &[wires[1], wires[3]], None, &[])
                                    .unwrap();
                            }
                            let mut b = QuantumBindings::default();
                            b.bundles.insert("t".into(), vec![wires[0]]);
                            b.bundles.insert("p".into(), vec![wires[1]]);
                            b.regions.insert(
                                "r".into(),
                                bennett_isa_ref::QuantumRegion {
                                    address_width: 1,
                                    cell_width: 1,
                                    cells: vec![vec![wires[2]], vec![wires[3]]],
                                },
                            );
                            b.borrowed.push(vec![wires[4]]);
                            o.execute_action("a", &mut q, &b, false, &[]).unwrap();
                            assert_eq!(q, expected, "input={input}, {leaf}\n{body}");
                            o.execute_action("a", &mut q, &b, true, &[]).unwrap();
                            assert_eq!(q, entry, "adjoint, input={input}, {leaf}\n{body}");
                        }
                    }
                }
            }
        }
    }
    assert_eq!(counts, (96, 32));
}

#[test]
fn generated_r5_certificate_and_clean_context_lifetimes_match_independent_maps() {
    let (mut accepted, mut rejected) = (0, 0);
    // Independent per-lifetime Boolean equations, irrespective of trip parity:
    // one X: beta'=beta^1; one CX: beta'=beta^t OR t'=t^beta.
    // Two identical gates cancel within the SAME lifetime.
    for n in 1..=4 {
        for gate in ["x b", "cx b t", "cx t b"] {
            for repetitions in 1..=2 {
                for nested in [false, true] {
                    let gates = vec![gate; repetitions].join("\n");
                    let inner = format!("qloop {n} {{\nb = qborrow any 1\n{gates}\nqreturn b\n}}");
                    let (body, bounds) = if nested {
                        (format!("qloop 2 {{\n{inner}\n}}"), vec![2, n])
                    } else {
                        (inner, vec![n])
                    };
                    for certified in [false, true] {
                        let text = if certified {
                            support::certified_borrow_identity(&body, &bounds)
                        } else {
                            support::borrow_lifetime_object(&body, &bounds)
                        };
                        if repetitions == 2 {
                            let o = Object::parse(&text).unwrap();
                            support::borrow_identity(&o);
                            if certified {
                                o.verify_certificate().unwrap();
                            }
                            accepted += 1;
                        } else {
                            let e = Object::parse(&text).unwrap_err().to_string();
                            assert!(e.contains("§5.6"), "{e}");
                            rejected += 1;
                        }
                    }
                }
            }
        }
        for dirty in [false, true] {
            for unrolled in [false, true] {
                let (lifetimes, bounds) = if unrolled {
                    (
                        (0..n)
                            .map(|j| format!("b{j} = qborrow any 1\ncx d b{j}\nqreturn b{j}"))
                            .collect::<Vec<_>>()
                            .join("\n"),
                        vec![],
                    )
                } else {
                    (
                        format!("qloop {n} {{\nb = qborrow any 1\ncx d b\nqreturn b\n}}"),
                        vec![n],
                    )
                };
                let x = if dirty { "x d\n" } else { "" };
                let body = format!("d = qalloc any 1\n{x}{lifetimes}\n{x}qfree d");
                for certified in [false, true] {
                    let text = if certified {
                        support::certified_borrow_identity(&body, &bounds)
                    } else {
                        support::borrow_lifetime_object(&body, &bounds)
                    };
                    if dirty {
                        assert!(Object::parse(&text)
                            .unwrap_err()
                            .to_string()
                            .contains("§5.6"));
                        rejected += 1;
                    } else {
                        let o = Object::parse(&text).unwrap();
                        support::borrow_identity(&o);
                        if certified {
                            o.verify_certificate().unwrap();
                        }
                        accepted += 1;
                    }
                }
            }
        }
    }
    assert_eq!((accepted, rejected), (64, 64));
}

#[test]
fn generated_r5_alpha_renaming_consumed_workspace_preserves_widths() {
    let mut count = 0;
    for n in 1..=3 {
        for width in 1..=3 {
            for second in ["d", "e"] {
                let body = format!("qloop {n} {{\nd = qalloc any 1\ncx t d\ncx t d\nqfree d\n{second} = qalloc any {width}\nqfree {second}\n}}");
                let o = Object::parse(&support::borrow_lifetime_object(&body, &[n])).unwrap();
                support::borrow_identity(&o);
                count += 1;
            }
        }
    }
    assert_eq!(count, 18);
}

#[test]
fn generated_r5_one_iteration_cleanup_and_nonpermutation_certificates() {
    let (mut accepted, mut rejected) = (0, 0);
    for inner_borrow in [false, true] {
        for wrapped in [false, true] {
            for dirty in [false, true] {
                let release = if inner_borrow {
                    "b = qborrow any 1\nqfree c\nqreturn b"
                } else {
                    "qfree c"
                };
                let release = if wrapped {
                    format!("qloop 1 {{\n{release}\n}}")
                } else {
                    release.into()
                };
                let b = if inner_borrow {
                    ""
                } else {
                    "b = qborrow any 1\n"
                };
                let ret = if inner_borrow { "" } else { "\nqreturn b" };
                let x = if dirty { "x c\n" } else { "" };
                let body = format!("c = qalloc any 1\n{x}{b}{release}{ret}");
                let bounds: &[u64] = if wrapped { &[1] } else { &[] };
                for certified in [false, true] {
                    let text = if certified {
                        support::certified_borrow_identity(&body, bounds)
                    } else {
                        support::borrow_lifetime_object(&body, bounds)
                    };
                    // Independent qalloc=0 and qloop-1 equivalence: precisely
                    // X-before-free is dirty, regardless of wrapper/certificate.
                    if dirty {
                        assert!(Object::parse(&text).is_err());
                        rejected += 1;
                    } else {
                        let o = Object::parse(&text).unwrap();
                        support::borrow_identity(&o);
                        if certified {
                            o.verify_certificate().unwrap();
                        }
                        accepted += 1;
                    }
                }
            }
        }
    }
    for gate in ["cz b t", "s b", "h b"] {
        for wrapped in [false, true] {
            let lifetime = format!("b = qborrow any 1\n{gate}\n{gate}\nqreturn b");
            let body = if wrapped {
                format!("qloop 1 {{\n{lifetime}\n}}")
            } else {
                lifetime
            };
            let bounds: &[u64] = if wrapped { &[1] } else { &[] };
            for certified in [false, true] {
                let text = if certified {
                    support::certified_borrow_identity_kind(&body, bounds, "action")
                } else {
                    support::borrow_lifetime_object(&body, bounds).replace("perm_action", "action")
                };
                assert!(Object::parse(&text)
                    .unwrap_err()
                    .to_string()
                    .contains("§5.6"));
                rejected += 1;
            }
        }
    }
    assert_eq!((accepted, rejected), (8, 20));
}

#[test]
fn generated_r6_disjoint_h_boundaries_match_identity_and_renamed_unrollings() {
    // c=0 throughout; CX(c,beta_j)=I for every fresh beta_j. H_t^2=I.
    // Move H across allocation/borrow, also inside loop lifetimes. No RNG use.
    let mut accepted = 0;
    for trips in 0..=3 {
        for position in 0..3 {
            for nested in [false, true] {
                for unrolled in [false, true] {
                    let lifetime = |name: &str| {
                        let (before, after) = if position == 2 {
                            ("", "h t\n")
                        } else {
                            ("h t\n", "")
                        };
                        format!("{before}{name} = qborrow any 1\n{after}cx c {name}\nqreturn {name}\nh t\n")
                    };
                    let (mut body, mut bounds) = if unrolled {
                        (
                            (0..trips)
                                .map(|j| lifetime(&format!("b{j}")))
                                .collect::<String>(),
                            vec![],
                        )
                    } else {
                        (
                            format!("qloop {trips} {{\n{}}}\n", lifetime("b")),
                            vec![trips],
                        )
                    };
                    if body.is_empty() {
                        body = "x t\nx t\n".into();
                    }
                    if nested {
                        body = format!("qloop 2 {{\n{body}}}\n");
                        bounds.insert(0, 2);
                    }
                    body = format!("c = qalloc any 1\n{body}qfree c");
                    if position == 0 {
                        // An additional pair commuted outside allocation gives
                        // a boundary before c exists as well as the inner ones.
                        body = format!("h t\n{body}\nh t");
                    }
                    let text = support::borrow_lifetime_object(&body, &bounds)
                        .replace("perm_action", "action");
                    let o = Object::parse(&text).unwrap_or_else(|e| panic!("{e}\n{body}"));
                    support::r6_map(&o, false);
                    accepted += 1;
                }
            }
        }
    }
    assert_eq!(accepted, 48);
}

#[test]
fn generated_r6_dynamic_width_lifetimes_match_identity_and_renamed_unrollings() {
    // Every allocation is zero (or an independently restored borrowed factor);
    // X on the last wire occurs twice in the same lifetime. Zero trips leave
    // the original width-1 d live. Renamed unrolling defines the same identity.
    let mut accepted = 0;
    for trips in 0..=3 {
        for width in 1..=3 {
            for borrowed in [false, true] {
                for nested in [false, true] {
                    for unrolled in [false, true] {
                        let (alloc, free) = if borrowed {
                            ("qborrow", "qreturn")
                        } else {
                            ("qalloc", "qfree")
                        };
                        let (body, bounds) = if unrolled {
                            let mut body = format!("d0 = {alloc} any 1\n");
                            let total = trips * if nested { 2 } else { 1 };
                            for j in 0..total {
                                body.push_str(&format!("{free} d{j}\nd{} = {alloc} any {width}\nx d{}[{}]\nx d{}[{}]\n",j+1,j+1,width-1,j+1,width-1));
                            }
                            body.push_str(&format!("{free} d{total}"));
                            (body, vec![])
                        } else {
                            let mut body=format!("qloop {trips} {{\n{free} d\nd = {alloc} any {width}\nx d[{}]\nx d[{}]\n}}",width-1,width-1);
                            let mut bounds = vec![trips];
                            if nested {
                                body = format!("qloop 2 {{\n{body}\n}}");
                                bounds.insert(0, 2);
                            }
                            (format!("d = {alloc} any 1\n{body}\n{free} d"), bounds)
                        };
                        let o = Object::parse(&support::borrow_lifetime_object(&body, &bounds))
                            .unwrap_or_else(|e| panic!("{e}\n{body}"));
                        support::r6_map(&o, false);
                        accepted += 1;
                    }
                }
            }
        }
    }
    assert_eq!(accepted, 96);
}

/// Independently expect I_t tensor I_bR; rejection concerns every lifetime,
/// including globally identity subjects. This family never consumes RNG draws.
fn r7_case(body: &str, bounds: &[u64], valid: bool, certified: bool) {
    let text = if certified {
        support::certified_borrow_identity_kind(body, bounds, "action")
    } else {
        support::borrow_lifetime_object(body, bounds).replace("perm_action", "action")
    };
    let result = Object::parse(&text);
    if valid {
        let o = result.unwrap_or_else(|e| panic!("{e}\n{body}"));
        support::r6_map(&o, false);
    } else {
        let error = result.unwrap_err().to_string();
        assert!(
            error.contains("§5.6") || error.contains("§6.2"),
            "{error}\n{body}"
        );
    }
}

#[test]
fn generated_r7_diagonal_control_and_inverse_boundaries() {
    let mut accepted = 0;
    let mut rejected = 0;
    // Diagonal gates preserve BOTH 0 and 1 basis controls. At 1 two CXs are
    // needed; one CX violates that fresh borrowed lifetime.
    for (gate, inverse) in [
        ("s c", "sdg c"),
        ("t c", "tdg c"),
        ("rz 0.5 c", "rz -0.5 c"),
        ("cz c t", "cz c t"),
        ("qpred c\nh t\nh t\nqunpred c", ""),
    ] {
        for initial in [0, 1] {
            for doubled in [false, true] {
                let x = if initial == 1 { "x c\n" } else { "" };
                let cx = if doubled { "cx c b\n" } else { "" };
                let body = format!("c = qalloc any 1\n{x}{gate}\nb = qborrow any 1\ncx c b\n{cx}qreturn b\n{inverse}\n{x}qfree c").replace("\n\n", "\n");
                let valid = initial == 0 || doubled;
                for certified in [false, true] {
                    r7_case(&body, &[], valid, certified);
                    if valid {
                        accepted += 1;
                    } else {
                        rejected += 1;
                    }
                }
            }
        }
    }
    // Matched non-diagonal inverses restore the allocation fact, even with a
    // disjoint intervening touch; a touch on c that changes its state does not.
    for (gate, inverse) in [
        ("h c", "h c"),
        ("rx 0.5 c", "rx -0.5 c"),
        ("ry 0.5 c", "ry -0.5 c"),
    ] {
        for middle in ["", "x t\nx t\n", "s c\n"] {
            for empty in [false, true] {
                let borrow_gate = if empty { "" } else { "cx c b\n" };
                let prefix = format!("{gate}\n{middle}{inverse}");
                // With S in between, explicit full inverse cleanup follows the
                // borrow. Empty lifetimes remain identity, despite unknown c.
                let tail = if middle == "s c\n" {
                    format!("{gate}\nsdg c\n{inverse}\n")
                } else {
                    String::new()
                };
                let body = format!("c = qalloc any 1\n{prefix}\nb = qborrow any 1\n{borrow_gate}qreturn b\n{tail}qfree c");
                let valid = middle != "s c\n" || empty;
                for certified in [false, true] {
                    r7_case(&body, &[], valid, certified);
                    if valid {
                        accepted += 1;
                    } else {
                        rejected += 1;
                    }
                }
            }
        }
    }
    // Non-permutation touches of B remain forbidden, including disabled
    // controls, diagonal inverse pairs and certified whole-region identities.
    for gates in [
        "s b\nsdg b",
        "h b\nh b",
        "qpred b\nh t\nh t\nqunpred b",
        "c = qalloc any 1\ncz c b\nqfree c",
    ] {
        for certified in [false, true] {
            r7_case(
                &format!("b = qborrow any 1\n{gates}\nqreturn b"),
                &[],
                false,
                certified,
            );
            rejected += 1;
        }
    }
    println!("R7 boundaries: {accepted} accepted / {rejected} rejected");
    assert_eq!((accepted, rejected), (60, 24));
}

#[test]
fn generated_r7_evolving_loops_match_per_return_and_renamed_unrollings() {
    let mut accepted = 0;
    let mut rejected = 0;
    for n in [1u64, 3, 4, 5, 8, 1 << 40] {
        for initial in [0u64, 1] {
            for shape in 0..3 {
                if shape == 1 && n != 4 || shape == 2 && n > 8 {
                    continue;
                }
                let (loop_body, bounds) = match shape {
                    1 => ("qloop 2 {\nqloop 2 {\nh t\nx c\n}\n}".into(), vec![2, 2]),
                    2 => (
                        (0..n)
                            .map(|_| "h t\nx c\n")
                            .collect::<String>()
                            .trim_end()
                            .into(),
                        vec![],
                    ),
                    _ => (format!("qloop {n} {{\nh t\nx c\n}}"), vec![n]),
                };
                let initial_x = if initial == 1 { "x c\n" } else { "" };
                let final_c = initial ^ (n & 1);
                let final_x = if final_c == 1 { "x c\n" } else { "" };
                let final_h = if n & 1 == 1 { "h t\n" } else { "" };
                let body = format!("c = qalloc any 1\n{initial_x}{loop_body}\nb = qborrow any 1\ncx c b\nqreturn b\n{final_h}{final_x}qfree c");
                for certified in [false, true] {
                    r7_case(&body, &bounds, final_c == 0, certified);
                    if final_c == 0 {
                        accepted += 1;
                    } else {
                        rejected += 1;
                    }
                }
            }
        }
        if n > 8 {
            continue;
        }
        // One-hot rotates 0 -> 1 -> 2 -> 3 -> 0. Return j is valid exactly
        // when j mod 4 != 3. Later inverse rotations do not repair a return.
        for shape in 0..3 {
            if shape == 1 && n != 4 || shape == 2 && n > 8 {
                continue;
            }
            let visit = |b: &str| {
                format!("h t\n{b} = qborrow any 1\ncx c[3] {b}\nqreturn {b}\nh t\nswap c[3] c[2]\nswap c[2] c[1]\nswap c[1] c[0]\n")
            };
            let undo = "h t\nswap c[1] c[0]\nswap c[2] c[1]\nswap c[3] c[2]\nh t\n";
            let (forward, inverse, bounds) = match shape {
                1 => (
                    format!("qloop 2 {{\nqloop 2 {{\n{}}}\n}}", visit("b")),
                    format!("qloop 2 {{\nqloop 2 {{\n{undo}}}\n}}"),
                    vec![2, 2, 2, 2],
                ),
                2 => (
                    (0..n).map(|j| visit(&format!("b{j}"))).collect(),
                    undo.repeat(n as usize),
                    vec![],
                ),
                _ => (
                    format!("qloop {n} {{\n{}}}", visit("b")),
                    format!("qloop {n} {{\n{undo}}}"),
                    vec![n, n],
                ),
            };
            let body = format!("c = qalloc any 4\nx c[0]\n{forward}\n{inverse}\nx c[0]\nqfree c")
                .replace("\n\n", "\n");
            for certified in [false, true] {
                r7_case(&body, &bounds, n < 4, certified);
                if n < 4 {
                    accepted += 1;
                } else {
                    rejected += 1;
                }
            }
        }
    }
    println!("R7 evolving loops: {accepted} accepted / {rejected} rejected");
    assert_eq!((accepted, rejected), (32, 38));
}

#[test]
fn generated_r7_large_noncycling_loop_proves_unknown_top_invariant() {
    // Intervening X touches prevent saved-H restoration in this fact domain,
    // although H X X H = I by independent matrices. c stays zero, so every
    // fresh borrowed lifetime is I. This forces the unknown-top proof route.
    let n = 1u64 << 40;
    for dirty in [false, true] {
        let x = if dirty { "x c\n" } else { "" };
        let body = format!("c = qalloc any 1\n{x}qloop {n} {{\nh t\nx t\nx t\nh t\nb = qborrow any 1\ncx c b\nqreturn b\n}}\n{x}qfree c");
        let text = support::borrow_lifetime_object(&body, &[n]).replace("perm_action", "action");
        let result = Object::parse(&text);
        assert_eq!(result.is_ok(), !dirty, "{result:?}");
    }
    println!("R7 invariant: 1 accepted / 1 rejected (static proof, no execution)");
}

#[test]
fn generated_r8_nested_causal_adjoint_stacks() {
    let pairs = [
        ("h c", "h c"),
        ("x c", "x c"),
        ("s c", "sdg c"),
        ("cx c t", "cx c t"),
        ("swap c t", "swap c t"),
        ("rx 0.25 c", "rx -0.25 c"),
        ("ry 0.5 c", "ry -0.5 c"),
    ];
    let mut accepted = 0;
    for depth in 1..=3 {
        for start in 0..pairs.len() {
            let computation: Vec<_> = std::iter::once(("h c", "h c"))
                .chain((0..depth).map(|k| pairs[(start + k) % pairs.len()]))
                .collect();
            let sequence = computation
                .iter()
                .map(|p| p.0)
                .chain(computation.iter().rev().map(|p| p.1))
                .collect::<Vec<_>>()
                .join("\n");
            for wrapped in [false, true] {
                let gates = if wrapped {
                    format!("qloop 3 {{\n{sequence}\n}}")
                } else {
                    sequence.clone()
                };
                for certified in [false, true] {
                    let body = format!(
                        "c = qalloc any 1\n{gates}\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c"
                    );
                    r7_case(&body, if wrapped { &[3] } else { &[] }, true, certified);
                    accepted += 1;
                }
            }
        }
    }
    // Independent stacks interleave; every sequence is literally U followed by U†.
    for certified in [false, true] {
        r7_case("c = qalloc any 1\nd = qalloc any 1\nh c\nrx 0.25 d\ns c\nx d\nsdg c\nx d\nh c\nrx -0.25 d\nb = qborrow any 1\ncx c b\ncx d b\nqreturn b\nqfree d\nqfree c", &[], true, certified);
        accepted += 1;
    }
    for certified in [false, true] {
        r7_case("c = qalloc any 1\nd = qalloc any 1\nh c\nswap c t\nswap c d\nswap c d\nswap c t\nh c\nb = qborrow any 1\ncx c b\ncx d b\nqreturn b\nqfree d\nqfree c", &[], true, certified);
        accepted += 1;
    }
    for certified in [false, true] {
        r7_case("c = qalloc any 1\nh c\nswap c t\nh c\nh c\nswap c t\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c", &[], true, certified);
        accepted += 1;
    }
    let mut rejected = 0;
    for inner in ["s c", "cx c t", "swap c t"] {
        let inverse = if inner == "s c" { "sdg c" } else { inner };
        for certified in [false, true] {
            // Overall cleanup is matched, but the borrow occurs BEFORE it. An
            // outstanding reader must prevent H from inventing a zero at that return.
            let body = format!("c = qalloc any 1\nh c\n{inner}\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nh c\n{inverse}\nh c\nqfree c");
            r7_case(&body, &[], false, certified);
            rejected += 1;
        }
    }
    eprintln!("R8 nested stacks: {accepted} accepted / {rejected} rejected");
}

#[test]
fn generated_r8_zero_trip_rebindings_share_executed_structure() {
    let mut accepted = 0;
    for dead in [
        "qfree c\nc = qalloc any 2",
        "qfree c\nc = qborrow any 2",
        "qfree c\nc = qcopy t",
        "qfree c\nc = qload r a0",
    ] {
        for nested in [false, true] {
            for renamed in [false, true] {
                for certified in [false, true] {
                    let (dead, bounds) = if nested {
                        (format!("qloop 0 {{\n{dead}\n}}"), vec![0, 0])
                    } else {
                        (dead.into(), vec![0])
                    };
                    let mut body = format!("qregion r 1 1\na0 = qalloc any 1\nc = qalloc any 1\nqloop 0 {{\n{dead}\n}}\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c\nqfree a0");
                    if renamed {
                        body = body
                            .replace("c =", "clean =")
                            .replace(" c ", " clean ")
                            .replace(" c\n", " clean\n");
                    }
                    r7_case(&body, &bounds, true, certified);
                    accepted += 1;
                }
            }
        }
    }
    // A declaration in a dead body still has function scope and entry lifetime.
    r7_case("qloop 0 {\nqregion r 1 1\n}\na0 = qalloc any 1\nc = qload r a0\nb = qborrow any 1\ncx c b\nqreturn b\nqunload r a0 c\nqfree a0", &[0], true, false);
    accepted += 1;
    let mut rejected = 0;
    for gate in ["h", "s"] {
        for nested in [false, true] {
            for certified in [false, true] {
                let dead = "qreturn b\nb = qalloc any 1";
                let (dead, bounds) = if nested {
                    (format!("qloop 0 {{\n{dead}\n}}"), vec![0, 0])
                } else {
                    (dead.into(), vec![0])
                };
                let body =
                    format!("b = qborrow any 1\nqloop 0 {{\n{dead}\n}}\n{gate} b\nqreturn b");
                r7_case(&body, &bounds, false, certified);
                rejected += 1;
            }
        }
    }
    for (body,bounds) in [
        ("c = qalloc any 1\nqloop 0 {\nc = qalloc any 2\n}\nqfree c",vec![0]),
        ("c = qalloc any 1\nqloop 1 {\nqfree c\nc = qalloc any 2\n}\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c",vec![1]),
    ] {
        let text = support::borrow_lifetime_object(body,&bounds).replace("perm_action","action");
        assert!(Object::parse(&text).unwrap_err().to_string().contains("§3.3"));
        rejected += 1;
    }
    eprintln!("R8 zero trips: {accepted} accepted / {rejected} rejected");
}

fn r8_call_family(depth: usize, readonly: bool, operation: &str, body: &str) -> String {
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let child_sig = if readonly {
        sig.replace("\"in\":[]", "\"in\":[1]")
    } else {
        sig.into()
    };
    let mut declarations = Vec::new();
    for k in 0..depth {
        let name = format!("u{k}");
        let inputs = if readonly { "p:wire in" } else { "" };
        let ins = if readonly { "p" } else { "" };
        let op = if k == 0 {
            if readonly {
                format!("qpred p\n{operation}\nqunpred p")
            } else {
                operation.into()
            }
        } else {
            format!("action_call @u{} (v) ({ins}) () ()", k - 1)
        };
        declarations.push((
            name,
            child_sig.clone(),
            format!("action @u{k} (v:wire mut) ({inputs}) () () ()\n^entry():\n{op}"),
        ));
    }
    declarations.push((
        "a".into(),
        sig.into(),
        format!(
            "action @a (t:wire mut) () () () ()\n^entry():\n{}",
            body.replace("@LAST", &format!("@u{}", depth - 1))
        ),
    ));
    support::object(
        &declarations
            .iter()
            .map(|(n, s, b)| (n.as_str(), s.as_str(), b.as_str()))
            .collect::<Vec<_>>(),
        "{}",
        "",
    )
}

#[test]
fn generated_r8_calls_transfer_per_wire_through_callee_chains() {
    let mut accepted = 0;
    let mut rejected = 0;
    for depth in 1..=3 {
        for readonly in [false, true] {
            for one in [false, true] {
                let init = if one { "x c\n" } else { "" };
                let call = if readonly {
                    "action_call @LAST (t) (c) () ()"
                } else {
                    "ctl c action_call @LAST (t) () () ()"
                };
                let body = format!("c = qalloc any 1\n{init}{call}\n{init}b = qborrow any 1\ncx c b\nqreturn b\nqfree c");
                let text = r8_call_family(depth, readonly, "h v", &body);
                let o = Object::parse(&text).unwrap();
                support::r6_map(&o, one); // literal controlled-H: I for zero, H for one
                accepted += 1;
            }
        }
        // Callee analysed net effect, rather than its action kind, preserves d=0.
        let body = "d = qalloc any 1\naction_call @LAST (d) () () ()\nb = qborrow any 1\ncx d b\nqreturn b\nadj action_call @LAST (d) () () ()\nqfree d";
        let o = Object::parse(&r8_call_family(depth, false, "h v\nx v\nx v\nh v", body)).unwrap();
        support::r6_map(&o, false);
        accepted += 1;
        // Unknown interface t controls a genuine H on clean d: d may be 1,
        // so borrowing through d is invalid despite matched cleanup afterward.
        let body = "d = qalloc any 1\nctl t action_call @LAST (d) () () ()\nb = qborrow any 1\ncx d b\nqreturn b\nctl t adj action_call @LAST (d) () () ()\nqfree d";
        let e = Object::parse(&r8_call_family(depth, false, "h v", body))
            .unwrap_err()
            .to_string();
        assert!(e.contains("§5.6"), "{e}");
        rejected += 1;
        // Both effective controls and direct operands remain structural touches.
        for call in [
            "ctl b action_call @LAST (t) () () ()",
            "action_call @LAST (b) () () ()",
        ] {
            let body = format!("b = qborrow any 1\n{call}\nqreturn b");
            let e = Object::parse(&r8_call_family(depth, false, "h v", &body))
                .unwrap_err()
                .to_string();
            assert!(e.contains("§5.6"), "{e}");
            rejected += 1;
        }
    }
    eprintln!("R8 call transfer: {accepted} accepted / {rejected} rejected");
}

#[test]
fn generated_r8_import_transfer_keeps_readonly_facts_but_blocks_history() {
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let imports = format!(
        r#"{{"u":{{"pinned_closure_hash":"{}","signature":{sig}}}}}"#,
        "1".repeat(64)
    );
    let build = |body: &str| {
        support::object(
            &[(
                "a",
                sig,
                &format!("action @a (t:wire mut) () () () ()\n^entry():\n{body}"),
            )],
            &imports,
            "",
        )
    };
    // Static only: imports have no executable definition yet. A disabled call
    // preserves c=0. An opaque reader of an unknown c cannot preserve its H
    // restoration history, even though c itself is a read-only call control.
    Object::parse(&build("c = qalloc any 1\nctl c action_call @u (t) () () ()\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c")).unwrap();
    let e = Object::parse(&build("c = qalloc any 1\nh c\nctl c action_call @u (t) () () ()\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nh c\nctl c adj action_call @u (t) () () ()\nh c\nqfree c")).unwrap_err().to_string();
    assert!(e.contains("§5.6"), "{e}");
    eprintln!("R8 import transfer (static): 1 accepted / 1 rejected");
}

#[test]
fn generated_r9_opaque_histories_match_independent_identity() {
    use bennett_isa_ref::{Argument, Budget};
    let call = "oracle_call @o tk (c) t f () ()";
    let tail = "b = qborrow any 1\ncx c b\nqreturn b\nqfree f\nqfree c";
    let mut accepted = 0;
    let mut rejected = 0;
    let mut positives = vec![(format!("{call}\n{call}"), vec![])];
    for n in [0, 1, 2, 3] {
        positives.push((format!("qloop {n} {{\n{call}\n{call}\n}}"), vec![n]));
    }
    positives.push((
        format!("qloop 1 {{\n{call}\n}}\nqloop 1 {{\n{call}\n}}"),
        vec![1, 1],
    ));
    positives.push(("qinplace @o @o tk c\nqinplace @o @o tk c".into(), vec![]));
    for (middle, bounds) in positives {
        let body = format!("c = qalloc any 1\nf = qalloc any 4\nh c\n{middle}\nh c\n{tail}");
        let o = support::rehash(&support::r9_opaque_object(&body, &bounds, None));
        for steps in if middle.contains("qinplace") {
            vec![1]
        } else {
            vec![0, 1]
        } {
            support::r9_identity(&o, &[Argument::Token(Budget::steps(steps))]);
        }
        accepted += 1;
    }
    // Two disjoint quantum/opaque histories interleave without sharing outputs.
    for reverse in [false, true] {
        let other = "oracle_call @o tk (e) u fr () ()";
        let middle = if reverse {
            format!("{call}\n{other}\n{other}\n{call}")
        } else {
            format!("{call}\n{other}\n{call}\n{other}")
        };
        let body = format!("c = qalloc any 1\ne = qalloc any 1\nu = qalloc any 1\nf = qalloc any 4\nfr = qalloc any 4\nh c\nh e\n{middle}\nh c\nh e\nqfree fr\nqfree u\nqfree e\n{tail}");
        let o = support::rehash(&support::r9_opaque_object(&body, &[], None));
        support::r9_identity(&o, &[Argument::Token(Budget::steps(1))]);
        accepted += 1;
    }
    // Local non-permutation call boundaries retain the analysed opaque events.
    let child = "action @child (v:wire mut, fr:wire[4] mut) (p:wire in) () (tk:token) ()\n^entry():\noracle_call @o tk (p) v fr () ()";
    for adj in [false, true] {
        let call = "action_call @child (t, f) (c) () (tk)";
        let middle = if adj {
            format!("adj {call}\n{call}")
        } else {
            format!("{call}\nadj {call}")
        };
        let body = format!("c = qalloc any 1\nf = qalloc any 4\nh c\n{middle}\nh c\n{tail}");
        let o = support::rehash(&support::r9_opaque_object(&body, &[], Some(child)));
        support::r9_identity(&o, &[Argument::Token(Budget::steps(1))]);
        accepted += 1;
    }
    // Full opaque keys: changed input, unmatched reader and changed control.
    let first = format!("c = qalloc any 1\nf = qalloc any 4\nh c\n{call}");
    for middle in [
        "".into(),
        format!("x c\n{call}\nx c"),
        format!("ctl c {call}"),
    ] {
        let body = format!("{first}\n{middle}\nh c\n{tail}");
        let body = body
            .lines()
            .filter(|l| !l.is_empty())
            .collect::<Vec<_>>()
            .join("\n");
        let text = support::r9_opaque_object(&body, &[], None);
        let e = Object::parse(&text).unwrap_err().to_string();
        assert!(e.contains("§5.6") || e.contains("§6.2"), "{e}");
        let rule = if e.contains("§5.6") {
            "§5.6"
        } else {
            "§6.2"
        };
        support::reject_cli(&text, rule, &format!("r9-unmatched-{rejected}"));
        rejected += 1;
    }
    // Two later borrowings cancel globally; neither invalid earlier return is waived.
    let body = format!("{first}\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nb = qborrow any 1\ncx c b\nqreturn b\nh c\n{call}\nh c\nqfree f\nqfree c");
    let text = support::r9_opaque_object(&body, &[], None);
    support::reject_cli(&text, "§5.6", "r9-split-opaque");
    rejected += 1;
    // Distinct oracle names require the explicit swapped g/ginv identity.
    for enclosing_h in [false, true] {
        let os = r#"{"abi":"0.8","args":[],"fault":4,"in":[1],"kind":"oracle","mem":[],"tgt":1,"tier":"q"}"#;
        let ac = r#"{"abi":"0.8","args":["token"],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
        let h = if enclosing_h { "h c\n" } else { "" };
        let body = format!("action @a (t:wire mut) () () (tk:token) ()\n^entry():\nc = qalloc any 1\n{h}qinplace @g @gi tk c\nqinplace @gi @g tk c\n{h}b = qborrow any 1\ncx c b\nqreturn b\nqfree c");
        let text = support::object(
            &[
                (
                    "g",
                    os,
                    "oracle @g (p:wire in) (v:wire) (fr:wire[4]) () ()\n^entry():\nlift.retval p",
                ),
                (
                    "gi",
                    os,
                    "oracle @gi (p:wire in) (v:wire) (fr:wire[4]) () ()\n^entry():\nlift.retval p",
                ),
                ("a", ac, &body),
            ],
            "{}",
            "",
        );
        let o = support::rehash(&text);
        support::r9_identity(&o, &[Argument::Token(Budget::steps(1))]);
        accepted += 1;
    }
    // Different token specialisations cannot cancel even when operand names agree.
    let body = format!("{first}\noracle_call @o tk2 (c) t f () ()\nh c\nb = qborrow any 1\ncx c b\nqreturn b\nh c\noracle_call @o tk2 (c) t f () ()\n{call}\nh c\nqfree f\nqfree c");
    let text = support::r9_opaque_object(&body, &[], None)
        .replace("\"args\":[\"token\"]", "\"args\":[\"token\",\"token\"]")
        .replace("(tk:token)", "(tk:token, tk2:token)");
    support::reject_cli(&text, "§5.6", "r9-token-specialisation");
    rejected += 1;
    eprintln!("R9 opaque: {accepted} accepted / {rejected} rejected");
}

#[test]
fn generated_r9_loop_bindings_match_independent_identity() {
    let mut accepted = 0;
    let mut rejected = 0;
    for n in [0, 1, 2, 3] {
        for nested in [false, true] {
            for renamed in [false, true] {
                let visit =
                    "b = qborrow any 1\ncx c b\nqreturn b\nqfree c\nc = qalloc any 1\nh t\nh t";
                let (middle, bounds) = if renamed {
                    let mut visits = Vec::new();
                    for i in 0..(if nested { n * 2 } else { n }) {
                        visits.push(format!("b{i} = qborrow any 1\ncx c{i} b{i}\nqreturn b{i}\nqfree c{i}\nc{} = qalloc any 1\nh t\nh t",i+1));
                    }
                    (
                        format!(
                            "c0 = qalloc any 1\n{}\nqfree c{}",
                            visits.join("\n"),
                            if nested { n * 2 } else { n }
                        ),
                        vec![],
                    )
                } else {
                    let (body, bounds) = if nested {
                        (format!("qloop 2 {{\n{visit}\n}}"), vec![n, 2])
                    } else {
                        (visit.into(), vec![n])
                    };
                    (
                        format!("c = qalloc any 1\nqloop {n} {{\n{body}\n}}\nqfree c"),
                        bounds,
                    )
                };
                let middle = middle
                    .lines()
                    .filter(|l| !l.is_empty())
                    .collect::<Vec<_>>()
                    .join("\n");
                let text = support::borrow_lifetime_object(&middle, &bounds)
                    .replace("perm_action", "action");
                let o = support::rehash(&text);
                support::r9_identity(&o, &[]);
                accepted += 1;
            }
        }
        if n > 0 {
            let body = format!("c = qalloc any 1\nx c\nqloop {n} {{\nb = qborrow any 1\ncx c b\nqreturn b\nx c\nqfree c\nc = qalloc any 1\nx c\nh t\nh t\n}}\nx c\nqfree c");
            let text =
                support::borrow_lifetime_object(&body, &[n]).replace("perm_action", "action");
            support::reject_cli(&text, "§5.6", &format!("r9-dirty-{n}"));
            rejected += 1;
        }
    }
    // First-visit width differs from the steady entry; indexed reads resolve anew.
    for n in [1, 2, 3] {
        let body = format!("c = qalloc any 1\nqloop {n} {{\nb = qborrow any 1\ncx c[0] b\nqreturn b\nqfree c\nc = qalloc any 2\nh t\nh t\n}}\nqfree c");
        let text = support::borrow_lifetime_object(&body, &[n]).replace("perm_action", "action");
        support::r9_identity(&support::rehash(&text), &[]);
        accepted += 1;
    }
    // Static huge count exercises recurring binding transfers, not loop expansion.
    let n = 1u64 << 40;
    let body = format!("c = qalloc any 1\nqloop {n} {{\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c\nc = qalloc any 1\nh t\nh t\n}}\nqfree c");
    let o = support::rehash(
        &support::borrow_lifetime_object(&body, &[n]).replace("perm_action", "action"),
    );
    o.check_hashes().unwrap(); // Static proof; dynamic fresh-wire execution is enumerative.
    accepted += 1;
    eprintln!("R9 loop bindings: {accepted} accepted / {rejected} rejected");
}

#[test]
fn generated_r10_effective_control_sets_match_independent_identity() {
    // Each O/O† pair is identity under conjunction; neither stack ordering nor
    // duplicate spelling changes that conjunction. ABI argument order stays fixed.
    let stacks: &[&[&str]] = &[
        &["p", "q"],
        &["q", "p"],
        &["p", "p", "q"],
        &["q", "p", "q"],
        &["p[0]", "q[0]"],
    ];
    let mut accepted = 0;
    for first in stacks {
        for second in stacks {
            for mode in 0..4 {
                let (op, inv) = match mode {
                    2 => (
                        "action_call @child (c, d, f) (p, q) () (tk)",
                        "adj action_call @child (c, d, f) (p, q) () (tk)",
                    ),
                    3 => ("qinplace @o @oi tk c", "qinplace @oi @o tk c"),
                    _ => (support::R10_ORACLE, support::R10_ORACLE),
                };
                let pair = support::r10_context(first, op) + &support::r10_context(second, inv);
                let middle = if mode == 1 {
                    format!("qloop 1 {{\nqloop 1 {{\n{pair}}}\n}}\n")
                } else {
                    pair
                };
                let bounds: &[u64] = if mode == 1 { &[1, 1] } else { &[] };
                let o =
                    support::rehash(&support::r10_object(&middle, bounds, mode == 2, mode == 3));
                for steps in if mode == 3 { &[1][..] } else { &[0, 1][..] } {
                    support::r10_identity(&o, *steps);
                }
                accepted += 1;
            }
        }
    }
    // Duplicate-only sets, with a differently spelled scalar alias.
    for second in [&["p", "p"][..], &["p[0]", "p"][..]] {
        let middle = support::r10_context(&["p"], support::R10_ORACLE)
            + &support::r10_context(second, support::R10_ORACLE);
        let o = support::rehash(&support::r10_object(&middle, &[], false, false));
        for b in [0, 1] {
            support::r10_identity(&o, b);
        }
        accepted += 1;
    }
    let first = support::r10_context(&["p", "q"], support::R10_ORACLE);
    let same = support::r10_context(&["q", "p"], support::R10_ORACLE);
    let absent = support::r10_context(&["p"], support::R10_ORACLE);
    let opposite = format!(
        "qpred q\nqif p {{\nx t\nx t\n}} {{\n{}\n}}\nqunpred q\n",
        support::R10_ORACLE
    );
    let mut rejected = 0;
    for (tag, middle, bounds) in [
        ("absent", first.clone() + &absent, vec![]),
        ("polarity", first.clone() + &opposite, vec![]),
        (
            "token",
            first.clone() + &same.replace("@o tk", "@o other"),
            vec![],
        ),
        ("version", first.clone() + "x p\n" + &same + "x p\n", vec![]),
        (
            "operand-order",
            first.clone() + &same.replace("(c) d f", "(d) c f"),
            vec![],
        ),
        (
            "callee",
            first.clone() + &same.replace("@o tk", "@oi tk"),
            vec![],
        ),
    ] {
        let mut text = support::r10_object(&middle, &bounds, false, tag == "callee");
        if tag == "token" {
            text = text
                .replace("(tk:token) ()", "(tk:token, other:token) ()")
                .replace(r#""args":["token"]"#, r#""args":["token","token"]"#);
        }
        if tag == "version" {
            text = text
                .replace(
                    "(t:wire mut) (p:wire in, q:wire in)",
                    "(t:wire mut, p:wire mut) (q:wire in)",
                )
                .replace(r#""in":[1,1]"#, r#""in":[1]"#)
                .replace(r#""mut":[1]"#, r#""mut":[1,1]"#);
        }
        support::reject_cli(&text, "§6.2", &format!("r10-{tag}"));
        rejected += 1;
    }
    println!("R10 effective controls: {accepted} accepted / {rejected} rejected");
}
#[test]
fn generated_r10_grammar_mutations_reject_without_normalization() {
    let mut rejected = 0;
    for text in support::r10_bad_headers() {
        support::reject_cli(&text, "§16.2", &format!("r10-generated-head-{rejected}"));
        rejected += 1;
    }
    for tag in ["fn-oracle", "inc-empty-bounds", "empty-lhs", "empty-bounds"] {
        support::reject_cli(&support::r10_text(tag), "§16.2", tag);
        rejected += 1;
    }
    // Header-like NAMEs are legal SSA names, not definition boundaries.
    let sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"action","mem":[],"mut":[1],"tier":"q"}"#;
    let mut accepted = 0;
    for name in ["fn", "cc", "r", "oracle", "action", "perm_action", "lproc"] {
        let body = format!(
            "action @a (t:wire mut) () () () ()\n^entry():\n{name} = qalloc any 1\nqfree {name}"
        );
        support::rehash(&support::object(&[("a", sig, &body)], "{}", ""));
        accepted += 1;
    }
    println!("R10 grammar: {accepted} accepted / {rejected} rejected");
}

#[test]
fn generated_r10_context_keys_cover_all_cleanup_families() {
    let mut accepted = 0;
    for (forward, inverse, memory) in [
        ("v = qcopy t", "quncopy t v", false),
        ("v = qload region addr", "qunload region addr v", true),
        ("qstore region addr t", "qstore region addr t", true),
        ("qexchm region addr t", "qexchm region addr t", true),
        ("qexch d t", "qexch d t", false),
        ("h d", "h d", false),
        ("s t", "sdg t", false),
        ("x d", "x d", false),
    ] {
        let prefix = if memory {
            "qregion region 1 1\naddr = qalloc any 1\n"
        } else {
            ""
        };
        let suffix = if memory { "qfree addr\n" } else { "" };
        let middle = format!(
            "{prefix}{}{}{suffix}",
            support::r10_context(&["p", "q"], forward),
            support::r10_context(&["q[0]", "p", "p[0]"], inverse)
        );
        let o = support::rehash(&support::r10_object(&middle, &[], false, false));
        support::r10_identity(&o, 1);
        accepted += 1;
    }
    // Both operations in the negative-polarity arm use the same semantic set.
    let one = support::r10_context(&["q"], support::R10_ORACLE);
    let two = support::r10_context(&["q[0]", "q"], support::R10_ORACLE);
    let middle = format!("qif p {{\nx t\nx t\n}} {{\n{one}{two}}}\n");
    let o = support::rehash(&support::r10_object(&middle, &[], false, false));
    for b in [0, 1] {
        support::r10_identity(&o, b);
    }
    accepted += 1;
    // A wrongly ordered pop remains invalid even though a semantic set agrees.
    let bad = support::r10_object(
        "qpred p\nqpred q\nx t\nqunpred p\nqunpred q\n",
        &[],
        false,
        false,
    );
    support::reject_cli(&bad, "§3.5", "r10-lexical-pop");
    println!("R10 cleanup contexts: {accepted} accepted / 1 rejected");
}
