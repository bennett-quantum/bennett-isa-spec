mod support;
use bennett_isa_ref::{sha256_hex, Budget, Object, OracleInput};
use support::{object, rehash};
const SIG: &str =
    r#"{"abi":"0.8","args":[],"fault":4,"in":[8],"kind":"oracle","mem":[],"tgt":8,"tier":"q"}"#;
const PROC: &str = r#"{"abi":"0.8","kind":"lproc","params":[8],"result":8,"tier":"q"}"#;
const CC: &str = r#"{"abi":"0.8","kind":"r","params":["i8"],"results":["i8"],"tier":"r"}"#;
const LAYOUT:&str="\nsection layout\n{\"A_tab\":0,\"H\":8,\"S_cells\":4,\"globals\":{},\"heap_base\":16,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n";
fn library() -> Object {
    rehash(&object(&[("inc",PROC,"lproc @inc (x:reg[8]) (r:reg[8]) () () ()\nregs { one:reg[8] }\n^entry():\none = lift.const.i8 1\nr = lift.add x one\nlift.ret r")],"{}",LAYOUT))
}
fn client(pin: Option<&str>) -> String {
    let pin = pin
        .map(|h| format!(",\"pinned_closure_hash\":\"{h}\""))
        .unwrap_or_default();
    let imports = format!("{{\"inc\":{{\"signature\":{PROC}{pin}}}}}");
    object(&[("main",SIG,"oracle @main (x:wire[8] in) (t:wire[8]) (f:wire[4]) () ()\nregs { r:reg[8] }\n^entry():\nr = lift.call @inc (x)\nlift.retval r")],&imports,LAYOUT)
}
#[test]
fn unresolved_lproc_has_fingerprint_and_no_executable_closure() {
    let o = rehash(&client(None));
    let identity = o.identity().unwrap();
    let mut bytes = b"inc\0".to_vec();
    bytes.extend(PROC.as_bytes());
    let fingerprint = sha256_hex(&bytes);
    assert!(identity.envelopes["main"].contains(&format!("lift.call @{fingerprint} (x)")));
    assert_eq!(identity.closure_hashes["main"], None);
    assert!(o
        .run_oracle(
            "main",
            &OracleInput::bits(&[2]),
            Budget {
                steps: 20,
                heap: 0,
                stack: 16,
                depth: 8
            }
        )
        .is_err());
}
#[test]
fn link_resolves_calls_and_pins_and_deduplicates() {
    let lib = library();
    let pin = lib.identity().unwrap().closure_hashes["inc"]
        .clone()
        .unwrap();
    let c = rehash(&client(Some(&pin)));
    let linked = Object::link(&[c, lib.clone(), lib]).unwrap();
    linked.check_hashes().unwrap();
    assert!(linked.section("imports").is_none());
    assert_eq!(linked.functions.len(), 2);
    let result = linked
        .run_oracle(
            "main",
            &OracleInput::bits(&[255]),
            Budget {
                steps: 6,
                heap: 0,
                stack: 16,
                depth: 8,
            },
        )
        .unwrap();
    assert_eq!((result.payload, result.code, result.steps), (0, 0, 5));
}
#[test]
fn link_rejects_unresolved_signature_pin_definition_and_layout_conflicts() {
    let lib = library();
    for c in [
        client(Some(&"0".repeat(64))),
        client(None).replace("\"params\":[8]", "\"params\":[16]"),
    ] {
        let error = match Object::parse(&c) {
            Ok(c) => Object::link(&[rehash(&c.text), lib.clone()]).unwrap_err(),
            Err(e) => e,
        };
        assert!(
            error.to_string().contains("§9.3") || error.to_string().contains("§3.3"),
            "{error}"
        );
    }
    assert!(Object::link(&[rehash(&client(None))]).is_err());
    let changed = rehash(&lib.text.replace("lift.const.i8 1", "lift.const.i8 2"));
    assert!(Object::link(&[lib.clone(), changed]).is_err());
    let changed = rehash(&lib.text.replace("\"null_size\":1", "\"null_size\":2"));
    assert!(Object::link(&[lib, changed]).is_err());
}
#[test]
fn classical_and_region_import_signatures_are_validated() {
    let imports = format!("{{\"f\":{{\"signature\":{CC}}}}}");
    let raw = object(
        &[(
            "caller",
            CC,
            "fn r @caller (x:i8) (r:i8) () () ()\n^entry():\nr = call @f x\nret r",
        )],
        &imports,
        "",
    );
    assert!(Object::parse(&raw).is_ok());
    let action_sig = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[8],"tier":"q"}"#;
    let imports = format!("{{\"f\":{{\"signature\":{action_sig}}}}}");
    let raw=object(&[("caller",action_sig,"perm_action @caller (x:wire[8] mut) () () () ()\n^entry():\naction_call @f (x) () () ()")],&imports,"");
    assert!(Object::parse(&raw).is_ok());
    let malformed = raw.replace("\"mut\":[8]", "\"mut\":[0]");
    assert!(Object::parse(&malformed).is_err());
}

#[test]
fn imported_quantum_region_image_resolves_by_data_signature() {
    let signature = r#"{"abi":"0.8","args":[],"bounds":[],"in":[],"kind":"perm_action","mem":[],"mut":[1],"tier":"q"}"#;
    let imports = r#"{"image":{"signature":{"abi":"0.8","kind":"data","size":1,"tier":"-"}}}"#;
    let caller = rehash(&object(
        &[(
            "a",
            signature,
            "perm_action @a (t:wire mut) () () () ()\n^entry():\nqregion r 0 8 @image",
        )],
        imports,
        "",
    ));
    let text=format!("bennett-object isa=0.8 object=0.8 launch=0.8 sched=0.8 cert=0.8 coverage=0.8 target=0.8 ptr_width=32\n\nsection symbols\n{{\"0\":{{\"interface_hash\":\"{}\",\"kind\":\"data\",\"name\":\"image\",\"signature\":{{\"abi\":\"0.8\",\"kind\":\"data\",\"size\":1,\"tier\":\"-\"}}}}}}\n\nsection data\nimage ff\n","0".repeat(64));
    let linked = Object::link(&[caller, rehash(&text)]).unwrap();
    linked.check_hashes().unwrap();
    let mut q = bennett_isa_ref::QuantumState::new();
    let t = q.allocate(1).unwrap();
    let mut binding = bennett_isa_ref::QuantumBindings::default();
    binding.bundles.insert("t".into(), t.clone());
    linked
        .execute_action("a", &mut q, &binding, false, &[])
        .unwrap();
    assert_eq!(q.read_basis(&t).unwrap(), 0);
}
