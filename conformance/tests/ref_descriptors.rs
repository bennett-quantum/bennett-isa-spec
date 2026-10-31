use bennett_isa_conformance::{
    json::{self, Json},
    ref_language::{Expander, Registry},
};
fn doc(parameters: &str, body: &str) -> String {
    format!(
        r#"{{"isa":"0.9","name":"ref","version":"0.9","recipes":[{{"kind":"test","parameters":{parameters},"body":{body},"scratch":[],"removable":[],"source":"§11.1 descriptor witness"}}]}}"#
    )
}
fn expand(
    d: &str,
    args: &str,
    bundles: &[(&str, usize)],
) -> Result<Vec<bennett_isa_conformance::ref_language::Leaf>, String> {
    Expander::new(d, Registry::new())?.expand("test", json::parse(args)?.as_arr().unwrap(), bundles)
}
#[test]
fn ordered_records_bind_heterogeneous_fields_lexically_and_reverse() {
    let p = r#"[{"name":"D","type":"records","fields":["base","extent",{"name":"cells","type":"wire","width":["*","extent",8]}],"order":["base"]}]"#;
    let b = r#"[{"visit":"D","body":[{"gate":"x","wires":["cells[base]"]}]}]"#;
    let args = r#"[[{"base":1,"extent":1,"cells":"a"},{"base":3,"extent":2,"cells":"b"}]]"#;
    let out = expand(&doc(p, b), args, &[("a", 8), ("b", 16)]).unwrap();
    assert_eq!(
        out.iter().map(|l| l.wires[0].clone()).collect::<Vec<_>>(),
        [("a".into(), 1), ("b".into(), 3)]
    );
    let rev = format!(r#"[{{"inverse":true,"body":{b}}}]"#);
    let out = expand(&doc(p, &rev), args, &[("a", 8), ("b", 16)]).unwrap();
    assert_eq!(out[0].wires, [("b".into(), 3)]);
    for bad in [
        args.replace("\"base\":3", "\"base\":0"),
        args.replace("\"extent\":2", "\"extent\":1"),
        args.replace("\"base\":3", "\"extra\":3"),
    ] {
        assert!(expand(&doc(p, b), &bad, &[("a", 8), ("b", 16)]).is_err());
    }
}
#[test]
fn regions_preserve_shape_role_origin_and_forward_cells() {
    let p = r#"[{"name":"R","type":"region","address":1,"cell":3,"role":"in"}]"#;
    let b = r#"[{"gate":"cx","wires":["R[0]","R[5]"]}]"#;
    let args = r#"[{"address":1,"cell":3,"role":"in","origin":"local:r","cells":"a"}]"#;
    assert_eq!(
        expand(&doc(p, b), args, &[("a", 6)]).unwrap()[0].wires,
        [("a".into(), 0), ("a".into(), 5)]
    );
    for (a, b) in [
        ("\"address\":1", "\"address\":0"),
        ("\"cell\":3", "\"cell\":6"),
        ("\"role\":\"in\"", "\"role\":\"mut\""),
        ("local:r", ""),
        ("local:r", "local:1r"),
    ] {
        assert!(expand(&doc(p, "[]"), &args.replace(a, b), &[("a", 6)]).is_err());
    }
}
#[test]
fn declared_outputs_share_lifetime_identity_across_invokes_and_captures() {
    let mut d=json::parse(&doc(r#"[{"name":"C","type":"class"}]"#,r#"[{"declare":"out","width":2,"class":"C","state":"clean","body":[{"invoke":"use","args":["out"]},{"gate":"x","wires":["out[1]"]},{"invoke":"use","args":["out"]}]}]"#)).unwrap();
    if let Json::Obj(m) = &mut d {
        if let Some(Json::Arr(rs)) = m.get_mut("recipes") {
            rs.push(json::parse(r#"{"kind":"use","parameters":[{"name":"D","type":"allocation"}],"body":[{"acquire":"D"},{"gate":"x","wires":["D[1]"]},{"relinquish":"D"}],"scratch":[],"removable":[],"source":"lifetime witness"}"#).unwrap());
        }
    }
    // First use releases the output: using it afterwards must reject.
    assert!(expand(&d.to_string(), r#"[{"class":"ancilla"}]"#, &[]).is_err());
    if let Json::Obj(m) = &mut d {
        if let Some(Json::Arr(rs)) = m.get_mut("recipes") {
            if let Json::Obj(r) = &mut rs[0] {
                r.insert("body".into(),json::parse(r#"[{"declare":"out","width":2,"class":"C","state":"clean","body":[{"invoke":"use","args":["out"]},{"gate":"x","wires":["out[1]"]},{"relinquish":"out"}]}]"#).unwrap());
            }
            if let Json::Obj(r) = &mut rs[1] {
                r.insert(
                    "body".into(),
                    json::parse(r#"[{"acquire":"D"},{"gate":"x","wires":["D[1]"]}]"#).unwrap(),
                );
            }
        }
    }
    let d = d.to_string();
    let out = expand(&d, r#"[{"class":"ancilla"}]"#, &[]).unwrap();
    assert_eq!(out.len(), 4);
    assert_eq!(out[0].allocation.as_ref().unwrap().2, "ancilla");
    assert_eq!(out[0].allocation, out[3].allocation);
    assert_eq!(out[1].wires, out[2].wires);
}

#[test]
fn region_vectors_are_closure_bound_and_shape_checked_even_without_call() {
    let hash = "b".repeat(64);
    let d = doc(
        r#"[{"name":"S","type":"symbol"},{"name":"A","type":"arguments","symbol":"S"}]"#,
        r#"[{"call":"S","args":"A"},{"call":"S","args":"A"}]"#,
    );
    let mut registry = Registry::new();
    registry.insert(hash.clone(),json::parse(r#"{"parameters":[{"name":"R","type":"region","address":1,"cell":3,"role":"in"}],"body":[{"gate":"cx","wires":["R[0]","R[5]"]}]}"#).unwrap());
    let mut e = Expander::new(&d, registry).unwrap();
    let args = json::parse(&format!(
        r#"["{hash}",[{{"address":1,"cell":3,"role":"in","origin":"interface:db","cells":"a"}}]]"#
    ))
    .unwrap();
    let out = e
        .expand("test", args.as_arr().unwrap(), &[("a", 6)])
        .unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out[0].wires, out[1].wires);
    assert_ne!(out[0].invocation, out[1].invocation);
    let bad = json::parse(
        &args
            .to_string()
            .replace("\"address\":1", "\"address\":0")
            .replace("\"cell\":3", "\"cell\":6"),
    )
    .unwrap();
    assert!(e
        .expand("test", bad.as_arr().unwrap(), &[("a", 6)])
        .is_err());
}
#[test]
fn declaration_capture_and_adjoint_keep_class_state_and_lifetimes() {
    let p = r#"[{"name":"C","type":"class"}]"#;
    let b = r#"[{"declare":"out","width":1,"class":"C","state":"borrowed","body":[{"acquire":"out"},{"gate":"x","wires":["out"]},{"gate":"x","wires":["out"]},{"relinquish":"out"},{"acquire":"out"},{"relinquish":"out"}]}]"#;
    for b in [b.to_string(), format!(r#"[{{"inverse":true,"body":{b}}}]"#)] {
        let leaves = expand(&doc(p, &b), r#"[{"class":"data"}]"#, &[]).unwrap();
        let allocations = leaves
            .iter()
            .filter_map(|l| l.allocation.as_ref())
            .collect::<Vec<_>>();
        assert_eq!(allocations[0], allocations[1]);
        assert_eq!(allocations[2], allocations[3]);
        assert_ne!(allocations[0], allocations[2]);
        assert_eq!(leaves[0].allocation.as_ref().unwrap().2, "data");
        assert_eq!(leaves[0].allocation.as_ref().unwrap().3, "borrowed");
        assert!(leaves.iter().all(|l| l.controls.is_empty()));
    }
}

#[test]
fn activation_records_bind_low_cell_slices_in_global_id_order() {
    let p = r#"[{"name":"L","type":"records","fields":["id","width","slot",{"name":"location","type":"wire","width":"width"}],"order":["id"]},{"name":"stack","type":"wire","width":12}]"#;
    let b = r#"[{"visit":"L","body":[{"for":"bit","from":0,"to":"width","body":[{"gate":"swap","wires":["location[bit]","stack[[\"+\",[\"*\",3,\"slot\"],\"bit\"]]"]}]}]}]"#;
    let args = r#"[[{"id":0,"width":2,"slot":0,"location":"a"},{"id":1,"width":1,"slot":1,"location":"b"}],"s"]"#;
    let out = expand(&doc(p, b), args, &[("a", 2), ("b", 1), ("s", 12)]).unwrap();
    assert_eq!(out.len(), 3);
    assert_eq!(
        out.iter().map(|l| l.wires[1].1).collect::<Vec<_>>(),
        [0, 1, 3]
    );
    assert!(out.iter().all(|l| l.wires[1].1 % 3 < 2));
    let bad = args.replace("\"id\":1", "\"id\":0");
    assert!(expand(&doc(p, b), &bad, &[("a", 2), ("b", 1), ("s", 12)]).is_err());
}

#[test]
fn record_actual_wires_resolve_at_capture_before_any_field_binding() {
    let p = r#"[{"name":"D","type":"records","fields":[{"name":"a","type":"wire","width":1},{"name":"b","type":"wire","width":1}],"order":[]}]"#;
    let d = doc(
        p,
        r#"[{"visit":"D","body":[{"gate":"cx","wires":["a","b"]}]}]"#,
    );
    let out = expand(
        &d,
        r#"[[{"a":"outer","b":"a"}]]"#,
        &[("a", 1), ("outer", 1)],
    )
    .unwrap();
    assert_eq!(out[0].wires, [("outer".into(), 0), ("a".into(), 0)]);
}

#[test]
fn record_widths_use_earlier_recipe_integers_and_actuals_keep_capture() {
    let p = r#"["N",{"name":"D","type":"records","fields":[{"name":"w","type":"wire","width":"N"}],"order":[]}]"#;
    let d = doc(
        p,
        r#"[{"visit":"D","body":[{"gate":"cx","wires":["w[0]","w[1]"]}]}]"#,
    );
    let out = expand(&d, r#"[2,[{"w":"a"}]]"#, &[("a", 2)]).unwrap();
    assert_eq!(out[0].wires, [("a".into(), 0), ("a".into(), 1)]);
}

#[test]
fn initializer_records_share_image_binding_and_shadow_across_types() {
    let p = r#"[{"name":"D","type":"records","fields":[{"name":"I","type":"image"},{"name":"cells","type":"wire","width":8}],"order":[]}]"#;
    let d = doc(
        p,
        r#"[{"visit":"D","body":[{"image":"I","wires":"cells"}]}]"#,
    );
    let out = expand(&d, r#"[[{"I":"81","cells":"a"}]]"#, &[("a", 8)]).unwrap();
    assert_eq!(out.iter().map(|l| l.wires[0].1).collect::<Vec<_>>(), [0, 7]);
    assert!(out.iter().all(|l| l.controls.is_empty()));
}

#[test]
fn declaration_handles_cannot_bypass_declared_library_scratch_lifo() {
    let body = r#"[{"declare":"a","width":1,"class":{"class":"any"},"state":"clean","body":[{"declare":"b","width":1,"class":{"class":"any"},"state":"clean","body":[{"acquire":"a"},{"acquire":"b"},{"relinquish":"a"},{"relinquish":"b"}]}]}]"#;
    let user = doc("[]", body);
    assert!(
        expand(&user, "[]", &[]).is_ok(),
        "caller result lifetimes may overlap"
    );
    let private = user.replace(r#""scratch":[]"#, r#""scratch":["a","b"]"#);
    assert!(
        expand(&private, "[]", &[]).is_err(),
        "declared library scratch must stay LIFO"
    );
}

#[test]
fn acquired_record_handles_retain_recipe_scratch_classification() {
    let d = doc(
        "[]",
        r#"[{"declare":"a","width":1,"class":{"class":"any"},"state":"clean","body":[{"declare":"b","width":1,"class":{"class":"any"},"state":"clean","body":[{"invoke":"use","args":[[{"storage":"a"},{"storage":"b"}]]}]}]}]"#,
    );
    let mut d = json::parse(&d).unwrap();
    if let Json::Obj(m) = &mut d {
        if let Some(Json::Arr(rs)) = m.get_mut("recipes") {
            rs.push(json::parse(r#"{"kind":"use","parameters":[{"name":"D","type":"records","fields":[{"name":"storage","type":"allocation"}],"order":[]}],"body":[{"visit":"D","body":[{"acquire":"storage"}]},{"visit":"D","body":[{"relinquish":"storage"}]}],"scratch":["storage"],"removable":[],"source":"K25/K27 private state descriptors"}"#).unwrap());
        }
    }
    assert!(
        expand(&d.to_string(), "[]", &[]).is_err(),
        "private record allocations cannot release in forward order"
    );
}
