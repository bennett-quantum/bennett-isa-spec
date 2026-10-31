use bennett_isa_conformance::{
    json::{self},
    ref_catalog::Machine,
    ref_language::{permutation, Expander, Registry},
};
#[test]
fn complete_call_interfaces_preserve_values_regions_and_callee_domains() {
    let hash = "c".repeat(64);
    let mut tuples = 0;
    for w in 1..=4 {
        for role in ["in", "mut"] {
            let registry_body = format!(
                r#"{{"parameters":[{{"name":"x","type":"wire","width":{w}}},{{"name":"y","type":"wire","width":{w}}},{{"name":"fr","type":"wire","width":4}},{{"name":"r","type":"region","address":0,"cell":{w},"role":"{role}"}},{{"name":"c","type":"value","isa_type":"i128"}},{{"name":"tk","type":"value","isa_type":"token"}}],"body":[{{"gate":"cx","wires":["x[0]","y[0]"]}},{{"gate":"cx","wires":["r[0]","y[0]"]}}]}}"#
            );
            let mut registry = Registry::new();
            registry.insert(hash.clone(), json::parse(&registry_body).unwrap());
            let mut e =
                Expander::new(include_str!("../../spec/ref-0.9-draft.json"), registry).unwrap();
            for kind in [
                "lift.oracle_call",
                "lift.oracle_uncall",
                "lift.perm_call",
                "oracle_call",
                "action_call",
            ] {
                if role == "mut" && kind.contains("oracle") {
                    continue;
                }
                // Tier/role restrictions are source well-formedness; this directly
                // checks the shared typed forwarding constructor at both roles.
                let args=json::parse(&format!(r#"["{hash}",["x","y","fr",{{"address":0,"cell":{w},"role":"{role}","origin":"interface:db","cells":"r"}},"18446744073709551617",["17","2","3","4"]]]"#)).unwrap();
                let bundles = [("x", w), ("y", w), ("r", w), ("fr", 4)];
                let leaves = e.expand(kind, args.as_arr().unwrap(), &bundles).unwrap();
                assert_eq!(leaves.iter().map(|l| l.ordinal).collect::<Vec<_>>(), [0, 1]);
                assert!(leaves
                    .iter()
                    .all(|l| l.invocation == leaves[0].invocation && !l.invocation.is_empty()));
                let m = Machine::compile(&permutation(&leaves).unwrap(), &bundles).unwrap();
                for basis in 0..1usize << (3 * w + 4) {
                    for enabled in [false, true] {
                        let mask = (1 << w) - 1;
                        let x = basis & mask;
                        let y = (basis >> w) & mask;
                        let r = (basis >> (2 * w)) & mask;
                        let f = (basis >> (3 * w)) & 15;
                        let mut s = m.state();
                        m.set(&mut s, "x", x as u64);
                        m.set(&mut s, "y", y as u64);
                        m.set(&mut s, "r", r as u64);
                        m.set(&mut s, "fr", f as u64);
                        let old = s.clone();
                        m.run(&mut s, enabled, false).unwrap();
                        assert_eq!(
                            m.get(&s, "y"),
                            (y ^ if enabled { (x ^ r) & 1 } else { 0 }) as u64
                        );
                        assert_eq!(m.get(&s, "x"), x as u64);
                        assert_eq!(m.get(&s, "r"), r as u64);
                        assert_eq!(m.get(&s, "fr"), f as u64);
                        m.run(&mut s, enabled, true).unwrap();
                        assert_eq!(s, old);
                        tuples += 1;
                    }
                }
                let bad = json::parse(&args.to_string().replace("\"address\":0", "\"address\":1"))
                    .unwrap();
                assert!(e.expand(kind, bad.as_arr().unwrap(), &bundles).is_err());
            }
        }
    }
    assert_eq!(tuples, 1048320);
    println!("call forwarding tuples {tuples}");
}
