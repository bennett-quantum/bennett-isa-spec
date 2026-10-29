use bennett_isa_ref::BitVector;
use std::cmp::Ordering;
fn bits(width: usize, n: u128) -> BitVector {
    BitVector::from_u128(width, n).unwrap()
}
#[test]
fn exhaustive_small_width_bitvector_arithmetic_and_signed_division() {
    for width in 1..=6 {
        let modulus = 1u128 << width;
        let mask = modulus - 1;
        let signed = |n: u128| {
            if n & (modulus >> 1) != 0 {
                n as i128 - modulus as i128
            } else {
                n as i128
            }
        };
        for a in 0..modulus {
            for b in 0..modulus {
                let x = bits(width, a);
                let y = bits(width, b);
                for (op, expected) in [
                    ("add", a.wrapping_add(b) & mask),
                    ("sub", a.wrapping_sub(b) & mask),
                    ("mul", a.wrapping_mul(b) & mask),
                    ("and", a & b),
                    ("or", a | b),
                    ("xor", a ^ b),
                    (
                        "shl",
                        if b < width as u128 {
                            (a << b) & mask
                        } else {
                            0
                        },
                    ),
                    ("lshr", if b < width as u128 { a >> b } else { 0 }),
                    (
                        "ashr",
                        // §4.1: "wrap; shift >= N -> 0" covers ashr too.
                        if b < width as u128 {
                            (signed(a) >> b) as u128 & mask
                        } else {
                            0
                        },
                    ),
                ] {
                    assert_eq!(
                        x.binary(op, &y).unwrap().to_u128().unwrap(),
                        expected,
                        "{width} {a} {op} {b}"
                    );
                }
                assert_eq!(x.unsigned_cmp(&y).unwrap(), a.cmp(&b));
                assert_eq!(x.signed_cmp(&y).unwrap(), signed(a).cmp(&signed(b)));
                for signed_op in [false, true] {
                    if b == 0
                        || (signed_op && signed(a) == -(modulus as i128 / 2) && signed(b) == -1)
                    {
                        assert!(x.div_rem(&y, signed_op).is_err());
                    } else {
                        let (q, r) = x.div_rem(&y, signed_op).unwrap();
                        let expected = if signed_op {
                            (
                                (signed(a) / signed(b)) as u128 & mask,
                                (signed(a) % signed(b)) as u128 & mask,
                            )
                        } else {
                            (a / b, a % b)
                        };
                        assert_eq!((q.to_u128().unwrap(), r.to_u128().unwrap()), expected);
                    }
                }
            }
        }
    }
}
#[test]
fn wide_carries_multiplication_division_decimal_and_extensions() {
    // At width257, (2^200+3)*(2^100+5) = 5*2^200+3*2^100+15 modulo2^257.
    let a = BitVector::parse_decimal(
        257,
        "1606938044258990275541962092341162602522202993782792835301379",
    )
    .unwrap();
    let b = BitVector::parse_decimal(257, "1267650600228229401496703205381").unwrap();
    let product = a.binary("mul", &b).unwrap();
    let mut expected = bits(257, 15);
    for bit in [100, 101, 200, 202] {
        expected.set(bit, true).unwrap();
    }
    assert_eq!(product, expected);
    let (q, r) = a.div_rem(&b, false).unwrap();
    assert_eq!(q.to_decimal(), "1267650600228229401496703205371");
    assert_eq!(r.to_u128().unwrap(), 28);
    assert_eq!(q.binary("mul", &b).unwrap().binary("add", &r).unwrap(), a);
    let negative = BitVector::parse_decimal(257, "-7").unwrap();
    let (q, r) = negative.div_rem(&bits(257, 2), true).unwrap();
    assert_eq!(q, bits(8, 253).resize(257, true).unwrap());
    assert_eq!(r, BitVector::parse_decimal(257, "-1").unwrap());
    assert_eq!(
        BitVector::from_le_bytes(9, &[0xff, 1])
            .unwrap()
            .to_decimal(),
        "511"
    );
    assert!(BitVector::from_le_bytes(9, &[0xff, 2]).is_err());
    assert_eq!(bits(257, 1).unsigned_cmp(&a).unwrap(), Ordering::Less);
    assert!(a.to_u128().is_err());
    assert!(bits(257, 1)
        .binary("shl", &bits(257, 256))
        .unwrap()
        .bit(256)
        .unwrap());
    assert_eq!(a.to_le_bytes().len(), 33);
    assert_eq!(BitVector::from_le_bytes(257, &a.to_le_bytes()).unwrap(), a);
}
