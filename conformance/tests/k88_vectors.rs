use bennett_isa_conformance::json::{self, Json};
#[test]
fn split_relation_and_one_slot_fragment_vector_are_independent() {
    let v = json::parse(include_str!("../fixtures/accepted-k88-09/vectors.json")).unwrap();
    let frame = v.get("frame").unwrap();
    let slots = frame.get("slots").unwrap().as_arr().unwrap();
    assert_eq!(slots.len(), 1);
    assert_eq!(slots[0].get("id").unwrap().as_str(), Some("mut:0"));
    let fragments = slots[0].get("fragments").unwrap().as_arr().unwrap();
    assert_eq!(fragments.len(), 2);
    for (i, c) in ["A", "B"].iter().enumerate() {
        assert_eq!(fragments[i].get("offset"), Some(&Json::Num(i as f64)));
        assert_eq!(fragments[i].get("width"), Some(&Json::Num(1.0)));
        assert_eq!(fragments[i].get("class").unwrap().as_str(), Some(*c));
    }
    for (n, row) in v
        .get("basis_relation")
        .unwrap()
        .as_arr()
        .unwrap()
        .iter()
        .enumerate()
    {
        assert_eq!(row.get("before"), Some(&Json::Num(n as f64)));
        assert_eq!(row.get("after"), Some(&Json::Num((n ^ 3) as f64)));
        assert_eq!(row.get("inverse"), Some(&Json::Num(n as f64)));
    }
}
