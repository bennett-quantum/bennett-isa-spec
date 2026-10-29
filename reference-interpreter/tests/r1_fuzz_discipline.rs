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
    for count in 0..6 {
        for wrapped in [false, true] {
            let (compute, uncompute) = if wrapped {
                (
                    "action_call @child (d, m) () () ()",
                    "adj action_call @child (d, m) () () ()",
                )
            } else {
                ("swap d m", "swap d m")
            };
            let body = format!("action @a (t:wire mut, m:wire mut) (p:wire in) () () ()\n^entry():\nd = qalloc any 1\nh d\n{compute}\n{}{uncompute}\nh d\nqfree d", "cx m t\n".repeat(count));
            let text = support::object(&[("child",cs,"perm_action @child (a:wire mut, b:wire mut) () () () ()\n^entry():\nswap a b"),("a",&ps,&body)],"{}", "");
            // Odd CX count copies the H-created bit into t: Bell entanglement
            // survives uncompute. Even count is exactly the identity.
            if count % 2 == 1 {
                assert!(Object::parse(&text)
                    .unwrap_err()
                    .to_string()
                    .contains("§6.2"));
            } else {
                let o = Object::parse(&text).unwrap();
                runs_clean(&o).unwrap();
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
