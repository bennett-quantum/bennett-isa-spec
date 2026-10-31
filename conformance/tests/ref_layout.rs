use bennett_isa_conformance::{
    json,
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
#[test]
fn lifted_layout_memory_full_symbolic_domains() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    for p in (1usize..=5).chain([8]) {
        for bytes in [1usize, 2, 3, 4, 16] {
            if (p == 5 && bytes != 16) || (p == 8 && bytes != 1) {
                continue;
            }
            for mapped in [false, true] {
                for op in ["load", "store"] {
                    let cells = if p == 8 { 4 } else { 1 << p };
                    let n = 8 * bytes;
                    let l = usize::BITS as usize - bytes.leading_zeros() as usize;
                    let ext = p.max(l) + 1;
                    let bundles = [
                        ("a", p),
                        ("base", p),
                        ("size", p),
                        ("live", 1),
                        ("pinned", 1),
                        ("atop", 1),
                        ("flag", 1),
                        ("faulted", 1),
                        ("code", 4),
                        ("v", n),
                        ("disp", n),
                        ("mem", 8 * cells),
                        ("mapped", 8 * cells),
                    ];
                    // One mapped byte in the interior; ordinary mem under it remains an
                    // arbitrary spectator. Partition endpoints and bindings are explicit.
                    let mapcount = if !mapped {
                        0
                    } else if p == 5 {
                        17
                    } else if p == 4 {
                        3
                    } else if p == 3 {
                        2
                    } else {
                        1
                    };
                    let mut records=(0..mapcount).map(|i|format!(r#"{{"base":{i},"extent":1,"cells":{{"range":"mapped","from":{},"to":{}}}}}"#,8*i,8*(i+1))).collect::<Vec<_>>();
                    if mapcount < cells {
                        records.push(format!(r#"{{"base":{mapcount},"extent":{},"cells":{{"range":"mem","from":{},"to":{}}}}}"#,cells-mapcount,8*mapcount,8*cells));
                    }
                    let layout = format!("[{}]", records.join(","));
                    let value_actual = if p == 8 { "a" } else { "v" };
                    let args=json::parse(&format!(r#"[{p},{l},{ext},1,1,4,{n},{bytes},"a","base","size","live","pinned","atop","flag","faulted","code","{value_actual}","disp",{layout}]"#)).unwrap();
                    let leaves = e
                        .expand(&format!("lift.{op}"), args.as_arr().unwrap(), &bundles)
                        .unwrap();
                    let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
                    let mut b = Bdd::new();
                    let mut state = m.symbolic_state();
                    for (name, _) in bundles {
                        if name != "flag" && name != "disp" {
                            for i in &m.inputs[name] {
                                state[*i] = b.variable();
                            }
                        }
                    }
                    let enabled = b.variable();
                    let old = state.clone();
                    let mut want = old.clone();
                    let get =
                        |name: &str| m.inputs[name].iter().map(|i| old[*i]).collect::<Vec<_>>();
                    let mut a = get("a");
                    a.resize(ext, 0);
                    let mut base = get("base");
                    base.resize(ext, 0);
                    let mut size = get("size");
                    size.resize(ext, 0);
                    let end = b.add(&a, &Bdd::constant(bytes, ext));
                    let top = b.add(&base, &size);
                    let below = b.less(&a, &base);
                    let above = b.less(&top, &end);
                    let outside = b.or(below, above);
                    let inside = b.not(outside);
                    let live = b.and(old[m.inputs["live"][0]], old[m.inputs["atop"][0]]);
                    let valid = b.and(live, inside);
                    let flag = b.not(valid);
                    // Restrict source layout/table legality: the live entry fits physical
                    // memory, with no wrap; all other table inputs remain arbitrary.
                    let maxend = Bdd::constant(cells, ext);
                    let overflow = b.less(&maxend, &top);
                    let domain = b.not(overflow);
                    if p == 8 {
                        // P>H includes both (0,4) and (4,0): 15 table pairs.
                        assert_eq!(b.satisfying_count(domain).unwrap(), 15 * (1u128 << 89));
                    }
                    let bad = b.and(enabled, flag);
                    want[m.inputs["flag"][0]] = bad;
                    let id = m.inputs["faulted"][0];
                    want[id] = b.xor(want[id], bad);
                    for id in &m.inputs["code"][..3] {
                        want[*id] = b.xor(want[*id], bad);
                    }
                    let success = b.and(enabled, valid);
                    for address in 0..cells {
                        let hit = b.equal(&get("a"), &Bdd::constant(address, p));
                        let hit = b.and(success, hit);
                        for byte in 0..bytes {
                            if address + byte < cells {
                                for bit in 0..8 {
                                    let cell = if address + byte < mapcount {
                                        m.inputs["mapped"][(address + byte) * 8 + bit]
                                    } else {
                                        m.inputs["mem"][(address + byte) * 8 + bit]
                                    };
                                    let v = m.inputs[value_actual][8 * byte + bit];
                                    let disp = m.inputs["disp"][8 * byte + bit];
                                    if op == "load" {
                                        let delta = b.xor(old[cell], old[v]);
                                        let delta = b.and(hit, delta);
                                        want[v] = b.xor(want[v], delta);
                                        let d = b.and(hit, old[v]);
                                        want[disp] = b.xor(want[disp], d);
                                    } else {
                                        let delta = b.xor(old[cell], old[v]);
                                        let delta = b.and(hit, delta);
                                        want[cell] = b.xor(want[cell], delta);
                                        let d = b.and(hit, old[cell]);
                                        want[disp] = b.xor(want[disp], d);
                                    }
                                }
                            }
                        }
                    }
                    m.run_symbolic(&mut b, &mut state, enabled, false, domain)
                        .unwrap();
                    for (i, v) in state.iter().enumerate() {
                        assert!(
                            b.equivalent(*v, want[i], domain),
                            "{op} P={p} bytes={bytes} mapped={mapped} bit={i}"
                        );
                    }
                    m.run_symbolic(&mut b, &mut state, enabled, true, domain)
                        .unwrap();
                    for (i, v) in state.iter().enumerate() {
                        assert!(b.equivalent(*v, old[i], domain));
                    }
                    println!("layout {op}: P={p}, bytes={bytes}, mapped={mapped}, full domain {} BDD nodes",b.nodes());
                }
            }
        }
    }
}

#[test]
fn memory_fault_write_ordinals_follow_all_assignment_exchanges() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let bundles = [
        ("a", 2),
        ("base", 2),
        ("size", 2),
        ("live", 1),
        ("pinned", 1),
        ("atop", 1),
        ("flag", 1),
        ("faulted", 1),
        ("code", 4),
        ("v", 8),
        ("disp", 8),
        ("mem", 32),
    ];
    let args=json::parse(r#"[2,1,3,1,1,4,8,1,"a","base","size","live","pinned","atop","flag","faulted","code","v","disp",[{"base":0,"extent":4,"cells":"mem"}]]"#).unwrap();
    for op in ["load", "store"] {
        let leaves = e
            .expand(&format!("lift.{op}"), args.as_arr().unwrap(), &bundles)
            .unwrap();
        let last_exchange = leaves
            .iter()
            .rposition(|l| {
                l.gate == "swap"
                    && l.wires
                        .iter()
                        .any(|(n, _)| n == if op == "load" { "v" } else { "mem" })
            })
            .unwrap();
        let first_fault = leaves
            .iter()
            .position(|l| l.wires.iter().any(|(n, _)| n == "faulted"))
            .unwrap();
        assert!(
            first_fault > last_exchange,
            "§3.6(d) fault writes follow §3.6(c) assignment effects: {op}"
        );
    }
}
