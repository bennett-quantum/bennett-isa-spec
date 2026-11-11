use bennett_isa_ref::{Budget, DerivedWidths, Object, OracleInput, OracleMachine};

const SOURCE: &str =
    include_str!("../../conformance/cases/F/F-return-delivery-width-09/return.bisa");
const BUDGET: Budget = Budget {
    steps: 4,
    heap: 1,
    stack: 8,
    depth: 1,
};

#[test]
fn unused_wide_location_sizes_every_return_delivery_frame() {
    let object = Object::parse(SOURCE).unwrap();
    let widths = DerivedWidths::derive(&object, "root", BUDGET).unwrap();
    assert_eq!(widths.slots, [8, 4, 2]);
    let frame = 2 * widths.p + widths.slots.iter().sum::<usize>() + 1;
    assert_eq!((frame, frame * BUDGET.steps as usize), (21, 84));
    let mut machine = OracleMachine::new(&object, "root", &OracleInput::bits(&[]), BUDGET).unwrap();
    assert_eq!(machine.widths(), &widths);
    let entry = machine.snapshot();
    for _ in 0..4 {
        machine.forward_step().unwrap();
    }
    let result = object
        .run_oracle("root", &OracleInput::bits(&[]), BUDGET)
        .unwrap();
    assert_eq!((result.payload, result.code, result.steps), (1, 0, 4));
    for _ in 0..4 {
        machine.inverse_step().unwrap();
    }
    assert_eq!(machine.snapshot(), entry);
    assert!(machine.frames_clean());
}

#[test]
fn closure_without_return_keeps_listed_slot_maxima() {
    let object = Object::parse(&SOURCE.replace("lift.ret v", "fault 4")).unwrap();
    assert_eq!(
        DerivedWidths::derive(&object, "root", BUDGET)
            .unwrap()
            .slots,
        [4, 4, 2]
    );
}
