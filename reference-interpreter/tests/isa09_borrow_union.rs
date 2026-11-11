use bennett_isa_ref::{BitVector, OrderedEntryUnion, Xoshiro256};
fn bit(n: u128) -> BitVector {
    BitVector::from_u128(1, n).unwrap()
}
fn product(ranges: &[(u128, u128)]) -> Vec<(BitVector, BitVector)> {
    ranges.iter().map(|(a, b)| (bit(*a), bit(*b))).collect()
}
#[test]
fn union_draws_one_rank_in_visit_order_and_fibers_consume_no_words() {
    let union = OrderedEntryUnion::from_products(&[
        product(&[(0, 0), (0, 0), (0, 1)]),
        product(&[(1, 1), (1, 1), (0, 1)]),
    ])
    .unwrap();
    assert_eq!(union.entries().len(), 4);
    for seed in [[0, 288230376151711744, 0, 0], [1, 72057594037927936, 0, 0]] {
        let mut rng = Xoshiro256(seed);
        let index = union.draw_index(&mut rng).unwrap();
        assert_eq!(index, 2);
        assert_eq!(
            union.entries()[index]
                .iter()
                .map(|b| b.to_u128().unwrap())
                .collect::<Vec<_>>(),
            [1, 1, 0]
        );
        let after = rng.0;
        let fiber = union.borrow_fiber(index, &[2]).unwrap();
        assert_eq!(fiber, [2, 3]);
        assert_eq!(rng.0, after);
    }
    let singleton = OrderedEntryUnion::from_products(&[product(&[(0, 0)])]).unwrap();
    let mut rng = Xoshiro256([1, 2, 3, 4]);
    let before = rng.0;
    assert_eq!(singleton.draw_index(&mut rng).unwrap(), 0);
    assert_eq!(rng.0, before);
}
#[test]
fn overlapping_products_deduplicate_complete_tuples_and_keep_last_component_fastest() {
    let union = OrderedEntryUnion::from_products(&[
        product(&[(0, 0), (0, 1), (0, 1)]),
        product(&[(0, 1), (0, 0), (0, 1)]),
    ])
    .unwrap();
    let rows = union
        .entries()
        .iter()
        .map(|row| row.iter().map(|b| b.to_u128().unwrap()).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    assert_eq!(
        rows,
        vec![
            vec![0, 0, 0],
            vec![0, 0, 1],
            vec![0, 1, 0],
            vec![0, 1, 1],
            vec![1, 0, 0],
            vec![1, 0, 1]
        ]
    );
    assert_eq!(union.borrow_fiber(4, &[2]).unwrap(), [4, 5]);
    assert!(union.borrow_fiber(0, &[3]).is_err());
    assert!(union.borrow_fiber(0, &[2, 2]).is_err());
    assert!(OrderedEntryUnion::from_products(&[]).is_err());
    assert!(OrderedEntryUnion::from_products(&[product(&[(1, 0)])]).is_err());
}
