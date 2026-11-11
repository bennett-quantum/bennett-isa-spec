mod support;
use bennett_isa_ref::{json::Json, Object};
use std::collections::BTreeMap;

fn fields(value: &mut Json) -> &mut BTreeMap<String, Json> {
    let Json::Object(fields) = value else {
        panic!("object")
    };
    fields
}

fn source_with_claim(union: bool, sampled: bool, mutate: impl FnOnce(&mut Json)) -> String {
    // Published §12.2 K89 witness: c starts clean; CX(c,b) preserves dirty b.
    let signature = r#"{"abi":"0.9","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let object = support::rehash(&support::object(&[("a", signature,
        "perm_action @a (t:wire mut) () () () ()\n^entry():\nc = qalloc any 1\nb = qborrow any 1\ncx c b\nqreturn b\nqfree c")], "{}", "").replace("=0.8", "=0.9"));
    let identity = object.identity().unwrap();
    let closure = identity.closure_hashes["a"].as_ref().unwrap();
    let occurrence = |point| {
        support::json(&format!(
            r#"{{"context_key":[],"symbol":"{}","point":"entry:{point}","iteration":[],"instance":[]}}"#,
            identity.interface_hashes["a"]
        ))
    };
    let mut product = support::json(
        r#"{"bundles":[{"name":"t","width":1,"lo":0,"hi":1}],"borrowed":[{"name":"b","width":1,"lo":0,"hi":1}],"regions":[],"workspace":[{"allocation":[],"offset":0,"width":1,"lo":0,"hi":0}]}"#,
    );
    let Json::Array(workspace) = fields(&mut product).get_mut("workspace").unwrap() else {
        unreachable!()
    };
    fields(&mut workspace[0]).insert(
        "allocation".into(),
        Json::Object(BTreeMap::from([
            ("source".into(), occurrence(0)),
            ("allocation_ordinal".into(), support::json("0")),
        ])),
    );
    let domain = if union {
        Json::Object(BTreeMap::from([(
            "entry_products".into(),
            Json::Array(vec![product]),
        )]))
    } else {
        product
    };
    let mut claim = support::json(&format!(
        r#"{{"kind":"exhaustive","domain":{},"expected":"borrow_frame","borrow_frame":{{"lifetime":0,"first":{},"last":{}}},"subject":{{"closure_hash":"{closure}","variant":"forward","inherited_controls":0}},"invocation":{{"args":[],"budget":["0","0","0","0"]}}}}"#,
        domain.canonical(),
        occurrence(2).canonical(),
        occurrence(2).canonical()
    ));
    if sampled {
        fields(&mut claim).insert("kind".into(), Json::String("sampled".into()));
        fields(&mut claim).insert(
            "sampler".into(),
            support::json(r#"{"algorithm":"xoshiro256**","seed":["1","0","0","0"],"count":1}"#),
        );
    }
    mutate(&mut claim);
    let template = Object::parse(include_str!(
        "../../conformance/fixtures/accepted-k84-09/baseline.bisa"
    ))
    .unwrap();
    let mut certificate = template.section("certificate").unwrap().clone();
    let bindings = fields(fields(&mut certificate).get_mut("bindings").unwrap());
    for (key, value) in [
        ("object_hash", &identity.object_hash),
        ("interface_hash", &identity.interface_hashes["a"]),
        ("closure_hash", closure),
        ("reference_program_hash", closure),
    ] {
        bindings.insert(key.into(), Json::String(value.clone()));
    }
    let regions = fields(fields(&mut certificate).get_mut("regions").unwrap());
    let mut region = regions.values().next().unwrap().clone();
    fields(&mut region).insert("claims".into(), Json::Array(vec![claim]));
    let verification = fields(fields(&mut region).get_mut("verification").unwrap());
    verification.insert("borrow_frames".into(), support::json("[0]"));
    verification.insert("unitary".into(), support::json("[]"));
    regions.clear();
    regions.insert(closure.clone(), region);
    fields(fields(&mut certificate).get_mut("composition").unwrap())
        .insert("multiplicities".into(), support::json("{}"));
    format!(
        "{}\nsection certificate\n{}\n",
        object.text,
        certificate.canonical()
    )
}

#[test]
fn k89_reader_admits_product_and_union_frame_metadata() {
    for union in [false, true] {
        for sampled in [false, true] {
            let source = source_with_claim(union, sampled, |_| {});
            Object::parse(&source)
                .unwrap_or_else(|error| panic!("union={union} sampled={sampled}: {error}"));
        }
    }
}

#[test]
fn k89_workspace_ranges_are_nonempty_disjoint_and_fit_their_width() {
    for union in [false, true] {
        for (key, value) in [("width", "0"), ("hi", "2"), ("lo", "1")] {
            let source = source_with_claim(union, false, |claim| {
                let domain = fields(claim).get_mut("domain").unwrap();
                let domain = if union {
                    let Json::Array(products) = fields(domain).get_mut("entry_products").unwrap()
                    else {
                        unreachable!()
                    };
                    &mut products[0]
                } else {
                    domain
                };
                let Json::Array(workspace) = fields(domain).get_mut("workspace").unwrap() else {
                    unreachable!()
                };
                fields(&mut workspace[0]).insert(key.into(), support::json(value));
            });
            assert!(
                Object::parse(&source).is_err(),
                "union={union}: accepted {key}={value}"
            );
        }
        let source = source_with_claim(union, false, |claim| {
            let domain = fields(claim).get_mut("domain").unwrap();
            let domain = if union {
                let Json::Array(products) = fields(domain).get_mut("entry_products").unwrap()
                else {
                    unreachable!()
                };
                &mut products[0]
            } else {
                domain
            };
            let Json::Array(workspace) = fields(domain).get_mut("workspace").unwrap() else {
                unreachable!()
            };
            workspace.push(workspace[0].clone());
        });
        assert!(
            Object::parse(&source).is_err(),
            "overlapping workspace components admitted"
        );
    }
}
