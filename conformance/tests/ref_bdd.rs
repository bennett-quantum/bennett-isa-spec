use bennett_isa_conformance::ref_bdd::Bdd;
#[test]
fn bdd_truth_tables_and_canonical_equalities() {
    let mut b = Bdd::new();
    let a = (0..4).map(|_| b.variable()).collect::<Vec<_>>();
    let c = (0..4).map(|_| b.variable()).collect::<Vec<_>>();
    let sum = b.add(&a, &c);
    let lt = b.less(&a, &c);
    let eq = b.equal(&a, &c);
    for x in 0..16 {
        for y in 0..16 {
            let assignment = x | (y << 4);
            assert_eq!(b.evaluate(lt, assignment), x < y);
            assert_eq!(b.evaluate(eq, assignment), x == y);
            let actual = sum
                .iter()
                .enumerate()
                .map(|(i, n)| usize::from(b.evaluate(*n, assignment)) << i)
                .sum::<usize>();
            assert_eq!(actual, (x + y) & 15);
        }
    }
    let xor = b.xor(a[0], c[0]);
    let and = b.and(a[0], c[0]);
    let or = b.or(a[0], c[0]);
    let decomposed = b.xor(xor, and);
    assert_eq!(or, decomposed);
    let double = b.xor(xor, c[0]);
    assert_eq!(double, a[0]);
}
