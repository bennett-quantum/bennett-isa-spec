use bennett_isa_conformance::json::{self, Json};
#[test]
fn upward_resource_witnesses_are_exact_integer_ceilings() {
    let v = json::parse(include_str!("../fixtures/accepted-k87-09/vectors.json")).unwrap();
    let scale = 10u128.pow(34);
    for (key, numerator) in [("third", 1), ("root_time", 2), ("root_occupancy", 4)] {
        let text = v.get(key).unwrap().as_str().unwrap();
        let n = text.replace('.', "").parse::<u128>().unwrap();
        let actual_scale = if numerator == 4 { scale / 10 } else { scale };
        assert!(3 * n >= numerator * actual_scale);
        assert!(3 * (n - 1) < numerator * actual_scale);
    }
    assert_eq!(
        v.get("integer_root").unwrap().as_arr().unwrap(),
        &[Json::Num(2.0), Json::Num(4.0), Json::Num(2.0)]
    );
    let source = include_str!("../fixtures/accepted-k87-09/two-flips.bisa");
    assert!(source.contains(
        "action_call @flip (t) () () ()\nqpred c\naction_call @flip (t) () () ()\nqunpred c\n"
    ));
}
