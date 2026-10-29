//! Codex suggestion (round 1): every §4.4 gate against hand-written matrices,
//! including disabled controls; and the §12.1 sampler's reference vector.
use bennett_isa_ref::{Complex, QuantumState, Xoshiro256};

fn close(a: Complex, b: (f64, f64)) -> bool {
    (a.re - b.0).abs() < 1e-12 && (a.im - b.1).abs() < 1e-12
}
/// Column `input` of the gate's matrix: apply to |input> on one wire.
fn column(op: &str, angle: Option<f64>, input: u128) -> Vec<Complex> {
    let mut q = QuantumState::new();
    let w = q.allocate(1).unwrap();
    if input == 1 {
        q.gate("x", &w, None, &[]).unwrap();
    }
    q.gate(op, &w, angle, &[]).unwrap();
    q.amplitudes().unwrap()
}
#[test]
fn single_wire_gates_match_section_4_4() {
    let r = std::f64::consts::FRAC_1_SQRT_2;
    let (c, s) = ((0.7f64 / 2.0).cos(), (0.7f64 / 2.0).sin());
    let (pc, ps) = (
        std::f64::consts::FRAC_PI_4.cos(),
        std::f64::consts::FRAC_PI_4.sin(),
    );
    // (gate, angle, column for |0>, column for |1>), entries (re, im).
    type Column = [(f64, f64); 2];
    let cases: Vec<(&str, Option<f64>, Column, Column)> = vec![
        (
            "x",
            None,
            [(0.0, 0.0), (1.0, 0.0)],
            [(1.0, 0.0), (0.0, 0.0)],
        ),
        ("h", None, [(r, 0.0), (r, 0.0)], [(r, 0.0), (-r, 0.0)]),
        (
            "s",
            None,
            [(1.0, 0.0), (0.0, 0.0)],
            [(0.0, 0.0), (0.0, 1.0)],
        ),
        (
            "sdg",
            None,
            [(1.0, 0.0), (0.0, 0.0)],
            [(0.0, 0.0), (0.0, -1.0)],
        ),
        ("t", None, [(1.0, 0.0), (0.0, 0.0)], [(0.0, 0.0), (pc, ps)]),
        (
            "tdg",
            None,
            [(1.0, 0.0), (0.0, 0.0)],
            [(0.0, 0.0), (pc, -ps)],
        ),
        // rx = [[c, -is], [-is, c]], ry = [[c, -s], [s, c]], rz = diag(e^-i/2, e^i/2).
        (
            "rx",
            Some(0.7),
            [(c, 0.0), (0.0, -s)],
            [(0.0, -s), (c, 0.0)],
        ),
        ("ry", Some(0.7), [(c, 0.0), (s, 0.0)], [(-s, 0.0), (c, 0.0)]),
        ("rz", Some(0.7), [(c, -s), (0.0, 0.0)], [(0.0, 0.0), (c, s)]),
    ];
    for (op, angle, col0, col1) in cases {
        for (input, col) in [(0, col0), (1, col1)] {
            let a = column(op, angle, input);
            assert!(
                close(a[0], col[0]) && close(a[1], col[1]),
                "{op}|{input}> = {a:?}"
            );
        }
    }
}
#[test]
fn controlled_gates_act_only_when_every_control_is_one() {
    // Basis order: wire 0 is the least significant bit.
    for (op, wires, input, expected) in [
        ("cx", 2, 0b01, 0b11),
        ("cx", 2, 0b10, 0b10),
        ("ccx", 3, 0b011, 0b111),
        ("ccx", 3, 0b001, 0b001),
        ("mcx", 4, 0b0111, 0b1111),
        ("mcx", 4, 0b0101, 0b0101),
        ("swap", 2, 0b01, 0b10),
    ] {
        let mut q = QuantumState::new();
        let w = q.allocate(wires).unwrap();
        for (i, wire) in w.iter().enumerate() {
            if input >> i & 1 == 1 {
                q.gate("x", &[*wire], None, &[]).unwrap();
            }
        }
        q.gate(op, &w, None, &[]).unwrap();
        assert_eq!(q.read_basis(&w).unwrap(), expected, "{op} on {input:b}");
    }
    // cz = diag(1, 1, 1, -1) after h on both wires: amplitudes (1,1,1,-1)/2.
    let mut q = QuantumState::new();
    let w = q.allocate(2).unwrap();
    q.gate("h", &w[..1], None, &[]).unwrap();
    q.gate("h", &w[1..], None, &[]).unwrap();
    q.gate("cz", &w, None, &[]).unwrap();
    let a = q.amplitudes().unwrap();
    for (i, e) in [(0.5, 0.0), (0.5, 0.0), (0.5, 0.0), (-0.5, 0.0)]
        .into_iter()
        .enumerate()
    {
        assert!(close(a[i], e), "{a:?}");
    }
    // A disabled predicate leaves h out entirely.
    let mut q = QuantumState::new();
    let w = q.allocate(2).unwrap();
    q.gate("h", &w[1..], None, &[w[0]]).unwrap();
    assert_eq!(q.read_basis(&w).unwrap(), 0);
}
#[test]
fn xoshiro256_star_star_reference_vector() {
    // Blackman-Vigna reference output for state (1, 2, 3, 4).
    let mut r = Xoshiro256([1, 2, 3, 4]);
    let words: Vec<u64> = (0..4).map(|_| r.next_word()).collect();
    assert_eq!(words, [11520, 0, 1509978240, 1215971899390074240]);
}
