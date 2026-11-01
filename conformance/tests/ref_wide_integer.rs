use bennett_isa_conformance::{
    json,
    ref_language::{Expander, Registry},
};
#[test]
fn literal_construction_keeps_bits_above_machine_words() {
    let mut e = Expander::new(include_str!("../../spec/ref-0.9.json"), Registry::new()).unwrap();
    for (decimal, width, bits) in [
        ("18446744073709551617", 65, vec![0, 64]),
        (
            "340282366920938463463374607431768211455",
            128,
            (0..128).collect(),
        ),
    ] {
        let args = json::parse(&format!(r#"[{width},"{decimal}","out"]"#)).unwrap();
        let leaves = e
            .expand("value.literal", args.as_arr().unwrap(), &[("out", width)])
            .unwrap();
        assert_eq!(
            leaves
                .iter()
                .map(|l| {
                    assert_eq!(l.gate, "x");
                    assert!(l.controls.is_empty());
                    l.wires[0].1
                })
                .collect::<Vec<_>>(),
            bits
        );
    }
}
