mod support;
use bennett_isa_ref::{sha256_hex, Object};

#[test]
fn fips_180_4_sha256_vectors() {
    for (input, expected) in [
        ("", "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        ("abc", "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        ("abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq", "248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"),
        ("abcdefghbcdefghicdefghijdefghijkefghijklfghijklmghijklmnhijklmnoijklmnopjklmnopqklmnopqrlmnopqrsmnopqrstnopqrstu", "cf5b16a778af8380036ce59e7b0492370b249b11e8f07a51afac45037afee9d1"),
    ] { assert_eq!(sha256_hex(input.as_bytes()), expected); }
    assert_eq!(
        sha256_hex(&vec![b'a'; 1_000_000]),
        "cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
    );
}

#[test]
fn all_four_published_hashes_and_printed_envelopes() {
    let spec = include_str!("../../spec/Bennett-ISA-Spec.md");
    let mut hashes = std::collections::BTreeMap::new();
    let mut envelopes = Vec::new();
    for fixture in [
        include_str!("fixtures/inc8.bisa"),
        include_str!("fixtures/sum_db.bisa"),
    ] {
        let object = Object::parse(fixture).unwrap();
        let identity = object.identity().unwrap();
        object.check_hashes().unwrap();
        for (name, hash) in identity.interface_hashes {
            assert_eq!(
                identity.closure_hashes[&name].as_deref(),
                Some(hash.as_str())
            );
            hashes.insert(name, hash);
        }
        envelopes.extend(identity.envelopes.into_values());
    }
    for (name, expected) in [
        (
            "inc8",
            "8ce5e64f15830d8e596ba9b4861def6565dd567b5804300f427d8b9566d5be93",
        ),
        (
            "zero",
            "7002ed5ea010907a0ae99065663f22fb7d18ca1d6cb171fed57e2594bd9367e6",
        ),
        (
            "sum",
            "74f110bbafde68a4785025e7e571ef871ddd83a09465ab9c1604f653a0cfcd92",
        ),
        (
            "sum_db",
            "68225d213c44812fe844cd295d8821127c3cf31682be4eb19303cb53f9b147f7",
        ),
    ] {
        assert_eq!(hashes[name], expected);
    }
    let mut count = 0;
    for block in spec.split("```\n").skip(1) {
        if block.starts_with("isa=0.8\n") {
            let envelope = block.split("```").next().unwrap();
            assert!(
                envelopes.iter().any(|e| e == envelope),
                "printed envelope differs byte-for-byte"
            );
            count += 1;
        }
    }
    assert_eq!(count, 2);
}

#[test]
fn object_hash_excludes_only_the_three_nonsemantic_sections() {
    let raw = include_str!("fixtures/inc8.bisa");
    let base = Object::parse(raw).unwrap().identity().unwrap();
    let annotated = format!("{raw}\nsection coverage\n{{\"entries\":[]}}\n");
    let after = Object::parse(&annotated).unwrap().identity().unwrap();
    assert_eq!(base.object_hash, sha256_hex(raw.as_bytes()));
    assert_eq!(base.object_hash, after.object_hash);
    assert!(
        Object::parse(&raw.replace("lift.const.i8 1", "lift.const.i8 2"))
            .unwrap()
            .check_hashes()
            .is_err()
    );
}

#[test]
fn reference_order_and_self_scc_are_observable() {
    let raw = include_str!("fixtures/sum_db.bisa");
    let base = Object::parse(raw).unwrap().identity().unwrap();
    let env = &base.envelopes["sum"];
    assert!(env.contains("lift.call @$0"));
    assert_eq!(
        base.interface_hashes["sum"],
        sha256_hex(format!("{}:0", sha256_hex(env.as_bytes())).as_bytes())
    );
    let shifted = Object::parse(&raw.replace("\"base\":16,\"image\"", "\"base\":17,\"image\""))
        .unwrap()
        .identity()
        .unwrap();
    assert_ne!(
        base.interface_hashes["sum"],
        shifted.interface_hashes["sum"]
    );
    assert_ne!(
        base.interface_hashes["sum_db"],
        shifted.interface_hashes["sum_db"]
    );
    assert_eq!(
        base.interface_hashes["zero"],
        shifted.interface_hashes["zero"]
    );
}

#[test]
fn unmodelled_effect_is_hashable_and_keeps_its_fault_semantics() {
    let o = Object::parse(include_str!("fixtures/classical.bisa")).unwrap();
    let identity = o.identity().unwrap();
    assert!(identity.envelopes["g"].contains("device clock {\"version\":\"0.8\"}\n"));
    assert!(!identity.envelopes["unknown_effect"].contains("device missing"));
}

#[test]
fn mutual_scc_hashing_and_canonical_lproc_member_order() {
    use bennett_isa_ref::{Budget, OracleInput};
    let o = Object::parse(include_str!("fixtures/mutual.bisa")).unwrap();
    let id = o.identity().unwrap();
    assert!(
        id.interface_hashes["f"] > id.interface_hashes["g"],
        "fixture must expose sorting by member hashes"
    );
    let digest = sha256_hex(format!("{}{}", id.envelopes["f"], id.envelopes["g"]).as_bytes());
    assert_eq!(
        id.interface_hashes["f"],
        sha256_hex(format!("{digest}:0").as_bytes())
    );
    assert_eq!(
        id.interface_hashes["g"],
        sha256_hex(format!("{digest}:1").as_bytes())
    );
    assert!(id.envelopes["f"].contains("lift.call @$1"));
    assert!(id.envelopes["g"].contains("lift.call @$0"));
    let r = o
        .run_oracle(
            "main",
            &OracleInput::bits(&[]),
            Budget {
                steps: 20,
                heap: 0,
                stack: 32,
                depth: 8,
            },
        )
        .unwrap();
    assert_eq!((r.payload, r.code), (4, 0));
}

#[test]
fn every_committed_fixture_has_valid_identity() {
    // The complete fixture audit is tests/r1_fixtures.rs (directory enumeration).
    // Independently hand-assembled device/layout envelope, checked by Python hashlib.
    let o = Object::parse(include_str!("fixtures/classical.bisa")).unwrap();
    assert_eq!(
        o.identity().unwrap().interface_hashes["g"],
        "e8a3dc5429049b16559aa6a4262779e3dcd37a758967def4e08c371a5b765fff"
    );
}

#[test]
fn council_r4_mutual_scc_has_independent_literal_envelopes() {
    // §9.2: f < g bytewise; headers are definitions, calls are references;
    // lprocs without lift.addr globals have NO layout leaves or part-(6) lines.
    let f = concat!(
        "isa=0.8\nptr_width=16\n",
        "{\"abi\":\"0.8\",\"kind\":\"lproc\",\"params\":[],\"result\":16,\"tier\":\"q\"}\n",
        "lproc @f () (r:reg[16]) () () ()\n^entry():\nr = lift.call @$1 ()\nlift.ret r\n"
    );
    let g = concat!(
        "isa=0.8\nptr_width=16\n",
        "{\"abi\":\"0.8\",\"kind\":\"lproc\",\"params\":[],\"result\":16,\"tier\":\"q\"}\n",
        "lproc @g () (r:reg[16]) () () ()\nregs { p:reg[1] }\n^entry():\n",
        "p = lift.const.i1 1\nqbranch p ^done ^recurse\n^done():\nr = lift.label ^entry\nlift.ret r\n",
        "^recurse():\nr = lift.call @$0 ()\nlift.ret r\n"
    );
    let digest = sha256_hex(format!("{f}{g}").as_bytes());
    let id = Object::parse(include_str!("fixtures/mutual.bisa"))
        .unwrap()
        .identity()
        .unwrap();
    assert_eq!(id.envelopes["f"], f);
    assert_eq!(id.envelopes["g"], g);
    assert_eq!(id.scc_members["f"], ["f", "g"]);
    assert_eq!(id.scc_hashes["f"], digest);
    assert_eq!(
        sha256_hex(format!("{digest}:0").as_bytes()),
        "e4028cf7347c2779e19c8193ef1039837944acebd4784e9c3a42fbe236e737ec"
    );
    assert_eq!(
        sha256_hex(format!("{digest}:1").as_bytes()),
        "7dc9c25aff5230a46ae697edf187d34ba72a03f7a3c1a520c6fc6bf5462c6374"
    );
    assert_eq!(
        id.interface_hashes["f"],
        sha256_hex(format!("{digest}:0").as_bytes())
    );
    assert_eq!(
        id.interface_hashes["g"],
        sha256_hex(format!("{digest}:1").as_bytes())
    );
}

#[test]
fn council_r4_cc_envelope_includes_service_but_not_unused_model() {
    // §4.8: effect is R-only, hostcall CC-only. A CC envelope includes its
    // service but not an unused declared model; both cannot be used directly.
    let raw = support::object(
        &[("mix", r#"{"abi":"0.8","kind":"cc","params":["ptr","i32"],"results":[],"tier":"cc"}"#,
        "fn cc @mix (p:ptr, n:i32) () () () ()\n^entry():\nr = hostcall a.service p n\nret")], "{}",
        "\nsection devices\n{\"models\":{\"clock\":{\"version\":\"0.8\"}},\"services\":{\"a.service\":{\"version\":\"1\"}}}\n\nsection layout\n{\"A_tab\":0,\"H\":0,\"S_cells\":0,\"globals\":{},\"heap_base\":1,\"heap_limit_max\":0,\"interface_regions\":{},\"null_size\":1,\"ptr_width\":32}\n"
    );
    let expected = concat!(
        "isa=0.8\nptr_width=32\n",
        "{\"abi\":\"0.8\",\"kind\":\"cc\",\"params\":[\"ptr\",\"i32\"],\"results\":[],\"tier\":\"cc\"}\n",
        "fn cc @mix (p:ptr, n:i32) () () () ()\n^entry():\nr = hostcall a.service p n\nret\n",
        "layout A_tab 0\nlayout H 0\nlayout S_cells 0\nlayout heap_base 1\nlayout heap_limit_max 0\nlayout null_size 1\nlayout ptr_width 32\n",
        "device a.service {\"version\":\"1\"}\n"
    );
    let id = Object::parse(&raw).unwrap().identity().unwrap();
    assert_eq!(id.envelopes["mix"], expected);
    assert_eq!(id.interface_hashes["mix"], sha256_hex(expected.as_bytes()));
}

#[test]
fn r11_oracle_layout_ownership_uses_exact_names() {
    let expected = concat!(
        "isa=0.8\nptr_width=32\n",
        "{\"abi\":\"0.8\",\"args\":[],\"fault\":4,\"in\":[1],\"kind\":\"oracle\",\"mem\":[],\"tgt\":1,\"tier\":\"q\"}\n",
        "oracle @a (x:wire in) (t:wire) (f:wire[4]) () ()\n^entry():\nlift.retval x\n",
        "layout A_tab 0\nlayout H 4\nlayout S_cells 0\nlayout heap_base 1\nlayout heap_limit_max 0\nlayout null_size 1\nlayout ptr_width 32\n"
    );
    assert_eq!(
        sha256_hex(expected.as_bytes()),
        "1b593b9027243623bfb18cc5b34c51d7df81d23526bc4987c14a5da2806b9521"
    );
    for tag in ["sol-layout-2", "sol-layout-3"] {
        let o = Object::parse(&support::r11_text(tag)).unwrap();
        assert_eq!(o.identity().unwrap().envelopes["a"], expected);
        support::r11_pin(tag);
    }
}

#[test]
fn r11_dotted_oracle_layout_ownership() {
    let expected = concat!(
        "isa=0.8\nptr_width=32\n",
        "{\"abi\":\"0.8\",\"args\":[],\"fault\":4,\"in\":[8],\"kind\":\"oracle\",\"mem\":[{\"A\":0,\"W\":8,\"role\":\"in\"}],\"tgt\":8,\"tier\":\"q\"}\n",
        "oracle @a (x:wire[8] in) (t:wire[8]) (f:wire[4]) (db:region[0,8] in) ()\n^entry():\nlift.retval x\n",
        "layout A_tab 1\nlayout H 8\nlayout S_cells 0\nlayout heap_base 128\nlayout heap_limit_max 0\n",
        "layout interface_regions.a.db {\"base\":16}\nlayout null_size 1\nlayout ptr_width 32\n"
    );
    assert_eq!(
        sha256_hex(expected.as_bytes()),
        "4671ca551ee57390d7f69d362e83180a1f3b29a3b8d2f45e2cd80962c6692aec"
    );
    for tag in ["codex-layout-32", "codex-layout-48"] {
        let o = Object::parse(&support::r11_text(tag)).unwrap();
        assert_eq!(o.identity().unwrap().envelopes["a"], expected);
        let o = support::r11_pin(tag);
        let mut input = bennett_isa_ref::OracleInput::bits(&[7]);
        input.regions.insert("db".into(), vec![0]);
        let result = o
            .run_oracle("a", &input, bennett_isa_ref::Budget::steps(1))
            .unwrap();
        assert_eq!((result.payload, result.code, result.steps), (7, 0, 1));
    }
    let changed = support::r11_text("codex-layout-32").replace("\"base\":16", "\"base\":17");
    assert_ne!(
        Object::parse(&changed)
            .unwrap()
            .identity()
            .unwrap()
            .interface_hashes["a"],
        sha256_hex(expected.as_bytes())
    );
}

#[test]
fn r11_dotted_owner_and_region_family() {
    for owner in ["a", "a.b", "a.b.c"] {
        let mut envelopes = vec![];
        for base in [32, 48] {
            let tag = format!("deep-{owner}-{base}");
            let o = support::r11_pin(&tag);
            envelopes.push(o.identity().unwrap().envelopes[owner].clone());
            let changed = support::r11_text(&tag).replace("\"base\":16", "\"base\":17");
            assert_ne!(
                Object::parse(&changed)
                    .unwrap()
                    .identity()
                    .unwrap()
                    .envelopes[owner],
                envelopes[0]
            );
        }
        assert_eq!(envelopes[0], envelopes[1]);
    }
}

#[test]
fn r12_cc_r_colliding_dotted_leaves() {
    // Codex's equal values make multiplicity independent of the tie reading.
    let expected = concat!(
        "isa=0.8\nptr_width=32\n",
        "{\"abi\":\"0.8\",\"kind\":\"cc\",\"params\":[],\"results\":[],\"tier\":\"cc\"}\n",
        "fn cc @main () () () () ()\n^entry():\nret\n",
        "layout A_tab 0\nlayout H 6\nlayout S_cells 0\nlayout heap_base 48\nlayout heap_limit_max 0\n",
        "layout interface_regions.a.b.c {\"base\":16}\n",
        "layout interface_regions.a.b.c {\"base\":16}\n",
        "layout null_size 1\nlayout ptr_width 32\n"
    );
    for tier in ["cc", "r"] {
        let expected = expected
            .replace("\"cc\"", &format!("\"{tier}\""))
            .replace("fn cc ", &format!("fn {tier} "));
        let o = Object::parse(&support::r12_text(&format!("codex-{tier}"))).unwrap();
        assert_eq!(o.identity().unwrap().envelopes["main"], expected);
        support::r12_pin(&format!("codex-{tier}"));
    }
}

#[test]
fn r12_cc_r_layout_paths_preserve_colliding_leaves() {
    // Sol deep-a-32 + @main: a < a.c in original-component order.
    let expected = concat!(
        "isa=0.8\nptr_width=32\n",
        "{\"abi\":\"0.8\",\"kind\":\"cc\",\"params\":[],\"results\":[],\"tier\":\"cc\"}\n",
        "fn cc @main () () () () ()\n^entry():\nret\n",
        "layout A_tab 1\nlayout H 8\nlayout S_cells 0\nlayout heap_base 128\nlayout heap_limit_max 0\n",
        "layout interface_regions.a.c.db {\"base\":16}\n",
        "layout interface_regions.a.c.db {\"base\":32}\n",
        "layout null_size 1\nlayout ptr_width 32\n"
    );
    for tier in ["cc", "r"] {
        for base in [16, 17] {
            let expected = expected
                .replace("\"cc\"", &format!("\"{tier}\""))
                .replace("fn cc ", &format!("fn {tier} "))
                .replace("\"base\":16", &format!("\"base\":{base}"));
            let tag = format!("sol-{tier}-{base}");
            let o = Object::parse(&support::r12_text(&tag)).unwrap();
            assert_eq!(o.identity().unwrap().envelopes["main"], expected);
            support::r12_pin(&tag);
        }
    }
}

#[test]
fn r12_dotted_leaf_collision_family() {
    for owner in ["a", "a.b", "a.b.c"] {
        for tier in ["cc", "r"] {
            let mut ids = vec![];
            for (b1, b2) in [(16, 16), (16, 32), (32, 16), (17, 32), (16, 33)] {
                let o = support::r12_pin(&format!("family-{owner}-{tier}-{b1}-{b2}"));
                let id = o.identity().unwrap();
                let lines: Vec<_> = id.envelopes["main"]
                    .lines()
                    .filter(|l| l.starts_with("layout interface_regions."))
                    .collect();
                assert_eq!(
                    lines,
                    [
                        format!("layout interface_regions.{owner}.c.db {{\"base\":{b1}}}"),
                        format!("layout interface_regions.{owner}.c.db {{\"base\":{b2}}}")
                    ]
                );
                ids.push(id);
            }
            for changed in [3, 4] {
                assert_ne!(
                    ids[1].interface_hashes["main"],
                    ids[changed].interface_hashes["main"]
                );
            }
            assert_eq!(ids[1].envelopes[owner], ids[4].envelopes[owner]);
            assert_eq!(
                ids[1].envelopes[&format!("{owner}.c")],
                ids[3].envelopes[&format!("{owner}.c")]
            );
        }
    }
}

#[test]
fn r12_dotted_global_lproc_keeps_exact_closure_selection() {
    let raw = include_str!("fixtures/sum_db.bisa").replace("zero", "global.with.dots");
    let o = Object::parse(&raw).unwrap();
    let id = o.identity().unwrap();
    let env = &id.envelopes["sum"];
    assert!(env.contains(
        "layout globals.global.with.dots {\"base\":16,\"image\":\"global.with.dots\",\"size\":1}\n"
    ));
    assert!(env.contains("data global.with.dots 00\n"));
    assert_eq!(env.lines().filter(|s| s.starts_with("layout ")).count(), 1);
    let changed = Object::parse(&raw.replace("\"base\":4096", "\"base\":4095"))
        .unwrap()
        .identity()
        .unwrap();
    assert_eq!(env, &changed.envelopes["sum"]);
    assert!(!id.envelopes["global.with.dots"].contains("layout "));
    let actions = Object::parse(include_str!("fixtures/actions.bisa"))
        .unwrap()
        .identity()
        .unwrap();
    assert!(actions
        .envelopes
        .values()
        .all(|env| !env.contains("layout ")));
}

#[test]
fn r12_interface_address_is_not_a_function_dependency() {
    let expected = concat!(
        "isa=0.8\nptr_width=32\n",
        "{\"abi\":\"0.8\",\"args\":[],\"fault\":4,\"in\":[],\"kind\":\"oracle\",\"mem\":[{\"A\":0,\"W\":8,\"role\":\"in\"}],\"tgt\":32,\"tier\":\"q\"}\n",
        "oracle @o () (t:wire[32]) (f:wire[4]) (o:region[0,8] in) ()\n",
        "regs { p:reg[32] }\n^entry():\np = lift.addr @o\nlift.retval p\n",
        "layout A_tab 0\nlayout H 6\nlayout S_cells 0\nlayout heap_base 48\nlayout heap_limit_max 0\n",
        "layout interface_regions.o.o {\"base\":16}\nlayout null_size 1\nlayout ptr_width 32\n"
    );
    for region in ["o", "db"] {
        let c = support::r12_address_case(region);
        let expected = expected
            .replace("(o:region", &format!("({region}:region"))
            .replace("lift.addr @o", &format!("lift.addr @{region}"))
            .replace(
                "interface_regions.o.o ",
                &format!("interface_regions.o.{region} "),
            );
        assert_eq!(
            sha256_hex(expected.as_bytes()),
            c.get("hash").unwrap().string().unwrap()
        );
        let o = Object::parse(&support::r12_address_text(region)).unwrap();
        let id = o.identity().unwrap();
        assert_eq!(id.envelopes["o"], expected);
        assert_eq!(
            id.interface_hashes["o"],
            c.get("hash").unwrap().string().unwrap()
        );
        assert_eq!(
            id.closure_hashes["o"].as_deref(),
            Some(c.get("hash").unwrap().string().unwrap())
        );
        assert_eq!(
            id.object_hash,
            c.get("object_hash").unwrap().string().unwrap()
        );
        // No false recursive ":0" hashing or part-(6) dependency line.
        assert_eq!(id.interface_hashes["o"], id.scc_hashes["o"]);
        o.check_hashes().unwrap();
    }
}

#[test]
fn r12_interface_symbol_kind_family() {
    let mut count = 0;
    for kind in [
        "cc",
        "r",
        "oracle",
        "lproc",
        "action",
        "perm_action",
        "data",
    ] {
        for imported in ["False", "True"] {
            let c = support::r12_address_case(&format!("family-{kind}-{imported}"));
            let o = Object::parse(c.get("text").unwrap().string().unwrap()).unwrap();
            let id = o.identity().unwrap();
            assert_eq!(
                id.envelopes["o"],
                c.get("envelope").unwrap().string().unwrap()
            );
            assert_eq!(
                id.interface_hashes["o"],
                c.get("hash").unwrap().string().unwrap()
            );
            assert_eq!(
                id.closure_hashes["o"].as_deref(),
                Some(c.get("hash").unwrap().string().unwrap())
            );
            o.check_hashes().unwrap();
            count += 1;
        }
    }
    assert_eq!(count, 14);
}

#[test]
fn r12_call_operand_stays_a_dependency_beside_local_address() {
    let mut hashes = vec![];
    for value in [0, 1] {
        let c = support::r12_address_case(&format!("call-{value}"));
        let o = Object::parse(c.get("text").unwrap().string().unwrap()).unwrap();
        let id = o.identity().unwrap();
        let env = &id.envelopes["o"];
        assert_eq!(env, c.get("envelope").unwrap().string().unwrap());
        assert!(env.contains("p = lift.addr @alias\n"));
        assert!(env.contains(&format!(
            "r = lift.call @{} ()\n",
            id.interface_hashes["alias"]
        )));
        assert_eq!(env.lines().last().unwrap(), id.interface_hashes["alias"]);
        assert_eq!(
            id.interface_hashes["o"],
            c.get("hash").unwrap().string().unwrap()
        );
        assert_eq!(
            id.closure_hashes["o"].as_deref(),
            Some(c.get("hash").unwrap().string().unwrap())
        );
        o.check_hashes().unwrap();
        hashes.push(id.interface_hashes["o"].clone());
    }
    assert_ne!(hashes[0], hashes[1]);
}

#[test]
fn r12_global_and_qregion_operands_remain_symbol_references() {
    for (tag, name) in [("global", "o"), ("image", "a")] {
        let c = support::r12_address_case(tag);
        let o = Object::parse(c.get("text").unwrap().string().unwrap()).unwrap();
        let id = o.identity().unwrap();
        assert_eq!(
            id.envelopes[name],
            c.get("envelope").unwrap().string().unwrap()
        );
        assert_eq!(
            id.interface_hashes[name],
            c.get("hash").unwrap().string().unwrap()
        );
        assert_eq!(
            id.closure_hashes[name].as_deref(),
            Some(c.get("hash").unwrap().string().unwrap())
        );
        assert_eq!(
            id.envelopes[name].lines().last().unwrap(),
            id.interface_hashes["alias"]
        );
        o.check_hashes().unwrap();
        if tag == "global" {
            let mut input = bennett_isa_ref::OracleInput::bits(&[]);
            input.regions.insert("db".into(), vec![0]);
            let r = o
                .run_oracle("o", &input, bennett_isa_ref::Budget::steps(2))
                .unwrap();
            assert_eq!((r.payload, r.code, r.steps), (24, 0, 2));
        }
    }
}
