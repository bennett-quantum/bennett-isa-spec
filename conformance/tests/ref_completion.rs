use bennett_isa_conformance::{
    json::{self, Json},
    ref_language::{Expander, Registry},
};
fn doc(recipes: &str) -> String {
    let mut rs = json::parse(recipes).unwrap();
    if let Json::Arr(rs) = &mut rs {
        for r in rs {
            if let Json::Obj(m) = r {
                m.insert("scratch".into(), Json::Arr(vec![]));
                m.insert("removable".into(), Json::Arr(vec![]));
                m.insert(
                    "source".into(),
                    Json::Str("§11.1 completion witness".into()),
                );
            }
        }
    }
    format!(r#"{{"isa":"0.9","name":"ref","version":"0.9","recipes":{rs}}}"#)
}
#[test]
fn image_bits_are_bound_unconditional_lsb_ordered_and_adjointed() {
    let d = doc(
        r#"[{"kind":"image","parameters":[{"name":"I","type":"image"},{"name":"w","type":"wire","width":8},{"name":"E","type":"environment"}],"body":[{"image":"I","wires":"w"}]}]"#,
    );
    let mut e = Expander::new(&d, Registry::new()).unwrap();
    let args = json::parse(r#"["8102","a","c"]"#).unwrap();
    assert!(e
        .expand("image", args.as_arr().unwrap(), &[("a", 8), ("c", 1)])
        .is_err());
    for image in ["81", "02", "00", "ff"] {
        let args = json::parse(&format!(r#"["{image}","a","c"]"#)).unwrap();
        let leaves = e
            .expand("image", args.as_arr().unwrap(), &[("a", 8), ("c", 1)])
            .unwrap();
        let byte = u8::from_str_radix(image, 16).unwrap();
        let bits = (0..8).filter(|i| byte >> i & 1 == 1).collect::<Vec<_>>();
        assert_eq!(
            leaves.iter().map(|l| l.wires[0].1).collect::<Vec<_>>(),
            bits
        );
        assert!(leaves
            .iter()
            .all(|l| l.gate == "x" && l.controls.is_empty()));
    }
    let d = d.replace(
        r#""body":[{"image":"I","wires":"w"}]"#,
        r#""body":[{"inverse":true,"body":[{"image":"I","wires":"w"}]}]"#,
    );
    let args = json::parse(r#"["81","a","c"]"#).unwrap();
    let leaves = Expander::new(&d, Registry::new())
        .unwrap()
        .expand("image", args.as_arr().unwrap(), &[("a", 8), ("c", 1)])
        .unwrap();
    assert_eq!(
        leaves.iter().map(|l| l.wires[0].1).collect::<Vec<_>>(),
        [7, 0]
    );
    for image in ["8A", "0", "zz"] {
        let args = json::parse(&format!(r#"["{image}","a","c"]"#)).unwrap();
        assert!(e
            .expand("image", args.as_arr().unwrap(), &[("a", 8), ("c", 1)])
            .is_err());
    }
}
#[test]
fn indexed_templates_bind_explicit_frame_and_preserve_capture() {
    let d = doc(
        r#"[{"kind":"rows","parameters":[{"name":"N","type":"integer"},{"name":"R","type":"templates","length":"N"},{"name":"frame","type":"wire","width":2}],"body":[{"for":"k","from":0,"to":"N","body":[{"apply":"R","index":"k","args":["frame"]}]}]}]"#,
    );
    let args=json::parse(r#"[2,[{"parameters":[{"name":"f","type":"wire","width":2}],"body":[{"gate":"cx","wires":["outer","f[0]"]}]},{"parameters":[{"name":"f","type":"wire","width":2}],"body":[{"gate":"cx","wires":["outer","f[1]"]}]}],"a"]"#).unwrap();
    let mut e = Expander::new(&d, Registry::new()).unwrap();
    let out = e
        .expand("rows", args.as_arr().unwrap(), &[("a", 2), ("outer", 1)])
        .unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].wires, [("outer".into(), 0), ("a".into(), 0)]);
    assert_eq!(out[1].wires, [("outer".into(), 0), ("a".into(), 1)]);
    let bad = json::parse(&args.to_string().replacen("[2,", "[1,", 1)).unwrap();
    assert!(e
        .expand("rows", bad.as_arr().unwrap(), &[("a", 2), ("outer", 1)])
        .is_err());
    let bad = d.replace(r#""index":"k""#, r#""index":2"#);
    assert!(Expander::new(&bad, Registry::new())
        .unwrap()
        .expand("rows", args.as_arr().unwrap(), &[("a", 2), ("outer", 1)])
        .is_err());
}

#[test]
fn complete_symbol_arguments_forward_heterogeneous_wires_values_and_tokens() {
    let d = doc(
        r#"[{"kind":"forward","parameters":[{"name":"S","type":"symbol"},{"name":"A","type":"arguments","symbol":"S"}],"body":[{"call":"S","args":"A"},{"call":"S","args":"A"}]}]"#,
    );
    let hash = "a".repeat(64);
    let mut registry = Registry::new();
    registry.insert(hash.clone(),json::parse(r#"{"parameters":[{"name":"a","type":"wire","width":1},{"name":"b","type":"wire","width":3},{"name":"v","type":"value","isa_type":"i128"},{"name":"f","type":"value","isa_type":"f64"},{"name":"tk","type":"value","isa_type":"token"}],"body":[{"gate":"cx","wires":["a","b[2]"]}]}"#).unwrap());
    let mut e = Expander::new(&d, registry).unwrap();
    let args = json::parse(&format!(
        r#"["{hash}",["a","b","18446744073709551617","0x7ff8000000000001",["2","0","0","0"]]]"#
    ))
    .unwrap();
    let leaves = e
        .expand("forward", args.as_arr().unwrap(), &[("a", 1), ("b", 3)])
        .unwrap();
    assert_eq!(leaves.len(), 2);
    assert_eq!(leaves[0].wires, [("a".into(), 0), ("b".into(), 2)]);
    assert_ne!(leaves[0].invocation, leaves[1].invocation);
    assert_eq!((leaves[0].ordinal, leaves[1].ordinal), (0, 0));
    for (from, to) in [
        (
            "18446744073709551617",
            "340282366920938463463374607431768211456",
        ),
        ("18446744073709551617", "01"),
        (r#"["2","0","0","0"]"#, r#"["2","0","0"]"#),
        (r#""a","b""#, r#""b","a""#),
    ] {
        let bad = json::parse(&args.to_string().replace(from, to)).unwrap();
        assert!(e
            .expand("forward", bad.as_arr().unwrap(), &[("a", 1), ("b", 3)])
            .is_err());
    }
}

#[test]
fn unused_template_shapes_and_types_are_rejected() {
    let d = doc(
        r#"[{"kind":"unused","parameters":[{"name":"T","type":"templates","length":1}],"body":[]}]"#,
    );
    for template in [
        r#"{"parameters":[{"name":"x","type":"private"}],"body":[]}"#,
        r#"{"parameters":["n","n"],"body":[]}"#,
        r#"{"parameters":[],"body":[{"private":"gate"}]}"#,
    ] {
        let args = json::parse(&format!("[[{template}]]")).unwrap();
        assert!(Expander::new(&d, Registry::new())
            .unwrap()
            .expand("unused", args.as_arr().unwrap(), &[])
            .is_err());
    }
}

#[test]
fn template_capture_is_not_rebound_by_legacy_integer_formals_or_cross_type_names() {
    let d = doc(
        r#"[{"kind":"use","parameters":[{"name":"T","type":"templates","length":1},{"name":"b","type":"wire","width":1},"n"],"body":[{"apply":"T","index":0,"args":[0]}]}]"#,
    );
    let args = json::parse(r#"[[{"parameters":["k"],"body":[{"gate":"x","wires":["b"]}]}],"a",1]"#)
        .unwrap();
    let leaves = Expander::new(&d, Registry::new())
        .unwrap()
        .expand("use", args.as_arr().unwrap(), &[("a", 1), ("b", 1)])
        .unwrap();
    assert_eq!(leaves[0].wires, [("b".into(), 0)]);
    let d = doc(
        r#"[{"kind":"use","parameters":["a",{"name":"T","type":"templates","length":1},{"name":"b","type":"wire","width":1}],"body":[{"apply":"T","index":0,"args":["b"]}]}]"#,
    );
    // Capture integer a=1, then shadow it with a wire formal named a.
    // The template must not continue resolving a as an integer.
    let args=json::parse(r#"[1,[{"parameters":[{"name":"a","type":"wire","width":1}],"body":[{"for":"i","from":0,"to":"a","body":[]}]}],"b"]"#).unwrap();
    // The actual captures at the external caller, so construct the integer
    // capture using an outer recipe's literal template actual.
    let mut parsed = json::parse(&d).unwrap();
    if let Json::Obj(m) = &mut parsed {
        if let Some(Json::Arr(rs)) = m.get_mut("recipes") {
            rs.push(json::parse(r#"{"kind":"wrapper","parameters":["a",{"name":"b","type":"wire","width":1}],"body":[{"invoke":"use","args":["a",[{"parameters":[{"name":"a","type":"wire","width":1}],"body":[{"for":"i","from":0,"to":"a","body":[]}]}],"b"]}],"scratch":[],"removable":[],"source":"K22 lexical shadow witness"}"#).unwrap());
        }
    }
    assert!(Expander::new(&parsed.to_string(), Registry::new())
        .unwrap()
        .expand(
            "wrapper",
            &[args.as_arr().unwrap()[0].clone(), Json::Str("b".into())],
            &[("b", 1)]
        )
        .is_err());
}
