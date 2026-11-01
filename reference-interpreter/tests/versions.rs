mod support;
use bennett_isa_ref::{Budget, Object, OracleInput};
use support::{object, rehash};
const INC: &str = include_str!("fixtures/inc8.bisa");

#[test]
fn isa09_admission_identity_and_execution_are_versioned() {
    let old = Object::parse(INC).unwrap();
    old.check_hashes().unwrap();
    let new = rehash(&INC.replace("0.8", "0.9"));
    new.check_hashes().unwrap();
    let a = old.identity().unwrap();
    let b = new.identity().unwrap();
    assert_ne!(a.interface_hashes, b.interface_hashes);
    assert_ne!(a.closure_hashes, b.closure_hashes);
    assert_ne!(a.object_hash, b.object_hash);
    assert!(b.envelopes["inc8"].starts_with("isa=0.9\n"));
    assert_eq!(
        b.interface_hashes["inc8"],
        "ed9865300b85a519c2379de98e36e635e72b5b78d8bdfca03dccf9a09aa3df5e"
    );
    for o in [&old, &new] {
        let result = o
            .run_oracle(
                "inc8",
                &OracleInput::bits(&[8]),
                Budget {
                    steps: 3,
                    heap: 0,
                    stack: 0,
                    depth: 0,
                },
            )
            .unwrap();
        assert_eq!((result.code, result.payload, result.steps), (0, 9, 3));
    }
    let linked = Object::link(&[new]).unwrap();
    assert!(linked.text.starts_with("bennett-object isa=0.9 object=0.9"));
    linked.check_hashes().unwrap();
    assert!(Object::link(&[old, linked])
        .unwrap_err()
        .to_string()
        .contains("version"));
}

#[test]
fn mixed_unknown_headers_signatures_and_imports_reject() {
    for v in ["0.8", "0.9"] {
        let source = INC.replace("0.8", v);
        for key in [
            "isa", "object", "launch", "sched", "cert", "coverage", "target",
        ] {
            for other in [
                if v == "0.8" { "0.9" } else { "0.8" },
                "1.0",
                "0.10",
                "00.9",
            ] {
                let bad = source.replacen(&format!("{key}={v}"), &format!("{key}={other}"), 1);
                assert!(Object::parse(&bad).unwrap_err().to_string().contains("§1"));
            }
        }
        for abi in [if v == "0.8" { "0.9" } else { "0.8" }, "1.0"] {
            let bad = source.replace(&format!("\"abi\":\"{v}\""), &format!("\"abi\":\"{abi}\""));
            assert!(Object::parse(&bad).is_err());
            let bad = object(&[("entry",r#"{"abi":"0.8","kind":"r","params":[],"results":[],"tier":"r"}"#,"fn r @entry () () () () ()\n^entry():\nret")], &format!(r#"{{"external":{{"signature":{{"abi":"{abi}","kind":"data","size":0,"tier":"-"}}}}}}"#), "").replace("0.8",v);
            // Replace only the import ABI after the source-version conversion.
            let bad = bad.replace(
                &format!("\"external\":{{\"signature\":{{\"abi\":\"{v}\""),
                &format!("\"external\":{{\"signature\":{{\"abi\":\"{abi}\""),
            );
            assert!(Object::parse(&bad).is_err(), "{bad}");
        }
    }
    assert!(Object::parse(&INC.replace("0.8", "1.0")).is_err());
}

#[test]
fn isa09_lproc_signature_has_mandatory_forwarding_arrays() {
    let sig = r#"{"abi":"0.9","args":[],"kind":"lproc","mem":[],"params":[8],"result":8,"tier":"q","tokens":[]}"#;
    let source = object(&[("id",sig,"lproc @id (x:reg[8]) (y:reg[8]) () () ()\n^entry():\ny = lift.mov x\nlift.ret y")],"{}","\nsection layout\n{\"A_tab\":0,\"H\":8,\"S_cells\":4,\"globals\":{},\"heap_base\":16,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n").replace("=0.8","=0.9");
    rehash(&source).check_hashes().unwrap();
    assert!(Object::parse(
        &source
            .replace("\"tokens\":[],", "")
            .replace(",\"tokens\":[]", "")
    )
    .is_err());
    assert!(Object::parse(&source.replace("=0.9", "=0.8").replace("\"0.9\"", "\"0.8\"")).is_err());
}

#[test]
fn default_expansion_and_certificate_versions_follow_object() {
    let raw = include_str!("support/r16-default-expansion.bisa");
    for v in ["0.8", "0.9"] {
        let source = raw.replace("0.8", v);
        let source = if v == "0.9" {
            source.replace(
                "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93",
                "ed9865300b85a519c2379de98e36e635e72b5b78d8bdfca03dccf9a09aa3df5e",
            )
        } else {
            source
        };
        let o = rehash(&source);
        assert_eq!(o.schedule_expansion().unwrap().unwrap().version, v);
        let mut metadata = o.metadata.clone();
        let versions = [
            "isa", "object", "launch", "sched", "cert", "coverage", "target",
        ]
        .map(|k| format!("\"{k}\":\"{v}\""))
        .join(",");
        // Borrow a complete certificate shape; version validation precedes its
        // identity checks, which intentionally remain bound to the old object.
        let cert = Object::parse(include_str!("fixtures/r21/domain-all-A0.bisa"))
            .unwrap()
            .section("certificate")
            .unwrap()
            .clone();
        let mut cert = match cert {
            bennett_isa_ref::json::Json::Object(m) => m,
            _ => unreachable!(),
        };
        cert.insert("versions".into(), support::json(&format!("{{{versions}}}")));
        for key in [
            "isa", "object", "launch", "sched", "cert", "coverage", "target",
        ] {
            let mut c = cert.clone();
            if let Some(bennett_isa_ref::json::Json::Object(m)) = c.get_mut("versions") {
                m.insert(
                    key.into(),
                    support::json(if v == "0.8" { "\"0.9\"" } else { "\"0.8\"" }),
                );
            }
            metadata.insert("certificate".into(), bennett_isa_ref::json::Json::Object(c));
            let text = format!(
                "{}\nsection certificate\n{}\n",
                o.text,
                metadata["certificate"].canonical()
            );
            let e = Object::parse(&text).unwrap_err();
            assert!(e.to_string().contains("certificate version"), "{e}");
        }
    }
}

#[test]
fn device_model_versions_follow_object_but_service_versions_are_independent() {
    let raw = include_str!("fixtures/classical.bisa");
    for v in ["0.8", "0.9"] {
        let source = raw.replace("0.8", v);
        let parsed = rehash(&source);
        parsed.check_hashes().unwrap();
        for other in [if v == "0.8" { "0.9" } else { "0.8" }, "1.0"] {
            let bad = source.replace(
                &format!("\"version\":\"{v}\""),
                &format!("\"version\":\"{other}\""),
            );
            assert!(Object::parse(&bad).is_err());
        }
    }
}
