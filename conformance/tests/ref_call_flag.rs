use bennett_isa_conformance::{
    json,
    ref_bdd::Bdd,
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
fn check(
    e: &mut Expander,
    d: usize,
    s: usize,
    r: usize,
    depth_limit: usize,
    stack_limit: usize,
) -> u64 {
    let width = s.max(usize::BITS as usize - r.leading_zeros() as usize) + 1;
    let bundles = [("depth", d), ("sp", s), ("t", 1)];
    let args = json::parse(&format!(
        r#"[{d},{s},{width},{r},{depth_limit},{stack_limit},"depth","sp","t"]"#
    ))
    .unwrap();
    let m = Machine::compile(
        &permutation(
            &e.expand("value.call.flag", args.as_arr().unwrap(), &bundles)
                .unwrap(),
        )
        .unwrap(),
        &bundles,
    )
    .unwrap();
    let mut b = Bdd::new();
    let mut state = m.symbolic_state();
    for name in ["depth", "sp", "t"] {
        for id in &m.inputs[name] {
            state[*id] = b.variable();
        }
    }
    let en = b.variable();
    let old = state.clone();
    let mut depth = m.inputs["depth"]
        .iter()
        .map(|i| old[*i])
        .collect::<Vec<_>>();
    depth.push(0);
    let depth_next = b.add(&depth, &Bdd::constant(1, d + 1));
    let depth_fault = b.less(&Bdd::constant(depth_limit, d + 1), &depth_next);
    let mut sp = m.inputs["sp"].iter().map(|i| old[*i]).collect::<Vec<_>>();
    sp.resize(width, 0);
    let next = b.add(&sp, &Bdd::constant(r, width));
    let stack_fault = b.less(&Bdd::constant(stack_limit, width), &next);
    let fault = b.or(depth_fault, stack_fault);
    let fault = b.and(en, fault);
    let mut expected = old.clone();
    let id = m.inputs["t"][0];
    expected[id] = b.xor(expected[id], fault);
    m.run_symbolic(&mut b, &mut state, en, false, 1).unwrap();
    assert_eq!(state, expected);
    m.run_symbolic(&mut b, &mut state, en, true, 1).unwrap();
    assert_eq!(state, old);
    1u64 << (d + s + 2)
}
#[test]
fn general_call_records_and_exact_example_counter_widths() {
    let mut e = Expander::new(
        include_str!("../../spec/ref-0.9-draft.json"),
        Registry::new(),
    )
    .unwrap();
    let mut count = 0;
    for d in 1..=4 {
        for s in 1..=4 {
            for r in [4, 5, 16] {
                for dl in [0, (1 << d) - 2] {
                    for sl in [0, 1 << (s - 1)] {
                        count += check(&mut e, d, s, r, dl, sl);
                    }
                }
            }
        }
    }
    // 16 width pairs × 3 record sizes × 2 depth limits × 2 stack limits.
    assert_eq!(count, 43200);
    assert_eq!(check(&mut e, 5, 10, 16, 17, 272), 131072);
    println!("generic call flags: {count} tuples across 192 configurations; §16.3 R=16, D=5, S=10, limits17/272: 131072 tuples");
}
