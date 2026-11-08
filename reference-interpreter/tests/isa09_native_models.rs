use bennett_isa_ref::target::Interval;
use bennett_isa_ref::target::{
    verify_channel_bound, verify_numerical_bound, verify_restricted_numerical_bound,
    ComplexInterval, IntervalMatrix,
};

fn matrix(rows: &[&[i128]]) -> IntervalMatrix {
    let entries = rows
        .iter()
        .flat_map(|row| row.iter())
        .map(|n| ComplexInterval::real(Rational::integer(*n), 64).unwrap())
        .collect();
    IntervalMatrix::new(rows.len(), rows[0].len(), entries).unwrap()
}

fn scalar_identity(size: usize, value: Rational) -> IntervalMatrix {
    let entries = (0..size)
        .flat_map(|row| {
            let value = value.clone();
            (0..size).map(move |col| {
                ComplexInterval::real(
                    if row == col {
                        value.clone()
                    } else {
                        Rational::integer(0)
                    },
                    64,
                )
                .unwrap()
            })
        })
        .collect();
    IntervalMatrix::new(size, size, entries).unwrap()
}

#[test]
fn phase_sensitive_frobenius_claim_uses_the_norm_not_its_square() {
    // §8.3/§12.1: controlled X vs controlled -X differs in its enabled
    // 2x2 block. Frobenius norm sqrt(4+4) lies strictly between 2 and 3.
    let cx = matrix(&[&[1, 0, 0, 0], &[0, 1, 0, 0], &[0, 0, 0, 1], &[0, 0, 1, 0]]);
    let minus_cx = matrix(&[&[1, 0, 0, 0], &[0, 1, 0, 0], &[0, 0, 0, -1], &[0, 0, -1, 0]]);
    assert!(verify_numerical_bound(&cx, &minus_cx, &Rational::integer(2)).is_err());
    verify_numerical_bound(&cx, &minus_cx, &Rational::integer(3)).unwrap();
    verify_numerical_bound(&cx, &cx, &Rational::integer(0)).unwrap();
}

#[test]
fn channel_dual_requires_every_strict_interval_pivot() {
    // §15 suite D's independently specified zero-channel witness:
    // Y=I4/8, mu=3/8; Tr_out(Y)=I2/4, so all three LDL pivots=1/8.
    let x = matrix(&[&[0, 1], &[1, 0]]);
    let minus_x = matrix(&[&[0, -1], &[-1, 0]]);
    let eighth = Rational::leaf("1", "8").unwrap();
    let mu = Rational::leaf("3", "8").unwrap();
    let y = scalar_identity(4, eighth);
    verify_channel_bound(&x, &minus_x, &y, &mu, &mu).unwrap();
    let zero = Rational::integer(0);
    assert!(verify_numerical_bound(&x, &minus_x, &zero).is_err());
    assert!(verify_channel_bound(&x, &x, &scalar_identity(4, zero.clone()), &zero, &zero).is_err());
    let wrong = matrix(&[&[0, 0, 0, 0], &[0, -1, 0, 0], &[0, 0, 1, 0], &[0, 0, 0, 1]]);
    assert!(verify_channel_bound(&x, &x, &wrong, &mu, &mu).is_err());
    assert!(verify_channel_bound(&x, &x, &y, &Rational::leaf("1", "4").unwrap(), &mu).is_err());
    assert!(verify_channel_bound(&x, &x, &y, &mu, &Rational::decimal("0.37").unwrap()).is_err());
}

#[test]
fn nonzero_complex_choi_terms_reach_both_dual_inequalities() {
    // For S=diag(1,i), I: J(S-I) has -1-i at (0,3) and -1+i
    // at (3,0). Y=2I4 gives LDL pivots 2,2,2,1 in Y±J.
    // Tr_out(Y)=4I2, so mu=5 gives the third matrix I2.
    let p = 64;
    let real = |n| ComplexInterval::real(Rational::integer(n), p).unwrap();
    let imaginary = ComplexInterval::new(
        Interval::point(Rational::integer(0), p).unwrap(),
        Interval::point(Rational::integer(1), p).unwrap(),
    )
    .unwrap();
    let s = IntervalMatrix::new(2, 2, vec![real(1), real(0), real(0), imaginary]).unwrap();
    let identity = matrix(&[&[1, 0], &[0, 1]]);
    let y = scalar_identity(4, Rational::integer(2));
    verify_channel_bound(
        &s,
        &identity,
        &y,
        &Rational::integer(5),
        &Rational::integer(5),
    )
    .unwrap();
    // The zero-channel witness is infeasible for this nonzero Choi matrix.
    let small = scalar_identity(4, Rational::leaf("1", "8").unwrap());
    assert!(verify_channel_bound(
        &s,
        &identity,
        &small,
        &Rational::leaf("3", "8").unwrap(),
        &Rational::integer(1)
    )
    .is_err());
    // Y has diagonal (2,1,1,2), Y03=Y30=-1. Y-J has corner
    // eigenvalues 1,3, while Y+J has 2±sqrt(5) and is indefinite.
    // With mu=4 the slack is I2. Swapping U,V swaps the two tests.
    let asymmetric = matrix(&[&[2, 0, 0, -1], &[0, 1, 0, 0], &[0, 0, 1, 0], &[-1, 0, 0, 2]]);
    assert!(verify_channel_bound(
        &s,
        &identity,
        &asymmetric,
        &Rational::integer(4),
        &Rational::integer(4)
    )
    .is_err());
    assert!(verify_channel_bound(
        &identity,
        &s,
        &asymmetric,
        &Rational::integer(4),
        &Rational::integer(4)
    )
    .is_err());
}

#[test]
fn restricted_processes_keep_the_complete_output_dimension_and_phase() {
    // K85's U=Z,V=I with J_I=|0> has equal columns. With J_I=|1>,
    // columns differ by -1: channel equality survives, phase equality fails.
    let zero_column = matrix(&[&[1], &[0]]);
    verify_restricted_numerical_bound(&zero_column, &zero_column, &Rational::integer(0)).unwrap();
    let one_column = matrix(&[&[0], &[1]]);
    let minus_column = matrix(&[&[0], &[-1]]);
    assert!(
        verify_restricted_numerical_bound(&one_column, &minus_column, &Rational::integer(0))
            .is_err()
    );
    let y = scalar_identity(2, Rational::leaf("1", "8").unwrap());
    let mu = Rational::leaf("3", "8").unwrap();
    verify_channel_bound(&one_column, &minus_column, &y, &mu, &mu).unwrap();
    // d_in=2,d_out=4: Choi dimension=8 and Tr_out(I8/8)=I2/2.
    let isometry = matrix(&[&[1, 0], &[0, 1], &[0, 0], &[0, 0]]);
    let y = scalar_identity(8, Rational::leaf("1", "8").unwrap());
    let mu = Rational::leaf("5", "8").unwrap();
    verify_channel_bound(&isometry, &isometry, &y, &mu, &mu).unwrap();
}
use bennett_isa_ref::target::Rational;

#[test]
fn decimal_and_binary64_bindings_remain_distinct_exact_rationals() {
    // §8.1 decimal 0.1 is 1/10; §8.3/K20 binary64 0.1 is
    // (2^52 + 2702159776422298) / 2^56, reduced by two.
    let decimal = Rational::decimal("0.1").unwrap();
    let angle = Rational::binary64(0.1f64.to_bits()).unwrap();
    assert_eq!(decimal.pair(), ("1".into(), "10".into()));
    assert_eq!(
        angle.pair(),
        ("3602879701896397".into(), "36028797018963968".into())
    );
    assert!(angle > decimal);
    assert_eq!(
        Rational::decimal("1e-400")
            .unwrap()
            .mul(&Rational::decimal("1e400").unwrap()),
        Rational::integer(1)
    );
    assert!(Rational::binary64(f64::INFINITY.to_bits()).is_err());
}

#[test]
fn interval_operations_round_outward_at_the_requested_binary_precision() {
    // Three significant binary bits enclose 1/3 by 0.0101₂ and 0.0110₂.
    let third = Interval::point(Rational::leaf("1", "3").unwrap(), 3).unwrap();
    assert_eq!(third.lower().pair(), ("5".into(), "16".into()));
    assert_eq!(third.upper().pair(), ("3".into(), "8".into()));
    let sum = third.add(&third).unwrap();
    assert_eq!(sum.lower().pair(), ("5".into(), "8".into()));
    assert_eq!(sum.upper().pair(), ("3".into(), "4".into()));
    let negative = third.neg();
    assert_eq!(negative.lower(), &third.upper().neg());
    assert_eq!(negative.upper(), &third.lower().neg());
    let two = Interval::point(Rational::integer(2), 4).unwrap();
    let root = two.sqrt().unwrap();
    // 11²/8² < 2 < 12²/8².
    assert_eq!(root.lower().pair(), ("11".into(), "8".into()));
    assert_eq!(root.upper().pair(), ("3".into(), "2".into()));
    assert!(Interval::point(Rational::integer(0), 0).is_err());
    assert!(two
        .div(&Interval::point(Rational::integer(0), 4).unwrap())
        .is_err());
}

#[test]
fn rational_leaves_are_canonical_and_arithmetic_never_overflows_host_integers() {
    // §8.3 leaves are reduced, denominator positive, zero exactly 0/1.
    for (n, d) in [
        ("2", "4"),
        ("0", "2"),
        ("01", "1"),
        ("-0", "1"),
        ("1", "0"),
        ("1", "-1"),
    ] {
        assert!(Rational::leaf(n, d).is_err(), "accepted {n}/{d}");
    }
    let big = Rational::leaf("340282366920938463463374607431768211457", "3").unwrap();
    let third = Rational::leaf("1", "3").unwrap();
    assert_eq!(
        big.sub(&third).pair(),
        ("340282366920938463463374607431768211456".into(), "3".into())
    );
    assert_eq!(third.add(&third).div(&third).unwrap(), Rational::integer(2));
    assert_eq!(
        Rational::leaf("-4", "3").unwrap().ceil(),
        Rational::integer(-1)
    );
    assert!(third.div(&Rational::integer(0)).is_err());
}
