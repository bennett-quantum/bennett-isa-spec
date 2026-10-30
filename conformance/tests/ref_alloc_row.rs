use bennett_isa_conformance::{
    json,
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
fn mux(b: &mut Bdd, mask: usize, old: usize, new: usize) -> usize {
    let d = b.xor(old, new);
    let d = b.and(mask, d);
    b.xor(old, d)
}
#[test]
fn allocator_four_slot_order_full_domains() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    for n in 1..=4 {
        for cells in [2usize, 4] {
            let p = (n + 2).max(5);
            let h = n + 2;
            let t = if cells == 2 { 2 } else { 3 };
            let r = 2 * p + 2;
            let limit = 1 << n;
            let bundles = [
                ("n", n),
                ("used", h),
                ("atop", t),
                ("table", cells * r),
                ("w", p),
                ("dp", p),
                ("de", r),
                ("du", h),
                ("da", t),
                ("flag", 1),
                ("faulted", 1),
                ("code", 4),
            ];
            let args=json::parse(&format!(r#"[{p},{n},{h},{t},{},{h},{n},{cells},16,{limit},4,"n","used","atop","table","w","dp","de","du","da","flag","faulted","code"]"#,h+1)).unwrap();
            let leaves = e
                .expand("lift.alloc", args.as_arr().unwrap(), &bundles)
                .unwrap();
            let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
            let mut b = Bdd::new();
            let mut s = m.symbolic_state();
            for name in ["n", "used", "atop", "w", "table", "faulted", "code"] {
                for id in &m.inputs[name] {
                    s[*id] = b.variable();
                }
            }
            let enabled = b.variable();
            let old = s.clone();
            let mut expected = old.clone();
            let length = m.inputs["n"].iter().map(|i| old[*i]).collect::<Vec<_>>();
            let used = m.inputs["used"].iter().map(|i| old[*i]).collect::<Vec<_>>();
            let atop = m.inputs["atop"].iter().map(|i| old[*i]).collect::<Vec<_>>();
            let mut mx = length.clone();
            mx.resize(h + 1, 0);
            let zero = b.equal(&length, &vec![0; n]);
            mx[0] = b.xor(mx[0], zero);
            let mut ux = used.clone();
            ux.resize(h + 1, 0);
            let sum = b.add(&ux, &mx);
            let fits = b.less(&Bdd::constant(limit, h + 1), &sum);
            let tableok = b.less(&atop, &Bdd::constant(cells, t));
            let tablefull = b.not(tableok);
            let flag = b.or(fits, tablefull);
            let failed = b.and(enabled, flag);
            let success = b.not(flag);
            let active = b.and(enabled, success);
            let invalid = b.less(&Bdd::constant(cells, t), &atop);
            let domain = b.not(invalid);
            expected[m.inputs["flag"][0]] = failed;
            let id = m.inputs["faulted"][0];
            expected[id] = b.xor(expected[id], failed);
            let id = m.inputs["code"][1];
            expected[id] = b.xor(expected[id], failed);
            let mut pointer = used.clone();
            pointer.resize(p, 0);
            let pointer = b.add(&pointer, &Bdd::constant(16, p));
            let next_atop = b.add(&atop, &Bdd::constant(1, t));
            for (name, slot, new) in [
                ("w", "dp", pointer.clone()),
                ("used", "du", sum[..h].to_vec()),
                ("atop", "da", next_atop),
            ] {
                for (bit, id) in m.inputs[name].iter().enumerate() {
                    expected[*id] = mux(&mut b, active, old[*id], new[bit]);
                    expected[m.inputs[slot][bit]] = b.and(active, old[*id]);
                }
            }
            let mut entry = pointer;
            let mut size = length;
            size.resize(p, 0);
            entry.extend(size);
            entry.extend([1, 0]);
            for index in 0..cells {
                let at = b.equal(&atop, &Bdd::constant(index, t));
                let selected = b.and(active, at);
                for (bit, new) in entry.iter().enumerate() {
                    let id = m.inputs["table"][index * r + bit];
                    expected[id] = mux(&mut b, selected, old[id], *new);
                    let captured = b.and(selected, old[id]);
                    let slot = m.inputs["de"][bit];
                    expected[slot] = b.xor(expected[slot], captured);
                }
            }
            m.run_symbolic(&mut b, &mut s, enabled, false, domain)
                .unwrap();
            for (bit, value) in s.iter().enumerate() {
                assert!(
                    b.equivalent(*value, expected[bit], domain),
                    "alloc n={n} C={cells} bit={bit}"
                );
            }
            m.run_symbolic(&mut b, &mut s, enabled, true, domain)
                .unwrap();
            for (bit, value) in s.iter().enumerate() {
                assert!(b.equivalent(*value, old[bit], domain));
            }
            println!(
                "lift.alloc N={n} H={h} P={p} C={cells}: full {}*2^{} tuples; {} BDD nodes",
                cells + 1,
                n + h + cells * r + p + 6,
                b.nodes()
            );
        }
    }
}
