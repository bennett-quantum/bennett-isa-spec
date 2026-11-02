//! Finite witness using the unchanged reference interpreter, not a new interpreter.
use bennett_isa_ref::{Argument, Budget, Object, OracleInput, OracleMachine};

fn main() {
    let path = std::env::args().nth(1).expect("oracle fixture path required");
    let text = std::fs::read_to_string(path).expect("read oracle fixture");
    let object = Object::parse(&text).expect("parse 0.9 oracle fixture");
    let mut tuples = 0;
    for b in 0..=2 {
        let budget = Budget::steps(b);
        let mut faults = 0;
        for x in 0..4 {
            let mut input = OracleInput::bits(&[x]);
            input.args = vec![Argument::Bits(1)];
            let expected = object.run_oracle("add2", &input, budget).unwrap();
            let pair = if b < 2 { (0, 1) } else { ((x + 1) % 4, 0) };
            assert_eq!((expected.payload, expected.code), pair);
            assert_eq!(expected.steps, b);
            let mut machine = OracleMachine::new(&object, "add2", &input, budget).unwrap();
            let initial = machine.snapshot();
            for _ in 0..b {
                machine.forward_step().unwrap();
            }
            assert_eq!(machine.outcome().unwrap(), expected);
            for target in 0..4 {
                for fault in 0..16 {
                    let after = machine.commit(target, fault).unwrap();
                    assert_eq!(after, (target ^ pair.0, fault ^ pair.1));
                    // Applying the same XOR-delivery map twice restores arbitrary entries.
                    assert_eq!((after.0 ^ pair.0, after.1 ^ pair.1), (target, fault));
                    faults += usize::from(expected.code != 0);
                    tuples += 1;
                }
            }
            for _ in 0..b {
                machine.inverse_step().unwrap();
            }
            assert_eq!(machine.snapshot(), initial, "full machine reversal at x={x}, B={b}");
            assert!(machine.frames_clean());
        }
        assert_eq!(faults, if b < 2 { 256 } else { 0 });
        println!("{{\"budget\":{b},\"tuples\":256,\"faults\":{faults}}}");
    }
    assert_eq!(tuples, 768);
}
