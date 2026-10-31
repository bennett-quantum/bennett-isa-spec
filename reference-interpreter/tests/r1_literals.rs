//! Council round 1: fable R1-B2, codex B12, muse B2 (§16.1 INT / FLOAT, §3.3, §4.8, §9.1).
mod support;
use bennett_isa_ref::{json::Json, Object};

const INC: &str = include_str!("fixtures/inc8.bisa");

fn rejects(text: &str, rule: &str) {
    match Object::parse(text) {
        Err(e) => assert!(e.to_string().contains(rule), "{e} must cite {rule}"),
        Ok(_) => panic!("ill-formed object accepted:\n{text}"),
    }
}
fn action(body: &str, bounds: &str) -> String {
    let sig_bounds = bounds.trim_start_matches(['0', '+']);
    let sig_bounds = if bounds.is_empty() {
        ""
    } else if sig_bounds.is_empty() {
        "0"
    } else {
        sig_bounds
    };
    let signature = format!(
        r#"{{"abi":"0.8","args":[],"bounds":[{sig_bounds}],"in":[1],"kind":"action","mem":[],"mut":[1],"tier":"q"}}"#
    );
    let regs = if bounds.is_empty() {
        String::new()
    } else {
        format!("regs {{  }} bounds {{{bounds}}}\n")
    };
    support::object(
        &[(
            "a",
            &signature,
            &format!("action @a (t:wire mut) (p:wire in) () () ()\n{regs}^entry():\n{body}"),
        )],
        "{}",
        "",
    )
}
fn cc(body: &str, results: &str) -> String {
    let signature =
        format!(r#"{{"abi":"0.8","kind":"cc","params":[],"results":[{results}],"tier":"cc"}}"#);
    let ret = results.trim_matches('"');
    support::object(
        &[(
            "f",
            &signature,
            &format!(
                "fn cc @f () ({}) () () ()\n^entry():\n{body}",
                if ret.is_empty() {
                    String::new()
                } else {
                    format!("r:{ret}")
                }
            ),
        )],
        "{}",
        "",
    )
}

#[test]
fn header_ptr_width_is_a_canonical_int() {
    for bad in ["ptr_width=032", "ptr_width=+32", "ptr_width=32.0"] {
        rejects(&INC.replacen("ptr_width=32", bad, 1), "§16.1");
    }
}
#[test]
fn fault_codes_are_canonical_ints() {
    for bad in ["fault 09", "fault 01", "fault +01", "fault -0"] {
        rejects(&INC.replace("lift.retval r", bad), "§16");
    }
    support::rehash(&INC.replace("lift.retval r", "fault 9"));
}
#[test]
fn type_widths_and_bounds_are_canonical_ints() {
    rejects(&INC.replace("x:wire[8] in", "x:wire[08] in"), "§16.1");
    assert!(bennett_isa_ref::Type::parse("qreg[08]")
        .unwrap_err()
        .to_string()
        .contains("§16.1"));
    rejects(&action("qloop 3 {\nx t\n}", "03"), "§16");
    rejects(&action("qloop 3 {\nx t\n}", "+3"), "§16");
    Object::parse(&action("qloop 3 {\nx t\n}", "3")).unwrap();
}
#[test]
fn action_literals_are_canonical_ints() {
    rejects(&action("qloop 03 {\nx t\n}", "3"), "§16.1");
    rejects(&action("d = qalloc any 01\nqfree d", ""), "§16.1");
    rejects(&action("d = qalloc any +1\nqfree d", ""), "§16.1");
    rejects(&action("qregion r 01 2\nx t", ""), "§16.1");
    Object::parse(&action("d = qalloc any 1\nqfree d", "")).unwrap();
}
#[test]
fn gate_angles_are_shortest_round_trip_floats() {
    for bad in ["0.70", "1.0e0", "00.7000", "+0.7", "7e-1", "0.7000"] {
        rejects(&action(&format!("rx {bad} t"), ""), "§16.1");
    }
    for good in ["0.7", "1.0", "-0.7", "1e+21", "1e-7"] {
        Object::parse(&action(&format!("rx {good} t\nrx {} t", neg(good)), ""))
            .unwrap_or_else(|e| panic!("{good}: {e}"));
    }
}
fn neg(s: &str) -> String {
    s.strip_prefix('-').map_or(format!("-{s}"), str::to_string)
}
#[test]
fn classical_const_range_and_qreg_literals() {
    rejects(&cc("r = const.i8 256\nret r", "\"i8\""), "§3.3");
    rejects(&cc("r = const.i8 -129\nret r", "\"i8\""), "§3.3");
    rejects(&cc("r = const.i8 01\nret r", "\"i8\""), "§16.1");
    support::rehash(&cc("r = const.i8 -128\nret r", "\"i8\""));
    support::rehash(&cc("r = const.i8 255\nret r", "\"i8\""));
    rejects(&cc("q = qreg.new any 01\nqreg.free q\nret", ""), "§16.1");
    rejects(
        &cc(
            "q = qreg.new any 2\ns = qreg.slice q 01 1\nqreg.free q\nret",
            "",
        ),
        "§16.1",
    );
}
#[test]
fn opcode_suffixes_follow_the_schema() {
    // §4.8: load.iN / store.iN; there is no load.ptr (codex B12).
    let text = cc("r = const.i8 1\nret r", "\"i8\"");
    let layout = "\nsection layout\n{\"A_tab\":2,\"H\":8,\"S_cells\":2,\"globals\":{},\"heap_base\":16,\"heap_limit_max\":64,\"interface_regions\":{},\"null_size\":16,\"ptr_width\":32}\n";
    let with = |body: &str, results: &str| {
        let mut t = cc(body, results);
        t.push_str(layout);
        t
    };
    let _ = text;
    rejects(
        &with("n = const.i32 8\np = alloc n\nq = load.ptr p\nret", ""),
        "§4.8",
    );
    rejects(
        &with("n = const.i32 8\np = alloc n\nr = load.i12 p\nret", ""),
        "§3.3",
    );
    support::rehash(&with(
        "n = const.i32 8\np = alloc n\nr = load.i16 p\nret",
        "",
    ));
}
#[test]
fn metadata_integers_above_2_53_minus_1_must_be_strings() {
    assert!(Json::parse("9007199254740992").unwrap().uint().is_err());
    assert!(Json::parse("9007199254740991").unwrap().uint().is_ok());
    assert!(Json::parse("\"9007199254740992\"").unwrap().uint().is_ok());
    // SPEC-QUESTIONS 17: a value at most 2^53 - 1 is a JSON number, never a string.
    assert!(Json::parse("\"16\"").unwrap().uint().is_err());
    let layout_number = INC.to_string()
        + "\nsection layout\n{\"A_tab\":0,\"H\":8,\"S_cells\":0,\"globals\":{},\"heap_base\":16,\"heap_limit_max\":9007199254740992,\"interface_regions\":{},\"null_size\":16,\"ptr_width\":32}\n";
    rejects(&layout_number, "§9.1");
}

#[test]
fn classical_call_has_one_spelling() {
    // SPEC-QUESTIONS 18: `r = call @f x` as printed in §4.1; `call @f (x)` is rejected.
    let sig = r#"{"abi":"0.8","kind":"r","params":["i8"],"results":["i8"],"tier":"r"}"#;
    let f = "fn r @f (x:i8) (r:i8) () () ()\n^entry():\nret x";
    let caller = |call: &str| {
        support::object(
            &[
                ("f", sig, f),
                (
                    "g",
                    sig,
                    &format!("fn r @g (x:i8) (r:i8) () () ()\n^entry():\n{call}\nret r"),
                ),
            ],
            "{}",
            "",
        )
    };
    support::rehash(&caller("r = call @f x"));
    rejects(&caller("r = call @f (x)"), "§16.2");
}

#[test]
fn r20_pinned_extreme_const_widths_parse_and_hash_without_panic() {
    for (source, pin, file_pin) in support::r20::CASES {
        assert_eq!(bennett_isa_ref::sha256_hex(source.as_bytes()), file_pin);
        let o = std::panic::catch_unwind(|| Object::parse(source))
            .expect("§3.3: admission must never panic")
            .unwrap();
        o.check_hashes().unwrap();
        let id = o.identity().unwrap();
        assert_eq!(id.interface_hashes["o"], pin);
        assert_eq!(id.closure_hashes["o"].as_deref(), Some(pin));
    }
}

#[test]
fn r20_extreme_const_execution_reports_host_resource() {
    use bennett_isa_ref::{BitVector, Budget, OracleVectorInput, OracleVectorMachine};
    for (source, _, _) in support::r20::CASES {
        let o = Object::parse(source).unwrap();
        let input = OracleVectorInput {
            ins: vec![BitVector::from_u128(8, 8).unwrap()],
            ..Default::default()
        };
        let sequential =
            std::panic::catch_unwind(|| o.run_oracle_vectors("o", &input, Budget::steps(1)))
                .expect("no execution panic")
                .unwrap_err();
        let machine = std::panic::catch_unwind(|| {
            OracleVectorMachine::new(&o, "o", &input, Budget::steps(1))
        })
        .expect("no machine construction panic")
        .err()
        .expect("host limit");
        for e in [sequential, machine] {
            let s = e.to_string();
            assert!(
                s.contains('§') && s.contains("host limit") && !s.contains("outside"),
                "{s}"
            );
        }
    }
}
