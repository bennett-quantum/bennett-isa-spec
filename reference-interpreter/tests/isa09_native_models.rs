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
